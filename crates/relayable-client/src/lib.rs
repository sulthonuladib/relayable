//! Relayable client library.
//!
//! This crate owns the connection lifecycle, the synchronous request/response
//! RPC surface, and terminal-stream reads over the local Unix-domain socket
//! transport. It depends on `relayable-model` and `relayable-protocol` only and
//! never on server internals (ADR-001 §7).
//!
//! # Concurrency
//!
//! The client is **synchronous and blocking**: it uses `std::io` over
//! `std::os::unix::net`, with no async runtime (`docs/adr/ADR-002-protocol-schema-and-transport.md`;
//! `full.md` §5.2). A dropped socket is surfaced as [`ClientError::Disconnected`]
//! on the next call; no I/O or protocol path panics.
//!
//! # Handshake (frozen by the CORE-001 slice note)
//!
//! The first frame on a connection is
//! `Envelope{ protocol_version: "1.0", request_id: Some(1), stream_id: 0,
//! body: Handshake{..} }`. The server echoes `request_id = 1` with
//! `Response.result = Hello`. Ordinary request ids then start at `2` and
//! increment monotonically per connection. Every control request uses
//! `stream_id = 0` (`relayable_protocol::CONTROL_STREAM_ID`).
//!
//! # Control RPC deadlines
//!
//! Every control RPC (and the initial handshake) waits at most
//! [`CONTROL_RPC_TIMEOUT`] for its response and then fails with
//! [`ClientError::Timeout`] instead of blocking forever. [`Client::wait_for_output`]
//! instead uses its caller-supplied timeout.
//!
//! # Re-exports
//!
//! Protocol types that appear in this crate's public API are re-exported so a
//! consumer such as the future `relayable-automation` (ADR-001
//! `automation -> {model, client}`) does not need a direct
//! `relayable-protocol` dependency:
//!
//! ```
//! use relayable_client::{
//!     AttachMode, Error, ErrorCode, Limits, ServerHello, SessionRecord,
//!     SessionSnapshotResponse, SessionSummary, TerminalSnapshot, TransportError,
//! };
//! let _ = AttachMode::Writer;
//! let _ = ErrorCode::Internal;
//! let _ = ServerHello::default();
//! let _ = SessionRecord::default();
//! let _ = SessionSnapshotResponse::default();
//! let _ = SessionSummary::default();
//! let _ = TerminalSnapshot::default();
//! let _ = Limits::default();
//! let _: Option<Error> = None;
//! fn _accepts_transport(_: TransportError) {}
//! ```

#![deny(unsafe_code)]

use std::collections::VecDeque;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use relayable_model::{BlockId, PtyProcessId, SessionId, WindowId};
use relayable_protocol::codec::CodecError;
use relayable_protocol::error_code::contract_str;
use relayable_protocol::ids;
use relayable_protocol::limits::DEFAULT_LIMITS;
use relayable_protocol::methods;
use relayable_protocol::transport::{default_socket_path, FrameConnection};
use relayable_protocol::v1::{
    envelope::Body, request::Operation, response::Result as ResponseResult, BlockAttachRequest,
    BlockDetachRequest, Envelope, FlowControl, Handshake, Heartbeat, Request, Response,
    SessionCreateRequest, SessionListResponse, SessionSnapshotRequest, TerminalInput,
    TerminalResize,
};
use relayable_protocol::{CONTROL_STREAM_ID, PROTOCOL_VERSION};

/// Protocol types that appear in this crate's public API, re-exported so
/// consumers (`relayable-automation`: ADR-001 `automation -> {model, client}`)
/// do not need a direct `relayable-protocol` dependency.
pub use relayable_protocol::transport::TransportError;
pub use relayable_protocol::v1::{
    AttachMode, Error, ErrorCode, Limits, ServerHello, SessionRecord, SessionSnapshotResponse,
    SessionSummary, TerminalSnapshot,
};

/// Maximum time a control RPC (and the initial handshake) waits for its
/// response before failing with [`ClientError::Timeout`].
pub const CONTROL_RPC_TIMEOUT: Duration = Duration::from_secs(10);

