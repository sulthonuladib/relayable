use std::time::Duration;

/// One bounded reconnect sweep: at most this many attempts.
pub const RECONNECT_MAX_ATTEMPTS: u32 = 3;
/// Fixed backoff between reconnect attempts.
pub const RECONNECT_BACKOFF: Duration = Duration::from_millis(500);
/// Per-plane read poll interval; keeps command handling responsive.
pub(super) const POLL_INTERVAL: Duration = Duration::from_millis(100);
/// Deadline for a single request/response round trip.
pub(super) const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Bound applied to an attach-time recovery snapshot before it is surfaced.
pub(super) const SNAPSHOT_TAIL_BYTES: usize = 32 * 1024;

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
