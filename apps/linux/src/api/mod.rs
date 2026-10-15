//! Live server client for the UI: two internal connections, one non-blocking
//! event/command surface (GPUI-free).
//!
//! # Two planes
//!
//! * **Control plane** — [`relayable_client::Client`] on its own connection for
//!   `session.create` / `session.list` / `session.view` and for `session.resume`
//!   and `collect_events`, so reconnect visuals are driven by **real** server
//!   events. It never calls `block.attach`, so it never contends for a writer
//!   lease (multi-client is supported).
//! * **Data plane** — a private [`relayable_protocol::transport::FrameConnection`]
//!   that mirrors the client library's handshake byte shape exactly, then
//!   `block.attach` (WRITER), then reads `TerminalOutput` frames, sending the
//!   mandatory per-frame `FlowControl` acknowledgement the server's credit
//!   window requires (`crates/relayable-server/src/session.rs`).
//!
//! # Threading
//!
//! [`ApiHandle::connect_live`] spawns one background thread per plane. The UI
//! thread only calls the non-blocking methods below; it never performs socket
//! I/O. Both planes are synchronous `std::io` over a Unix socket, matching
//! ADR-002 (no async runtime).
//!
//! # Flow-control accounting (interpretation)
//!
//! The server charges the credit window per **output payload byte**
//! (`Subscriber::try_consume(bytes)`, `session.rs`; `add_credit(ack_bytes)`), and
//! the reference client acknowledges `output.bytes.len()`
//! (`relayable-client/src/lib.rs`, `wait_for_output_inner`). This module
//! therefore acknowledges the exact `TerminalOutput.bytes` length — the byte
//! length of the terminal-data frame's payload. Envelope/framing overhead is
//! **not** acked; acking it would permanently pin the window at its cap.
//!
//! The data plane holds **many** concurrent attachments, so it acks output for
//! every attached stream it currently holds, not just one. The server permits up
//! to its advertised `max_concurrent_streams` (default 8) streams per connection
//! (`crates/relayable-protocol/src/limits.rs`, enforced in
//! `crates/relayable-server/src/dispatch.rs`); the handshake's advertised limit
//! is logged at connect time.
//!
//! # Reconnect policy (client-side, Relayable-original)
//!
//! `session.resume` is core CORE-002 behavior, but a *reconnect policy* is
//! explicitly unimplemented in the core (`protocol/compatibility/core-002-method-set.md`
//! §G). This module implements **one bounded reconnect loop** with a fixed small
//! backoff; it emits `Reconnecting { attempt }` then `Connected`/`Error`. This
//! is a **client-side Relayable-original policy, not core protocol behavior**.
//!
//! # Mock
//!
//! [`ApiHandle::connect_mock`] is the **only** way to get synthetic data. It is
//! clearly labelled ([`ApiHandle::is_mock`] returns `true`) and never pretends
//! to be a live connection. Live is the default.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use relayable_client::{Client, ClientConfig, ClientError};
use relayable_model::SessionId;
use relayable_protocol::codec::CodecError;
use relayable_protocol::methods;
use relayable_protocol::transport::{FrameConnection, TransportError};
use relayable_protocol::v1::{
    envelope::Body, request::Operation, response::Result as ResponseResult, AttachMode,
    BlockAttachRequest, BlockDetachRequest, Envelope, FlowControl, Handshake, Request,
    SessionState, TerminalInput, TerminalOutput, TerminalResize,
};
use relayable_protocol::{CONTROL_STREAM_ID, PROTOCOL_VERSION};
use vte::{Parser, Perform};

/// One bounded reconnect sweep: at most this many attempts.
pub const RECONNECT_MAX_ATTEMPTS: u32 = 3;
/// Fixed backoff between reconnect attempts.
pub const RECONNECT_BACKOFF: Duration = Duration::from_millis(500);
/// Per-plane read poll interval; keeps command handling responsive.
const POLL_INTERVAL: Duration = Duration::from_millis(100);
/// Deadline for a single request/response round trip.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Bound applied to an attach-time recovery snapshot before it is surfaced.
const SNAPSHOT_TAIL_BYTES: usize = 32 * 1024;

/// Connection lifecycle state. Shared and safe to read during render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected and not trying.
    Disconnected,
    /// A connection attempt is in progress.
    Connecting,
    /// Connected and usable.
    Connected,
    /// A reconnect attempt is in progress.
    Reconnecting {
        /// 1-based attempt number.
        attempt: u32,
    },
    /// Permanently failed (policy exhausted) with a message.
    Error(String),
}

/// A session row for the session picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSummaryView {
    /// Wire session id.
    pub session_id: String,
    /// Optional label.
    pub label: Option<String>,
    /// Lower-case state (`active` / `closed` / `unspecified`).
    pub state: String,
}

