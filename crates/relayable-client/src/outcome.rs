use relayable_model::{BlockId, PtyProcessId, WindowId};
use relayable_protocol::v1::{AttachMode, SessionRecord, TerminalSnapshot};

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
