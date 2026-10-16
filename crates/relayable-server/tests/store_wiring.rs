//! STORE-001 slice B: server wiring to `relayable-store`.
//!
//! These tests spawn the real `relayable-server` binary with a hermetic
//! environment (`env_clear`; only `PATH`, `RELAYABLE_SERVER`, `SHELL`, `TERM`,
//! plus the variables under test), drive it over a real Unix socket with real
//! protocol frames, and then inspect the on-disk SQLite store after a clean
//! shutdown releases the single-writer lock.
//!
//! Coverage (assignment slice B):
//! 1. create+close persists a `closed` row, one tombstone, and ordered
//!    `session_created`/`session_closed` audit events; raw terminal bytes are
//!    absent from the DB (with a positive control that the DB scan works).
//! 2. startup reconciliation marks a stale `active` row `interrupted`, and the
//!    stale session id is never resurrected as a live session.
//! 3. a second server refuses to start while the first owns the state directory
//!    (`StateDirLocked`), and the single-writer lock refuses an in-process
//!    second `Store::open` too.
//! 4. with no resolvable state directory the server serves sessions normally,
//!    creates no database/WAL/lock artifact under a fresh child cwd, and logs
//!    the persistence-disabled warning.
//! 5. a state directory that is a regular file, and a real store write failure
//!    (broken `sessions` table) both degrade to in-memory operation with a
//!    warning while protocol operations still succeed (the fail-soft policy).
//! 6. `RELAYABLE_STORE_AUDIT_MAX_ROWS` bounds the startup-pruned audit table
//!    end to end, proving the documented retention overrides are honored.
//! 7. the close-time prune refreshes its age anchor: with
//!    `RELAYABLE_STORE_AUDIT_MAX_AGE_MILLIS=0`, a session's `session_created`
//!    audit row (written after startup) is age-pruned when it closes.
//!
//! Everything is bounded: short deadlines, explicit child shutdown, and a
//! `Drop` that kills the server and every tracked PTY child.

use std::io::{ErrorKind, Read};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use relayable_protocol::methods;
use relayable_protocol::transport::{FrameConnection, TransportError};
use relayable_protocol::v1::{
    envelope::Body, request, response, AttachMode, BlockAttachRequest, Envelope, ErrorCode,
    Handshake, Request, SessionCloseRequest, SessionCreateRequest, SessionListRequest,
    SessionSnapshotRequest, TerminalInput,
};
use relayable_protocol::PROTOCOL_VERSION;
use relayable_store::{AuditEventKind, SessionCreate, Store, StoreError, StoredSessionState};

/// Generous bound for a server to become ready or a response to arrive.
const WAIT: Duration = Duration::from_secs(10);
/// Bound for graceful shutdown / process cleanup.
const SHUTDOWN: Duration = Duration::from_secs(5);

static NONCE: AtomicU64 = AtomicU64::new(0);

fn nonce(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let counter = NONCE.fetch_add(1, Ordering::SeqCst);
    format!("{prefix}-{:x}-{nanos:x}-{counter:x}", std::process::id())
}