/// Static client identity sent in the handshake.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Client kind (for example `cli`).
    pub client_kind: String,
    /// Operating-system platform (`std::env::consts::OS`).
    pub platform: String,
    /// Client version (`CARGO_PKG_VERSION`).
    pub client_version: String,
}

impl ClientConfig {
    /// Build a config for `kind`, filling platform and version from the build.
    #[must_use]
    pub fn new(kind: &str) -> Self {
        Self {
            client_kind: kind.to_string(),
            platform: std::env::consts::OS.to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Failure modes of a client operation.
#[derive(Debug)]
pub enum ClientError {
    /// The socket could not be opened (missing or refused).
    Connect(io::Error),
    /// A framed transport operation failed.
    Transport(TransportError),
    /// The peer sent a malformed or unexpected frame.
    Protocol(String),
    /// The server answered with a structured error; the code, message, and
    /// details are preserved.
    Server(Error),
    /// A read deadline expired: a control RPC (see [`CONTROL_RPC_TIMEOUT`]) or
    /// a [`Client::wait_for_output`] call.
    Timeout,
    /// The peer closed the connection.
    Disconnected,
    /// The handshake never completed.
    NotHandshaken,
}

impl ClientError {
    /// The server-reported code, when this error came from the server.
    #[must_use]
    pub fn server_error_code(&self) -> Option<ErrorCode> {
        match self {
            Self::Server(err) => ErrorCode::try_from(err.code).ok(),
            _ => None,
        }
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(err) => write!(f, "could not connect to the server: {err}"),
            Self::Transport(err) => write!(f, "transport error: {err}"),
            Self::Protocol(msg) => write!(f, "protocol error: {msg}"),
            Self::Server(err) => {
                let code = ErrorCode::try_from(err.code).unwrap_or(ErrorCode::Unspecified);
                write!(f, "server error {}: {}", contract_str(code), err.message)
            }
            Self::Timeout => f.write_str("timed out waiting for the server"),
            Self::Disconnected => f.write_str("the server disconnected"),
            Self::NotHandshaken => f.write_str("the client is not handshaken"),
        }
    }
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Connect(err) => Some(err),
            Self::Transport(err) => Some(err),
            _ => None,
        }
    }
}

/// The result of `session.create`.
#[derive(Debug)]
pub struct CreatedSession {
    /// The created session record.
    pub session: SessionRecord,
    /// The window created with the session.
    pub window_id: WindowId,
    /// The terminal block created with the session.
    pub block_id: BlockId,
    /// The server-owned PTY/process identity.
    pub pty_process_id: PtyProcessId,
    /// The OS process id of the program.
    pub program_pid: u32,
}

/// The result of `block.attach`.
#[derive(Debug)]
pub struct AttachOutcome {
    /// The stream id assigned to this attachment.
    pub stream_id: u64,
    /// The negotiated attachment mode.
    pub mode: AttachMode,
    /// Whether this attachment holds the writer lease.
    pub writer_lease: bool,
    /// The recovery snapshot returned at attach time.
    pub snapshot: TerminalSnapshot,
}

/// A synchronous Relayable control client.
pub struct Client {
    conn: FrameConnection,
    socket_path: PathBuf,
    hello: ServerHello,
    limits: Limits,
    config: ClientConfig,
    next_request_id: u64,
    pending: VecDeque<Envelope>,
    control_rpc_timeout: Duration,
}

impl Client {
    /// Connect to `socket_path` and perform the handshake.
    ///
    /// Callers resolve `RELAYABLE_SERVER` themselves, or use [`Client::from_env`].
    ///
    /// # Errors
    ///
    /// [`ClientError::Connect`] if the socket cannot be opened,
    /// [`ClientError::Transport`] if the handshake frame cannot be exchanged,
    /// and [`ClientError::Server`] if the server rejects the handshake.
    pub fn connect(socket_path: &Path, config: ClientConfig) -> Result<Self, ClientError> {
        let conn = FrameConnection::connect(socket_path).map_err(ClientError::Connect)?;
        Self::from_connection(conn, socket_path.to_path_buf(), config)
    }

