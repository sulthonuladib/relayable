//! Server-wide registry: sessions, clients, connections, and the retained
//! control-event log. This is the single authority for server-owned state.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use relayable_model::{
    BlockId, Client, ClientId, ConnectionState, PtyProcessId, ServerId, SessionId, WindowId,
};
use relayable_protocol::v1::{
    self, ClientRecord, Envelope, ErrorCode, Handshake, SessionRecord, SessionResumeResponse,
    SessionState, SessionSummary,
};

use crate::config::ServerConfig;
use crate::connection::ConnectionShared;
use crate::events::{
    client_connection_state_event, client_disconnected_event, decide_resume, session_state_event,
    writer_lease_event, DisconnectReason, EventBus, ResumeDecision,
};
use crate::lock;
use crate::session::{BlockInit, BlockRuntime, SessionRuntime};
use crate::snapshot;

/// Data returned to `session.create`.
pub(crate) struct SessionCreateData {
    pub(crate) session: SessionRecord,
    pub(crate) window_id: WindowId,
    pub(crate) block_id: BlockId,
    pub(crate) pty_process_id: PtyProcessId,
    pub(crate) program_pid: Option<u32>,
}

/// The result of a `session.close` attempt.
pub(crate) enum CloseSessionOutcome {
    /// The child was confirmed dead; the session is now CLOSED.
    Closed(SessionRecord),
    /// The child could not be confirmed dead; the session remains ACTIVE.
    NotConfirmed { pid: Option<u32> },
    /// No such session.
    Unknown,
}

/// Server-wide shared state.
pub(crate) struct ServerShared {
    pub(crate) config: ServerConfig,
    pub(crate) server_id: ServerId,
    pub(crate) started_at_millis: u64,
    /// The single server-wide retained control-event log.
    pub(crate) events: Arc<EventBus>,
    pub(crate) sessions: Mutex<BTreeMap<SessionId, Arc<SessionRuntime>>>,
    pub(crate) clients: Mutex<BTreeMap<ClientId, Client>>,
    pub(crate) connections: Mutex<Vec<Arc<ConnectionShared>>>,
    pub(crate) connection_threads: Mutex<Vec<JoinHandle<()>>>,
    pub(crate) shutdown: AtomicBool,
}

impl ServerShared {
    pub(crate) fn new(config: ServerConfig) -> Self {
        let events = Arc::new(EventBus::new(config.event_retention_events));
        Self {
            config,
            server_id: ServerId::new(),
            started_at_millis: now_millis(),
            events,
            sessions: Mutex::new(BTreeMap::new()),
            clients: Mutex::new(BTreeMap::new()),
            connections: Mutex::new(Vec::new()),
            connection_threads: Mutex::new(Vec::new()),
            shutdown: AtomicBool::new(false),
        }
    }

    /// The log head sequence; the same total order used by snapshots and events.
    pub(crate) fn sequence(&self) -> u64 {
        self.events.current_sequence()
    }

    /// Create a session with a default window and terminal block, spawning the
    /// child immediately. The child is owned by the registry, never by a client.
    pub(crate) fn create_session(&self, label: Option<String>) -> Result<SessionCreateData> {
        let session_id = SessionId::new();
        let window_id = WindowId::new();
        let block_id = BlockId::new();
        let pty_process_id = PtyProcessId::new();
        let initial_size = self.config.initial_size;

        let mut environment: Vec<(OsString, OsString)> = Vec::new();
        if std::env::var_os("TERM").is_none() {
            environment.push((OsString::from("TERM"), OsString::from("xterm-256color")));
        }

        let pty = relayable_pty::PtyHandle::spawn(
            self.config.shell.as_os_str(),
            &[],
            relayable_pty::PtySize {
                cols: initial_size.cols,
                rows: initial_size.rows,
                pixel_width: initial_size.pixel_width,
                pixel_height: initial_size.pixel_height,
            },
            None,
            &environment,
        )
        .context("failed to spawn the session shell")?;

        let block = Arc::new(
            BlockRuntime::new(
                block_id,
                window_id,
                session_id,
                pty_process_id,
                pty,
                BlockInit {
                    initial_size,
                    output_tail_bytes: self.config.output_tail_bytes,
                    events: Some(Arc::clone(&self.events)),
                },
            )
            .context("failed to start the block runtime")?,
        );
        if let Err(error) = block.start_threads() {
            // Do not leak a spawned child if the block threads cannot start.
            block.close(Duration::from_millis(200));
            return Err(error.context("failed to start the block threads"));
        }

        let program_pid = block.program_pid;
        let session = Arc::new(SessionRuntime::new(
            session_id,
            label,
            now_millis(),
            window_id,
            block,
        ));
        let record = snapshot::session_record(&session);

        // Insert first, then append the event outside every entity lock (§C).
        lock(&self.sessions).insert(session_id, session);
        self.events.append(session_state_event(
            &session_id.to_string(),
            record.revision,
            SessionState::Active,
        ));

        Ok(SessionCreateData {
            session: record,
            window_id,
            block_id,
            pty_process_id,
            program_pid,
        })
    }