/// An event delivered to the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiEvent {
    /// The connection state changed.
    ConnectionStateChanged(ConnectionState),
    /// A fresh session list.
    SessionsListed(Vec<SessionSummaryView>),
    /// `session.create` succeeded.
    SessionCreated {
        /// Session wire id.
        session_id: String,
        /// Window wire id.
        window_id: String,
        /// Terminal block wire id.
        block_id: String,
        /// Program process id.
        program_pid: u32,
        /// Server PTY/process identity.
        pty_process_id: String,
    },
    /// A writer attach succeeded; `screen_bytes` seeds the terminal model.
    Attached {
        /// Attached stream id.
        stream_id: u64,
        /// Snapshot columns.
        cols: u16,
        /// Snapshot rows.
        rows: u16,
        /// Bounded recovery tail from the attach-time snapshot.
        screen_bytes: Vec<u8>,
    },
    /// Live terminal output.
    Output {
        /// Source stream id.
        stream_id: u64,
        /// Per-stream monotonic sequence.
        stream_seq: u64,
        /// Raw VT bytes.
        bytes: Vec<u8>,
    },
    /// A resize was accepted by the server.
    Resized {
        /// Stream id.
        stream_id: u64,
        /// New columns.
        cols: u16,
        /// New rows.
        rows: u16,
    },
    /// The pane title/cwd changed (from OSC 0/2 and OSC 7 in the output stream).
    PaneTitleChanged {
        /// Program/title, when known.
        program: Option<String>,
        /// Working directory, when known.
        cwd: Option<String>,
    },
    /// A structured server event or acknowledgement, for the status line.
    ServerEvent {
        /// Stable kind label.
        kind: String,
        /// Human-readable detail.
        detail: String,
    },
    /// The connection was lost.
    Disconnected {
        /// Reason label.
        reason: String,
    },
    /// An error message.
    Error {
        /// Message.
        message: String,
    },
}

/// A command from the UI. All variants are cheap to clone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiCommand {
    /// Reload the session list.
    RefreshSessions,
    /// Create a session.
    CreateSession {
        /// Optional label.
        label: Option<String>,
    },
    /// Attach the data plane to a block as writer.
    AttachBlock {
        /// Block wire id.
        block_id: String,
        /// Take over an existing writer lease.
        takeover: bool,
    },
    /// Detach a stream.
    Detach {
        /// Stream id.
        stream_id: u64,
    },
    /// Send terminal input.
    Input {
        /// Stream id.
        stream_id: u64,
        /// Bytes to write.
        bytes: Vec<u8>,
    },
    /// Resize a terminal stream.
    Resize {
        /// Stream id.
        stream_id: u64,
        /// New columns.
        cols: u16,
        /// New rows.
        rows: u16,
        /// Optional pixel width.
        pixel_width: Option<u32>,
        /// Optional pixel height.
        pixel_height: Option<u32>,
    },
    /// Stop both planes.
    Shutdown,
}

/// The UI-facing handle. `Send + Sync`; no method blocks or panics.
pub struct ApiHandle {
    is_mock: bool,
    socket_path: PathBuf,
    control_tx: Mutex<Sender<ApiCommand>>,
    data_tx: Mutex<Sender<ApiCommand>>,
    events: Mutex<Receiver<ApiEvent>>,
    state: Arc<Mutex<ConnectionState>>,
}

/// Lock a mutex, recovering from poisoning instead of panicking.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn emit(events: &Sender<ApiEvent>, event: ApiEvent) {
    // Unbounded channel: the UI is expected to drain each frame.
    let _ = events.send(event);
}

impl ApiHandle {
    /// Connect live to `socket_path`, spawning both planes on background
    /// threads.
    #[must_use]
    pub fn connect_live(socket_path: PathBuf) -> ApiHandle {
        let (control_tx, control_rx) = channel();
        let (data_tx, data_rx) = channel();
        let (events_tx, events_rx) = channel();
        let state = Arc::new(Mutex::new(ConnectionState::Connecting));

        {
            let path = socket_path.clone();
            let tx = events_tx.clone();
            let state = Arc::clone(&state);
            spawn("relayable-api-control", move || {
                control_thread(path, control_rx, tx, state);
            });
        }
        {
            let path = socket_path.clone();
            let tx = events_tx.clone();
            spawn("relayable-api-data", move || {
                data_thread(path, data_rx, tx);
            });
        }
        // Keep `events_tx` alive for the process lifetime: it is cloned into the
        // workers above, but holding one here makes the absence of a sender a
        // shutdown signal that never fires spuriously.

        ApiHandle {
            is_mock: false,
            socket_path,
            control_tx: Mutex::new(control_tx),
            data_tx: Mutex::new(data_tx),
            events: Mutex::new(events_rx),
            state,
        }
    }

    /// A clearly-labelled in-process mock with synthetic data.
    ///
    /// Live is the default; this constructor exists so the UI can be developed
    /// without a server. [`ApiHandle::is_mock`] returns `true`.
    #[must_use]
    pub fn connect_mock() -> ApiHandle {
        let (control_tx, control_rx) = channel();
        let (data_tx, data_rx) = channel();
        let (events_tx, events_rx) = channel();
        let state = Arc::new(Mutex::new(ConnectionState::Connected));

        {
            let tx = events_tx.clone();
            spawn("relayable-api-mock-control", move || {
                mock_control_thread(control_rx, tx);
            });
        }
        {
            let tx = events_tx.clone();
            spawn("relayable-api-mock-data", move || {
                mock_data_thread(data_rx, tx);
            });
        }

        ApiHandle {
            is_mock: true,
            socket_path: PathBuf::from("<mock>"),
            control_tx: Mutex::new(control_tx),
            data_tx: Mutex::new(data_tx),
            events: Mutex::new(events_rx),
            state,
        }
    }

    /// Whether this handle is the in-process mock.
    #[must_use]
    pub fn is_mock(&self) -> bool {
        self.is_mock
    }

    /// The socket path this handle targets (`<mock>` for the mock).
    #[must_use]
    pub fn socket_path(&self) -> PathBuf {
        self.socket_path.clone()
    }