    /// Connect using `$RELAYABLE_SERVER` if set and non-empty, else the
    /// protocol crate's default socket path.
    ///
    /// # Errors
    ///
    /// See [`Client::connect`].
    pub fn from_env(config: ClientConfig) -> Result<Self, ClientError> {
        let path = match std::env::var_os("RELAYABLE_SERVER") {
            Some(value) if !value.is_empty() => PathBuf::from(value),
            _ => default_socket_path(),
        };
        Self::connect(&path, config)
    }

    /// The path this client connected to.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// The server's handshake reply.
    #[must_use]
    pub fn server_hello(&self) -> &ServerHello {
        &self.hello
    }

    /// The effective limits: the server-advertised limits, or the protocol
    /// defaults when the server advertised none.
    #[must_use]
    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    /// Create a session (`session.create`).
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn create_session(&mut self, label: Option<&str>) -> Result<CreatedSession, ClientError> {
        let result = self.request_operation(
            methods::SESSION_CREATE,
            Operation::SessionCreate(SessionCreateRequest {
                label: label.map(str::to_owned),
            }),
        )?;
        match result {
            ResponseResult::SessionCreate(response) => {
                let session = response
                    .session
                    .ok_or_else(|| missing("session.create response session"))?;
                let window_id = parse_id(ids::window_id_from_wire(&response.window_id), "window")?;
                let block_id = parse_id(ids::block_id_from_wire(&response.block_id), "block")?;
                let pty_process_id = parse_id(
                    ids::pty_process_id_from_wire(&response.pty_process_id),
                    "pty",
                )?;
                Ok(CreatedSession {
                    session,
                    window_id,
                    block_id,
                    pty_process_id,
                    program_pid: response.program_pid,
                })
            }
            _ => Err(unexpected("session.create", &result)),
        }
    }

    /// List sessions (`session.list`).
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn list_sessions(&mut self) -> Result<Vec<SessionSummary>, ClientError> {
        let result = self.request_operation(
            methods::SESSION_LIST,
            Operation::SessionList(relayable_protocol::v1::SessionListRequest {}),
        )?;
        match result {
            ResponseResult::SessionList(SessionListResponse { sessions, .. }) => Ok(sessions),
            _ => Err(unexpected("session.list", &result)),
        }
    }

    /// Fetch a session snapshot (`session.view`).
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn session_snapshot(
        &mut self,
        session_id: &SessionId,
    ) -> Result<SessionSnapshotResponse, ClientError> {
        let result = self.request_operation(
            methods::SESSION_VIEW,
            Operation::SessionView(SessionSnapshotRequest {
                session_id: ids::session_id_to_wire(session_id),
            }),
        )?;
        match result {
            ResponseResult::SessionView(response) => Ok(response),
            _ => Err(unexpected("session.view", &result)),
        }
    }

    /// Close a session (`session.close`) and return the closed record.
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn close_session(&mut self, session_id: &SessionId) -> Result<SessionRecord, ClientError> {
        let result = self.request_operation(
            methods::SESSION_CLOSE,
            Operation::SessionClose(relayable_protocol::v1::SessionCloseRequest {
                session_id: ids::session_id_to_wire(session_id),
            }),
        )?;
        match result {
            ResponseResult::SessionClose(response) => response
                .session
                .ok_or_else(|| missing("session.close response session")),
            _ => Err(unexpected("session.close", &result)),
        }
    }

    /// Attach to a terminal block (`block.attach`).
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn attach_block(
        &mut self,
        block_id: &BlockId,
        mode: AttachMode,
    ) -> Result<AttachOutcome, ClientError> {
        let result = self.request_operation(
            methods::BLOCK_ATTACH,
            Operation::BlockAttach(BlockAttachRequest {
                block_id: ids::block_id_to_wire(block_id),
                mode: mode as i32,
            }),
        )?;
        match result {
            ResponseResult::BlockAttach(response) => Ok(AttachOutcome {
                stream_id: response.stream_id,
                mode: AttachMode::try_from(response.mode).unwrap_or(AttachMode::Unspecified),
                writer_lease: response.writer_lease,
                snapshot: response
                    .snapshot
                    .ok_or_else(|| missing("block.attach snapshot"))?,
            }),
            _ => Err(unexpected("block.attach", &result)),
        }
    }