fn child_path() -> std::ffi::OsString {
    std::env::var_os("PATH").unwrap_or_else(|| {
        std::ffi::OsString::from("/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
    })
}

fn is_timeout(error: &std::io::Error) -> bool {
    matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut)
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack.len() >= needle.len()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

/// A foreground `relayable-server run` subprocess plus every PTY child it
/// created. Dropping it terminates everything, even on a test failure.
struct ServerProcess {
    child: Option<Child>,
    socket: PathBuf,
    socket_dir: tempfile::TempDir,
    /// Fresh working directory for the child, so "no persistence artifact was
    /// created" can be asserted over a directory the test controls rather than
    /// the cargo test process cwd.
    cwd: tempfile::TempDir,
    /// The child's stderr, piped so warnings can be asserted after exit.
    stderr: Option<ChildStderr>,
    pids: Vec<u32>,
}

impl ServerProcess {
    /// Spawn the server with `RELAYABLE_STATE_DIR` set only when `state_dir` is
    /// `Some`. The environment is otherwise hermetic.
    fn spawn(state_dir: Option<&Path>) -> Self {
        Self::spawn_with(state_dir, &[])
    }

    /// Spawn the server with `RELAYABLE_STATE_DIR` (when `Some`) plus the given
    /// extra environment variables, otherwise hermetic.
    fn spawn_with(state_dir: Option<&Path>, extra_env: &[(&str, &str)]) -> Self {
        let socket_dir = tempfile::Builder::new()
            .prefix("relayable-store-wiring-")
            .tempdir()
            .expect("create socket temp dir");
        let socket = socket_dir.path().join("relayable.sock");
        let cwd = tempfile::Builder::new()
            .prefix("relayable-store-wiring-cwd-")
            .tempdir()
            .expect("create child cwd temp dir");
        let mut command = Command::new(env!("CARGO_BIN_EXE_relayable-server"));
        command
            .arg("run")
            .env_clear()
            .env("PATH", child_path())
            .env("RELAYABLE_SERVER", &socket)
            .env("SHELL", "/bin/sh")
            .env("TERM", "xterm-256color")
            .current_dir(cwd.path())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        if let Some(state_dir) = state_dir {
            command.env("RELAYABLE_STATE_DIR", state_dir);
        }
        for (name, value) in extra_env {
            command.env(name, value);
        }
        let mut child = command.spawn().expect("spawn relayable-server");
        let stderr = child.stderr.take();
        let server = Self {
            child: Some(child),
            socket,
            socket_dir,
            cwd,
            stderr,
            pids: Vec::new(),
        };
        assert!(
            server.wait_ready(WAIT),
            "server never became ready on {:?}",
            server.socket
        );
        server
    }

    fn wait_ready(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if self.socket.exists() && UnixStream::connect(&self.socket).is_ok() {
                return true;
            }
            thread::sleep(Duration::from_millis(20));
        }
        false
    }

    fn connect(&self) -> FrameConnection {
        let deadline = Instant::now() + WAIT;
        loop {
            match FrameConnection::connect(&self.socket) {
                Ok(connection) => return connection,
                Err(_) if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
                Err(error) => panic!("connect to {}: {error}", self.socket.display()),
            }
        }
    }

    fn track(&mut self, pid: u32) {
        assert!(pid > 0, "tracked pid must be nonzero");
        self.pids.push(pid);
    }

    fn socket_dir(&self) -> &Path {
        self.socket_dir.path()
    }

    /// The child's fresh working directory.
    fn cwd(&self) -> &Path {
        self.cwd.path()
    }

    /// Drain and return the child's stderr. Blocks until EOF, so call only after
    /// the child has exited (`shutdown` or a confirmed exit).
    fn take_stderr(&mut self) -> String {
        let mut text = String::new();
        if let Some(mut pipe) = self.stderr.take() {
            let _ = pipe.read_to_string(&mut text);
        }
        text
    }

    /// Send `SIGTERM`, wait for graceful exit (which runs the store checkpoint
    /// and releases the lock), and return the exit status.
    fn shutdown(&mut self) -> ExitStatus {
        let mut child = self.child.take().expect("server child still owned");
        let pid = child.id();
        // SAFETY: signalling a process this test started.
        unsafe {
            libc::kill(pid as libc::pid_t, libc::SIGTERM);
        }
        let deadline = Instant::now() + SHUTDOWN;
        loop {
            match child.try_wait().expect("try_wait") {
                Some(status) => return status,
                None if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
                None => {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("server did not exit after SIGTERM");
                }
            }
        }
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        for pid in self.pids.clone() {
            // SAFETY: best-effort cleanup of a process this test started.
            unsafe {
                libc::kill(pid as libc::pid_t, libc::SIGKILL);
            }
        }
    }
}

/// Minimal protocol client: handshake + request/response, buffering terminal
/// output so a response is never lost to a racing output frame.
struct Client {
    connection: FrameConnection,
    next_id: u64,
    output: Vec<u8>,
}

impl Client {
    fn new(connection: FrameConnection) -> Self {
        Self {
            connection,
            next_id: 1,
            output: Vec::new(),
        }
    }

    fn send(&mut self, envelope: &Envelope) {
        self.connection.send(envelope).expect("send envelope");
    }