    /// Active sessions only; closed tombstones are excluded from `session.list`.
    pub(crate) fn list_active_sessions(&self) -> Vec<SessionSummary> {
        lock(&self.sessions)
            .values()
            .filter(|session| !session.is_closed())
            .map(|session| SessionSummary {
                session_id: session.id.to_string(),
                label: session.label.clone(),
                state: SessionState::Active as i32,
                revision: session.revision(),
            })
            .collect()
    }

    pub(crate) fn snapshot(
        &self,
        session_id: SessionId,
    ) -> Option<relayable_protocol::v1::SessionSnapshotResponse> {
        let session = lock(&self.sessions).get(&session_id).cloned()?;
        Some(snapshot::build(self, &session))
    }

    /// Close a session idempotently, retaining a tombstone record.
    ///
    /// The session is marked CLOSED only when the child is confirmed reaped;
    /// otherwise it is left ACTIVE (subscribers and lease intact) so a retry is
    /// possible.
    pub(crate) fn close_session(&self, session_id: SessionId) -> CloseSessionOutcome {
        let Some(session) = lock(&self.sessions).get(&session_id).cloned() else {
            return CloseSessionOutcome::Unknown;
        };
        if session.is_closed() {
            return CloseSessionOutcome::Closed(snapshot::session_record(&session));
        }
        let outcome = session.block.close(Duration::from_millis(500));
        if !outcome.reaped {
            return CloseSessionOutcome::NotConfirmed {
                pid: session.block.program_pid,
            };
        }
        if let Some((client_id, stream_id)) = outcome.released_lease {
            self.events.append(writer_lease_event(
                &session.id.to_string(),
                &session.block.id.to_string(),
                stream_id,
                session.block.revision(),
                &client_id.to_string(),
                "",
                false,
            ));
        }
        if let Some(status) = &outcome.status {
            eprintln!(
                "relayable-server: session {session_id} child exited (code {}, signal {:?})",
                status.exit_code, status.signal
            );
            session.block.emit_child_exited(status);
        }
        session.mark_closed();
        self.events.append(session_state_event(
            &session.id.to_string(),
            session.revision(),
            SessionState::Closed,
        ));
        CloseSessionOutcome::Closed(snapshot::session_record(&session))
    }

    /// Look up a block in a live (non-closed) session only.
    pub(crate) fn find_block(&self, block_id: BlockId) -> Option<Arc<BlockRuntime>> {
        lock(&self.sessions)
            .values()
            .filter(|session| !session.is_closed())
            .find(|session| session.block.id == block_id)
            .map(|session| Arc::clone(&session.block))
    }

    /// Every live client record visible in a snapshot.
    ///
    /// Disconnected clients are removed from the registry on disconnect, so
    /// this lists live clients only (CORE-001 snapshot semantics).
    pub(crate) fn client_records(&self) -> Vec<ClientRecord> {
        lock(&self.clients)
            .values()
            .map(snapshot::client_record)
            .collect()
    }