    /// Detach from a terminal block (`block.detach`).
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn detach_block(&mut self, stream_id: u64) -> Result<(), ClientError> {
        let result = self.request_operation(
            methods::BLOCK_DETACH,
            Operation::BlockDetach(BlockDetachRequest { stream_id }),
        )?;
        match result {
            ResponseResult::BlockDetach(_) => Ok(()),
            _ => Err(unexpected("block.detach", &result)),
        }
    }

    /// Send terminal input (`terminal.input`); returns the accepted byte count.
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn send_input(&mut self, stream_id: u64, bytes: &[u8]) -> Result<u64, ClientError> {
        let result = self.request_operation(
            methods::TERMINAL_INPUT,
            Operation::TerminalInput(TerminalInput {
                stream_id,
                bytes: bytes.to_vec(),
            }),
        )?;
        match result {
            ResponseResult::TerminalInput(accepted) => Ok(accepted.accepted_bytes),
            _ => Err(unexpected("terminal.input", &result)),
        }
    }

    /// Resize a terminal stream (`terminal.resize`).
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn resize(&mut self, stream_id: u64, cols: u16, rows: u16) -> Result<(), ClientError> {
        let result = self.request_operation(
            methods::TERMINAL_RESIZE,
            Operation::TerminalResize(TerminalResize {
                stream_id,
                cols: u32::from(cols),
                rows: u32::from(rows),
                pixel_width: None,
                pixel_height: None,
            }),
        )?;
        match result {
            ResponseResult::TerminalResize(_) => Ok(()),
            _ => Err(unexpected("terminal.resize", &result)),
        }
    }

    /// Read `TerminalOutput` frames for `stream_id` until `needle` appears in
    /// the accumulated bytes, or `timeout` elapses ([`ClientError::Timeout`]).
    ///
    /// While reading, consumed bytes are acknowledged with a `FlowControl`
    /// frame, a `Heartbeat` is sent whenever an idle read deadline expires, and
    /// the socket read timeout is held at or below the heartbeat interval.
    /// Frames that do not belong to the target stream (events, responses for
    /// other request ids, other streams) are retained and never lost.
    ///
    /// # Errors
    ///
    /// [`ClientError::Timeout`] when `needle` does not appear before the
    /// deadline, and any other [`ClientError`] from the transport.
    pub fn wait_for_output(
        &mut self,
        stream_id: u64,
        needle: &[u8],
        timeout: Duration,
    ) -> Result<Vec<u8>, ClientError> {
        let mut deferred: VecDeque<Envelope> = VecDeque::new();
        let result = self.wait_for_output_inner(stream_id, needle, timeout, &mut deferred);
        // Restore everything we set aside, preserving arrival order.
        for env in deferred.into_iter().rev() {
            self.pending.push_front(env);
        }
        result
    }

    // --- internals -------------------------------------------------------

    fn from_connection(
        conn: FrameConnection,
        socket_path: PathBuf,
        config: ClientConfig,
    ) -> Result<Self, ClientError> {
        let mut client = Self {
            conn,
            socket_path,
            hello: ServerHello::default(),
            limits: DEFAULT_LIMITS,
            config,
            next_request_id: 1,
            pending: VecDeque::new(),
            control_rpc_timeout: CONTROL_RPC_TIMEOUT,
        };
        let hello = client.perform_handshake()?;
        client.limits = hello.limits.unwrap_or(DEFAULT_LIMITS);
        client.hello = hello;
        Ok(client)
    }