    fn recv_frame(&mut self, deadline: Instant) -> Option<Envelope> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return None;
        }
        let _ = self.connection.set_read_timeout(Some(remaining));
        match self.connection.recv() {
            Ok(Some(envelope)) => Some(envelope),
            Ok(None) => None,
            Err(TransportError::Io(error)) if is_timeout(&error) => None,
            Err(_) => None,
        }
    }

    fn handshake(&mut self) {
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: Some(1),
            stream_id: 0,
            sequence: None,
            trace_id: None,
            timestamp_unix_millis: None,
            body: Some(Body::Handshake(Handshake {
                client_kind: "store-wiring".to_string(),
                platform: "linux".to_string(),
                client_version: "0".to_string(),
                protocol_version: PROTOCOL_VERSION.to_string(),
                capabilities: Vec::new(),
            })),
        };
        self.send(&envelope);
        self.next_id = 2;
        let response = self
            .recv_frame(Instant::now() + WAIT)
            .expect("handshake response");
        assert_eq!(response.request_id, Some(1));
    }

    fn request(&mut self, method: &str, operation: request::Operation) -> response::Result {
        let request_id = self.next_id;
        self.next_id += 1;
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: Some(request_id),
            stream_id: 0,
            sequence: None,
            trace_id: None,
            timestamp_unix_millis: None,
            body: Some(Body::Request(Request {
                method: method.to_string(),
                operation: Some(operation),
            })),
        };
        self.send(&envelope);
        let deadline = Instant::now() + WAIT;
        loop {
            let envelope = self
                .recv_frame(deadline)
                .unwrap_or_else(|| panic!("no response for request {request_id}"));
            if let Some(Body::TerminalData(output)) = &envelope.body {
                self.output.extend_from_slice(&output.bytes);
            }
            if envelope.request_id != Some(request_id) {
                continue;
            }
            if let Some(Body::Response(response)) = envelope.body {
                return response
                    .result
                    .unwrap_or_else(|| panic!("response {request_id} has no result"));
            }
        }
    }

    fn create(&mut self, label: &str) -> relayable_protocol::v1::SessionCreateResponse {
        match self.request(
            methods::SESSION_CREATE,
            request::Operation::SessionCreate(SessionCreateRequest {
                label: Some(label.to_string()),
            }),
        ) {
            response::Result::SessionCreate(response) => response,
            other => panic!("session.create failed: {other:?}"),
        }
    }

    fn list(&mut self) -> Vec<relayable_protocol::v1::SessionSummary> {
        match self.request(
            methods::SESSION_LIST,
            request::Operation::SessionList(SessionListRequest {}),
        ) {
            response::Result::SessionList(list) => list.sessions,
            other => panic!("session.list failed: {other:?}"),
        }
    }

    fn view(&mut self, session_id: &str) -> response::Result {
        self.request(
            methods::SESSION_VIEW,
            request::Operation::SessionView(SessionSnapshotRequest {
                session_id: session_id.to_string(),
            }),
        )
    }

    fn close(&mut self, session_id: &str) {
        match self.request(
            methods::SESSION_CLOSE,
            request::Operation::SessionClose(SessionCloseRequest {
                session_id: session_id.to_string(),
            }),
        ) {
            response::Result::SessionClose(closed) => {
                assert_eq!(
                    closed.session.expect("closed session").state,
                    relayable_protocol::v1::SessionState::Closed as i32
                );
            }
            other => panic!("session.close failed: {other:?}"),
        }
    }

    fn attach_writer(&mut self, block_id: &str) -> u64 {
        match self.request(
            methods::BLOCK_ATTACH,
            request::Operation::BlockAttach(BlockAttachRequest {
                block_id: block_id.to_string(),
                mode: AttachMode::Writer as i32,
                takeover: false,
            }),
        ) {
            response::Result::BlockAttach(attached) => {
                assert!(attached.writer_lease, "writer attach must hold the lease");
                attached.stream_id
            }
            other => panic!("block.attach failed: {other:?}"),
        }
    }

    fn input(&mut self, stream_id: u64, text: &str) {
        match self.request(
            methods::TERMINAL_INPUT,
            request::Operation::TerminalInput(TerminalInput {
                stream_id,
                bytes: text.as_bytes().to_vec(),
            }),
        ) {
            response::Result::TerminalInput(accepted) => {
                assert!(accepted.accepted_bytes > 0, "input must be queued")
            }
            other => panic!("terminal.input failed: {other:?}"),
        }
    }

    fn read_until(&mut self, needle: &str, deadline: Instant) {
        loop {
            if contains(&self.output, needle.as_bytes()) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for {needle:?}; collected: {:?}",
                String::from_utf8_lossy(&self.output)
            );
            if let Some(envelope) = self.recv_frame(deadline) {
                if let Some(Body::TerminalData(output)) = envelope.body {
                    self.output.extend_from_slice(&output.bytes);
                }
            }
        }
    }
}