    /// All live sessions, for graceful shutdown.
    pub(crate) fn live_sessions(&self) -> Vec<Arc<SessionRuntime>> {
        lock(&self.sessions)
            .values()
            .filter(|session| !session.is_closed())
            .cloned()
            .collect()
    }

    pub(crate) fn request_shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }

    /// Insert a fresh client record in the initial transport state.
    pub(crate) fn insert_client(&self, client: Client) {
        lock(&self.clients).insert(client.id, client);
    }

    /// Apply a completed handshake and emit `client_connection_state_changed`
    /// (`negotiated`).
    pub(crate) fn client_negotiated(&self, client_id: ClientId, handshake: &Handshake) {
        let changed = {
            let mut clients = lock(&self.clients);
            match clients.get_mut(&client_id) {
                Some(client) => {
                    client.kind = handshake.client_kind.clone();
                    client.platform = handshake.platform.clone();
                    client.version = handshake.client_version.clone();
                    client.connection_state = ConnectionState::Negotiated;
                    true
                }
                None => false,
            }
        };
        if changed {
            self.events.append(client_connection_state_event(
                &client_id,
                ConnectionState::Negotiated.as_str(),
            ));
        }
    }

    /// Mark a client `Ready` after a successful view/resume and emit
    /// `client_connection_state_changed` (`ready`) on a real transition only.
    pub(crate) fn client_ready(&self, client_id: ClientId) {
        let changed = {
            let mut clients = lock(&self.clients);
            match clients.get_mut(&client_id) {
                Some(client) if client.connection_state != ConnectionState::Ready => {
                    client.connection_state = ConnectionState::Ready;
                    true
                }
                _ => false,
            }
        };
        if changed {
            self.events.append(client_connection_state_event(
                &client_id,
                ConnectionState::Ready.as_str(),
            ));
        }
    }

    /// Emit the two ordered disconnect events and then remove the client
    /// record:
    ///
    /// 1. `client_connection_state_changed{disconnected}`
    /// 2. `client_disconnected{reason}`
    ///
    /// The transition is observable through that event pair; the record itself
    /// is removed so a later snapshot lists live clients only (CORE-001
    /// snapshot semantics). `flow_control_overflow` and `server_shutdown`
    /// disconnect through this same path, so their records are removed too.
    pub(crate) fn client_disconnected(&self, client_id: ClientId, reason: DisconnectReason) {
        self.events.append_pair(
            client_connection_state_event(&client_id, ConnectionState::Disconnected.as_str()),
            client_disconnected_event(&client_id, reason),
        );
        lock(&self.clients).remove(&client_id);
    }

    /// Handle `session.resume` under the single bus-lock critical section.
    ///
    /// Holds the bus lock across cursor validation, payload construction (the
    /// snapshot reads the sessions map, the allowed exception), response
    /// enqueue, and subscriber registration, so the frozen §C "no gap, no
    /// duplicate" ordering holds.
    ///
    /// Returns `Ok(true)` when the response frame was enqueued, `Ok(false)` when
    /// the queue was full (the caller drops the connection after the lock is
    /// released), or `Err` for an unknown session.
    pub(crate) fn resume_session(
        &self,
        request_id: u64,
        session_id: SessionId,
        last_sequence_applied: u64,
        connection: &Arc<ConnectionShared>,
    ) -> Result<bool, ErrorCode> {
        let mut state = self.events.lock_state();
        let current = state.current();
        let floor = state.floor();
        let retained_event_count = u32::try_from(state.count()).unwrap_or(u32::MAX);
        let Some(session) = lock(&self.sessions).get(&session_id).cloned() else {
            return Err(ErrorCode::UnknownEntity);
        };

        let response = match decide_resume(last_sequence_applied, current, floor) {
            ResumeDecision::Replay => SessionResumeResponse {
                snapshot_required: false,
                snapshot_required_reason: String::new(),
                snapshot: None,
                events: state
                    .events
                    .iter()
                    .filter(|event| event.sequence > last_sequence_applied)
                    .cloned()
                    .collect(),
                snapshot_sequence: last_sequence_applied,
                current_sequence: current,
                retained_floor_sequence: floor,
                retained_event_count,
            },
            ResumeDecision::Snapshot(reason) => {
                let snapshot = snapshot::build(self, &session);
                SessionResumeResponse {
                    snapshot_required: true,
                    snapshot_required_reason: reason.to_string(),
                    snapshot: Some(snapshot),
                    events: Vec::new(),
                    snapshot_sequence: current,
                    current_sequence: current,
                    retained_floor_sequence: floor,
                    retained_event_count,
                }
            }
        };

        let envelope = Envelope {
            protocol_version: relayable_protocol::PROTOCOL_VERSION.to_string(),
            request_id: Some(request_id),
            stream_id: 0,
            sequence: None,
            trace_id: None,
            timestamp_unix_millis: None,
            body: Some(v1::envelope::Body::Response(v1::Response {
                request_id,
                result: Some(v1::response::Result::SessionResume(response)),
            })),
        };
        let enqueued = connection.enqueue_envelope(envelope).is_ok();
        if enqueued {
            state.subscribe(connection);
        }
        Ok(enqueued)
    }
}