    fn perform_handshake(&mut self) -> Result<ServerHello, ClientError> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: Some(request_id),
            stream_id: CONTROL_STREAM_ID,
            body: Some(Body::Handshake(Handshake {
                client_kind: self.config.client_kind.clone(),
                platform: self.config.platform.clone(),
                client_version: self.config.client_version.clone(),
                protocol_version: PROTOCOL_VERSION.to_string(),
                capabilities: Vec::new(),
            })),
            ..Envelope::default()
        };
        self.set_control_read_timeout()?;
        self.send(&envelope)?;
        let response = self.await_response(request_id)?;
        match response.result {
            Some(ResponseResult::Hello(hello)) => Ok(hello),
            Some(ResponseResult::Error(err)) => Err(ClientError::Server(err)),
            _ => Err(ClientError::Protocol(
                "expected a server hello from the handshake".to_string(),
            )),
        }
    }

    fn request_operation(
        &mut self,
        method: &str,
        operation: Operation,
    ) -> Result<ResponseResult, ClientError> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        self.set_control_read_timeout()?;
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: Some(request_id),
            stream_id: CONTROL_STREAM_ID,
            body: Some(Body::Request(Request {
                method: method.to_string(),
                operation: Some(operation),
            })),
            ..Envelope::default()
        };
        self.send(&envelope)?;
        let response = self.await_response(request_id)?;
        match response.result {
            Some(ResponseResult::Error(err)) => Err(ClientError::Server(err)),
            Some(result) => Ok(result),
            None => Err(ClientError::Protocol(format!(
                "response to {method} carried no result"
            ))),
        }
    }

    /// Read until the response for `request_id` arrives. Unrelated frames are
    /// moved to `deferred` (and restored by the caller) so nothing is lost and
    /// no queued frame is re-examined.
    fn await_response(&mut self, request_id: u64) -> Result<Response, ClientError> {
        let mut deferred: VecDeque<Envelope> = VecDeque::new();
        let result = self.await_response_inner(request_id, &mut deferred);
        for env in deferred.into_iter().rev() {
            self.pending.push_front(env);
        }
        result
    }

    fn await_response_inner(
        &mut self,
        request_id: u64,
        deferred: &mut VecDeque<Envelope>,
    ) -> Result<Response, ClientError> {
        loop {
            let envelope = match self.pending.pop_front() {
                Some(env) => env,
                None => self.recv_network()?,
            };
            match envelope.body {
                Some(Body::Response(response)) => {
                    if response.request_id == request_id {
                        return Ok(response);
                    }
                    return Err(ClientError::Protocol(format!(
                        "response for unexpected request id {}",
                        response.request_id
                    )));
                }
                Some(
                    Body::Event(_)
                    | Body::TerminalData(_)
                    | Body::FlowControl(_)
                    | Body::Heartbeat(_),
                ) => deferred.push_back(envelope),
                _ => {
                    return Err(ClientError::Protocol(
                        "unexpected frame while awaiting a response".to_string(),
                    ));
                }
            }
        }
    }

    fn wait_for_output_inner(
        &mut self,
        stream_id: u64,
        needle: &[u8],
        timeout: Duration,
        deferred: &mut VecDeque<Envelope>,
    ) -> Result<Vec<u8>, ClientError> {
        let heartbeat =
            Duration::from_millis(u64::from(self.limits.heartbeat_interval_millis.max(1)));
        let start = Instant::now();
        let deadline = start.checked_add(timeout).unwrap_or(start);
        let mut accumulated: Vec<u8> = Vec::new();

        loop {
            if needle.is_empty() || contains(&accumulated, needle) {
                return Ok(accumulated);
            }
            if Instant::now() >= deadline {
                return Err(ClientError::Timeout);
            }

            let remaining = deadline.saturating_duration_since(Instant::now());
            let read_timeout = if remaining < heartbeat {
                remaining
            } else {
                heartbeat
            };
            self.conn
                .set_read_timeout(Some(read_timeout))
                .map_err(|err| ClientError::Transport(TransportError::Io(err)))?;

            let frame = match self.pending.pop_front() {
                Some(env) => Some(env),
                None => match self.recv_network() {
                    Ok(env) => Some(env),
                    Err(ClientError::Timeout) => None,
                    Err(err) => return Err(err),
                },
            };

            match frame {
                None => self.send_heartbeat()?,
                Some(envelope) => match envelope.body {
                    Some(Body::TerminalData(output)) if output.stream_id == stream_id => {
                        let len = output.bytes.len() as u64;
                        accumulated.extend_from_slice(&output.bytes);
                        if len > 0 {
                            self.send_flow_control(stream_id, len)?;
                        }
                    }
                    _ => deferred.push_back(envelope),
                },
            }
        }
    }

    fn recv_network(&mut self) -> Result<Envelope, ClientError> {
        match self.conn.recv() {
            Ok(Some(envelope)) => Ok(envelope),
            Ok(None) => Err(ClientError::Disconnected),
            // A read timeout can surface either directly (`TransportError::Io`)
            // or wrapped by the frame codec (`TransportError::Codec` around
            // `CodecError::Io`); both mean the deadline expired.
            Err(TransportError::Io(err)) if is_timeout(&err) => Err(ClientError::Timeout),
            Err(TransportError::Codec(CodecError::Io(err))) if is_timeout(&err) => {
                Err(ClientError::Timeout)
            }
            Err(err) => Err(ClientError::Transport(err)),
        }
    }

    fn send(&mut self, envelope: &Envelope) -> Result<(), ClientError> {
        self.conn.send(envelope).map_err(ClientError::Transport)
    }

    fn send_heartbeat(&mut self) -> Result<(), ClientError> {
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            stream_id: CONTROL_STREAM_ID,
            body: Some(Body::Heartbeat(Heartbeat { sequence: None })),
            ..Envelope::default()
        };
        self.send(&envelope)
    }

    fn send_flow_control(&mut self, stream_id: u64, ack_bytes: u64) -> Result<(), ClientError> {
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            stream_id,
            body: Some(Body::FlowControl(FlowControl {
                stream_id,
                ack_bytes,
            })),
            ..Envelope::default()
        };
        self.send(&envelope)
    }

    /// Arm the socket read timeout used while waiting for a control response.
    ///
    /// On expiry the read returns a would-block error, which
    /// [`Client::recv_network`] maps to [`ClientError::Timeout`]. A well-behaved
    /// peer that has sent nothing leaves the connection usable; a partially
    /// consumed frame would force a fresh connect.
    fn set_control_read_timeout(&mut self) -> Result<(), ClientError> {
        self.conn
            .set_read_timeout(Some(self.control_rpc_timeout))
            .map_err(|err| ClientError::Transport(TransportError::Io(err)))
    }
}

