use std::time::Duration;

/// One bounded reconnect sweep: at most this many attempts.
pub const RECONNECT_MAX_ATTEMPTS: u32 = 3;
/// Fixed backoff between reconnect attempts.
pub const RECONNECT_BACKOFF: Duration = Duration::from_millis(500);
/// Per-plane read poll interval; keeps command handling responsive.
pub(super) const POLL_INTERVAL: Duration = Duration::from_millis(100);
/// Window for the data plane's one bounded blocking read per pump.
///
/// The read pumps the socket for this long (`Client::collect_events`) and the
/// pump then acknowledges everything it pulled, for every attached stream. The
/// window bounds a stream's unacknowledged volume to its production during one
/// window, and the server closes the whole connection when a stream overruns its
/// output bounds: the per-stream credit window (`credit_window_bytes`, 256 KiB)
/// or the 8192-message outbound queue (`OUTBOUND_QUEUE_MESSAGES`). Measured
/// fan-out on this host is well inside those bounds at 1 ms: the 400 KiB
/// acceptance burst delivered 400290 bytes at `delivered_in_ms=40 rate_mb_s=9.4`
/// — the only committed run of that measurement, `evidence/ui-003-live-probe.log`
/// line 170 — so at most roughly 9 KiB of output is outstanding when a window
/// closes (one window of production at that measured rate). 1 ms also bounds the
/// added latency of one output batch to one millisecond, far below a rendered
/// frame.
///
/// Cost when idle: one `set_read_timeout` + `recv` per millisecond on the data
/// thread, and no work at all for streams that are already drained. The window is
/// deliberately not [`POLL_INTERVAL`]: a 100 ms window would let a 2.6 MB/s
/// producer spend the whole credit window, and losing the connection detaches
/// every pane.
///
/// This window cannot repair the one flow-control case the measurement found:
/// a child writing tiny pieces fast (line-buffered `yes` produced ~15-47 byte
/// frames at a rate the client could not acknowledge) out-runs the client's ack
/// rate, because acknowledging a frame costs the client two `recv` calls plus a
/// `FlowControl` write while the server produces a frame with one `read`; the
/// connection then closes on the first output bound the stream crosses. In all
/// four recorded runs the server logged `stream 2 outbound channel full` — the
/// bounded outbound queue — while the credit window's `credit exhausted` line does
/// not appear in them; both bounds live in the same `Subscriber::deliver`. That is
/// the per-chunk ack model of the protocol, not this module's loop; see
/// `evidence/ui-003-flow-control.log`.
pub(super) const DATA_READ_WINDOW: Duration = Duration::from_millis(1);
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
    ///
    /// **No per-chunk sequence number.** The previous raw data plane copied
    /// `TerminalOutput.stream_seq` into this event; the data plane now reads
    /// through `relayable_client::Client::next_output`, which returns the bytes
    /// only and exposes no sequence, and no consumer in this tree reads one. A
    /// counter reconstructed here could not match the server's numbering, so the
    /// field is gone instead of being invented. Order is still preserved: chunks
    /// are emitted in the order the client returns them.
    Output {
        /// Source stream id.
        stream_id: u64,
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
        /// Session wire id the block belongs to (`session:<uuid>`).
        ///
        /// `block.attach` is addressed by block id alone; the session travels with
        /// the command so the data plane can name the attach through
        /// `relayable_client::Client::attach_with_takeover`, which takes both ids.
        session_id: String,
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
