use relayable_model::SessionId;
use relayable_protocol::ids;
use relayable_protocol::methods;
use relayable_protocol::v1::{
    request::Operation, response::Result as ResponseResult, SessionCreateRequest,
    SessionListResponse, SessionRecord, SessionResumeRequest, SessionResumeResponse,
    SessionSnapshotRequest, SessionSnapshotResponse, SessionSummary,
};

use crate::client::Client;
use crate::error::ClientError;
use crate::outcome::CreatedSession;
use crate::rpc::{missing, parse_id, unexpected};

impl Client {
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
}