    /// Enqueue a command. Never blocks and never panics, even after shutdown.
    pub fn send(&self, cmd: ApiCommand) {
        let to_control = matches!(
            &cmd,
            ApiCommand::RefreshSessions | ApiCommand::CreateSession { .. } | ApiCommand::Shutdown
        );
        let to_data = matches!(
            &cmd,
            ApiCommand::AttachBlock { .. }
                | ApiCommand::Detach { .. }
                | ApiCommand::Input { .. }
                | ApiCommand::Resize { .. }
                | ApiCommand::Shutdown
        );
        if to_control {
            let _ = lock(&self.control_tx).send(cmd.clone());
        }
        if to_data {
            let _ = lock(&self.data_tx).send(cmd);
        }
    }

    /// Take one pending event, if any.
    #[must_use]
    pub fn try_recv(&self) -> Option<ApiEvent> {
        lock(&self.events).try_recv().ok()
    }

    /// Take all pending events.
    #[must_use]
    pub fn drain(&self) -> Vec<ApiEvent> {
        let receiver = lock(&self.events);
        let mut events = Vec::new();
        while let Ok(event) = receiver.try_recv() {
            events.push(event);
        }
        events
    }

    /// A snapshot of the shared connection state.
    #[must_use]
    pub fn state(&self) -> ConnectionState {
        lock(&self.state).clone()
    }
}

fn spawn<F: FnOnce() + Send + 'static>(name: &str, body: F) {
    std::thread::Builder::new()
        .name(name.to_string())
        .spawn(body)
        .expect("failed to spawn a relayable API worker thread");
}

fn set_state(state: &Mutex<ConnectionState>, events: &Sender<ApiEvent>, next: ConnectionState) {
    *lock(state) = next.clone();
    emit(events, ApiEvent::ConnectionStateChanged(next));
}

// --- control plane ---------------------------------------------------------

fn control_thread(
    path: PathBuf,
    commands: Receiver<ApiCommand>,
    events: Sender<ApiEvent>,
    state: Arc<Mutex<ConnectionState>>,
) {
    let mut client: Option<Client> = None;
    let mut attempts = 0u32;
    let mut first_attempt = true;
    let mut last_session: Option<SessionId> = None;
    let mut last_sequence = 0u64;

    loop {
        let mut shutdown = false;
        while let Ok(command) = commands.try_recv() {
            match command {
                ApiCommand::Shutdown => {
                    shutdown = true;
                    break;
                }
                ApiCommand::RefreshSessions => match client.as_mut() {
                    Some(client) => list_sessions(client, &events),
                    None => emit(
                        &events,
                        ApiEvent::Error {
                            message: "session list unavailable while disconnected".to_string(),
                        },
                    ),
                },
                ApiCommand::CreateSession { label } => match client.as_mut() {
                    Some(client) => {
                        create_session(client, label.as_deref(), &events, &mut last_session);
                    }
                    None => emit(
                        &events,
                        ApiEvent::Error {
                            message: "session create unavailable while disconnected".to_string(),
                        },
                    ),
                },
                _ => {}
            }
        }
        if shutdown {
            return;
        }

        if client.is_none() {
            if attempts >= RECONNECT_MAX_ATTEMPTS {
                // Policy exhausted: stay in Error but keep serving commands.
                std::thread::sleep(POLL_INTERVAL);
                continue;
            }
            attempts += 1;
            if !first_attempt {
                set_state(
                    &state,
                    &events,
                    ConnectionState::Reconnecting { attempt: attempts },
                );
            }
            first_attempt = false;
            match Client::connect(&path, ClientConfig::new("ui")) {
                Ok(mut connected) => {
                    attempts = 0;
                    set_state(&state, &events, ConnectionState::Connected);
                    resume_after_connect(
                        &mut connected,
                        &events,
                        &mut last_session,
                        &mut last_sequence,
                    );
                    client = Some(connected);
                }
                Err(error) => {
                    if attempts >= RECONNECT_MAX_ATTEMPTS {
                        let message = format!("could not connect to {}: {error}", path.display());
                        set_state(&state, &events, ConnectionState::Error(message.clone()));
                        emit(&events, ApiEvent::Error { message });
                    } else {
                        emit(
                            &events,
                            ApiEvent::Error {
                                message: format!("connect attempt {attempts} failed: {error}"),
                            },
                        );
                    }
                }
            }
            continue;
        }

        let Some(active) = client.as_mut() else {
            continue;
        };
        match active.collect_events(POLL_INTERVAL) {
            Ok(received) => {
                for event in received {
                    last_sequence = last_sequence.max(event.sequence);
                    emit(&events, translate_wire_event(event));
                }
            }
            Err(ClientError::Disconnected) | Err(ClientError::Transport(_)) => {
                emit(
                    &events,
                    ApiEvent::Disconnected {
                        reason: "control connection lost".to_string(),
                    },
                );
                client = None;
            }
            Err(error) => emit(
                &events,
                ApiEvent::Error {
                    message: error.to_string(),
                },
            ),
        }
    }
}

fn list_sessions(client: &mut Client, events: &Sender<ApiEvent>) {
    match client.list_sessions() {
        Ok(sessions) => emit(
            events,
            ApiEvent::SessionsListed(
                sessions
                    .into_iter()
                    .map(|session| SessionSummaryView {
                        session_id: session.session_id,
                        label: session.label,
                        state: session_state_str(session.state),
                    })
                    .collect(),
            ),
        ),
        Err(error) => emit(
            events,
            ApiEvent::Error {
                message: error.to_string(),
            },
        ),
    }
}