fn parse_id<T>(parsed: Result<T, relayable_model::IdError>, kind: &str) -> Result<T, ClientError> {
    parsed.map_err(|err| ClientError::Protocol(format!("invalid {kind} id in response: {err}")))
}

fn missing(field: &str) -> ClientError {
    ClientError::Protocol(format!("response is missing {field}"))
}

fn unexpected(method: &str, result: &ResponseResult) -> ClientError {
    ClientError::Protocol(format!(
        "unexpected response variant for {method}: {}",
        variant_name(result)
    ))
}

fn variant_name(result: &ResponseResult) -> &'static str {
    match result {
        ResponseResult::Error(_) => "error",
        ResponseResult::Hello(_) => "hello",
        ResponseResult::SessionCreate(_) => "session_create",
        ResponseResult::SessionList(_) => "session_list",
        ResponseResult::SessionView(_) => "session_view",
        ResponseResult::SessionClose(_) => "session_close",
        ResponseResult::BlockAttach(_) => "block_attach",
        ResponseResult::BlockDetach(_) => "block_detach",
        ResponseResult::TerminalInput(_) => "terminal_input",
        ResponseResult::TerminalResize(_) => "terminal_resize",
    }
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack.len() >= needle.len()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn is_timeout(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use relayable_protocol::v1::SessionState;
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::thread;

    fn response_for(request_id: u64, result: ResponseResult) -> Envelope {
        Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: Some(request_id),
            stream_id: CONTROL_STREAM_ID,
            body: Some(Body::Response(Response {
                request_id,
                result: Some(result),
            })),
            ..Envelope::default()
        }
    }

    fn fake_server(mut conn: FrameConnection) {
        conn.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let handshake = conn.recv().unwrap().unwrap();
        assert!(matches!(handshake.body, Some(Body::Handshake(_))));
        assert_eq!(handshake.request_id, Some(1));
        conn.send(&response_for(
            1,
            ResponseResult::Hello(ServerHello {
                protocol_version: PROTOCOL_VERSION.to_string(),
                capabilities: vec!["test".to_string()],
                limits: Some(Limits {
                    max_frame_bytes: 4096,
                    ..DEFAULT_LIMITS
                }),
            }),
        ))
        .unwrap();

        let list = conn.recv().unwrap().unwrap();
        let request_id = list.request_id.unwrap();
        assert!(matches!(list.body, Some(Body::Request(_))));
        conn.send(&response_for(
            request_id,
            ResponseResult::SessionList(SessionListResponse {
                sessions: vec![SessionSummary {
                    session_id: "session:01a10cf3-2655-7e48-9b9c-43dd6612bef5".to_string(),
                    label: Some("demo".to_string()),
                    state: SessionState::Active as i32,
                    revision: 1,
                }],
                sequence: 3,
            }),
        ))
        .unwrap();
    }

    fn client_connected() -> (Client, thread::JoinHandle<()>) {
        let (client_stream, server_stream) = UnixStream::pair().unwrap();
        let server = thread::spawn(move || {
            fake_server(FrameConnection::from_stream(
                server_stream,
                DEFAULT_LIMITS.max_frame_bytes,
            ));
        });
        let client = Client::from_connection(
            FrameConnection::from_stream(client_stream, DEFAULT_LIMITS.max_frame_bytes),
            PathBuf::from("/tmp/relayable-test.sock"),
            ClientConfig::new("test"),
        )
        .unwrap();
        (client, server)
    }

    #[test]
    fn handshake_captures_hello_and_limits() {
        let (client, server) = client_connected();
        assert_eq!(client.server_hello().capabilities, vec!["test".to_string()]);
        assert_eq!(client.limits().max_frame_bytes, 4096);
        assert_eq!(client.socket_path(), Path::new("/tmp/relayable-test.sock"));
        drop(client);
        let _ = server.join();
    }

    #[test]
    fn list_sessions_returns_the_server_list() {
        let (mut client, server) = client_connected();
        let sessions = client.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].label.as_deref(), Some("demo"));
        drop(client);
        let _ = server.join();
    }

    /// A real listening Unix socket whose peer completes the handshake and then
    /// never answers a control RPC must not hang the caller: `list_sessions`
    /// must return [`ClientError::Timeout`] inside a bounded window. The
    /// production deadline is shortened in-test so the assertion is fast.
    #[test]
    fn control_rpc_times_out_when_the_server_never_replies() {
        let dir = std::env::temp_dir().join(format!(
            "relayable-client-rpc-timeout-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("relayable.sock");
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).unwrap();

        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut conn = FrameConnection::from_stream(stream, DEFAULT_LIMITS.max_frame_bytes);

            // Complete the handshake so the client gets past `connect`.
            let handshake = conn.recv().unwrap().unwrap();
            let request_id = handshake.request_id.unwrap();
            conn.send(&response_for(
                request_id,
                ResponseResult::Hello(ServerHello {
                    protocol_version: PROTOCOL_VERSION.to_string(),
                    capabilities: Vec::new(),
                    limits: None,
                }),
            ))
            .unwrap();

            // Consume the next request but deliberately never answer it, and
            // outlive the client's shortened deadline before closing.
            let request = conn.recv().unwrap().unwrap();
            assert!(matches!(request.body, Some(Body::Request(_))));
            thread::sleep(Duration::from_secs(2));
            drop(conn);
        });

        let mut client = Client::connect(&path, ClientConfig::new("test")).unwrap();
        client.control_rpc_timeout = Duration::from_millis(250);

        let started = Instant::now();
        let err = client.list_sessions().unwrap_err();
        let elapsed = started.elapsed();

        assert!(matches!(err, ClientError::Timeout), "got {err:?}");
        assert!(
            elapsed >= Duration::from_millis(200),
            "returned before the deadline: {elapsed:?}"
        );
        assert!(
            elapsed < Duration::from_secs(2),
            "did not respect the deadline: {elapsed:?}"
        );

        drop(client);
        let _ = server.join();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn server_error_code_is_exposed() {
        let err = ClientError::Server(Error {
            code: ErrorCode::UnknownEntity as i32,
            message: "nope".to_string(),
            details: std::collections::HashMap::new(),
            retryable: false,
        });
        assert_eq!(err.server_error_code(), Some(ErrorCode::UnknownEntity));
        assert!(err.to_string().contains("unknown_entity"));
        assert!(ClientError::Timeout.server_error_code().is_none());
    }

    #[test]
    fn contains_matches_across_chunk_boundaries() {
        assert!(contains(b"hello world", b"lo wo"));
        assert!(!contains(b"hello", b"world"));
        assert!(!contains(b"", b"x"));
    }
}
