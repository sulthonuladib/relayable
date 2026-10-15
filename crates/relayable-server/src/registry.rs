//! Server-wide registry: sessions, clients, connections, and the global
//! sequence. This is the single authority for server-owned state.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use relayable_model::{BlockId, Client, ClientId, PtyProcessId, ServerId, SessionId, WindowId};
use relayable_protocol::v1::{SessionRecord, SessionState, SessionSummary};

use crate::config::ServerConfig;
use crate::connection::ConnectionShared;
use crate::lock;
use crate::session::{BlockRuntime, SessionRuntime};
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
    sequence: AtomicU64,
    pub(crate) sessions: Mutex<BTreeMap<SessionId, Arc<SessionRuntime>>>,
    pub(crate) clients: Mutex<BTreeMap<ClientId, Client>>,
    pub(crate) connections: Mutex<Vec<Arc<ConnectionShared>>>,
    pub(crate) connection_threads: Mutex<Vec<JoinHandle<()>>>,
    pub(crate) shutdown: AtomicBool,
}

impl ServerShared {
    pub(crate) fn new(config: ServerConfig) -> Self {
        Self {
            config,
            server_id: ServerId::new(),
            started_at_millis: now_millis(),
            sequence: AtomicU64::new(0),
            sessions: Mutex::new(BTreeMap::new()),
            clients: Mutex::new(BTreeMap::new()),
            connections: Mutex::new(Vec::new()),
            connection_threads: Mutex::new(Vec::new()),
            shutdown: AtomicBool::new(false),
        }
    }

    pub(crate) fn sequence(&self) -> u64 {
        self.sequence.load(Ordering::SeqCst)
    }

    fn bump_sequence(&self) -> u64 {
        self.sequence.fetch_add(1, Ordering::SeqCst) + 1
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
                initial_size,
                pty,
                self.config.output_tail_bytes,
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

        self.bump_sequence();
        lock(&self.sessions).insert(session_id, session);

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
        if let Some(status) = &outcome.status {
            eprintln!(
                "relayable-server: session {session_id} child exited (code {}, signal {:?})",
                status.exit_code, status.signal
            );
        }
        session.mark_closed();
        self.bump_sequence();
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

    pub(crate) fn client_records(&self) -> Vec<relayable_protocol::v1::ClientRecord> {
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

    fn test_config() -> ServerConfig {
        ServerConfig {
            socket_path: std::env::temp_dir().join("relayable-registry-test.sock"),
            shell: OsString::from("/bin/sh"),
            initial_size: relayable_model::TerminalSize::DEFAULT_SIZE,
            limits: relayable_protocol::limits::DEFAULT_LIMITS,
            output_tail_bytes: 32_768,
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
}