fn create_session(
    client: &mut Client,
    label: Option<&str>,
    events: &Sender<ApiEvent>,
    last_session: &mut Option<SessionId>,
) {
    match client.create_session(label) {
        Ok(created) => {
            if let Ok(session_id) = created.session.session_id.parse::<SessionId>() {
                *last_session = Some(session_id);
            }
            emit(
                events,
                ApiEvent::SessionCreated {
                    session_id: created.session.session_id,
                    window_id: created.window_id.to_string(),
                    block_id: created.block_id.to_string(),
                    program_pid: created.program_pid,
                    pty_process_id: created.pty_process_id.to_string(),
                },
            );
        }
        Err(error) => emit(
            events,
            ApiEvent::Error {
                message: error.to_string(),
            },
        ),
    }
}

/// After a (re)connect, fetch a fresh `session.view` and `session.resume` from
/// the last applied cursor so the UI is synchronized from real server state.
fn resume_after_connect(
    client: &mut Client,
    events: &Sender<ApiEvent>,
    last_session: &mut Option<SessionId>,
    last_sequence: &mut u64,
) {
    let Some(session_id) = *last_session else {
        return;
    };

    match client.session_snapshot(&session_id) {
        Ok(snapshot) => emit(
            events,
            ApiEvent::ServerEvent {
                kind: "session.view".to_string(),
                detail: format!("session={session_id} sequence={}", snapshot.sequence),
            },
        ),
        Err(error) => emit(
            events,
            ApiEvent::Error {
                message: format!("session.view failed: {error}"),
            },
        ),
    }

    match client.resume_session(&session_id, Some(*last_sequence)) {
        Ok(resume) => {
            emit(
                events,
                ApiEvent::ServerEvent {
                    kind: "session.resume".to_string(),
                    detail: format!(
                        "snapshot_required={} snapshot_sequence={} current_sequence={} events={}",
                        resume.snapshot_required,
                        resume.snapshot_sequence,
                        resume.current_sequence,
                        resume.events.len()
                    ),
                },
            );
            for event in resume.events {
                *last_sequence = (*last_sequence).max(event.sequence);
                emit(events, translate_wire_event(event));
            }
            *last_sequence = (*last_sequence).max(resume.current_sequence);
        }
        Err(error) => emit(
            events,
            ApiEvent::Error {
                message: format!("session.resume failed: {error}"),
            },
        ),
    }
}

fn session_state_str(state: i32) -> String {
    match SessionState::try_from(state) {
        Ok(SessionState::Active) => "active".to_string(),
        Ok(SessionState::Closed) => "closed".to_string(),
        _ => "unspecified".to_string(),
    }
}

fn translate_wire_event(event: relayable_protocol::v1::Event) -> ApiEvent {
    use relayable_protocol::v1::event::Event as Kind;
    let (kind, detail) = match event.event {
        Some(Kind::ClientConnectionStateChanged(e)) => (
            "client_connection_state_changed",
            format!("client_id={} state={}", e.client_id, e.connection_state),
        ),
        Some(Kind::SessionStateChanged(e)) => (
            "session_state_changed",
            format!(
                "session_id={} state={}",
                e.session_id,
                session_state_str(e.state)
            ),
        ),
        Some(Kind::ClientDisconnected(e)) => (
            "client_disconnected",
            format!("client_id={} reason={}", e.client_id, e.reason),
        ),
        Some(Kind::WriterLeaseChanged(e)) => (
            "writer_lease_changed",
            format!(
                "session_id={} block_id={} stream_id={} previous={} new={} held={}",
                e.session_id,
                e.block_id,
                e.stream_id,
                e.previous_client_id,
                e.new_client_id,
                e.lease_held
            ),
        ),
        Some(Kind::TerminalChildExited(e)) => (
            "terminal_child_exited",
            format!(
                "session_id={} block_id={} exit_code={} signal={} exited={}",
                e.session_id, e.block_id, e.exit_code, e.signal, e.exited
            ),
        ),
        None => ("unknown", format!("sequence={}", event.sequence)),
    };
    ApiEvent::ServerEvent {
        kind: kind.to_string(),
        detail,
    }
}

// --- data plane ------------------------------------------------------------

/// One data-plane read outcome.
enum FrameOutcome {
    /// A complete frame arrived (boxed: `Envelope` is large).
    Frame(Box<Envelope>),
    /// The read deadline expired with no frame (not a disconnect).
    Idle,
    /// The peer closed the connection cleanly.
    Closed,
}

/// The private data-plane connection.
struct DataPlane {
    conn: FrameConnection,
    events: Sender<ApiEvent>,
    next_request_id: u64,
    /// Every stream this connection currently holds (`block.attach` succeeded and
    /// no `block.detach` has completed). N concurrent live attachments.
    stream_ids: BTreeSet<u64>,
    /// Per-stream OSC parser + title/cwd state, so interleaved output from
    /// different streams cannot corrupt each other's escape parsing.
    titles: HashMap<u64, StreamTitle>,
}

/// Per-stream OSC 0/2 title and OSC 7 cwd state.
#[derive(Default)]
struct StreamTitle {
    parser: Parser,
    tracker: TitleTracker,
}