fn expect_error(result: response::Result, code: ErrorCode) {
    match result {
        response::Result::Error(error) => {
            assert_eq!(error.code, code as i32, "expected {code:?}, got {error:?}");
        }
        other => panic!("expected an error, got {other:?}"),
    }
}

/// Run a second `relayable-server run` against the same state dir but a distinct
/// socket, bounded by `SHUTDOWN`; returns its status and stderr.
fn run_second_server(state_dir: &Path) -> (ExitStatus, String) {
    let dir = tempfile::Builder::new()
        .prefix("relayable-store-wiring-second-")
        .tempdir()
        .expect("create second temp dir");
    let socket = dir.path().join("second.sock");
    let mut child = Command::new(env!("CARGO_BIN_EXE_relayable-server"))
        .arg("run")
        .env_clear()
        .env("PATH", child_path())
        .env("RELAYABLE_SERVER", &socket)
        .env("SHELL", "/bin/sh")
        .env("RELAYABLE_STATE_DIR", state_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn second relayable-server");
    let deadline = Instant::now() + SHUTDOWN;
    let status = loop {
        match child.try_wait().expect("try_wait") {
            Some(status) => break status,
            None if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
            None => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("a second server must refuse to start while the first owns the state dir");
            }
        }
    };
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stderr.take() {
        let _ = pipe.read_to_string(&mut stderr);
    }
    (status, stderr)
}

/// Recursively collect file paths under `root`.
fn walk_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// Read the main DB file plus any WAL bytes, so a scan cannot miss a committed
/// write that has not been checkpointed.
fn read_db_bytes(state_dir: &Path) -> Vec<u8> {
    let mut bytes = std::fs::read(Store::db_path(state_dir)).unwrap_or_default();
    let wal = state_dir.join(format!("{}-wal", Store::db_file_name()));
    if let Ok(wal_bytes) = std::fs::read(wal) {
        bytes.extend(wal_bytes);
    }
    bytes
}

#[test]
fn create_and_close_persist_metadata_without_terminal_bytes() {
    let state_dir = tempfile::tempdir().expect("state temp dir");
    let mut server = ServerProcess::spawn(Some(state_dir.path()));
    let mut client = Client::new(server.connect());
    client.handshake();

    let label = nonce("persist-label");
    let created = client.create(&label);
    let session_id = created
        .session
        .as_ref()
        .expect("session record")
        .session_id
        .clone();
    let block_id = created.block_id.clone();
    server.track(created.program_pid);

    // Send a unique terminal sentinel through the real PTY and prove it was
    // echoed, so the byte-scan below is meaningful.
    let stream_id = client.attach_writer(&block_id);
    let sentinel = nonce("persist-sentinel");
    client.input(stream_id, &format!("printf '{sentinel}\\n'\n"));
    client.read_until(&sentinel, Instant::now() + WAIT);

    client.close(&session_id);
    assert!(server.shutdown().success(), "server must exit cleanly");

    // Raw bytes: the label is present (positive control), terminal bytes are not.
    let bytes = read_db_bytes(state_dir.path());
    assert!(
        contains(&bytes, label.as_bytes()),
        "positive control: the session label must be persisted in the DB"
    );
    assert!(
        !contains(&bytes, sentinel.as_bytes()),
        "terminal bytes must never be persisted in the metadata DB"
    );

    // The store is the source of truth for the persisted definition.
    let store = Store::open(state_dir.path()).expect("reopen the store after shutdown");
    let session = store
        .session(&session_id)
        .expect("read session")
        .expect("session row must exist");
    assert_eq!(session.state, StoredSessionState::Closed);
    assert!(
        session.closed_at_millis.is_some(),
        "a closed row must carry closed_at_millis"
    );

    let tombstones: Vec<_> = store
        .tombstones()
        .expect("read tombstones")
        .into_iter()
        .filter(|tombstone| tombstone.session_id == session_id)
        .collect();
    assert_eq!(tombstones.len(), 1, "exactly one tombstone for the session");

    let kinds: Vec<AuditEventKind> = store
        .audit_events()
        .expect("read audit events")
        .into_iter()
        .filter(|event| event.session_id.as_deref() == Some(session_id.as_str()))
        .map(|event| event.kind)
        .collect();
    let created_at = kinds
        .iter()
        .position(|kind| *kind == AuditEventKind::SessionCreated)
        .expect("session_created audit event");
    let closed_at = kinds
        .iter()
        .position(|kind| *kind == AuditEventKind::SessionClosed)
        .expect("session_closed audit event");
    assert!(
        created_at < closed_at,
        "audit order must be session_created then session_closed"
    );
}

