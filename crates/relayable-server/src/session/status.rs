//! The block runtime's small value types: normalized program status and the
//! single writer lease.

use relayable_model::{ClientId, ProgramState};
use relayable_protocol::v1::ProgramStatus;
use relayable_pty::PtyExitStatus;

/// Normalized program status tracked by the block runtime.
#[derive(Debug, Clone)]
pub(super) struct StoredStatus {
    state: ProgramState,
    exit_code: Option<u32>,
    signal: Option<String>,
}

impl StoredStatus {
    pub(super) fn running() -> Self {
        Self {
            state: ProgramState::Running,
            exit_code: None,
            signal: None,
        }
    }

    pub(super) fn exited(status: &PtyExitStatus) -> Self {
        Self {
            state: ProgramState::Exited,
            exit_code: Some(status.exit_code),
            signal: status.signal.clone(),
        }
    }

    pub(super) fn to_wire(&self) -> ProgramStatus {
        ProgramStatus {
            state: match self.state {
                ProgramState::Running => relayable_protocol::v1::ProgramState::Running as i32,
                ProgramState::Exited => relayable_protocol::v1::ProgramState::Exited as i32,
            },
            exit_code: self.exit_code,
            signal: self.signal.clone(),
            pwd: None,
            title: None,
        }
    }
}

/// The single writer lease for a block, held by one `(client, stream)` pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WriterLease {
    pub(super) client_id: ClientId,
    pub(super) stream_id: u64,
}