impl DataPlane {
    fn connect(path: &std::path::Path, events: Sender<ApiEvent>) -> Result<Self, String> {
        let conn = FrameConnection::connect(path)
            .map_err(|error| format!("could not connect to {}: {error}", path.display()))?;
        conn.set_read_timeout(Some(POLL_INTERVAL))
            .map_err(|error| format!("could not set read timeout: {error}"))?;

        let mut plane = Self {
            conn,
            events,
            next_request_id: 1,
            stream_ids: BTreeSet::new(),
            titles: HashMap::new(),
        };
        plane.perform_handshake()?;
        Ok(plane)
    }

    fn perform_handshake(&mut self) -> Result<(), String> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: Some(request_id),
            stream_id: CONTROL_STREAM_ID,
            body: Some(Body::Handshake(Handshake {
                client_kind: "ui".to_string(),
                platform: std::env::consts::OS.to_string(),
                client_version: env!("CARGO_PKG_VERSION").to_string(),
                protocol_version: PROTOCOL_VERSION.to_string(),
                capabilities: Vec::new(),
            })),
            ..Envelope::default()
        };
        self.conn
            .send(&envelope)
            .map_err(|error| format!("handshake send failed: {error}"))?;
        match self.await_result(request_id)? {
            ResponseResult::Hello(hello) => {
                if let Some(limits) = hello.limits.as_ref() {
                    eprintln!(
                        "relayable-data: server advertised max_concurrent_streams={} credit_window_bytes={}",
                        limits.max_concurrent_streams, limits.credit_window_bytes
                    );
                }
                Ok(())
            }
            ResponseResult::Error(error) => Err(format!("handshake rejected: {}", error.message)),
            other => Err(format!("unexpected handshake response: {other:?}")),
        }
    }

    fn attach(&mut self, block_id: &str, takeover: bool) -> Result<(), String> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        let envelope = self.request_envelope(
            request_id,
            methods::BLOCK_ATTACH,
            Operation::BlockAttach(BlockAttachRequest {
                block_id: block_id.to_string(),
                mode: AttachMode::Writer as i32,
                takeover,
            }),
        );
        self.conn
            .send(&envelope)
            .map_err(|error| format!("block.attach send failed: {error}"))?;

        match self.await_result(request_id)? {
            ResponseResult::BlockAttach(response) => {
                let snapshot = response.snapshot.unwrap_or_default();
                let cols = clamp_dimension(snapshot.cols);
                let rows = clamp_dimension(snapshot.rows);
                let screen_bytes = tail(&snapshot.screen_bytes, SNAPSHOT_TAIL_BYTES);
                self.stream_ids.insert(response.stream_id);
                // Emit `Attached` before the snapshot's OSC title/cwd so a
                // listener can attribute the following `PaneTitleChanged` to the
                // stream that was just attached (the event carries no id itself).
                emit(
                    &self.events,
                    ApiEvent::Attached {
                        stream_id: response.stream_id,
                        cols,
                        rows,
                        screen_bytes: screen_bytes.clone(),
                    },
                );
                self.observe(response.stream_id, &screen_bytes);
                Ok(())
            }
            ResponseResult::Error(error) => Err(format!("block.attach failed: {}", error.message)),
            other => Err(format!("unexpected block.attach response: {other:?}")),
        }
    }

    fn detach(&mut self, stream_id: u64) -> Result<(), String> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        let envelope = self.request_envelope(
            request_id,
            methods::BLOCK_DETACH,
            Operation::BlockDetach(BlockDetachRequest { stream_id }),
        );
        self.conn
            .send(&envelope)
            .map_err(|error| format!("block.detach send failed: {error}"))?;
        match self.await_result(request_id)? {
            ResponseResult::BlockDetach(_) => {
                self.stream_ids.remove(&stream_id);
                self.titles.remove(&stream_id);
                Ok(())
            }
            ResponseResult::Error(error) => Err(format!("block.detach failed: {}", error.message)),
            other => Err(format!("unexpected block.detach response: {other:?}")),
        }
    }

    fn send_input(&mut self, stream_id: u64, bytes: Vec<u8>) -> Result<(), String> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        let envelope = self.request_envelope(
            request_id,
            methods::TERMINAL_INPUT,
            Operation::TerminalInput(TerminalInput {
                stream_id,
                bytes: bytes.clone(),
            }),
        );
        self.conn
            .send(&envelope)
            .map_err(|error| format!("terminal.input send failed: {error}"))?;
        match self.await_result(request_id)? {
            ResponseResult::TerminalInput(accepted) => {
                emit(
                    &self.events,
                    ApiEvent::ServerEvent {
                        kind: "TerminalInputAccepted".to_string(),
                        detail: format!(
                            "stream_id={} accepted_bytes={}",
                            accepted.stream_id, accepted.accepted_bytes
                        ),
                    },
                );
                Ok(())
            }
            ResponseResult::Error(error) => {
                Err(format!("terminal.input failed: {}", error.message))
            }
            other => Err(format!("unexpected terminal.input response: {other:?}")),
        }
    }

    fn send_resize(
        &mut self,
        stream_id: u64,
        cols: u16,
        rows: u16,
        pixel_width: Option<u32>,
        pixel_height: Option<u32>,
    ) -> Result<(), String> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        let envelope = self.request_envelope(
            request_id,
            methods::TERMINAL_RESIZE,
            Operation::TerminalResize(TerminalResize {
                stream_id,
                cols: u32::from(cols),
                rows: u32::from(rows),
                pixel_width,
                pixel_height,
            }),
        );
        self.conn
            .send(&envelope)
            .map_err(|error| format!("terminal.resize send failed: {error}"))?;
        match self.await_result(request_id)? {
            ResponseResult::TerminalResize(accepted) => {
                emit(
                    &self.events,
                    ApiEvent::Resized {
                        stream_id: accepted.stream_id,
                        cols: clamp_dimension(accepted.cols),
                        rows: clamp_dimension(accepted.rows),
                    },
                );
                emit(
                    &self.events,
                    ApiEvent::ServerEvent {
                        kind: "TerminalResizeAccepted".to_string(),
                        detail: format!(
                            "stream_id={} cols={} rows={}",
                            accepted.stream_id, accepted.cols, accepted.rows
                        ),
                    },
                );
                Ok(())
            }
            ResponseResult::Error(error) => {
                Err(format!("terminal.resize failed: {}", error.message))
            }
            other => Err(format!("unexpected terminal.resize response: {other:?}")),
        }
    }

    fn request_envelope(&self, request_id: u64, method: &str, operation: Operation) -> Envelope {
        Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: Some(request_id),
            stream_id: CONTROL_STREAM_ID,
            body: Some(Body::Request(Request {
                method: method.to_string(),
                operation: Some(operation),
            })),
            ..Envelope::default()
        }
    }

    /// Read until the response for `request_id`, handling output frames on the
    /// way. Returns the response's `result`.
    fn await_result(&mut self, request_id: u64) -> Result<ResponseResult, String> {
        let deadline = Instant::now() + REQUEST_TIMEOUT;
        loop {
            if Instant::now() >= deadline {
                return Err("timed out waiting for a server response".to_string());
            }
            match self.recv_frame()? {
                FrameOutcome::Frame(envelope) => match envelope.body {
                    Some(Body::Response(response)) if response.request_id == request_id => {
                        return response
                            .result
                            .ok_or_else(|| "response carried no result".to_string());
                    }
                    Some(Body::TerminalData(output)) => self.handle_output(output),
                    Some(Body::Response(_) | Body::Heartbeat(_) | Body::FlowControl(_)) => {}
                    Some(Body::Event(event)) => {
                        emit(&self.events, translate_wire_event(event));
                    }
                    _ => {}
                },
                FrameOutcome::Idle => {}
                FrameOutcome::Closed => return Err("server closed the connection".to_string()),
            }
        }
    }

    /// One output pump; a read timeout is not an error.
    fn pump(&mut self) -> Result<(), String> {
        match self.recv_frame()? {
            FrameOutcome::Frame(envelope) => {
                match envelope.body {
                    Some(Body::TerminalData(output)) => self.handle_output(output),
                    Some(Body::Event(event)) => {
                        emit(&self.events, translate_wire_event(event));
                    }
                    Some(Body::Heartbeat(_) | Body::FlowControl(_) | Body::Response(_)) => {}
                    _ => {}
                }
                Ok(())
            }
            FrameOutcome::Idle => Ok(()),
            FrameOutcome::Closed => Err("server closed the connection".to_string()),
        }
    }

    /// One read. A read-deadline expiry is [`FrameOutcome::Idle`]; a clean EOF is
    /// [`FrameOutcome::Closed`]; anything else is an error.
    fn recv_frame(&mut self) -> Result<FrameOutcome, String> {
        match self.conn.recv() {
            Ok(Some(envelope)) => Ok(FrameOutcome::Frame(Box::new(envelope))),
            Ok(None) => Ok(FrameOutcome::Closed),
            Err(TransportError::Io(error)) if is_timeout(&error) => Ok(FrameOutcome::Idle),
            Err(TransportError::Codec(CodecError::Io(error))) if is_timeout(&error) => {
                Ok(FrameOutcome::Idle)
            }
            Err(error) => Err(error.to_string()),
        }
    }

    fn handle_output(&mut self, output: TerminalOutput) {
        let TerminalOutput {
            stream_id,
            stream_seq,
            bytes,
        } = output;
        let ack = bytes.len() as u64;
        // Emit the output first so the UI can associate the following
        // `PaneTitleChanged` (which carries no stream id) with this stream.
        emit(
            &self.events,
            ApiEvent::Output {
                stream_id,
                stream_seq,
                bytes: bytes.clone(),
            },
        );
        self.observe(stream_id, &bytes);
        if should_ack(&self.stream_ids, stream_id) && ack > 0 {
            // The server charges its credit window per output payload byte.
            let envelope = Envelope {
                protocol_version: PROTOCOL_VERSION.to_string(),
                stream_id,
                body: Some(Body::FlowControl(FlowControl {
                    stream_id,
                    ack_bytes: ack,
                })),
                ..Envelope::default()
            };
            let _ = self.conn.send(&envelope);
        }
    }

    /// Feed bytes to the lightweight OSC title/cwd tracker for one stream.
    fn observe(&mut self, stream_id: u64, bytes: &[u8]) {
        let entry = self.titles.entry(stream_id).or_default();
        let mut performer = OscPerformer {
            tracker: &mut entry.tracker,
            changed: false,
        };
        entry.parser.advance(&mut performer, bytes);
        if performer.changed {
            let program = entry.tracker.title.clone();
            let cwd = entry.tracker.cwd.clone();
            emit(&self.events, ApiEvent::PaneTitleChanged { program, cwd });
        }
    }
}

