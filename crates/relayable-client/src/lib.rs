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
    SessionCreateRequest, SessionListResponse, SessionResumeRequest, SessionSnapshotRequest,
    TerminalInput, TerminalResize,
};
use relayable_protocol::{CONTROL_STREAM_ID, PROTOCOL_VERSION};

/// Protocol types that appear in this crate's public API, re-exported so
/// consumers (`relayable-automation`: ADR-001 `automation -> {model, client}`)
/// do not need a direct `relayable-protocol` dependency.
pub use relayable_protocol::transport::TransportError;
pub use relayable_protocol::v1::{
    AttachMode, BlockAttachResponse, Error, ErrorCode, Event, Limits, ServerHello, SessionRecord,
    SessionResumeResponse, SessionSnapshotResponse, SessionSummary, TerminalSnapshot,
};

/// Maximum time a control RPC (and the initial handshake) waits for its
/// response before failing with [`ClientError::Timeout`].
pub const CONTROL_RPC_TIMEOUT: Duration = Duration::from_secs(10);

/// Maximum number of received `Event` frames buffered inside a [`Client`] when
/// the caller has not drained them yet.
///
/// CORE-002 events are control-class frames the client may receive at any time;
/// [`Client::drain_events`], [`Client::wait_for_event`], and
/// [`Client::collect_events`] surface them. The buffer is bounded so a client
/// that never drains cannot grow without limit: when full, the **oldest**
/// buffered event is evicted (FIFO) before the newest is stored. Evicted events
/// are not lost from the server's retained event log, so a later
/// [`Client::resume_session`] from the last applied cursor can re-fetch them.
/// Terminal bytes never live in this buffer and are never evicted by this bound.
pub const MAX_PENDING_EVENTS: usize = 4096;

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
    pending_events: VecDeque<Event>,
    /// When this client last actually sent a `Heartbeat`.
    ///
    /// Persisted across calls on purpose: repeated short waits must accumulate
    /// toward the outbound cadence instead of each call resetting it. Only
    /// [`Client::heartbeat_if_due`] advances it, and only after a heartbeat is
    /// sent.
    last_heartbeat_sent: Instant,
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

    /// Resume a session from a server-wide event-log cursor (`session.resume`).
    ///
    /// `None` sends `last_sequence_applied = 0`, the frozen "no cursor"
    /// sentinel (`protocol/compatibility/core-002-method-set.md` §A); the server
    /// then returns a fresh snapshot rather than a false replay. The response
    /// body is returned unchanged: the client does not reinterpret
    /// `snapshot_required` or reorder the returned events. Live events that
    /// follow arrive as `Event` frames and are surfaced by
    /// [`Client::wait_for_event`], [`Client::collect_events`], or
    /// [`Client::drain_events`].
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn resume_session(
        &mut self,
        session: &SessionId,
        last_sequence_applied: Option<u64>,
    ) -> Result<SessionResumeResponse, ClientError> {
        let result = self.request_operation(
            methods::SESSION_RESUME,
            Operation::SessionResume(SessionResumeRequest {
                session_id: ids::session_id_to_wire(session),
                last_sequence_applied: last_sequence_applied.unwrap_or(0),
            }),
        )?;
        match result {
            ResponseResult::SessionResume(response) => Ok(response),
            _ => Err(unexpected("session.resume", &result)),
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
                takeover: false,
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

    /// Attach to a terminal block (`block.attach`) without taking over an
    /// existing writer lease (`takeover = false`).
    ///
    /// `session` is accepted for call-site parity with
    /// [`Client::attach_with_takeover`]; `block.attach` itself is addressed by
    /// `block` plus the acting client identity on the local socket
    /// (`protocol/compatibility/core-002-method-set.md` §E).
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn attach(
        &mut self,
        session: &SessionId,
        block: &BlockId,
        mode: AttachMode,
    ) -> Result<BlockAttachResponse, ClientError> {
        self.attach_with_takeover(session, block, mode, false)
    }

    /// Attach to a terminal block (`block.attach`), optionally taking over the
    /// single writer lease from the current holder (CORE-002, contract §5).
    ///
    /// `takeover = false` is exactly [`Client::attach`] / [`Client::attach_block`]
    /// (a second writer is rejected with `input_not_owned`). `takeover = true`
    /// asks the server to transfer the lease and emit `writer_lease_changed`.
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn attach_with_takeover(
        &mut self,
        _session: &SessionId,
        block: &BlockId,
        mode: AttachMode,
        takeover: bool,
    ) -> Result<BlockAttachResponse, ClientError> {
        let result = self.request_operation(
            methods::BLOCK_ATTACH,
            Operation::BlockAttach(BlockAttachRequest {
                block_id: ids::block_id_to_wire(block),
                mode: mode as i32,
                takeover,
            }),
        )?;
        match result {
            ResponseResult::BlockAttach(response) => Ok(response),
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

    /// Read the next `TerminalOutput` frame for `stream_id` and return its
    /// bytes, or `Ok(None)` when `timeout` elapses with no such frame.
    ///
    /// Each returned chunk is acknowledged exactly once through the same
    /// `FlowControl` credit path as [`Client::wait_for_output`], so a caller
    /// that keeps consuming keeps the server's credit window refilled. Unlike
    /// [`Client::wait_for_output`], an expired deadline is **not** an error:
    /// bytes are acknowledged only when they are returned in the same call, so
    /// a timeout never acknowledges and then discards output. A zero `timeout`
    /// never waits on the deadline and performs no socket read: it serves any
    /// already-buffered target bytes and otherwise returns `Ok(None)` promptly.
    /// The only I/O it may still perform is the pending ack or heartbeat write.
    /// Frames for other streams, events, and other control frames are
    /// retained in arrival order and never dropped; events stay reachable
    /// through [`Client::drain_events`], [`Client::wait_for_event`], and
    /// [`Client::collect_events`].
    ///
    /// # Errors
    ///
    /// [`ClientError::Disconnected`] if the peer closes the connection, and any
    /// other [`ClientError`] from the transport or from acknowledging output.
    pub fn next_output(
        &mut self,
        stream_id: u64,
        timeout: Duration,
    ) -> Result<Option<Vec<u8>>, ClientError> {
        let heartbeat = self.heartbeat_interval();
        let start = Instant::now();
        let deadline = start.checked_add(timeout).unwrap_or(start);

        // Serve anything already buffered before touching the socket, at any
        // timeout. The scan is bounded by the queue length and leaves every
        // frame it does not consume in its original arrival order (a bare
        // rotation would move frames popped before a match behind frames that
        // followed it), so repeated calls stay lossless and ordered for every
        // stream.
        let buffered = self.pending.len();
        let mut deferred: VecDeque<Envelope> = VecDeque::new();
        for _ in 0..buffered {
            let Some(envelope) = self.pending.pop_front() else {
                break;
            };
            match envelope.body {
                Some(Body::TerminalData(output)) if output.stream_id == stream_id => {
                    for env in deferred.into_iter().rev() {
                        self.pending.push_front(env);
                    }
                    let len = output.bytes.len() as u64;
                    self.send_flow_control(stream_id, len)?;
                    return Ok(Some(output.bytes));
                }
                Some(Body::Event(event)) => self.enqueue_event(event),
                _ => deferred.push_back(envelope),
            }
        }
        for env in deferred.into_iter().rev() {
            self.pending.push_front(env);
        }

        loop {
            self.heartbeat_if_due(heartbeat)?;
            if Instant::now() >= deadline {
                return Ok(None);
            }
            let read_timeout = self.next_read_timeout(deadline, heartbeat);
            if read_timeout.is_zero() {
                continue;
            }
            self.conn
                .set_read_timeout(Some(read_timeout))
                .map_err(|err| ClientError::Transport(TransportError::Io(err)))?;
            let envelope = match self.recv_network() {
                Ok(envelope) => envelope,
                Err(ClientError::Timeout) => continue,
                Err(err) => return Err(err),
            };
            match envelope.body {
                Some(Body::TerminalData(output)) if output.stream_id == stream_id => {
                    let len = output.bytes.len() as u64;
                    self.send_flow_control(stream_id, len)?;
                    return Ok(Some(output.bytes));
                }
                Some(Body::Event(event)) => self.enqueue_event(event),
                _ => self.pending.push_back(envelope),
            }
        }
    }

    /// Return every `Event` frame already buffered inside the client, in
    /// arrival order, without reading the socket.
    ///
    /// Events are buffered by [`Client::wait_for_event`],
    /// [`Client::collect_events`], and whenever a control RPC drains a frame it
    /// is not waiting for. The buffer is bounded by [`MAX_PENDING_EVENTS`]; see
    /// that constant for the overflow policy.
    #[must_use]
    pub fn drain_events(&mut self) -> Vec<Event> {
        self.pending_events.drain(..).collect()
    }

    /// Block until the next `Event` frame arrives, or `timeout` elapses.
    ///
    /// Returns `Ok(None)` if no event arrived before the deadline. While
    /// waiting the method sends a `Heartbeat` whenever its own send cadence
    /// ([`Limits::heartbeat_interval_millis`], measured from the last heartbeat
    /// it *sent*) elapses. Inbound frames — including the server's own
    /// `Heartbeat` frames — never reset that timer, and the blocking read is
    /// capped so the loop wakes often enough to send on schedule. Non-event
    /// frames are retained (moved to the internal pending queue), never
    /// dropped.
    ///
    /// # Errors
    ///
    /// [`ClientError::Disconnected`] if the peer closes the connection, and any
    /// other [`ClientError`] from the transport.
    pub fn wait_for_event(&mut self, timeout: Duration) -> Result<Option<Event>, ClientError> {
        if let Some(event) = self.pending_events.pop_front() {
            return Ok(Some(event));
        }
        let heartbeat = self.heartbeat_interval();
        let start = Instant::now();
        let deadline = start.checked_add(timeout).unwrap_or(start);
        loop {
            if Instant::now() >= deadline {
                return Ok(None);
            }
            self.heartbeat_if_due(heartbeat)?;
            if Instant::now() >= deadline {
                return Ok(None);
            }
            let read_timeout = self.next_read_timeout(deadline, heartbeat);
            if read_timeout.is_zero() {
                continue;
            }
            if let Some(envelope) = self.read_event_frame(read_timeout)? {
                match envelope.body {
                    Some(Body::Event(event)) => return Ok(Some(event)),
                    _ => self.pending.push_back(envelope),
                }
            }
        }
    }

    /// Read `Event` frames until `timeout` elapses, returning every event
    /// received in that window in arrival order.
    ///
    /// Events already buffered inside the client are returned first. The method
    /// heartbeats and retains non-event frames exactly like
    /// [`Client::wait_for_event`]: the outbound cadence depends only on the last
    /// heartbeat this client sent, never on inbound frames. An empty vector
    /// means the deadline expired with no event, not an error.
    ///
    /// # Errors
    ///
    /// [`ClientError::Disconnected`] if the peer closes the connection, and any
    /// other [`ClientError`] from the transport.
    pub fn collect_events(&mut self, timeout: Duration) -> Result<Vec<Event>, ClientError> {
        let mut events: Vec<Event> = self.pending_events.drain(..).collect();
        let heartbeat = self.heartbeat_interval();
        let start = Instant::now();
        let deadline = start.checked_add(timeout).unwrap_or(start);
        loop {
            if Instant::now() >= deadline {
                return Ok(events);
            }
            self.heartbeat_if_due(heartbeat)?;
            if Instant::now() >= deadline {
                return Ok(events);
            }
            let read_timeout = self.next_read_timeout(deadline, heartbeat);
            if read_timeout.is_zero() {
                continue;
            }
            if let Some(envelope) = self.read_event_frame(read_timeout)? {
                match envelope.body {
                    Some(Body::Event(event)) => events.push(event),
                    _ => self.pending.push_back(envelope),
                }
            }
        }
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
            pending_events: VecDeque::new(),
            last_heartbeat_sent: Instant::now(),
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
                Some(Body::Event(event)) => self.enqueue_event(event),
                Some(Body::TerminalData(_) | Body::FlowControl(_) | Body::Heartbeat(_)) => {
                    deferred.push_back(envelope);
                }
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
        let heartbeat = self.heartbeat_interval();
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

            self.heartbeat_if_due(heartbeat)?;
            if Instant::now() >= deadline {
                return Err(ClientError::Timeout);
            }
            let read_timeout = self.next_read_timeout(deadline, heartbeat);
            if read_timeout.is_zero() {
                continue;
            }
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
                // A read-deadline expiry is not an error here: the loop top
                // sends a heartbeat if the send cadence elapsed, or returns
                // `Timeout` once the caller's deadline passes.
                None => {}
                Some(envelope) => match envelope.body {
                    Some(Body::TerminalData(output)) if output.stream_id == stream_id => {
                        let len = output.bytes.len() as u64;
                        accumulated.extend_from_slice(&output.bytes);
                        if len > 0 {
                            self.send_flow_control(stream_id, len)?;
                        }
                    }
                    Some(Body::Event(event)) => self.enqueue_event(event),
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

    /// Store a received event in the bounded pending list, evicting the oldest
    /// buffered event when full (see [`MAX_PENDING_EVENTS`]).
    fn enqueue_event(&mut self, event: Event) {
        if self.pending_events.len() >= MAX_PENDING_EVENTS {
            self.pending_events.pop_front();
        }
        self.pending_events.push_back(event);
    }

    fn heartbeat_interval(&self) -> Duration {
        Duration::from_millis(u64::from(self.limits.heartbeat_interval_millis.max(1)))
    }

    /// Send a heartbeat if the outbound cadence has elapsed since this client
    /// last sent one, advancing the shared baseline only when a heartbeat is
    /// actually sent.
    ///
    /// The baseline lives on the `Client` so it survives across calls: repeated
    /// short waits accumulate toward the cadence instead of resetting it, and a
    /// heartbeat is never sent more often than once per cadence. Inbound frames
    /// never feed this timer.
    fn heartbeat_if_due(&mut self, cadence: Duration) -> Result<(), ClientError> {
        if Instant::now().duration_since(self.last_heartbeat_sent) >= cadence {
            self.send_heartbeat()?;
            self.last_heartbeat_sent = Instant::now();
        }
        Ok(())
    }

    /// The blocking-read cap that wakes the caller when either its own deadline
    /// or the next outbound heartbeat falls first.
    fn next_read_timeout(&self, deadline: Instant, cadence: Duration) -> Duration {
        let now = Instant::now();
        let remaining = deadline.saturating_duration_since(now);
        let until_heartbeat = cadence.saturating_sub(now.duration_since(self.last_heartbeat_sent));
        remaining.min(until_heartbeat)
    }

    /// Read one frame for the event-reading methods. The caller owns the
    /// outbound heartbeat cadence; an expired read deadline is reported as
    /// `Ok(None)` so the caller can re-check its timers rather than mistaking it
    /// for a transport error.
    fn read_event_frame(
        &mut self,
        read_timeout: Duration,
    ) -> Result<Option<Envelope>, ClientError> {
        self.conn
            .set_read_timeout(Some(read_timeout))
            .map_err(|err| ClientError::Transport(TransportError::Io(err)))?;
        match self.recv_network() {
            Ok(envelope) => Ok(Some(envelope)),
            Err(ClientError::Timeout) => Ok(None),
            Err(err) => Err(err),
        }
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
        ResponseResult::SessionResume(_) => "session_resume",
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
    use relayable_protocol::v1::{
        event, ClientConnectionStateChanged, SessionState, TerminalOutput,
    };
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::sync::mpsc;
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

    /// Receive one envelope for a stub server, treating a closed connection or a
    /// torn-down socket as "the stub is finished" instead of panicking.
    ///
    /// The tests assert on the *client* side, so a stub that loses a teardown
    /// race with a dropping client should just stop. `Ok(None)` is EOF; a
    /// timeout/reset `Err` is the same teardown condition observed from the read
    /// side. Returns `None` in either case.
    fn recv_or_finished(conn: &mut FrameConnection) -> Option<Envelope> {
        match conn.recv() {
            Ok(Some(envelope)) => Some(envelope),
            Ok(None) | Err(_) => None,
        }
    }

    fn fake_server(mut conn: FrameConnection) {
        conn.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let Some(handshake) = recv_or_finished(&mut conn) else {
            return;
        };
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

        // A client that only exercises the handshake (e.g.
        // `handshake_captures_hello_and_limits`) drops without ever sending a
        // `session.list`; its EOF must end the stub cleanly, not panic.
        let Some(list) = recv_or_finished(&mut conn) else {
            return;
        };
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

    /// Complete the client handshake on a server-side connection, advertising
    /// `limits`. Returns `false` (without panicking) if the client has already
    /// gone away, so the caller skips its script.
    fn complete_handshake(conn: &mut FrameConnection, limits: Limits) -> bool {
        conn.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let Some(handshake) = recv_or_finished(conn) else {
            return false;
        };
        let request_id = handshake.request_id.unwrap();
        assert!(matches!(handshake.body, Some(Body::Handshake(_))));
        conn.send(&response_for(
            request_id,
            ResponseResult::Hello(ServerHello {
                protocol_version: PROTOCOL_VERSION.to_string(),
                capabilities: vec!["test".to_string()],
                limits: Some(limits),
            }),
        ))
        .unwrap();
        true
    }

    /// Spawn a paired server thread that completes the handshake and then runs
    /// `script` against the connection.
    fn spawn_scripted_server<F>(script: F) -> (Client, thread::JoinHandle<()>)
    where
        F: FnOnce(&mut FrameConnection) + Send + 'static,
    {
        let limits = Limits {
            max_frame_bytes: 4096,
            ..DEFAULT_LIMITS
        };
        spawn_scripted_server_with_limits(limits, script)
    }

    /// Like [`spawn_scripted_server`], but advertises `limits` (so a test can
    /// pick a short heartbeat cadence).
    fn spawn_scripted_server_with_limits<F>(
        limits: Limits,
        script: F,
    ) -> (Client, thread::JoinHandle<()>)
    where
        F: FnOnce(&mut FrameConnection) + Send + 'static,
    {
        let (client_stream, server_stream) = UnixStream::pair().unwrap();
        let server = thread::spawn(move || {
            let mut conn =
                FrameConnection::from_stream(server_stream, DEFAULT_LIMITS.max_frame_bytes);
            if complete_handshake(&mut conn, limits) {
                script(&mut conn);
            }
        });
        let client = Client::from_connection(
            FrameConnection::from_stream(client_stream, DEFAULT_LIMITS.max_frame_bytes),
            PathBuf::from("/tmp/relayable-test.sock"),
            ClientConfig::new("test"),
        )
        .unwrap();
        (client, server)
    }

    fn block_attach_response() -> BlockAttachResponse {
        BlockAttachResponse {
            stream_id: 7,
            mode: AttachMode::Writer as i32,
            writer_lease: true,
            snapshot: Some(TerminalSnapshot::default()),
        }
    }

    fn block_attach_of(envelope: Envelope) -> BlockAttachRequest {
        match envelope.body {
            Some(Body::Request(Request {
                operation: Some(Operation::BlockAttach(request)),
                ..
            })) => request,
            other => panic!("expected a block.attach request, got {other:?}"),
        }
    }

    fn session_resume_of(envelope: Envelope) -> SessionResumeRequest {
        match envelope.body {
            Some(Body::Request(Request {
                operation: Some(Operation::SessionResume(request)),
                ..
            })) => request,
            other => panic!("expected a session.resume request, got {other:?}"),
        }
    }

    fn event_envelope(sequence: u64) -> Envelope {
        Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: None,
            stream_id: CONTROL_STREAM_ID,
            sequence: Some(sequence),
            body: Some(Body::Event(Event {
                sequence,
                revision: 0,
                session_id: String::new(),
                event: Some(event::Event::ClientConnectionStateChanged(
                    ClientConnectionStateChanged {
                        client_id: "client:test".to_string(),
                        connection_state: "ready".to_string(),
                    },
                )),
            })),
            ..Envelope::default()
        }
    }

    /// Spawn a server that captures exactly one `block.attach` request and
    /// answers it with [`block_attach_response`].
    fn attach_server() -> (Client, thread::JoinHandle<()>, mpsc::Receiver<Envelope>) {
        let (tx, rx) = mpsc::channel();
        let (client, server) = spawn_scripted_server(move |conn| {
            let Some(request) = recv_or_finished(conn) else {
                return;
            };
            let request_id = request.request_id.unwrap();
            tx.send(request).unwrap();
            conn.send(&response_for(
                request_id,
                ResponseResult::BlockAttach(block_attach_response()),
            ))
            .unwrap();
        });
        (client, server, rx)
    }

    /// Spawn a server that captures exactly one `session.resume` request and
    /// answers it with a snapshot-required response.
    fn resume_server() -> (Client, thread::JoinHandle<()>, mpsc::Receiver<Envelope>) {
        let (tx, rx) = mpsc::channel();
        let (client, server) = spawn_scripted_server(move |conn| {
            let Some(request) = recv_or_finished(conn) else {
                return;
            };
            let request_id = request.request_id.unwrap();
            tx.send(request).unwrap();
            conn.send(&response_for(
                request_id,
                ResponseResult::SessionResume(SessionResumeResponse {
                    snapshot_required: true,
                    snapshot_required_reason: "no_cursor".to_string(),
                    ..SessionResumeResponse::default()
                }),
            ))
            .unwrap();
        });
        (client, server, rx)
    }

    /// A server-to-client `TerminalOutput` frame exactly as the server builds it
    /// (`crates/relayable-server/src/connection.rs::terminal_output_envelope`).
    fn terminal_data(stream_id: u64, stream_seq: u64, bytes: &[u8]) -> Envelope {
        Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: None,
            stream_id,
            body: Some(Body::TerminalData(TerminalOutput {
                stream_id,
                stream_seq,
                bytes: bytes.to_vec(),
            })),
            ..Envelope::default()
        }
    }

    /// A framed transport read timeout, as opposed to a real close/teardown.
    fn is_transport_timeout(err: &TransportError) -> bool {
        match err {
            TransportError::Io(io) => is_timeout(io),
            TransportError::Codec(CodecError::Io(io)) => is_timeout(io),
            TransportError::Codec(_) => false,
        }
    }

    /// Spawn a server that completes the handshake, answers exactly one
    /// `block.attach` with [`block_attach_response`], sends `script` frames, and
    /// then forwards every inbound envelope into an `mpsc` channel until the
    /// client closes the connection or `drain` elapses (so a misbehaving test
    /// cannot hang the suite).
    fn spawn_output_server(
        script: Vec<Envelope>,
        drain: Duration,
    ) -> (Client, thread::JoinHandle<()>, mpsc::Receiver<Envelope>) {
        let (tx, rx) = mpsc::channel();
        let (client, server) = spawn_scripted_server(move |conn| {
            let Some(request) = recv_or_finished(conn) else {
                return;
            };
            let request_id = request.request_id.unwrap();
            conn.send(&response_for(
                request_id,
                ResponseResult::BlockAttach(block_attach_response()),
            ))
            .unwrap();
            for envelope in &script {
                if conn.send(envelope).is_err() {
                    return;
                }
            }
            let deadline = Instant::now() + drain;
            while Instant::now() < deadline {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if conn
                    .set_read_timeout(Some(remaining.min(Duration::from_millis(50))))
                    .is_err()
                {
                    break;
                }
                match conn.recv() {
                    Ok(Some(envelope)) => {
                        let _ = tx.send(envelope);
                    }
                    Ok(None) => break,
                    Err(err) if is_transport_timeout(&err) => {}
                    Err(_) => break,
                }
            }
        });
        (client, server, rx)
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

    #[test]
    fn attach_sets_takeover_false_and_attach_with_takeover_sets_true() {
        let session = SessionId::new();
        let block = BlockId::new();

        let (mut client, server, rx) = attach_server();
        let response = client.attach(&session, &block, AttachMode::Writer).unwrap();
        assert_eq!(response.stream_id, 7);
        assert_eq!(response.mode, AttachMode::Writer as i32);
        assert!(response.writer_lease);
        let request = block_attach_of(rx.recv().unwrap());
        assert!(!request.takeover, "plain attach must not take over");
        assert_eq!(request.block_id, ids::block_id_to_wire(&block));
        drop(client);
        let _ = server.join();

        let (mut client, server, rx) = attach_server();
        let response = client
            .attach_with_takeover(&session, &block, AttachMode::Writer, true)
            .unwrap();
        assert_eq!(response.stream_id, 7);
        let request = block_attach_of(rx.recv().unwrap());
        assert!(request.takeover, "attach_with_takeover must set takeover");
        assert_eq!(request.block_id, ids::block_id_to_wire(&block));
        drop(client);
        let _ = server.join();
    }

    #[test]
    fn resume_session_encodes_the_last_applied_sequence() {
        let session = SessionId::new();

        let (mut client, server, rx) = resume_server();
        client.resume_session(&session, None).unwrap();
        let request = session_resume_of(rx.recv().unwrap());
        assert_eq!(
            request.last_sequence_applied, 0,
            "None is the no-cursor sentinel"
        );
        assert_eq!(request.session_id, ids::session_id_to_wire(&session));
        drop(client);
        let _ = server.join();

        let (mut client, server, rx) = resume_server();
        client.resume_session(&session, Some(41)).unwrap();
        let request = session_resume_of(rx.recv().unwrap());
        assert_eq!(request.last_sequence_applied, 41);
        drop(client);
        let _ = server.join();
    }

    #[test]
    fn wait_for_event_surfaces_a_live_event() {
        let (mut client, server) = spawn_scripted_server(|conn| {
            conn.send(&event_envelope(7)).unwrap();
            // Keep the connection open until the client is done.
            let _ = conn.recv();
        });

        let event = client
            .wait_for_event(Duration::from_secs(2))
            .unwrap()
            .expect("an event should arrive");
        assert_eq!(event.sequence, 7);
        assert!(matches!(
            event.event,
            Some(event::Event::ClientConnectionStateChanged(_))
        ));
        assert!(client.drain_events().is_empty());

        drop(client);
        let _ = server.join();
    }

    #[test]
    fn events_are_retained_across_control_rpcs_and_reachable_by_drain() {
        let (mut client, server) = spawn_scripted_server(|conn| {
            let request = conn.recv().unwrap().unwrap();
            let request_id = request.request_id.unwrap();
            // The event arrives between the request and its response: it must be
            // buffered without corrupting the pending control RPC.
            conn.send(&event_envelope(9)).unwrap();
            conn.send(&response_for(
                request_id,
                ResponseResult::SessionList(SessionListResponse {
                    sessions: vec![SessionSummary {
                        session_id: "session:01a10cf3-2655-7e48-9b9c-43dd6612bef5".to_string(),
                        label: Some("demo".to_string()),
                        state: SessionState::Active as i32,
                        revision: 1,
                    }],
                    sequence: 9,
                }),
            ))
            .unwrap();
        });

        let sessions = client.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].label.as_deref(), Some("demo"));

        let events = client.drain_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sequence, 9);
        assert!(client.drain_events().is_empty());

        drop(client);
        let _ = server.join();
    }

    #[test]
    fn events_buffered_during_a_rpc_can_be_waited_for_later() {
        let (mut client, server) = spawn_scripted_server(|conn| {
            conn.send(&event_envelope(11)).unwrap();
            let request = conn.recv().unwrap().unwrap();
            let request_id = request.request_id.unwrap();
            conn.send(&response_for(
                request_id,
                ResponseResult::SessionList(SessionListResponse {
                    sessions: Vec::new(),
                    sequence: 11,
                }),
            ))
            .unwrap();
            // Keep the connection open until the client is done.
            let _ = conn.recv();
        });

        // The list RPC drains the event frame into the bounded pending list.
        assert!(client.list_sessions().unwrap().is_empty());
        let event = client
            .wait_for_event(Duration::from_millis(50))
            .unwrap()
            .expect("buffered event");
        assert_eq!(event.sequence, 11);
        assert!(client
            .wait_for_event(Duration::from_millis(10))
            .unwrap()
            .is_none());

        drop(client);
        let _ = server.join();
    }

    /// Receiving frames must never reset the outbound heartbeat deadline: a
    /// client blocked in one long `collect_events` call while the server streams
    /// events every few milliseconds (so the client's blocking read always
    /// returns a frame and never reaches a read timeout) must still send its own
    /// `Heartbeat` frames on its advertised cadence. With the previous
    /// "send-on-read-timeout" logic this produced zero heartbeats.
    #[test]
    fn inbound_frames_do_not_starve_the_outbound_heartbeat() {
        const CADENCE_MS: u64 = 40;
        let limits = Limits {
            max_frame_bytes: 4096,
            heartbeat_interval_millis: CADENCE_MS as u32,
            ..DEFAULT_LIMITS
        };
        let (hb_tx, hb_rx) = mpsc::channel();
        let (mut client, server) = spawn_scripted_server_with_limits(limits, move |conn| {
            // Stream events faster than the heartbeat cadence, for longer than
            // the client's collect window, so inbound frames keep the client's
            // blocking read busy the whole time.
            let start = Instant::now();
            let mut sequence = 0u64;
            while start.elapsed() < Duration::from_millis(320) {
                sequence += 1;
                conn.send(&event_envelope(sequence)).unwrap();
                thread::sleep(Duration::from_millis(5));
            }
            // The client has returned from `collect_events` by now; drain the
            // heartbeats it sent while the window was open.
            conn.set_read_timeout(Some(Duration::from_millis(100)))
                .unwrap();
            let mut heartbeats = 0usize;
            let drain_deadline = Instant::now() + Duration::from_millis(300);
            while Instant::now() < drain_deadline {
                match conn.recv() {
                    Ok(Some(envelope)) => {
                        if matches!(envelope.body, Some(Body::Heartbeat(_))) {
                            heartbeats += 1;
                        }
                    }
                    Ok(None) | Err(_) => break,
                }
            }
            hb_tx.send(heartbeats).unwrap();
        });

        let events = client.collect_events(Duration::from_millis(300)).unwrap();
        assert!(!events.is_empty(), "the server streamed events");
        let heartbeats = hb_rx.recv().unwrap();
        assert!(
            heartbeats >= 1,
            "expected an outbound heartbeat while inbound frames kept arriving, got {heartbeats}"
        );

        drop(client);
        let _ = server.join();
    }

    /// The outbound baseline must survive **across calls**: a pump loop that
    /// waits in short repeated slices spanning the cadence still emits
    /// heartbeats — with a per-call baseline it emits none and is idle-dropped —
    /// yet never emits faster than the cadence.
    #[test]
    fn short_repeated_waits_accumulate_toward_the_heartbeat_cadence() {
        const CADENCE_MS: u64 = 40;
        let limits = Limits {
            max_frame_bytes: 4096,
            heartbeat_interval_millis: CADENCE_MS as u32,
            ..DEFAULT_LIMITS
        };
        let (hb_tx, hb_rx) = mpsc::channel();
        let (mut client, server) = spawn_scripted_server_with_limits(limits, move |conn| {
            let start = Instant::now();
            let mut sequence = 0u64;
            while start.elapsed() < Duration::from_millis(320) {
                sequence += 1;
                conn.send(&event_envelope(sequence)).unwrap();
                thread::sleep(Duration::from_millis(5));
            }
            conn.set_read_timeout(Some(Duration::from_millis(100)))
                .unwrap();
            let mut heartbeats = 0usize;
            let drain_deadline = Instant::now() + Duration::from_millis(300);
            while Instant::now() < drain_deadline {
                match conn.recv() {
                    Ok(Some(envelope)) => {
                        if matches!(envelope.body, Some(Body::Heartbeat(_))) {
                            heartbeats += 1;
                        }
                    }
                    Ok(None) | Err(_) => break,
                }
            }
            hb_tx.send(heartbeats).unwrap();
        });

        // Pump in 10 ms slices for ~300 ms, the way a CLI/output pump would.
        let pump_deadline = Instant::now() + Duration::from_millis(300);
        let mut events_seen = 0usize;
        while Instant::now() < pump_deadline {
            events_seen += client
                .collect_events(Duration::from_millis(10))
                .unwrap()
                .len();
        }
        assert!(events_seen > 0, "the server streamed events");

        let heartbeats = hb_rx.recv().unwrap();
        assert!(
            heartbeats >= 1,
            "short repeated waits must accumulate toward the cadence, got {heartbeats}"
        );
        // ~300 ms / 40 ms cadence is at most ~8 sends; allow slack for scheduling
        // but reject per-call flooding (a 10 ms slice would send ~30 times).
        assert!(
            heartbeats <= 12,
            "heartbeats must not be sent faster than the cadence, got {heartbeats}"
        );

        drop(client);
        let _ = server.join();
    }

    /// CLIENT-API-001 rule 1: repeated `next_output` calls return the target
    /// stream's frames exactly once each, in arrival order, with no byte loss.
    #[test]
    fn next_output_returns_successive_target_frames_without_loss() {
        let target = 7u64;
        let (mut client, server, rx) = spawn_output_server(
            vec![
                terminal_data(target, 1, b"first-"),
                terminal_data(target, 2, b"second"),
            ],
            Duration::from_millis(500),
        );
        client
            .attach_block(&BlockId::new(), AttachMode::Writer)
            .unwrap();

        let first = client
            .next_output(target, Duration::from_millis(500))
            .unwrap()
            .expect("first target frame");
        let second = client
            .next_output(target, Duration::from_millis(500))
            .unwrap()
            .expect("second target frame");
        assert_eq!(first, b"first-");
        assert_eq!(second, b"second");
        let mut joined = first;
        joined.extend_from_slice(&second);
        assert_eq!(joined, b"first-second", "no byte may be lost between calls");

        drop(client);
        let _ = server.join();
        drop(rx);
    }

    /// CLIENT-API-001 rule 3: a normal timeout is `Ok(None)`, never
    /// [`ClientError::Timeout`].
    #[test]
    fn next_output_returns_none_on_timeout() {
        let target = 7u64;
        let (mut client, server, rx) = spawn_output_server(Vec::new(), Duration::from_millis(300));
        client
            .attach_block(&BlockId::new(), AttachMode::Writer)
            .unwrap();

        let result = client
            .next_output(target, Duration::from_millis(50))
            .expect("a timeout is not an error");
        assert!(result.is_none(), "expected Ok(None), got {result:?}");

        drop(client);
        let _ = server.join();
        drop(rx);
    }

    /// CLIENT-API-001 rule 5: `Duration::ZERO` serves frames already buffered in
    /// `pending` (deferred by an earlier call) and returns `Ok(None)` promptly
    /// with an empty queue, never reading the socket.
    #[test]
    fn zero_timeout_serves_buffered_frames_without_blocking() {
        let target = 7u64;
        let (mut client, server, rx) = spawn_output_server(
            vec![
                terminal_data(target, 1, b"buffered-bytes"),
                event_envelope(21),
            ],
            Duration::from_millis(500),
        );
        client
            .attach_block(&BlockId::new(), AttachMode::Writer)
            .unwrap();

        // `wait_for_event` reads the target data frame first (deferring it into
        // `pending`) and then the event, so the bytes are only reachable through
        // public API.
        let event = client
            .wait_for_event(Duration::from_millis(500))
            .unwrap()
            .expect("buffered event");
        assert_eq!(event.sequence, 21);

        let started = Instant::now();
        let bytes = client
            .next_output(target, Duration::ZERO)
            .unwrap()
            .expect("buffered target frame");
        assert_eq!(bytes, b"buffered-bytes");
        assert!(
            started.elapsed() < Duration::from_millis(250),
            "a zero-timeout drain must not block: {:?}",
            started.elapsed()
        );

        // With an empty queue the same call returns promptly without a read.
        let started = Instant::now();
        let nothing = client.next_output(target, Duration::ZERO).unwrap();
        assert!(nothing.is_none());
        assert!(
            started.elapsed() < Duration::from_millis(250),
            "an empty zero-timeout call must not block: {:?}",
            started.elapsed()
        );

        drop(client);
        let _ = server.join();
        drop(rx);
    }

    /// CLIENT-API-001 rule 2: every returned chunk is acknowledged exactly once
    /// with `ack_bytes == chunk.len()`, and the totals agree.
    #[test]
    fn one_flow_control_ack_per_returned_chunk() {
        let target = 7u64;
        let chunks: &[&[u8]] = &[b"aaaa", b"bb", b"cccccc"];
        let script = chunks
            .iter()
            .enumerate()
            .map(|(index, bytes)| terminal_data(target, index as u64 + 1, bytes))
            .collect();
        let (mut client, server, rx) = spawn_output_server(script, Duration::from_millis(500));
        client
            .attach_block(&BlockId::new(), AttachMode::Writer)
            .unwrap();

        let mut returned: Vec<Vec<u8>> = Vec::new();
        for _ in 0..chunks.len() {
            returned.push(
                client
                    .next_output(target, Duration::from_millis(500))
                    .unwrap()
                    .expect("scripted frame"),
            );
        }
        drop(client);

        let mut acks = Vec::new();
        while let Ok(envelope) = rx.recv_timeout(Duration::from_millis(500)) {
            if let Some(Body::FlowControl(flow)) = envelope.body {
                acks.push(flow);
            }
        }
        let _ = server.join();

        assert_eq!(
            acks.len(),
            chunks.len(),
            "exactly one ack per returned chunk"
        );
        for (chunk, ack) in returned.iter().zip(&acks) {
            assert_eq!(ack.stream_id, target);
            assert_eq!(ack.ack_bytes, chunk.len() as u64);
        }
        let returned_total: u64 = returned.iter().map(|chunk| chunk.len() as u64).sum();
        let acked_total: u64 = acks.iter().map(|ack| ack.ack_bytes).sum();
        assert_eq!(acked_total, returned_total);
    }

    /// CLIENT-API-001 rule 2 for a zero-length frame: the protocol layer does
    /// not reject `ack_bytes == 0` (nothing validates a `FlowControl` body; the
    /// server's `add_credit` is a no-op for zero), so the ack is still sent
    /// exactly once and the "one ack per returned chunk" invariant holds.
    #[test]
    fn zero_length_chunk_is_acked_exactly_once() {
        let target = 7u64;
        let (mut client, server, rx) = spawn_output_server(
            vec![terminal_data(target, 1, b"")],
            Duration::from_millis(500),
        );
        client
            .attach_block(&BlockId::new(), AttachMode::Writer)
            .unwrap();

        let chunk = client
            .next_output(target, Duration::from_millis(500))
            .unwrap()
            .expect("zero-length frame");
        assert!(chunk.is_empty());
        drop(client);

        let mut acks = Vec::new();
        while let Ok(envelope) = rx.recv_timeout(Duration::from_millis(500)) {
            if let Some(Body::FlowControl(flow)) = envelope.body {
                acks.push(flow);
            }
        }
        let _ = server.join();
        assert_eq!(acks.len(), 1, "exactly one ack for the empty chunk");
        assert_eq!(acks[0].stream_id, target);
        assert_eq!(acks[0].ack_bytes, 0);
    }

    /// CLIENT-API-001 rules 1 and 7: a non-target `TerminalData` frame is never
    /// acked during a target read and is later returned intact; an `Event` frame
    /// stays retrievable through the event API.
    #[test]
    fn non_target_frames_are_retained_and_not_acked() {
        let target = 7u64;
        let other = 9u64;
        let (mut client, server, rx) = spawn_output_server(
            vec![
                terminal_data(target, 1, b"alpha"),
                terminal_data(other, 1, b"beta"),
                event_envelope(33),
                terminal_data(target, 2, b"gamma"),
            ],
            // Keep the stub alive for the whole test: the last ack writes happen
            // well after the first read, so a loaded machine must not be able to
            // race the stub's drain window and turn an ack into EPIPE.
            Duration::from_secs(2),
        );
        client
            .attach_block(&BlockId::new(), AttachMode::Writer)
            .unwrap();

        let first = client
            .next_output(target, Duration::from_millis(500))
            .unwrap()
            .expect("target frame");
        assert_eq!(first, b"alpha");

        // Only the target stream may be acked so far.
        let mut first_wave = Vec::new();
        while let Ok(envelope) = rx.recv_timeout(Duration::from_millis(300)) {
            if let Some(Body::FlowControl(flow)) = envelope.body {
                first_wave.push(flow);
            }
        }
        assert_eq!(first_wave.len(), 1, "exactly the target frame is acked");
        assert_eq!(first_wave[0].stream_id, target);
        assert_eq!(first_wave[0].ack_bytes, b"alpha".len() as u64);
        assert!(
            first_wave.iter().all(|ack| ack.stream_id != other),
            "the other stream must not be acked during a target read"
        );

        let second = client
            .next_output(target, Duration::from_millis(500))
            .unwrap()
            .expect("second target frame");
        assert_eq!(second, b"gamma");
        assert_eq!(
            client
                .drain_events()
                .iter()
                .map(|event| event.sequence)
                .collect::<Vec<_>>(),
            vec![33],
            "the event frame stays retrievable"
        );

        let other_bytes = client
            .next_output(other, Duration::from_millis(500))
            .unwrap_or_else(|err| {
                panic!(
                    "next_output for the other stream failed (the stub may have exited early): {err:?}"
                )
            })
            .expect("other stream frame");
        assert_eq!(
            other_bytes, b"beta",
            "the other stream's bytes are complete"
        );

        drop(client);
        let _ = server.join();
    }

    /// CLIENT-API-001 rule 8: while `next_output` waits for a delayed frame it
    /// keeps the outbound heartbeat cadence.
    #[test]
    fn next_output_heartbeats_while_waiting_for_a_delayed_frame() {
        const CADENCE_MS: u64 = 50;
        let target = 7u64;
        let limits = Limits {
            max_frame_bytes: 4096,
            heartbeat_interval_millis: CADENCE_MS as u32,
            ..DEFAULT_LIMITS
        };
        let (hb_tx, hb_rx) = mpsc::channel();
        let (mut client, server) = spawn_scripted_server_with_limits(limits, move |conn| {
            let Some(request) = recv_or_finished(conn) else {
                return;
            };
            let request_id = request.request_id.unwrap();
            conn.send(&response_for(
                request_id,
                ResponseResult::BlockAttach(block_attach_response()),
            ))
            .unwrap();

            // Count the heartbeats the client sends while it waits for a frame
            // that is deliberately withheld, then release it.
            let deadline = Instant::now() + Duration::from_millis(250);
            let mut heartbeats = 0usize;
            while Instant::now() < deadline {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if conn
                    .set_read_timeout(Some(remaining.min(Duration::from_millis(50))))
                    .is_err()
                {
                    break;
                }
                match conn.recv() {
                    Ok(Some(envelope)) => {
                        if matches!(envelope.body, Some(Body::Heartbeat(_))) {
                            heartbeats += 1;
                        }
                    }
                    Ok(None) => break,
                    Err(err) if is_transport_timeout(&err) => {}
                    Err(_) => break,
                }
            }
            hb_tx.send(heartbeats).unwrap();
            let _ = conn.send(&terminal_data(target, 1, b"delayed"));
            // Keep the connection open until the client is done, so the client's
            // next heartbeat read/write does not race the server drop.
            conn.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            let _ = conn.recv();
        });

        client
            .attach_block(&BlockId::new(), AttachMode::Writer)
            .unwrap();
        let bytes = client
            .next_output(target, Duration::from_millis(600))
            .unwrap()
            .expect("delayed frame");
        assert_eq!(bytes, b"delayed");

        let heartbeats = hb_rx.recv().unwrap();
        assert!(
            heartbeats >= 1,
            "waiting for a delayed frame must keep the heartbeat cadence, got {heartbeats}"
        );
        // ~250 ms at a 50 ms cadence is at most ~5 sends; mirror the
        // pre-existing cadence test and reject a runaway heartbeat loop without
        // being timing-fragile.
        assert!(
            heartbeats <= 12,
            "heartbeats must not be sent faster than the cadence, got {heartbeats}"
        );

        drop(client);
        let _ = server.join();
    }
}
