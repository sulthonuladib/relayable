use std::fmt;

/// Lifecycle state of a session (contract §2 `SessionRecord.state`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionState {
    /// The session is live.
    Active,
    /// The session has been closed.
    Closed,
}

/// Kind of a block. Only `Terminal` exists in the first release (contract §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockKind {
    /// A terminal block.
    Terminal,
}

/// Lifecycle state of the program running inside a terminal block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProgramState {
    /// The program is running.
    Running,
    /// The program has exited.
    Exited,
}

/// Normalized program status reported by a terminal.
///
/// This is a value type; it is **not** an OSC parser. Ingestion of terminal
/// status bytes belongs to the terminal layer (ADR-001 "ProgramStatus").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramStatus {
    /// Current program state.
    pub state: ProgramState,
    /// Child exit code, when the program has exited normally.
    pub exit_code: Option<u32>,
    /// Terminating signal name, when the program was signalled.
    pub signal: Option<String>,
    /// Last known working directory.
    pub pwd: Option<String>,
    /// Last known title.
    pub title: Option<String>,
}

/// Connection lifecycle state (contract §4).
///
/// The state names are Relayable-defined (no REX evidence); the field name
/// `connection_state` is reused verbatim from the reference docs (matrix §12,
/// §13). `as_str` returns exactly the spelling in the contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    /// Initial state, or reached when the transport drops.
    Disconnected,
    /// Bytes flow but no semantics are negotiated yet.
    TransportConnected,
    /// Handshake reply exchanged; version and capabilities agree.
    Negotiated,
    /// Authentication accepted.
    Authenticated,
    /// Snapshot requested and in flight.
    Synchronizing,
    /// Live state is usable.
    Ready,
    /// A stale sequence or reconnect forced resynchronization.
    Resynchronizing,
    /// Client shutdown in progress; draining only.
    Closing,
    /// Terminal state reached after transport close.
    Closed,
    /// Terminal failure with a recorded reason.
    Failed(String),
}

impl ConnectionState {
    /// The exact contract §4 spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Disconnected => "Disconnected",
            Self::TransportConnected => "TransportConnected",
            Self::Negotiated => "Negotiated",
            Self::Authenticated => "Authenticated",
            Self::Synchronizing => "Synchronizing",
            Self::Ready => "Ready",
            Self::Resynchronizing => "Resynchronizing",
            Self::Closing => "Closing",
            Self::Closed => "Closed",
            Self::Failed(_) => "Failed",
        }
    }
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failed(reason) => write!(f, "Failed({reason})"),
            other => f.write_str(other.as_str()),
        }
    }
}