/// Connect the data plane and, on success, tell the UI it is usable again.
///
/// The signal is a [`ApiEvent::ServerEvent`] (not a new [`ApiEvent`] variant,
/// which is frozen) so the UI can re-attach panes whose streams were invalidated
/// by a data-plane-only disconnect.
fn connect_data_plane(
    path: &std::path::Path,
    events: &Sender<ApiEvent>,
) -> Result<DataPlane, String> {
    let plane = DataPlane::connect(path, events.clone())?;
    emit(
        events,
        ApiEvent::ServerEvent {
            kind: "data_plane_connected".to_string(),
            detail: path.display().to_string(),
        },
    );
    Ok(plane)
}

fn data_thread(path: PathBuf, commands: Receiver<ApiCommand>, events: Sender<ApiEvent>) {
    let mut plane = match connect_data_plane(&path, &events) {
        Ok(plane) => Some(plane),
        Err(error) => {
            emit(&events, ApiEvent::Error { message: error });
            None
        }
    };
    let mut attempts = 0u32;

    loop {
        if plane.is_none() {
            if attempts >= RECONNECT_MAX_ATTEMPTS {
                std::thread::sleep(POLL_INTERVAL);
                // Still drain commands so `Shutdown` is honoured.
                while let Ok(command) = commands.try_recv() {
                    if matches!(command, ApiCommand::Shutdown) {
                        return;
                    }
                }
                continue;
            }
            attempts += 1;
            match connect_data_plane(&path, &events) {
                Ok(connected) => {
                    attempts = 0;
                    plane = Some(connected);
                }
                Err(error) => {
                    emit(
                        &events,
                        ApiEvent::Error {
                            message: format!(
                                "data-plane connect attempt {attempts} failed: {error}"
                            ),
                        },
                    );
                    std::thread::sleep(RECONNECT_BACKOFF);
                    continue;
                }
            }
        }

        let mut shutdown = false;
        while let Ok(command) = commands.try_recv() {
            match command {
                ApiCommand::Shutdown => {
                    shutdown = true;
                    break;
                }
                ApiCommand::AttachBlock { block_id, takeover } => {
                    if let Some(plane) = plane.as_mut() {
                        if let Err(error) = plane.attach(&block_id, takeover) {
                            emit(&events, ApiEvent::Error { message: error });
                        }
                    }
                }
                ApiCommand::Detach { stream_id } => {
                    if let Some(plane) = plane.as_mut() {
                        if let Err(error) = plane.detach(stream_id) {
                            emit(&events, ApiEvent::Error { message: error });
                        }
                    }
                }
                ApiCommand::Input { stream_id, bytes } => {
                    if let Some(plane) = plane.as_mut() {
                        if let Err(error) = plane.send_input(stream_id, bytes) {
                            emit(&events, ApiEvent::Error { message: error });
                        }
                    }
                }
                ApiCommand::Resize {
                    stream_id,
                    cols,
                    rows,
                    pixel_width,
                    pixel_height,
                } => {
                    if let Some(plane) = plane.as_mut() {
                        if let Err(error) =
                            plane.send_resize(stream_id, cols, rows, pixel_width, pixel_height)
                        {
                            emit(&events, ApiEvent::Error { message: error });
                        }
                    }
                }
                _ => {}
            }
        }
        if shutdown {
            return;
        }

        if let Some(active_plane) = plane.as_mut() {
            if let Err(error) = active_plane.pump() {
                emit(
                    &events,
                    ApiEvent::Disconnected {
                        reason: format!("data connection lost: {error}"),
                    },
                );
                plane = None;
            }
        }
    }
}