/// Current Unix-epoch time in milliseconds; informational only.
pub(crate) fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use relayable_protocol::v1::event;

    fn test_config() -> ServerConfig {
        ServerConfig {
            socket_path: std::env::temp_dir().join("relayable-registry-test.sock"),
            shell: OsString::from("/bin/sh"),
            initial_size: relayable_model::TerminalSize::DEFAULT_SIZE,
            limits: relayable_protocol::limits::DEFAULT_LIMITS,
            output_tail_bytes: 32_768,
            event_retention_events: 4096,
        }
    }

    #[test]
    fn create_list_view_close_round_trip() {
        let shared = ServerShared::new(test_config());
        let created = shared
            .create_session(Some("demo".to_string()))
            .expect("create");
        let session_id = created.session.session_id.clone();

        // list includes the new session
        let listed = shared.list_active_sessions();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].session_id, session_id);
        assert_eq!(listed[0].state, SessionState::Active as i32);
        assert_eq!(listed[0].revision, 1);

        // snapshot exposes the block and its geometry/status
        let parsed = session_id.parse().expect("session id");
        let snapshot = shared.snapshot(parsed).expect("snapshot");
        assert_eq!(snapshot.blocks.len(), 1);
        let block = &snapshot.blocks[0];
        assert_eq!(block.cols, 80);
        assert_eq!(block.rows, 24);
        assert_eq!(
            block.program_status.as_ref().expect("status").state,
            relayable_protocol::v1::ProgramState::Running as i32
        );
        assert_eq!(block.pty_process_id, created.pty_process_id.to_string());
        assert_eq!(block.program_pid, created.program_pid.unwrap_or(0));

        // close is idempotent and removes the session from `list`
        let closed = match shared.close_session(parsed) {
            CloseSessionOutcome::Closed(record) => record,
            _ => panic!("expected a confirmed close"),
        };
        assert_eq!(closed.state, SessionState::Closed as i32);
        assert!(shared.list_active_sessions().is_empty());
        let closed_again = match shared.close_session(parsed) {
            CloseSessionOutcome::Closed(record) => record,
            _ => panic!("expected an idempotent close"),
        };
        assert_eq!(closed_again.state, SessionState::Closed as i32);

        // a closed session's block is no longer resolvable and its session is
        // still viewable as a tombstone
        assert!(shared.find_block(created.block_id).is_none());
        assert!(shared.snapshot(parsed).is_some());

        // unknown ids are absent
        assert!(shared.snapshot(SessionId::new()).is_none());
        assert!(matches!(
            shared.close_session(SessionId::new()),
            CloseSessionOutcome::Unknown
        ));

        // create/close each emitted `session_state_changed`
        assert!(shared.sequence() >= 2);
    }

    #[test]
    fn close_session_is_not_confirmed_when_child_survives() {
        let shared = ServerShared::new(test_config());
        let created = shared.create_session(None).expect("create");
        let parsed = created.session.session_id.parse().expect("session id");
        let block = shared.find_block(created.block_id).expect("block");

        // Force `close` to report an unconfirmed exit without killing the child.
        block.set_force_unreaped(true);
        match shared.close_session(parsed) {
            CloseSessionOutcome::NotConfirmed { pid } => {
                assert_eq!(pid, created.program_pid);
            }
            _ => panic!("expected NotConfirmed"),
        }
        // The session stays ACTIVE and is still listed.
        assert_eq!(shared.list_active_sessions().len(), 1);
        let snapshot = shared.snapshot(parsed).expect("still viewable");
        assert_eq!(
            snapshot.session.expect("session").state,
            SessionState::Active as i32
        );

        // Cleanup: real close now that the hook is off.
        block.set_force_unreaped(false);
        assert!(matches!(
            shared.close_session(parsed),
            CloseSessionOutcome::Closed(_)
        ));
    }

    #[test]
    fn disconnected_client_records_are_removed() {
        let shared = ServerShared::new(test_config());
        let client_id = ClientId::new();
        shared.insert_client(Client {
            id: client_id,
            kind: "test".to_string(),
            platform: "linux".to_string(),
            version: "0".to_string(),
            connection_state: ConnectionState::Ready,
            principal: None,
            connected_at: relayable_model::Timestamp::new(1),
        });
        assert_eq!(shared.client_records().len(), 1);

        shared.client_disconnected(client_id, DisconnectReason::PeerClosed);
        assert!(
            shared.client_records().is_empty(),
            "a disconnected record must not be listed in a snapshot"
        );
    }

    #[test]
    fn disconnect_emits_two_ordered_events() {
        let shared = ServerShared::new(test_config());
        let client_id = ClientId::new();
        shared.insert_client(Client {
            id: client_id,
            kind: "test".to_string(),
            platform: "linux".to_string(),
            version: "0".to_string(),
            connection_state: ConnectionState::Ready,
            principal: None,
            connected_at: relayable_model::Timestamp::new(1),
        });
        shared.client_disconnected(client_id, DisconnectReason::IdleTimeout);

        {
            let state = shared.events.lock_state();
            let events: Vec<_> = state.events.iter().collect();
            assert_eq!(events.len(), 2, "exactly two ordered disconnect events");
            assert_eq!(
                events[1].sequence,
                events[0].sequence + 1,
                "the disconnect pair must be adjacent"
            );
            match events[0].event.as_ref().expect("state event") {
                event::Event::ClientConnectionStateChanged(change) => {
                    assert_eq!(change.client_id, client_id.to_string());
                    assert_eq!(change.connection_state, "Disconnected");
                }
                other => panic!("expected client_connection_state_changed, got {other:?}"),
            }
            match events[1].event.as_ref().expect("disconnect event") {
                event::Event::ClientDisconnected(change) => {
                    assert_eq!(change.client_id, client_id.to_string());
                    assert_eq!(change.reason, "idle_timeout");
                }
                other => panic!("expected client_disconnected, got {other:?}"),
            }
        }
        // The transition is observable through the pair, then the record goes.
        assert!(shared.client_records().is_empty());
    }

    #[test]
    fn session_lifecycle_emits_state_change_events() {
        let shared = ServerShared::new(test_config());
        let created = shared
            .create_session(Some("events".to_string()))
            .expect("create");
        let parsed = created.session.session_id.parse().expect("session id");

        let active = latest_session_state(&shared).expect("create event");
        assert_eq!(active.state, SessionState::Active as i32);

        assert!(matches!(
            shared.close_session(parsed),
            CloseSessionOutcome::Closed(_)
        ));
        let closed = latest_session_state(&shared).expect("close event");
        assert_eq!(closed.state, SessionState::Closed as i32);
    }

    /// The most recent `session_state_changed` payload in the retained log.
    fn latest_session_state(
        shared: &ServerShared,
    ) -> Option<relayable_protocol::v1::SessionStateChanged> {
        shared
            .events
            .lock_state()
            .events
            .iter()
            .rev()
            .find_map(|event| match &event.event {
                Some(event::Event::SessionStateChanged(change)) => Some(change.clone()),
                _ => None,
            })
    }
}