#[test]
fn startup_reconciles_stale_active_rows_and_never_resurrects_them() {
    let state_dir = tempfile::tempdir().expect("state temp dir");
    let stale_id = relayable_model::SessionId::new().to_string();
    // Recent enough to survive the default 90-day session age prune, so this
    // test observes reconciliation rather than retention.
    let stale_created_at_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0);
    {
        let store = Store::open(state_dir.path()).expect("pre-create the store");
        store
            .record_session_created(&SessionCreate {
                session_id: stale_id.clone(),
                label: Some("stale-definition".to_string()),
                created_at_millis: stale_created_at_millis,
                revision: 1,
            })
            .expect("record stale active session");
    } // dropping the store releases the lock

    let mut server = ServerProcess::spawn(Some(state_dir.path()));
    let mut client = Client::new(server.connect());
    client.handshake();

    // The reconciled row is historical definition only: it is not live.
    let listed = client.list();
    assert!(
        !listed.iter().any(|summary| summary.session_id == stale_id),
        "a reconciled (interrupted) session must not be listed live"
    );
    expect_error(client.view(&stale_id), ErrorCode::UnknownEntity);

    assert!(server.shutdown().success(), "server must exit cleanly");

    let store = Store::open(state_dir.path()).expect("reopen the store");
    let session = store
        .session(&stale_id)
        .expect("read session")
        .expect("stale row must still exist as a definition");
    assert_eq!(session.state, StoredSessionState::Interrupted);
}

#[test]
fn second_writer_is_refused_at_the_server_boundary() {
    let state_dir = tempfile::tempdir().expect("state temp dir");
    let mut server = ServerProcess::spawn(Some(state_dir.path()));

    // Belt-and-suspenders: the OS lock refuses an in-process reopen too.
    match Store::open(state_dir.path()) {
        Err(StoreError::StateDirLocked(path)) => {
            assert_eq!(path, state_dir.path(), "the refusal names the state dir")
        }
        Err(other) => panic!("expected StateDirLocked, got {other:?}"),
        Ok(_) => {
            panic!("expected StateDirLocked while the server owns the dir, but open succeeded")
        }
    }

    // The real boundary: a second server exits non-zero and says why.
    let (status, stderr) = run_second_server(state_dir.path());
    assert!(
        !status.success(),
        "a second server must not start as a second writer"
    );
    assert!(
        stderr.contains("already owns the state directory"),
        "second-server stderr must contain the refusal, got {stderr:?}"
    );
    assert!(
        stderr.contains(&state_dir.path().display().to_string()),
        "second-server stderr must name the state directory, got {stderr:?}"
    );

    // The first server still owns and serves.
    let mut client = Client::new(server.connect());
    client.handshake();
    assert!(server.shutdown().success());
}

