use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};

use relayable_client::{Client, ClientConfig, ClientError};
use relayable_model::SessionId;
use relayable_protocol::v1::SessionState;

use super::data::data_thread;
use super::mock::{mock_control_thread, mock_data_thread};
use super::types::{
    ApiCommand, ApiEvent, ConnectionState, SessionSummaryView, POLL_INTERVAL,
    RECONNECT_MAX_ATTEMPTS,
};

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

pub(super) fn emit(events: &Sender<ApiEvent>, event: ApiEvent) {
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

pub(super) fn translate_wire_event(event: relayable_protocol::v1::Event) -> ApiEvent {
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
