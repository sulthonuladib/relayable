//! End-to-end test: a real `relayable-server` subprocess driven by raw protocol
//! frames.
//!
//! Nothing is mocked. The test spawns the compiled binary, talks to it over a
//! real Unix socket using `relayable_protocol`'s `FrameConnection` (the
//! `relayable-client` crate is intentionally not a dependency), spawns a real
//! `/bin/sh` PTY child, and cleans up every process it started.

use std::io::{ErrorKind, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use relayable_protocol::limits::DEFAULT_LIMITS;
use relayable_protocol::methods;
use relayable_protocol::transport::{FrameConnection, TransportError};
use relayable_protocol::v1::{
    envelope::Body, request, response, AttachMode, BlockAttachRequest, BlockDetachRequest,
    Envelope, Error, ErrorCode, Handshake, Request, SessionCloseRequest, SessionCreateRequest,
    SessionResumeRequest, SessionSnapshotRequest, SessionSnapshotResponse, TerminalInput,
    TerminalResize,
};
use relayable_protocol::PROTOCOL_VERSION;

const WAIT: Duration = Duration::from_secs(10);
const RETRY: Duration = Duration::from_secs(3);

static UNIQUE: AtomicU64 = AtomicU64::new(0);

fn unique_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let unique = UNIQUE.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!(
        "relayable-e2e-{}-{nanos}-{unique}",
        std::process::id()
    ))
}

fn is_timeout(error: &std::io::Error) -> bool {
    matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut)
}

fn contains(haystack: &[u8], needle: &str) -> bool {
    let needle = needle.as_bytes();
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn assert_tagged_uuid_v7(text: &str, kind: &str) {
    let prefix = format!("{kind}:");
    let rest = text
        .strip_prefix(&prefix)
        .unwrap_or_else(|| panic!("{text:?} does not use the {prefix:?} prefix"));
    assert_eq!(rest.len(), 36, "uuid length for {text:?}");
    let hyphens = [8usize, 13, 18, 23];
    for index in hyphens {
        assert_eq!(
            rest.as_bytes()[index],
            b'-',
            "hyphen at {index} in {rest:?}"
        );
    }
    let hex: Vec<char> = rest.chars().filter(|c| *c != '-').collect();
    assert_eq!(hex.len(), 32, "hex digits in {rest:?}");
    assert!(
        hex.iter()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "lowercase hex in {rest:?}"
    );
    assert_eq!(hex[12], '7', "UUIDv7 version nibble in {rest:?}");
}

/// Owns the server child plus every PTY child pid it created, so a failing test
/// cannot leak processes.
struct ServerGuard {
    child: Child,
    socket: PathBuf,
    dir: PathBuf,
    pids: Vec<u32>,
}

impl ServerGuard {
    fn spawn() -> Self {
        Self::spawn_with_env(&[])
    }

    /// Spawn the server with extra `RELAYABLE_*` environment overrides.
    fn spawn_with_env(extra: &[(&str, &str)]) -> Self {
        let dir = unique_dir();
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let socket = dir.join("relayable.sock");
        let mut command = Command::new(env!("CARGO_BIN_EXE_relayable-server"));
        command
            .arg("run")
            .env("RELAYABLE_SERVER", &socket)
            .env("SHELL", "/bin/sh");
        for (key, value) in extra {
            command.env(key, value);
        }
        let child = command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn relayable-server");
        // Wait for the listener to accept.
        let deadline = Instant::now() + WAIT;
        loop {
            if UnixStream::connect(&socket).is_ok() {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "server never listened on {socket:?}"
            );
            thread::sleep(Duration::from_millis(20));
        }
        Self {
            child,
            socket,
            dir,
            pids: Vec::new(),
        }
    }

    fn track(&mut self, pid: u32) {
        assert!(pid > 0, "tracked pid must be nonzero");
        self.pids.push(pid);
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
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        for pid in self.pids.clone() {
            kill_process_tree(pid);
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Kill a pid and every process sharing its session (best-effort cleanup).
fn kill_process_tree(pid: u32) {
    // SAFETY: `getsid` is a read-only query used only for cleanup.
    let sid = unsafe { libc::getsid(pid as libc::pid_t) };
    if sid >= 0 {
        for member in session_members(sid) {
            // SAFETY: best-effort cleanup of processes this test started.
            unsafe {
                libc::kill(member, libc::SIGKILL);
            }
        }
    }
    // SAFETY: best-effort cleanup of a process this test started.
    unsafe {
        libc::kill(pid as libc::pid_t, libc::SIGKILL);
    }
}

/// Enumerate pids in `session`, by reading `/proc`.
fn session_members(session: libc::pid_t) -> Vec<libc::pid_t> {
    let mut members = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return members;
    };
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<libc::pid_t>().ok())
        else {
            continue;
        };
        if let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) {
            if stat_session(&stat) == Some(session) {
                members.push(pid);
            }
        }
    }
    members
}

/// Parse the session id from a `/proc/<pid>/stat` line.
fn stat_session(stat: &str) -> Option<libc::pid_t> {
    let rest = stat.rsplit_once(") ")?.1;
    let mut fields = rest.split_whitespace();
    let _state = fields.next()?;
    let _ppid = fields.next()?;
    let _pgrp = fields.next()?;
    fields.next()?.parse().ok()
}