#[test]
fn without_a_state_directory_the_server_runs_without_persistence() {
    // No RELAYABLE_STATE_DIR, no XDG_STATE_HOME, no HOME: unresolvable.
    let mut server = ServerProcess::spawn(None);
    let mut client = Client::new(server.connect());
    client.handshake();

    let created = client.create("no-persistence");
    let session_id = created
        .session
        .as_ref()
        .expect("session record")
        .session_id
        .clone();
    server.track(created.program_pid);
    assert!(
        client
            .list()
            .iter()
            .any(|summary| summary.session_id == session_id),
        "the server must still serve sessions without persistence"
    );
    client.close(&session_id);
    assert!(server.shutdown().success());
    let stderr = server.take_stderr();

    // The child's cwd is a fresh temp dir owned by this test: no database, WAL,
    // or lock artifact may appear anywhere under it (the socket temp dir is
    // scanned too for completeness).
    let mut files = Vec::new();
    walk_files(server.cwd(), &mut files);
    walk_files(server.socket_dir(), &mut files);
    let artifacts: Vec<&PathBuf> = files
        .iter()
        .filter(|path| is_persistence_artifact(path))
        .collect();
    assert!(
        artifacts.is_empty(),
        "no database/WAL/lock artifact may be created when the state dir is \
         unresolvable, found {artifacts:?}"
    );

    // The documented warning must be emitted so the operator knows persistence
    // is off for this run.
    assert!(
        stderr.contains("persistence disabled"),
        "stderr must explain that persistence is disabled, got {stderr:?}"
    );
}

/// True when `path`'s file name looks like one of the store's on-disk
/// artifacts (`*.db`, `*.db-wal`, `*.lock`).
fn is_persistence_artifact(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    name.ends_with(".db") || name.ends_with(".db-wal") || name.ends_with(".lock")
}

/// A state directory that exists as a regular FILE is "any other open error":
/// the server degrades to in-memory operation and still serves sessions.
#[test]
fn state_dir_that_is_a_regular_file_degrades_instead_of_failing_startup() {
    let dir = tempfile::tempdir().expect("temp dir");
    let file = dir.path().join("not-a-state-directory");
    std::fs::write(&file, b"this is a file, not a state directory").expect("write file");

    let mut server = ServerProcess::spawn(Some(&file));
    let mut client = Client::new(server.connect());
    client.handshake();

    let created = client.create("file-state-dir");
    let session_id = created
        .session
        .as_ref()
        .expect("session record")
        .session_id
        .clone();
    server.track(created.program_pid);
    client.close(&session_id);
    assert!(
        server.shutdown().success(),
        "server must still exit cleanly"
    );
    let stderr = server.take_stderr();

    assert!(
        stderr.contains("persistence"),
        "a non-lock open error must log a persistence warning, got {stderr:?}"
    );
    // The pre-existing file is left untouched (we never truncate or replace it).
    assert_eq!(
        std::fs::read(&file).expect("read the file back"),
        b"this is a file, not a state directory"
    );
}

/// The documented fail-soft policy: a real store write failure while serving
/// must NOT fail the protocol operation; it is logged and the session still
/// creates and closes.
#[test]
fn store_write_failure_does_not_fail_protocol_operations() {
    let state_dir = tempfile::tempdir().expect("state temp dir");
    {
        // Create a valid schema first so `schema_migrations` reads version 1.
        let store = Store::open(state_dir.path()).expect("create schema");
        assert_eq!(store.schema_version().expect("schema version"), 1);
    }
    {
        // Break `sessions`: reconciliation still works, but the typed INSERT
        // cannot satisfy the extra NOT NULL column.
        let conn = rusqlite::Connection::open(Store::db_path(state_dir.path())).expect("raw open");
        conn.execute_batch(
            "DROP TABLE sessions;\n\
             CREATE TABLE sessions (\n\
                 session_id TEXT PRIMARY KEY NOT NULL,\n\
                 label TEXT,\n\
                 created_at_millis INTEGER NOT NULL,\n\
                 closed_at_millis INTEGER,\n\
                 state TEXT NOT NULL,\n\
                 last_known_revision INTEGER NOT NULL DEFAULT 1,\n\
                 broken_col TEXT NOT NULL\n\
             );",
        )
        .expect("break the sessions table");
    }

    let mut server = ServerProcess::spawn(Some(state_dir.path()));
    let mut client = Client::new(server.connect());
    client.handshake();

    // Both operations succeed even though every store write fails.
    let created = client.create("store-write-failure");
    let session_id = created
        .session
        .as_ref()
        .expect("session record")
        .session_id
        .clone();
    server.track(created.program_pid);
    assert!(
        client
            .list()
            .iter()
            .any(|summary| summary.session_id == session_id),
        "the session must still be served from authoritative memory"
    );
    client.close(&session_id);
    assert!(
        server.shutdown().success(),
        "server must still exit cleanly"
    );
    let stderr = server.take_stderr();

    assert!(
        stderr.contains("persistence"),
        "a store write failure must log a persistence warning, got {stderr:?}"
    );
    assert!(
        stderr.contains("record_session_created failed"),
        "the warning must name the failed store write, got {stderr:?}"
    );
}

