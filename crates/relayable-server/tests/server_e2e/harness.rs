//! Shared end-to-end harness: a real `relayable-server` subprocess and a raw
//! protocol client.
//!
//! Nothing is mocked. `ServerGuard` spawns the compiled binary and kills every
//! PTY child it tracked; `Client` drives raw `relayable_protocol` frames over a
//! real Unix socket.

use std::io::ErrorKind;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use relayable_protocol::methods;
use relayable_protocol::transport::{FrameConnection, TransportError};
use relayable_protocol::v1::{
    envelope::Body, request, response, AttachMode, BlockAttachRequest, BlockDetachRequest,
    Envelope, Error, ErrorCode, Handshake, Request, SessionCreateRequest, SessionResumeRequest,
    SessionSnapshotRequest, SessionSnapshotResponse, TerminalInput,
};
use relayable_protocol::PROTOCOL_VERSION;

pub(crate) const WAIT: Duration = Duration::from_secs(10);
pub(crate) const RETRY: Duration = Duration::from_secs(3);

pub(crate) static UNIQUE: AtomicU64 = AtomicU64::new(0);

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

pub(crate) fn assert_tagged_uuid_v7(text: &str, kind: &str) {
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
pub(crate) struct ServerGuard {
    pub(crate) child: Child,
    pub(crate) socket: PathBuf,
    dir: PathBuf,
    pids: Vec<u32>,
}

impl ServerGuard {
    pub(crate) fn spawn() -> Self {
        Self::spawn_with_env(&[])
    }

    /// Spawn the server with extra `RELAYABLE_*` environment overrides.
    pub(crate) fn spawn_with_env(extra: &[(&str, &str)]) -> Self {
        let dir = unique_dir();
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let socket = dir.join("relayable.sock");
        // Hermetic persistence: each spawned server owns its own state dir. With
        // the store wired in (STORE-001 slice B), the machine-global default
        // ($HOME/.local/state/relayable) is guarded by a single-writer lock, so
        // parallel tests inheriting $HOME would otherwise contend and refuse to
        // start. A per-test state dir keeps every existing assertion intact.
        let state_dir = dir.join("state");
        let mut command = Command::new(env!("CARGO_BIN_EXE_relayable-server"));
        command
            .arg("run")
            .env("RELAYABLE_SERVER", &socket)
            .env("RELAYABLE_STATE_DIR", &state_dir)
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

    pub(crate) fn track(&mut self, pid: u32) {
        assert!(pid > 0, "tracked pid must be nonzero");
        self.pids.push(pid);
    }

    pub(crate) fn connect(&self) -> FrameConnection {
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
pub(crate) fn parse_descendant_pid(bytes: &[u8]) -> Option<u32> {
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
pub(crate) fn extract_markers(bytes: &[u8]) -> Vec<u32> {
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

pub(crate) struct Client {
    connection: FrameConnection,
    pub(crate) next_id: u64,
    /// Every terminal byte seen while waiting for responses, so a read that
    /// races the response is not lost.
    pub(crate) output: Vec<u8>,
}

impl Client {
    pub(crate) fn new(connection: FrameConnection) -> Self {
        Self {
            connection,
            next_id: 1,
            output: Vec::new(),
        }
    }

    pub(crate) fn send(&mut self, envelope: &Envelope) {
        self.connection.send(envelope).expect("send envelope");
    }

    pub(crate) fn recv_frame(&mut self, deadline: Instant) -> Option<Envelope> {
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

    pub(crate) fn handshake(&mut self) -> relayable_protocol::v1::ServerHello {
        self.handshake_as("e2e")
    }

    pub(crate) fn handshake_as(
        &mut self,
        client_kind: &str,
    ) -> relayable_protocol::v1::ServerHello {
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

    pub(crate) fn request(
        &mut self,
        method: &str,
        operation: request::Operation,
    ) -> response::Result {
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

    pub(crate) fn read_until(&mut self, needle: &str, deadline: Instant) -> Vec<u8> {
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

pub(crate) fn expect_error(result: response::Result, code: ErrorCode) -> Error {
    match result {
        response::Result::Error(error) => {
            assert_eq!(error.code, code as i32, "expected {code:?}, got {error:?}");
            error
        }
        other => panic!("expected an error, got {other:?}"),
    }
}

pub(crate) fn create(
    client: &mut Client,
    label: &str,
) -> relayable_protocol::v1::SessionCreateResponse {
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

pub(crate) fn view(client: &mut Client, session_id: &str) -> SessionSnapshotResponse {
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

pub(crate) fn attach(
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

pub(crate) fn input(client: &mut Client, stream_id: u64, text: &str) -> u64 {
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

pub(crate) fn resume(
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

pub(crate) fn attach_takeover(
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
pub(crate) fn next_event(
    client: &mut Client,
    deadline: Instant,
) -> Option<relayable_protocol::v1::Event> {
    loop {
        let envelope = client.recv_frame(deadline)?;
        if let Some(Body::Event(event)) = envelope.body {
            return Some(event);
        }
    }
}

pub(crate) fn proc_exists(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

pub(crate) fn wait_gone(pid: u32, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !proc_exists(pid) {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }
    false
}

pub(crate) fn detach(client: &mut Client, stream_id: u64) {
    match client.request(
        methods::BLOCK_DETACH,
        request::Operation::BlockDetach(BlockDetachRequest { stream_id }),
    ) {
        response::Result::BlockDetach(_) => {}
        other => panic!("block.detach failed: {other:?}"),
    }
}