// --- OSC title/cwd tracking ------------------------------------------------

/// The latest OSC 0/2 title and OSC 7 cwd seen on the output stream.
#[derive(Debug, Default, Clone)]
struct TitleTracker {
    title: Option<String>,
    cwd: Option<String>,
}

/// A `vte` performer that only records OSC 0/2 (title) and OSC 7 (cwd).
struct OscPerformer<'a> {
    tracker: &'a mut TitleTracker,
    changed: bool,
}

impl Perform for OscPerformer<'_> {
    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.len() < 2 {
            return;
        }
        match params[0] {
            b"0" | b"2" => {
                let title = params[1..]
                    .iter()
                    .map(|part| String::from_utf8_lossy(part))
                    .collect::<Vec<_>>()
                    .join(";");
                self.tracker.title = Some(title.trim().to_string());
                self.changed = true;
            }
            b"7" => {
                let raw = String::from_utf8_lossy(params[1]).to_string();
                self.tracker.cwd = Some(strip_file_uri(&raw));
                self.changed = true;
            }
            _ => {}
        }
    }
}

/// Turn `file://host/path` into `/path`; leave anything else untouched.
fn strip_file_uri(raw: &str) -> String {
    let Some(rest) = raw.strip_prefix("file://") else {
        return raw.to_string();
    };
    match rest.find('/') {
        Some(index) => rest[index..].to_string(),
        None => rest.to_string(),
    }
}

// --- shared helpers --------------------------------------------------------

fn is_timeout(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
    )
}

/// Whether an output frame's stream is one this connection currently holds and
/// therefore must credit-ack. Pure so it can be unit-tested without a socket.
fn should_ack(attached: &BTreeSet<u64>, stream_id: u64) -> bool {
    attached.contains(&stream_id)
}

fn clamp_dimension(value: u32) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

fn tail(bytes: &[u8], max: usize) -> Vec<u8> {
    if bytes.len() <= max {
        return bytes.to_vec();
    }
    bytes[bytes.len() - max..].to_vec()
}

// --- mock ------------------------------------------------------------------