/// F1 end-to-end: the documented `RELAYABLE_STORE_AUDIT_MAX_ROWS` override is
/// honored by the server's startup prune.
#[test]
fn startup_prune_honors_audit_max_rows_override() {
    const SEEDED: usize = 60;
    const CAP: usize = 50;

    let state_dir = tempfile::tempdir().expect("state temp dir");
    {
        let store = Store::open(state_dir.path()).expect("seed store");
        for index in 0..SEEDED {
            store
                .record_session_created(&SessionCreate {
                    session_id: format!("seed-{index:03}"),
                    label: None,
                    created_at_millis: 1_000 + index as i64,
                    revision: 1,
                })
                .expect("seed session");
        }
        assert_eq!(
            store.audit_events().expect("audit").len(),
            SEEDED,
            "the seed must create one audit row per session"
        );
    }

    let mut server = ServerProcess::spawn_with(
        Some(state_dir.path()),
        &[("RELAYABLE_STORE_AUDIT_MAX_ROWS", "50")],
    );
    let mut client = Client::new(server.connect());
    client.handshake();
    let created = client.create("post-prune");
    let session_id = created
        .session
        .as_ref()
        .expect("session record")
        .session_id
        .clone();
    server.track(created.program_pid);
    client.close(&session_id);
    assert!(server.shutdown().success(), "server must exit cleanly");

    let store = Store::open(state_dir.path()).expect("reopen the store");
    let rows = store.audit_events().expect("audit").len();
    assert!(
        rows <= CAP,
        "RELAYABLE_STORE_AUDIT_MAX_ROWS=50 must bound audit rows, got {rows}"
    );
}

/// F7: the close-time prune refreshes its age anchor, so rows written after
/// server startup can still age out.
///
/// `RELAYABLE_STORE_AUDIT_MAX_AGE_MILLIS=0` makes the age cutoff exactly the
/// prune-time `now`. The `session_created` audit row is written tens of
/// milliseconds before the close-time prune, so a **fresh** anchor deletes it
/// while the frozen startup anchor (set before the session existed) cannot.
#[test]
fn close_prune_refreshes_age_anchor() {
    let state_dir = tempfile::tempdir().expect("state temp dir");
    let mut server = ServerProcess::spawn_with(
        Some(state_dir.path()),
        &[("RELAYABLE_STORE_AUDIT_MAX_AGE_MILLIS", "0")],
    );
    let mut client = Client::new(server.connect());
    client.handshake();
    let created = client.create("age-anchor");
    let session_id = created
        .session
        .as_ref()
        .expect("session record")
        .session_id
        .clone();
    server.track(created.program_pid);
    // Guarantee the created audit row is strictly older than the close-time
    // prune anchor, so a refreshed `now` deletes it and a frozen one cannot.
    thread::sleep(Duration::from_millis(50));
    client.close(&session_id);
    assert!(server.shutdown().success(), "server must exit cleanly");

    let store = Store::open(state_dir.path()).expect("reopen the store");
    let events = store.audit_events().expect("audit");
    assert!(
        !events.iter().any(|event| {
            event.kind == AuditEventKind::SessionCreated
                && event.session_id.as_deref() == Some(session_id.as_str())
        }),
        "RELAYABLE_STORE_AUDIT_MAX_AGE_MILLIS=0 must age-prune the created audit row at \
         close with a fresh anchor; remaining rows: {events:?}"
    );
}