/// Extract the pid printed by `printf 'DESC=%s\n' "$!"`.
///
/// The echoed command line contains `DESC=%s`, so scan every occurrence and
/// accept the first one followed by digits (the expanded output).
fn parse_descendant_pid(bytes: &[u8]) -> Option<u32> {
    let text = String::from_utf8_lossy(bytes);
    for (index, _) in text.match_indices("DESC=") {
        let start = index + "DESC=".len();
        let digits: String = text[start..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if !digits.is_empty() {
            if let Ok(pid) = digits.parse() {
                return Some(pid);
            }
        }
    }
    None
}

/// Extract `MARK-########` values in order.
fn extract_markers(bytes: &[u8]) -> Vec<u32> {
    let text = String::from_utf8_lossy(bytes);
    let mut markers = Vec::new();
    for part in text.split("MARK-").skip(1) {
        let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
        if digits.len() == 8 {
            if let Ok(value) = digits.parse::<u32>() {
                markers.push(value);
            }
        }
    }
    markers
}

struct Client {
    connection: FrameConnection,
    next_id: u64,
    /// Every terminal byte seen while waiting for responses, so a read that
    /// races the response is not lost.
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

    fn handshake(&mut self) -> relayable_protocol::v1::ServerHello {
        self.handshake_as("e2e")
    }

    fn handshake_as(&mut self, client_kind: &str) -> relayable_protocol::v1::ServerHello {
        self.next_id = 1;
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: Some(1),
            stream_id: 0,
            sequence: None,
            trace_id: None,
            timestamp_unix_millis: None,
            body: Some(Body::Handshake(Handshake {
                client_kind: client_kind.to_string(),
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
        match response.body {
            Some(Body::Response(response)) => match response.result {
                Some(response::Result::Hello(hello)) => hello,
                other => panic!("expected a hello, got {other:?}"),
            },
            other => panic!("expected a response, got {other:?}"),
        }
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

    fn read_until(&mut self, needle: &str, deadline: Instant) -> Vec<u8> {
        loop {
            if contains(&self.output, needle) {
                return self.output.clone();
            }
            if Instant::now() >= deadline {
                panic!(
                    "timed out waiting for {needle:?}; collected: {:?}",
                    String::from_utf8_lossy(&self.output)
                );
            }
            if let Some(envelope) = self.recv_frame(deadline) {
                if let Some(Body::TerminalData(output)) = envelope.body {
                    self.output.extend_from_slice(&output.bytes);
                }
            }
        }
    }
}

fn expect_error(result: response::Result, code: ErrorCode) -> Error {
    match result {
        response::Result::Error(error) => {
            assert_eq!(error.code, code as i32, "expected {code:?}, got {error:?}");
            error
        }
        other => panic!("expected an error, got {other:?}"),
    }
}

fn create(client: &mut Client, label: &str) -> relayable_protocol::v1::SessionCreateResponse {
    match client.request(
        methods::SESSION_CREATE,
        request::Operation::SessionCreate(SessionCreateRequest {
            label: Some(label.to_string()),
        }),
    ) {
        response::Result::SessionCreate(response) => response,
        other => panic!("session.create failed: {other:?}"),
    }
}

fn view(client: &mut Client, session_id: &str) -> SessionSnapshotResponse {
    match client.request(
        methods::SESSION_VIEW,
        request::Operation::SessionView(SessionSnapshotRequest {
            session_id: session_id.to_string(),
        }),
    ) {
        response::Result::SessionView(response) => response,
        other => panic!("session.view failed: {other:?}"),
    }
}

fn attach(
    client: &mut Client,
    block_id: &str,
    mode: AttachMode,
) -> relayable_protocol::v1::BlockAttachResponse {
    match client.request(
        methods::BLOCK_ATTACH,
        request::Operation::BlockAttach(BlockAttachRequest {
            block_id: block_id.to_string(),
            mode: mode as i32,
            takeover: false,
        }),
    ) {
        response::Result::BlockAttach(response) => response,
        other => panic!("block.attach failed: {other:?}"),
    }
}

fn input(client: &mut Client, stream_id: u64, text: &str) -> u64 {
    match client.request(
        methods::TERMINAL_INPUT,
        request::Operation::TerminalInput(TerminalInput {
            stream_id,
            bytes: text.as_bytes().to_vec(),
        }),
    ) {
        response::Result::TerminalInput(accepted) => accepted.accepted_bytes,
        other => panic!("terminal.input failed: {other:?}"),
    }
}

fn resume(
    client: &mut Client,
    session_id: &str,
    last_sequence_applied: u64,
) -> relayable_protocol::v1::SessionResumeResponse {
    match client.request(
        methods::SESSION_RESUME,
        request::Operation::SessionResume(SessionResumeRequest {
            session_id: session_id.to_string(),
            last_sequence_applied,
        }),
    ) {
        response::Result::SessionResume(response) => response,
        other => panic!("session.resume failed: {other:?}"),
    }
}

fn attach_takeover(
    client: &mut Client,
    block_id: &str,
) -> relayable_protocol::v1::BlockAttachResponse {
    match client.request(
        methods::BLOCK_ATTACH,
        request::Operation::BlockAttach(BlockAttachRequest {
            block_id: block_id.to_string(),
            mode: AttachMode::Writer as i32,
            takeover: true,
        }),
    ) {
        response::Result::BlockAttach(response) => response,
        other => panic!("block.attach takeover failed: {other:?}"),
    }
}

/// Read the next control event, ignoring terminal output and responses.
fn next_event(client: &mut Client, deadline: Instant) -> Option<relayable_protocol::v1::Event> {
    loop {
        let envelope = client.recv_frame(deadline)?;
        if let Some(Body::Event(event)) = envelope.body {
            return Some(event);
        }
    }
}

fn proc_exists(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

fn wait_gone(pid: u32, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !proc_exists(pid) {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }
    false
}

fn detach(client: &mut Client, stream_id: u64) {
    match client.request(
        methods::BLOCK_DETACH,
        request::Operation::BlockDetach(BlockDetachRequest { stream_id }),
    ) {
        response::Result::BlockDetach(_) => {}
        other => panic!("block.detach failed: {other:?}"),
    }
}

#[test]
fn server_end_to_end() {
    let mut server = ServerGuard::spawn();

    // (a) handshake
    let mut client = Client::new(server.connect());
    let hello = client.handshake();
    assert_eq!(hello.protocol_version, "1.0");
    assert_eq!(hello.limits, Some(DEFAULT_LIMITS));

    // (b) session.create
    let created = create(&mut client, "e2e");
    let session_id = created
        .session
        .as_ref()
        .expect("session record")
        .session_id
        .clone();
    let window_id = created.window_id.clone();
    let block_id = created.block_id.clone();
    let pty_process_id = created.pty_process_id.clone();
    let program_pid = created.program_pid;
    assert_tagged_uuid_v7(&session_id, "session");
    assert_tagged_uuid_v7(&window_id, "window");
    assert_tagged_uuid_v7(&block_id, "block");
    assert_tagged_uuid_v7(&pty_process_id, "pty");
    assert!(program_pid > 0, "program_pid must be a real pid");
    server.track(program_pid);

    // (c) session.list
    match client.request(
        methods::SESSION_LIST,
        request::Operation::SessionList(relayable_protocol::v1::SessionListRequest {}),
    ) {
        response::Result::SessionList(list) => {
            assert!(
                list.sessions
                    .iter()
                    .any(|summary| summary.session_id == session_id),
                "session.list must include the new session"
            );
        }
        other => panic!("session.list failed: {other:?}"),
    }

    // (d) session.view
    let snapshot = view(&mut client, &session_id);
    assert!(snapshot.sequence >= 1);
    let session = snapshot.session.as_ref().expect("session");
    assert_eq!(session.session_id, session_id);
    assert_eq!(
        session.state,
        relayable_protocol::v1::SessionState::Active as i32
    );
    assert_eq!(session.revision, 1);
    assert_eq!(snapshot.windows.len(), 1);
    assert_eq!(snapshot.windows[0].window_id, window_id);
    assert_eq!(snapshot.blocks.len(), 1);
    let block = &snapshot.blocks[0];
    assert_eq!(block.block_id, block_id);
    assert_eq!(block.cols, 80);
    assert_eq!(block.rows, 24);
    assert_eq!(block.pty_process_id, pty_process_id);
    assert_eq!(block.program_pid, program_pid);
    assert_eq!(
        block.program_status.as_ref().expect("status").state,
        relayable_protocol::v1::ProgramState::Running as i32
    );
    assert!(
        !snapshot.clients.is_empty(),
        "the connected client is listed"
    );

    // (e) block.attach (writer)
    let attached = attach(&mut client, &block_id, AttachMode::Writer);
    let stream_id = attached.stream_id;
    assert!(stream_id >= 1, "stream ids start at 1");
    assert_eq!(attached.mode, AttachMode::Writer as i32);
    assert!(attached.writer_lease);
    let attached_snapshot = attached.snapshot.expect("terminal snapshot");
    assert_eq!(attached_snapshot.cols, 80);
    assert_eq!(attached_snapshot.rows, 24);

    // (f) terminal I/O round-trip proves a real PTY
    assert!(input(&mut client, stream_id, "tty\n") > 0);
    client.read_until("/dev/pts/", Instant::now() + WAIT);

    // (g) stty sees the initial size
    input(&mut client, stream_id, "stty size\n");
    client.read_until("24 80", Instant::now() + WAIT);

    // (h) resize, then stty and the snapshot both report 100x40
    match client.request(
        methods::TERMINAL_RESIZE,
        request::Operation::TerminalResize(TerminalResize {
            stream_id,
            cols: 100,
            rows: 40,
            pixel_width: None,
            pixel_height: None,
        }),
    ) {
        response::Result::TerminalResize(resized) => {
            assert_eq!(resized.cols, 100);
            assert_eq!(resized.rows, 40);
        }
        other => panic!("terminal.resize failed: {other:?}"),
    }
    input(&mut client, stream_id, "stty size\n");
    client.read_until("40 100", Instant::now() + WAIT);
    let resized_snapshot = view(&mut client, &session_id);
    assert_eq!(resized_snapshot.blocks[0].cols, 100);
    assert_eq!(resized_snapshot.blocks[0].rows, 40);

    // (i) disconnect survival: set a shell variable, drop the connection, then
    // reconnect and prove the SAME shell process (and variable) survived.
    let nonce = format!("survival-{}", UNIQUE.fetch_add(1, Ordering::SeqCst));
    input(
        &mut client,
        stream_id,
        &format!("RELAYABLE_SURVIVAL={nonce}\n"),
    );
    // Non-echoable confirmation that the shell stored the variable: the echoed
    // input line contains `SURV-%s`, not the expanded value.
    input(
        &mut client,
        stream_id,
        "printf 'SURV-%s\\n' \"$RELAYABLE_SURVIVAL\"\n",
    );
    client.read_until(&format!("SURV-{nonce}"), Instant::now() + WAIT);
    drop(client); // socket close

    // (i2) the child is still alive after the client disconnect
    assert!(
        proc_exists(program_pid),
        "the PTY child must survive the client disconnect"
    );

    let mut client2 = Client::new(server.connect());
    client2.handshake();
    let snapshot2 = view(&mut client2, &session_id);
    assert_eq!(
        snapshot2.session.as_ref().expect("session").session_id,
        session_id
    );
    assert_eq!(
        snapshot2.session.as_ref().expect("session").state,
        relayable_protocol::v1::SessionState::Active as i32
    );
    let block2 = &snapshot2.blocks[0];
    assert_eq!(block2.block_id, block_id);
    assert_eq!(block2.pty_process_id, pty_process_id);
    assert_eq!(block2.program_pid, program_pid);
    assert_eq!(
        block2.program_status.as_ref().expect("status").state,
        relayable_protocol::v1::ProgramState::Running as i32
    );

    // The lease is freed asynchronously once the server notices EOF; retry.
    let attach_deadline = Instant::now() + RETRY;
    let stream2 = loop {
        match client2.request(
            methods::BLOCK_ATTACH,
            request::Operation::BlockAttach(BlockAttachRequest {
                block_id: block_id.clone(),
                mode: AttachMode::Writer as i32,
                takeover: false,
            }),
        ) {
            response::Result::BlockAttach(attached) => break attached.stream_id,
            response::Result::Error(error)
                if error.code == ErrorCode::InputNotOwned as i32
                    && Instant::now() < attach_deadline =>
            {
                thread::sleep(Duration::from_millis(50));
            }
            other => panic!("post-reconnect attach failed: {other:?}"),
        }
    };
    assert!(stream2 >= 1);
    assert!(proc_exists(program_pid));
    input(&mut client2, stream2, "echo $RELAYABLE_SURVIVAL\n");
    client2.read_until(&nonce, Instant::now() + WAIT);

    // (j) session.close stops the child and removes the session from list
    match client2.request(
        methods::SESSION_CLOSE,
        request::Operation::SessionClose(SessionCloseRequest {
            session_id: session_id.clone(),
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
    match client2.request(
        methods::SESSION_LIST,
        request::Operation::SessionList(relayable_protocol::v1::SessionListRequest {}),
    ) {
        response::Result::SessionList(list) => {
            assert!(
                !list
                    .sessions
                    .iter()
                    .any(|summary| summary.session_id == session_id),
                "closed sessions must not appear in session.list"
            );
        }
        other => panic!("session.list failed: {other:?}"),
    }
    assert!(
        wait_gone(program_pid, RETRY),
        "the PTY child must be gone after session.close"
    );

    // (k) protocol conformance.

    // Oversized length prefix: rejected before any body and the connection is
    // closed without a server panic.
    let mut raw = UnixStream::connect(&server.socket).expect("raw connect");
    raw.write_all(&(DEFAULT_LIMITS.max_frame_bytes + 1).to_le_bytes())
        .expect("write oversized prefix");
    raw.flush().ok();
    raw.set_read_timeout(Some(RETRY)).ok();
    let mut byte = [0u8; 1];
    match raw.read(&mut byte) {
        Ok(0) | Err(_) => {}
        Ok(_) => panic!("server answered an oversized frame"),
    }

    // Zero-length frame: closed.
    let mut raw = UnixStream::connect(&server.socket).expect("raw connect");
    raw.write_all(&0u32.to_le_bytes())
        .expect("write zero prefix");
    raw.flush().ok();
    raw.set_read_timeout(Some(RETRY)).ok();
    match raw.read(&mut byte) {
        Ok(0) | Err(_) => {}
        Ok(_) => panic!("server answered a zero-length frame"),
    }

    // Truncated body: closed.
    let mut raw = UnixStream::connect(&server.socket).expect("raw connect");
    raw.write_all(&10u32.to_le_bytes()).expect("write prefix");
    raw.write_all(&[1, 2, 3]).expect("write partial body");
    raw.shutdown(Shutdown::Write).expect("half-close");
    raw.set_read_timeout(Some(RETRY)).ok();
    match raw.read(&mut byte) {
        Ok(0) | Err(_) => {}
        Ok(_) => panic!("server answered a truncated frame"),
    }

    // The server is still alive and serving after the malformed frames.
    let mut probe = Client::new(server.connect());
    probe.handshake();
    match probe.request(
        methods::SESSION_LIST,
        request::Operation::SessionList(relayable_protocol::v1::SessionListRequest {}),
    ) {
        response::Result::SessionList(_) => {}
        other => panic!("server unhealthy after malformed frames: {other:?}"),
    }

    // Unknown method string -> METHOD_NOT_FOUND.
    let mut unknown_method = Client::new(server.connect());
    unknown_method.handshake();
    let unknown_id = unknown_method.next_id;
    unknown_method.next_id += 1;
    unknown_method.send(&Envelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        request_id: Some(unknown_id),
        stream_id: 0,
        sequence: None,
        trace_id: None,
        timestamp_unix_millis: None,
        body: Some(Body::Request(Request {
            method: "bogus.method".to_string(),
            operation: None,
        })),
    });
    let result = loop {
        let envelope = unknown_method
            .recv_frame(Instant::now() + WAIT)
            .expect("response");
        if envelope.request_id == Some(unknown_id) {
            if let Some(Body::Response(response)) = envelope.body {
                break response.result.expect("result");
            }
        }
    };
    expect_error(result, ErrorCode::MethodNotFound);

    // Unknown session id -> UNKNOWN_ENTITY.
    let missing_session = relayable_model::SessionId::new().to_string();
    let mut unknown_session = Client::new(server.connect());
    unknown_session.handshake();
    expect_error(
        unknown_session.request(
            methods::SESSION_VIEW,
            request::Operation::SessionView(SessionSnapshotRequest {
                session_id: missing_session,
            }),
        ),
        ErrorCode::UnknownEntity,
    );

    // Writer-lease conflict and observer input rejection.
    let mut holder = Client::new(server.connect());
    holder.handshake();
    let held = create(&mut holder, "lease");
    let held_session = held.session.as_ref().expect("session").session_id.clone();
    let held_block = held.block_id.clone();
    let held_pid = held.program_pid;
    server.track(held_pid);
    let held_stream = attach(&mut holder, &held_block, AttachMode::Writer).stream_id;

    let mut other = Client::new(server.connect());
    other.handshake();
    expect_error(
        other.request(
            methods::BLOCK_ATTACH,
            request::Operation::BlockAttach(BlockAttachRequest {
                block_id: held_block.clone(),
                mode: AttachMode::Writer as i32,
                takeover: false,
            }),
        ),
        ErrorCode::InputNotOwned,
    );
    let observer = attach(&mut other, &held_block, AttachMode::Observer);
    assert_eq!(observer.mode, AttachMode::Observer as i32);
    assert!(!observer.writer_lease);
    expect_error(
        other.request(
            methods::TERMINAL_INPUT,
            request::Operation::TerminalInput(TerminalInput {
                stream_id: observer.stream_id,
                bytes: b"whoami\n".to_vec(),
            }),
        ),
        ErrorCode::InputNotOwned,
    );

    // Close the lease session so the cleanup guard has nothing live.
    match holder.request(
        methods::SESSION_CLOSE,
        request::Operation::SessionClose(SessionCloseRequest {
            session_id: held_session,
        }),
    ) {
        response::Result::SessionClose(_) => {}
        other => panic!("close lease session failed: {other:?}"),
    }
    assert!(wait_gone(held_pid, RETRY));
    let _ = held_stream;
}

#[test]
fn graceful_shutdown_kills_live_children() {
    let mut server = ServerGuard::spawn();
    let mut client = Client::new(server.connect());
    client.handshake();
    let created = create(&mut client, "shutdown");
    let pid = created.program_pid;
    server.track(pid);
    assert!(proc_exists(pid), "child must be alive before shutdown");

    // SIGTERM the *foreground* server: its handler must run the graceful
    // shutdown path, which kills every live child before exiting.
    let server_pid = server.child.id();
    // SAFETY: signalling a process this test started.
    unsafe {
        libc::kill(server_pid as libc::pid_t, libc::SIGTERM);
    }
    let deadline = Instant::now() + WAIT;
    loop {
        match server.child.try_wait().expect("try_wait") {
            Some(_) => break,
            None if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
            None => panic!("server did not exit after SIGTERM"),
        }
    }
    assert!(
        wait_gone(pid, RETRY),
        "graceful shutdown must kill the live PTY child"
    );
}

/// M1: a descendant that inherited the PTY slave must not keep `session.close`
/// (or server shutdown) blocked. `/bin/sh` here enables job control, so the
/// background `sleep` is in its own process group but the child's session; the
/// server must terminate the whole session tree.
#[test]
fn session_close_terminates_descendants_holding_the_slave() {
    let mut server = ServerGuard::spawn();
    let mut client = Client::new(server.connect());
    client.handshake();
    let created = create(&mut client, "m1");
    let session_id = created
        .session
        .as_ref()
        .expect("session")
        .session_id
        .clone();
    let block_id = created.block_id.clone();
    let child_pid = created.program_pid;
    server.track(child_pid);
    let stream_id = attach(&mut client, &block_id, AttachMode::Writer).stream_id;

    // Start a background descendant that keeps the PTY slave open, and print its
    // pid so we can assert it dies too. The echoed command contains `DESC=%s`,
    // so we wait for the *expanded* `DESC=<digits>` output.
    input(
        &mut client,
        stream_id,
        "sleep 300 & printf 'DESC=%s\\n' \"$!\"\n",
    );
    let deadline = Instant::now() + WAIT;
    let descendant_pid = loop {
        if let Some(pid) = parse_descendant_pid(&client.output) {
            break pid;
        }
        assert!(
            Instant::now() < deadline,
            "no descendant pid in output: {:?}",
            String::from_utf8_lossy(&client.output)
        );
        if let Some(envelope) = client.recv_frame(deadline) {
            if let Some(Body::TerminalData(output)) = envelope.body {
                client.output.extend_from_slice(&output.bytes);
            }
        }
    };
    server.track(descendant_pid);
    assert!(proc_exists(descendant_pid));
    assert!(proc_exists(child_pid));

    let started = Instant::now();
    match client.request(
        methods::SESSION_CLOSE,
        request::Operation::SessionClose(SessionCloseRequest {
            session_id: session_id.clone(),
        }),
    ) {
        response::Result::SessionClose(closed) => assert_eq!(
            closed.session.expect("closed session").state,
            relayable_protocol::v1::SessionState::Closed as i32
        ),
        other => panic!("session.close failed: {other:?}"),
    }
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(5),
        "session.close took {elapsed:?}"
    );
    assert!(wait_gone(child_pid, RETRY), "direct child must be gone");
    assert!(
        wait_gone(descendant_pid, RETRY),
        "descendant holding the slave must be gone"
    );
}

/// M2: a program that never reads must not wedge the connection or leak the
/// writer lease; a full input queue answers `rate_limited`.
#[test]
fn blocking_program_does_not_wedge_the_connection() {
    let mut server = ServerGuard::spawn();
    let mut client = Client::new(server.connect());
    client.handshake();
    let created = create(&mut client, "m2");
    let session_id = created
        .session
        .as_ref()
        .expect("session")
        .session_id
        .clone();
    let block_id = created.block_id.clone();
    server.track(created.program_pid);
    let stream_id = attach(&mut client, &block_id, AttachMode::Writer).stream_id;

    // Foreground program that never reads, with echo disabled so the large
    // input is not echoed back as output (which would trip output backpressure).
    assert!(input(&mut client, stream_id, "stty -echo; sleep 300\n") > 0);
    thread::sleep(Duration::from_millis(100));

    // ~1 MiB across frames that each fit the 1 MiB frame limit.
    let frame = vec![b'x'; 128 * 1024 - 64];
    let mut accepted = 0usize;
    for _ in 0..8 {
        let started = Instant::now();
        match client.request(
            methods::TERMINAL_INPUT,
            request::Operation::TerminalInput(TerminalInput {
                stream_id,
                bytes: frame.clone(),
            }),
        ) {
            response::Result::TerminalInput(ok) => accepted += ok.accepted_bytes as usize,
            other => panic!("unexpected terminal.input result: {other:?}"),
        }
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "terminal.input response was slow (> 2 s)"
        );
    }
    assert!(
        accepted >= frame.len(),
        "at least one frame must be accepted"
    );

    // The connection thread is still responsive.
    let snapshot = view(&mut client, &session_id);
    assert_eq!(snapshot.blocks[0].block_id, block_id);

    // The writer lease is still held; a second connection cannot take it.
    let mut other = Client::new(server.connect());
    other.handshake();
    expect_error(
        other.request(
            methods::BLOCK_ATTACH,
            request::Operation::BlockAttach(BlockAttachRequest {
                block_id: block_id.clone(),
                mode: AttachMode::Writer as i32,
                takeover: false,
            }),
        ),
        ErrorCode::InputNotOwned,
    );

    // Keep enqueueing until the bounded queue reports `rate_limited`.
    let mut rate_limited = false;
    for _ in 0..200 {
        match client.request(
            methods::TERMINAL_INPUT,
            request::Operation::TerminalInput(TerminalInput {
                stream_id,
                bytes: frame.clone(),
            }),
        ) {
            response::Result::TerminalInput(_) => {}
            response::Result::Error(error) if error.code == ErrorCode::RateLimited as i32 => {
                assert!(error.retryable);
                rate_limited = true;
                break;
            }
            other => panic!("unexpected overflow result: {other:?}"),
        }
    }
    assert!(
        rate_limited,
        "the bounded input queue must report rate_limited"
    );

    // Still responsive after overflow.
    assert_eq!(view(&mut client, &session_id).blocks[0].block_id, block_id);

    // Detaching frees the lease; another writer can attach normally.
    detach(&mut client, stream_id);
    let mut third = Client::new(server.connect());
    third.handshake();
    let attached = attach(&mut third, &block_id, AttachMode::Writer);
    assert!(attached.writer_lease);
}

/// M4: the attach snapshot and subscriber registration are atomic, so
/// `screen_bytes` plus the delivered frames contain an unbroken, non-duplicated
/// marker sequence.
#[test]
fn attach_snapshot_and_frames_are_contiguous() {
    let mut server = ServerGuard::spawn();
    let mut producer = Client::new(server.connect());
    producer.handshake();
    let created = create(&mut producer, "m4");
    let session_id = created
        .session
        .as_ref()
        .expect("session")
        .session_id
        .clone();
    let block_id = created.block_id.clone();
    server.track(created.program_pid);
    let stream_a = attach(&mut producer, &block_id, AttachMode::Writer).stream_id;

    // A slow, continuous marker stream so the observer attaches mid-stream.
    input(
        &mut producer,
        stream_a,
        "i=0; while [ $i -lt 5000 ]; do printf 'MARK-%08d\\n' \"$i\"; i=$((i+1)); if [ $((i % 500)) -eq 0 ]; then sleep 0.05; fi; done\n",
    );
    thread::sleep(Duration::from_millis(150));

    let mut observer = Client::new(server.connect());
    observer.handshake();
    let attached = attach(&mut observer, &block_id, AttachMode::Observer);
    let screen = attached.snapshot.expect("snapshot").screen_bytes;

    observer.read_until("MARK-00004999", Instant::now() + Duration::from_secs(15));
    let mut combined = screen;
    combined.extend_from_slice(&observer.output);

    let markers = extract_markers(&combined);
    assert!(
        markers.len() > 100,
        "expected a substantial marker run, got {}",
        markers.len()
    );
    for pair in markers.windows(2) {
        assert_eq!(
            pair[1],
            pair[0] + 1,
            "gap or duplicate in the marker stream: {} then {}",
            pair[0],
            pair[1]
        );
    }

    match producer.request(
        methods::SESSION_CLOSE,
        request::Operation::SessionClose(SessionCloseRequest { session_id }),
    ) {
        response::Result::SessionClose(_) => {}
        other => panic!("cleanup close failed: {other:?}"),
    }
}

/// m3/m7: a closed session's block is unresolvable, and stale stream entries
/// pointing at the dead block answer `unknown_entity`.
#[test]
fn closed_session_block_and_streams_are_unknown() {
    let mut server = ServerGuard::spawn();
    let mut client = Client::new(server.connect());
    client.handshake();
    let created = create(&mut client, "m3");
    let session_id = created
        .session
        .as_ref()
        .expect("session")
        .session_id
        .clone();
    let block_id = created.block_id.clone();
    let child_pid = created.program_pid;
    server.track(child_pid);
    let stream_id = attach(&mut client, &block_id, AttachMode::Writer).stream_id;

    match client.request(
        methods::SESSION_CLOSE,
        request::Operation::SessionClose(SessionCloseRequest {
            session_id: session_id.clone(),
        }),
    ) {
        response::Result::SessionClose(_) => {}
        other => panic!("session.close failed: {other:?}"),
    }
    assert!(wait_gone(child_pid, RETRY));

    // m3: the closed block cannot be attached.
    expect_error(
        client.request(
            methods::BLOCK_ATTACH,
            request::Operation::BlockAttach(BlockAttachRequest {
                block_id: block_id.clone(),
                mode: AttachMode::Writer as i32,
                takeover: false,
            }),
        ),
        ErrorCode::UnknownEntity,
    );
    // m7: stale stream entries answer unknown_entity.
    expect_error(
        client.request(
            methods::TERMINAL_INPUT,
            request::Operation::TerminalInput(TerminalInput {
                stream_id,
                bytes: b"x".to_vec(),
            }),
        ),
        ErrorCode::UnknownEntity,
    );
    expect_error(
        client.request(
            methods::BLOCK_DETACH,
            request::Operation::BlockDetach(BlockDetachRequest { stream_id }),
        ),
        ErrorCode::UnknownEntity,
    );
}

/// CORE-002: `session.resume` replay/snapshot shapes and writer-lease takeover.
#[test]
fn resume_shapes_and_writer_takeover() {
    let mut server = ServerGuard::spawn();
    let mut a = Client::new(server.connect());
    a.handshake();
    let created = create(&mut a, "resume");
    let session_id = created
        .session
        .as_ref()
        .expect("session")
        .session_id
        .clone();
    let block_id = created.block_id.clone();
    server.track(created.program_pid);

    // First view completes synchronization (Ready) and emits the `ready` event.
    let _ = view(&mut a, &session_id);
    // A second view observes the log head with no further state change.
    let head = view(&mut a, &session_id).sequence;

    // No cursor: snapshot fallback, reason `no_cursor`, events EMPTY.
    let no_cursor = resume(&mut a, &session_id, 0);
    assert!(no_cursor.snapshot_required);
    assert_eq!(no_cursor.snapshot_required_reason, "no_cursor");
    assert!(no_cursor.events.is_empty(), "never a false replay");
    let snapshot = no_cursor.snapshot.as_ref().expect("snapshot payload");
    assert_eq!(no_cursor.snapshot_sequence, no_cursor.current_sequence);
    assert_eq!(snapshot.sequence, no_cursor.snapshot_sequence);

    // Cursor at the head: replay with an empty event range and no snapshot.
    let at_head = resume(&mut a, &session_id, head);
    assert!(!at_head.snapshot_required);
    assert!(at_head.snapshot.is_none());
    assert!(at_head.events.is_empty());
    assert_eq!(at_head.snapshot_sequence, head);
    assert_eq!(at_head.current_sequence, head);

    // Cursor ahead of the server: snapshot fallback, reason `cursor_ahead`.
    let ahead = resume(&mut a, &session_id, head + 100);
    assert!(ahead.snapshot_required);
    assert_eq!(ahead.snapshot_required_reason, "cursor_ahead");
    assert!(ahead.events.is_empty());

    // Cursor at the floor: the contiguous retained suffix replays in order.
    let from_floor = resume(&mut a, &session_id, 1);
    assert!(!from_floor.snapshot_required);
    assert!(
        !from_floor.events.is_empty(),
        "session create and lease events are retained"
    );
    assert_eq!(from_floor.events.first().map(|e| e.sequence), Some(2));
    assert_eq!(from_floor.events.last().map(|e| e.sequence), Some(head));
    for pair in from_floor.events.windows(2) {
        assert_eq!(
            pair[1].sequence,
            pair[0].sequence + 1,
            "replayed events must be contiguous with no duplicates"
        );
    }

    // Takeover: a second writer takes the single lease, and the previously
    // subscribed client A receives `writer_lease_changed`.
    let stream_a = attach(&mut a, &block_id, AttachMode::Writer).stream_id;
    let mut b = Client::new(server.connect());
    b.handshake();
    let taken = attach_takeover(&mut b, &block_id);
    assert!(taken.writer_lease, "B now holds the lease");

    let deadline = Instant::now() + WAIT;
    let event = loop {
        let event = next_event(&mut a, deadline).expect("a control event");
        if matches!(
            event.event,
            Some(relayable_protocol::v1::event::Event::WriterLeaseChanged(_))
        ) {
            break event;
        }
    };
    match event.event {
        Some(relayable_protocol::v1::event::Event::WriterLeaseChanged(change)) => {
            assert!(change.lease_held, "the lease is held by the new writer");
            assert_eq!(change.block_id, block_id);
            assert!(
                !change.previous_client_id.is_empty(),
                "a takeover names the previous holder"
            );
            assert!(!change.new_client_id.is_empty());
        }
        other => panic!("expected writer_lease_changed, got {other:?}"),
    }

    // The demoted holder's input is rejected, never silently dropped.
    expect_error(
        a.request(
            methods::TERMINAL_INPUT,
            request::Operation::TerminalInput(TerminalInput {
                stream_id: stream_a,
                bytes: b"should be rejected\n".to_vec(),
            }),
        ),
        ErrorCode::InputNotOwned,
    );
    // B holds the lease and can write.
    assert!(input(&mut b, taken.stream_id, "echo takeover\n") > 0);

    match b.request(
        methods::SESSION_CLOSE,
        request::Operation::SessionClose(SessionCloseRequest {
            session_id: session_id.clone(),
        }),
    ) {
        response::Result::SessionClose(_) => {}
        other => panic!("cleanup close failed: {other:?}"),
    }
}

/// CORE-002: disconnect emits the transition event pair and then removes the
/// client record, so a snapshot lists live clients only (CORE-001 semantics),
/// while the session and child survive.
#[test]
fn disconnected_client_record_is_removed() {
    let mut server = ServerGuard::spawn();
    let mut x = Client::new(server.connect());
    x.handshake();
    let created = create(&mut x, "removed");
    let session_id = created
        .session
        .as_ref()
        .expect("session")
        .session_id
        .clone();
    let program_pid = created.program_pid;
    server.track(program_pid);

    let before = view(&mut x, &session_id);
    let x_id = before
        .clients
        .iter()
        .find(|client| client.kind == "e2e")
        .map(|client| client.client_id.clone())
        .expect("the connected client is listed in the snapshot");
    drop(x);

    let mut y = Client::new(server.connect());
    y.handshake();
    let deadline = Instant::now() + RETRY;
    let mut removed = false;
    loop {
        let snapshot = view(&mut y, &session_id);
        if snapshot
            .clients
            .iter()
            .all(|client| client.client_id != x_id)
        {
            removed = true;
            break;
        }
        if Instant::now() >= deadline {
            break;
        }
        thread::sleep(Duration::from_millis(30));
    }
    assert!(
        removed,
        "the disconnected client record must be removed from the snapshot"
    );
    assert!(
        proc_exists(program_pid),
        "the session and child must survive the client disconnect"
    );
}

/// A client-to-server heartbeat envelope; it carries no application data but
/// resets the server reader's idle deadline.
fn heartbeat_envelope() -> Envelope {
    Envelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        request_id: None,
        stream_id: 0,
        sequence: None,
        trace_id: None,
        timestamp_unix_millis: None,
        body: Some(Body::Heartbeat(relayable_protocol::v1::Heartbeat {
            sequence: None,
        })),
    }
}

/// The real reader path: a silent client must be classified `idle_timeout`
/// (never `transport_error`), while a heartbeating observer stays connected and
/// receives the disconnect event pair. This exercises the codec-wrapped timeout
/// shape over a live socket.
#[test]
fn idle_timeout_reason_is_reported_to_a_subscriber() {
    let mut server = ServerGuard::spawn_with_env(&[
        ("RELAYABLE_IDLE_TIMEOUT_MILLIS", "700"),
        ("RELAYABLE_HEARTBEAT_INTERVAL_MILLIS", "200"),
    ]);

    let mut observer = Client::new(server.connect());
    observer.handshake_as("e2e-observer");
    let created = create(&mut observer, "idle");
    let session_id = created
        .session
        .as_ref()
        .expect("session")
        .session_id
        .clone();
    let program_pid = created.program_pid;
    server.track(program_pid);
    // Subscribe the observer to live control events.
    let _ = resume(&mut observer, &session_id, 0);

    // A second client handshakes and is then deliberately silent.
    let mut victim = Client::new(server.connect());
    victim.handshake_as("e2e-victim");
    let victim_id = view(&mut observer, &session_id)
        .clients
        .iter()
        .find(|client| client.kind == "e2e-victim")
        .map(|client| client.client_id.clone())
        .expect("the victim is listed live");

    // Keep the observer alive with heartbeats well inside the idle window,
    // while the silent victim idles out.
    let deadline = Instant::now() + Duration::from_secs(4);
    let mut reason = None;
    'outer: while Instant::now() < deadline {
        observer.send(&heartbeat_envelope());
        let read_until = Instant::now() + Duration::from_millis(150);
        while let Some(envelope) = observer.recv_frame(read_until) {
            if let Some(Body::Event(event)) = envelope.body {
                if let Some(relayable_protocol::v1::event::Event::ClientDisconnected(
                    disconnected,
                )) = event.event
                {
                    if disconnected.client_id == victim_id {
                        reason = Some(disconnected.reason);
                        break 'outer;
                    }
                }
            }
        }
    }

    assert_eq!(reason.as_deref(), Some("idle_timeout"));
    assert!(
        proc_exists(program_pid),
        "the session and child must survive the idle disconnect"
    );
    // The victim is gone from the snapshot; the observer is still listed.
    let snapshot = view(&mut observer, &session_id);
    assert!(
        snapshot
            .clients
            .iter()
            .all(|client| client.client_id != victim_id),
        "the victim record must be removed"
    );
    assert!(
        snapshot
            .clients
            .iter()
            .any(|client| client.kind == "e2e-observer"),
        "the heartbeating observer stays connected"
    );
}