fn mock_control_thread(commands: Receiver<ApiCommand>, events: Sender<ApiEvent>) {
    emit(
        &events,
        ApiEvent::ConnectionStateChanged(ConnectionState::Connected),
    );
    emit(
        &events,
        ApiEvent::SessionsListed(vec![SessionSummaryView {
            session_id: "session:mock".to_string(),
            label: Some("mock session".to_string()),
            state: "active".to_string(),
        }]),
    );
    // Blocking receive: process every command for as long as the handle lives.
    while let Ok(command) = commands.recv() {
        match command {
            ApiCommand::Shutdown => return,
            ApiCommand::RefreshSessions => emit(
                &events,
                ApiEvent::SessionsListed(vec![SessionSummaryView {
                    session_id: "session:mock".to_string(),
                    label: Some("mock session".to_string()),
                    state: "active".to_string(),
                }]),
            ),
            ApiCommand::CreateSession { label: _ } => emit(
                &events,
                ApiEvent::SessionCreated {
                    session_id: "session:mock".to_string(),
                    window_id: "window:mock".to_string(),
                    block_id: "block:mock".to_string(),
                    program_pid: 0,
                    pty_process_id: "pty:mock".to_string(),
                },
            ),
            _ => {}
        }
    }
}

fn mock_data_thread(commands: Receiver<ApiCommand>, events: Sender<ApiEvent>) {
    // Blocking receive: process every command for as long as the handle lives.
    while let Ok(command) = commands.recv() {
        match command {
            ApiCommand::Shutdown => return,
            ApiCommand::AttachBlock { .. } => {
                emit(
                    &events,
                    ApiEvent::Attached {
                        stream_id: 1,
                        cols: 80,
                        rows: 24,
                        screen_bytes: b"mock$ ".to_vec(),
                    },
                );
                emit(
                    &events,
                    ApiEvent::Output {
                        stream_id: 1,
                        stream_seq: 1,
                        bytes: b"mock output\r\n".to_vec(),
                    },
                );
            }
            ApiCommand::Input { stream_id, bytes } => emit(
                &events,
                ApiEvent::Output {
                    stream_id,
                    stream_seq: 2,
                    bytes,
                },
            ),
            ApiCommand::Resize {
                stream_id,
                cols,
                rows,
                ..
            } => emit(
                &events,
                ApiEvent::Resized {
                    stream_id,
                    cols,
                    rows,
                },
            ),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn api_handle_is_send_and_sync() {
        assert_send_sync::<ApiHandle>();
    }

    #[test]
    fn mock_handle_is_labelled_and_non_blocking() {
        let handle = ApiHandle::connect_mock();
        assert!(handle.is_mock());
        assert_eq!(handle.socket_path(), PathBuf::from("<mock>"));
        // `send` must not block or panic; `try_recv` must not block.
        handle.send(ApiCommand::RefreshSessions);
        let _ = handle.try_recv();
        let _ = handle.drain();
        handle.send(ApiCommand::Shutdown);
    }

    #[test]
    fn mock_replies_to_create_and_attach() {
        let handle = ApiHandle::connect_mock();
        handle.send(ApiCommand::CreateSession {
            label: Some("demo".to_string()),
        });
        handle.send(ApiCommand::AttachBlock {
            block_id: "block:mock".to_string(),
            takeover: false,
        });

        let mut seen_created = false;
        let mut seen_attached = false;
        for _ in 0..200 {
            for event in handle.drain() {
                match event {
                    ApiEvent::SessionCreated { block_id, .. } => {
                        assert_eq!(block_id, "block:mock");
                        seen_created = true;
                    }
                    ApiEvent::Attached { cols, rows, .. } => {
                        assert_eq!((cols, rows), (80, 24));
                        seen_attached = true;
                    }
                    _ => {}
                }
            }
            if seen_created && seen_attached {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(seen_created, "mock must answer session.create");
        assert!(seen_attached, "mock must answer block.attach");
        handle.send(ApiCommand::Shutdown);
    }

    #[test]
    fn strip_file_uri_handles_host_and_bare_paths() {
        assert_eq!(strip_file_uri("file://host/tmp/x"), "/tmp/x");
        assert_eq!(strip_file_uri("file:///tmp/x"), "/tmp/x");
        assert_eq!(strip_file_uri("/tmp/x"), "/tmp/x");
        assert_eq!(strip_file_uri("relative"), "relative");
    }

    #[test]
    fn tail_bounds_a_long_snapshot() {
        let bytes: Vec<u8> = (0..100u8).collect();
        let tailed = tail(&bytes, 10);
        assert_eq!(tailed.len(), 10);
        assert_eq!(tailed, (90..100u8).collect::<Vec<u8>>());
    }

    #[test]
    fn osc_tracker_records_title_and_cwd() {
        let mut tracker = TitleTracker::default();
        let mut performer = OscPerformer {
            tracker: &mut tracker,
            changed: false,
        };
        performer.osc_dispatch(&[b"2", b"my-title"], false);
        performer.osc_dispatch(&[b"7", b"file://host/tmp/work"], false);
        assert!(performer.changed);
        assert_eq!(tracker.title.as_deref(), Some("my-title"));
        assert_eq!(tracker.cwd.as_deref(), Some("/tmp/work"));
    }

    #[test]
    fn every_attached_stream_is_credit_acked_not_just_one() {
        let mut attached = BTreeSet::new();
        attached.insert(3u64);
        attached.insert(7u64);
        assert!(should_ack(&attached, 3));
        assert!(should_ack(&attached, 7));
        assert!(!should_ack(&attached, 4));
        attached.remove(&7);
        assert!(!should_ack(&attached, 7));
        assert!(should_ack(&attached, 3));
    }
}
