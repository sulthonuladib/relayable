use crate::ids::{
    BlockId, ClientId, PtyProcessId, Revision, Sequence, ServerId, SessionId, Timestamp, WindowId,
};
use crate::size::TerminalSize;
use crate::state::{BlockKind, ConnectionState, ProgramStatus, SessionState};

/// A server-owned session group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// Stable identity.
    pub id: SessionId,
    /// Mutable human label; never identity.
    pub label: Option<String>,
    /// Optimistic-concurrency revision.
    pub revision: Revision,
    /// Lifecycle state.
    pub state: SessionState,
    /// Creation time.
    pub created_at: Timestamp,
}

/// A window within a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    /// Stable identity.
    pub id: WindowId,
    /// Owning session.
    pub session_id: SessionId,
    /// Mutable human label; never identity.
    pub label: Option<String>,
    /// Optimistic-concurrency revision.
    pub revision: Revision,
    /// Whether the window is active.
    pub active: bool,
}

/// A block within a window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// Stable identity.
    pub id: BlockId,
    /// Owning window.
    pub window_id: WindowId,
    /// Owning session (scoping; ADR-001 Decision C).
    pub session_id: SessionId,
    /// Block kind.
    pub kind: BlockKind,
    /// Mutable human label; never identity.
    pub label: Option<String>,
    /// Optimistic-concurrency revision.
    pub revision: Revision,
    /// Terminal size.
    pub size: TerminalSize,
    /// Normalized program status.
    pub program: ProgramStatus,
    /// Server-owned PTY/process identity.
    pub pty_process_id: PtyProcessId,
    /// The OS process id of the program, when known.
    pub program_pid: Option<u32>,
    /// Whether a writer lease is currently attached.
    pub writer_attached: bool,
}

/// An attached client record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Client {
    /// Stable identity.
    pub id: ClientId,
    /// Client kind (for example a platform-agnostic name).
    pub kind: String,
    /// Client platform.
    pub platform: String,
    /// Client version.
    pub version: String,
    /// Connection lifecycle state.
    pub connection_state: ConnectionState,
    /// Authenticated principal, when known (mechanism deferred).
    pub principal: Option<String>,
    /// When the client connected.
    pub connected_at: Timestamp,
}

/// A server instance descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerInfo {
    /// Stable identity.
    pub id: ServerId,
    /// Advertised protocol version, `major.minor`.
    pub protocol_version: String,
    /// Current server-wide sequence.
    pub sequence: Sequence,
    /// When the server started.
    pub started_at: Timestamp,
}
