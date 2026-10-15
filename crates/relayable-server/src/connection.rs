//! Per-connection protocol front end.
//!
//! Each accepted connection gets a reader thread (owns a `FrameConnection`
//! clone, validates the handshake, and dispatches inbound frames) and a writer
//! thread (owns another clone, drains a bounded outbound queue, and emits
//! idle heartbeats). Shared state is behind `Arc<Mutex<..>>` with short lock
//! scopes; no lock is held across a blocking socket write.

use std::collections::{HashMap, VecDeque};
use std::io::ErrorKind;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use relayable_model::{Client, ClientId, ConnectionState, Timestamp};
use relayable_protocol::transport::{FrameConnection, TransportError};
use relayable_protocol::v1::{
    envelope::Body, request, response, Envelope, Error, ErrorCode, Heartbeat, ServerHello,
};
use relayable_protocol::PROTOCOL_VERSION;

use crate::dispatch;
use crate::events::{writer_lease_event, DisconnectReason};
use crate::lock;
use crate::registry::ServerShared;
use crate::session::BlockRuntime;

/// Maximum frames queued for a connection's writer thread.
///
/// Payload bytes are independently bounded by the per-stream credit window; this
/// is a secondary bound that absorbs a burst of many small PTY-output frames
/// without closing a client that is actively reading. A client that stays
/// behind is closed once the queue is full (see `Subscriber::deliver`).
const OUTBOUND_QUEUE_MESSAGES: usize = 8192;

/// A message for the connection's writer thread.
pub(crate) enum WriterMsg {
    /// A fully-formed envelope to serialize and write.
    Frame(Box<Envelope>),
    /// Flush what is queued, then stop the writer.
    Close,
}

/// State shared between a connection's reader and writer threads, plus a socket
/// handle used to force both out of a blocking read/write.
pub(crate) struct ConnectionShared {
    pub(crate) tx: SyncSender<WriterMsg>,
    close: AtomicBool,
    reason: Mutex<Option<DisconnectReason>>,
    stream: UnixStream,
}

impl ConnectionShared {
    /// Ask the writer to flush and stop.
    pub(crate) fn request_close(&self) {
        let _ = self.tx.try_send(WriterMsg::Close);
    }

    /// Non-blocking enqueue of an event or response envelope into this
    /// connection's writer queue.
    ///
    /// Used by the event-bus fan-out while the bus lock is held, so it must
    /// never block and never re-enter teardown. `Err` means the bounded queue
    /// is full or the writer is gone; the caller drops the connection after
    /// releasing the bus lock.
    pub(crate) fn enqueue_envelope(
        &self,
        envelope: Envelope,
    ) -> Result<(), TrySendError<WriterMsg>> {
        self.tx.try_send(WriterMsg::Frame(Box::new(envelope)))
    }

    /// Abort the connection promptly: record the first disconnect `reason`,
    /// mark it closed, queue a stop, and shut the socket down so a blocked
    /// `recv`/`send` returns immediately. The session and child are untouched.
    pub(crate) fn close_now(&self, reason: DisconnectReason) {
        {
            let mut current = lock(&self.reason);
            if current.is_none() {
                *current = Some(reason);
            }
        }
        self.close.store(true, Ordering::SeqCst);
        let _ = self.tx.try_send(WriterMsg::Close);
        let _ = self.stream.shutdown(Shutdown::Both);
    }

    /// The recorded disconnect reason, if `close_now` ran.
    pub(crate) fn take_reason(&self) -> Option<DisconnectReason> {
        lock(&self.reason).take()
    }
}

/// Bounded duplicate-request window: the most recent `capacity` request ids.
pub(crate) struct RequestWindow {
    seen: VecDeque<u64>,
    capacity: usize,
}

impl RequestWindow {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            seen: VecDeque::new(),
            capacity: capacity.max(1),
        }
    }

    /// Record `id`; returns `false` when it is a duplicate in the window.
    pub(crate) fn record(&mut self, id: u64) -> bool {
        if self.seen.contains(&id) {
            return false;
        }
        if self.seen.len() == self.capacity {
            self.seen.pop_front();
        }
        self.seen.push_back(id);
        true
    }
}

/// A stream this connection has attached to a block.
#[derive(Clone)]
pub(crate) struct StreamEntry {
    pub(crate) block: Arc<BlockRuntime>,
    pub(crate) writer: bool,
}

/// Reader-thread-local state for one connection.
pub(crate) struct ConnectionCtx {
    pub(crate) shared: Arc<ServerShared>,
    pub(crate) client_id: ClientId,
    pub(crate) conn: Arc<ConnectionShared>,
    tx: SyncSender<WriterMsg>,
    pub(crate) dedup: Mutex<RequestWindow>,
    pub(crate) next_stream: AtomicU64,
    pub(crate) streams: Mutex<HashMap<u64, StreamEntry>>,
}

impl ConnectionCtx {
    fn send(&self, envelope: Envelope) {
        match self.tx.try_send(WriterMsg::Frame(Box::new(envelope))) {
            Ok(()) => {}
            Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => {
                self.conn.close_now(DisconnectReason::TransportError);
            }
        }
    }

    pub(crate) fn send_result(&self, request_id: u64, result: response::Result) {
        self.send(Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: Some(request_id),
            stream_id: 0,
            sequence: None,
            trace_id: None,
            timestamp_unix_millis: None,
            body: Some(Body::Response(relayable_protocol::v1::Response {
                request_id,
                result: Some(result),
            })),
        });
    }

    pub(crate) fn send_error(
        &self,
        request_id: u64,
        code: ErrorCode,
        message: &str,
        details: HashMap<String, String>,
        retryable: bool,
    ) {
        self.send_result(
            request_id,
            response::Result::Error(Error {
                code: code as i32,
                message: message.to_string(),
                details,
                retryable,
            }),
        );
    }

    /// Release every stream this connection attached and free the writer leases.
    ///
    /// Each freed lease emits `writer_lease_changed` after the entity lock is
    /// released (never while holding an entity lock).
    pub(crate) fn release_streams(&self) {
        let entries: Vec<(u64, StreamEntry)> = {
            let mut streams = lock(&self.streams);
            streams.drain().collect()
        };
        for (stream_id, entry) in entries {
            entry.block.remove_subscriber(self.client_id, stream_id);
            if entry.writer && entry.block.release_writer(self.client_id, stream_id) {
                self.shared.events.append(writer_lease_event(
                    &entry.block.session_id.to_string(),
                    &entry.block.id.to_string(),
                    stream_id,
                    entry.block.revision(),
                    &self.client_id.to_string(),
                    "",
                    false,
                ));
            }
        }
    }
}

/// Spawn a reader thread for an accepted connection.
pub(crate) fn spawn_connection(shared: Arc<ServerShared>, stream: UnixStream) -> JoinHandle<()> {
    thread::spawn(move || run_connection(shared, stream))
}

fn run_connection(shared: Arc<ServerShared>, stream: UnixStream) {
    let client_id = ClientId::new();
    shared.insert_client(Client {
        id: client_id,
        kind: String::new(),
        platform: String::new(),
        version: String::new(),
        connection_state: ConnectionState::TransportConnected,
        principal: None,
        connected_at: Timestamp::new(crate::registry::now_millis()),
    });

    let max_frame_bytes = shared.config.limits.max_frame_bytes;
    let read_timeout = Duration::from_millis(u64::from(shared.config.limits.idle_timeout_millis));
    let heartbeat =
        Duration::from_millis(u64::from(shared.config.limits.heartbeat_interval_millis));

    let shutdown_stream = match stream.try_clone() {
        Ok(clone) => clone,
        Err(_) => {
            shared.client_disconnected(client_id, DisconnectReason::TransportError);
            return;
        }
    };
    let reader_stream = match stream.try_clone() {
        Ok(clone) => clone,
        Err(_) => {
            shared.client_disconnected(client_id, DisconnectReason::TransportError);
            return;
        }
    };

    let (tx, rx) = mpsc::sync_channel::<WriterMsg>(OUTBOUND_QUEUE_MESSAGES);
    let conn = Arc::new(ConnectionShared {
        tx: tx.clone(),
        close: AtomicBool::new(false),
        reason: Mutex::new(None),
        stream: shutdown_stream,
    });
    lock(&shared.connections).push(Arc::clone(&conn));

    let writer_handle = thread::spawn({
        let conn = Arc::clone(&conn);
        let shared = Arc::clone(&shared);
        move || {
            writer_loop(
                FrameConnection::from_stream(stream, max_frame_bytes),
                rx,
                conn,
                shared,
                heartbeat,
            )
        }
    });

    let mut reader = FrameConnection::from_stream(reader_stream, max_frame_bytes);
    let _ = reader.set_read_timeout(Some(read_timeout));

    let ctx = ConnectionCtx {
        shared: Arc::clone(&shared),
        client_id,
        conn: Arc::clone(&conn),
        tx,
        dedup: Mutex::new(RequestWindow::new(64)),
        next_stream: AtomicU64::new(0),
        streams: Mutex::new(HashMap::new()),
    };

    // The first close reason wins; a reader-detected cause is the fallback.
    let detected = if handshake(&mut reader, &ctx) {
        loop {
            if conn.close.load(Ordering::SeqCst) {
                break None;
            }
            match reader.recv() {
                Ok(Some(envelope)) => dispatch::handle_envelope(&ctx, envelope),
                Ok(None) => break Some(DisconnectReason::PeerClosed),
                Err(error) => {
                    let reason = classify_read_error(&error);
                    if reason != DisconnectReason::IdleTimeout {
                        eprintln!("relayable-server: client {client_id} disconnected: {error}");
                    }
                    break Some(reason);
                }
            }
        }
    } else {
        None
    };
    let reason = conn
        .take_reason()
        .or(detected)
        .unwrap_or(DisconnectReason::TransportError);

    // Tear down the transport only: the session, block, PTY, and child stay up.
    ctx.release_streams();
    shared.events.unsubscribe(&conn);
    conn.request_close();
    lock(&shared.connections).retain(|candidate| !Arc::ptr_eq(candidate, &conn));
    shared.client_disconnected(client_id, reason);
    let _ = writer_handle.join();
}

fn handshake(reader: &mut FrameConnection, ctx: &ConnectionCtx) -> bool {
    let envelope = match reader.recv() {
        Ok(Some(envelope)) => envelope,
        Ok(None) => return false,
        Err(error) => {
            eprintln!(
                "relayable-server: client {} closed during handshake: {error}",
                ctx.client_id
            );
            return false;
        }
    };
    let request_id = envelope.request_id.unwrap_or(0);
    let Some(Body::Handshake(handshake)) = envelope.body else {
        ctx.send_error(
            request_id,
            ErrorCode::BadRequest,
            "the first frame must be a handshake",
            HashMap::new(),
            false,
        );
        return false;
    };
    if envelope.request_id != Some(1) {
        ctx.send_error(
            request_id,
            ErrorCode::BadRequest,
            "the handshake must use request_id 1",
            HashMap::new(),
            false,
        );
        return false;
    }
    if major_version(&handshake.protocol_version) != Some(1) {
        let mut details = HashMap::new();
        details.insert("accepted".to_string(), "1.x".to_string());
        ctx.send_error(
            request_id,
            ErrorCode::IncompatibleVersion,
            &format!(
                "unsupported protocol version {:?}; accepted 1.x",
                handshake.protocol_version
            ),
            details,
            false,
        );
        return false;
    }

    ctx.shared.client_negotiated(ctx.client_id, &handshake);

    ctx.send_result(
        request_id,
        response::Result::Hello(ServerHello {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: Vec::new(),
            limits: Some(ctx.shared.config.limits),
        }),
    );
    true
}

fn writer_loop(
    mut connection: FrameConnection,
    receiver: Receiver<WriterMsg>,
    shared: Arc<ConnectionShared>,
    server: Arc<ServerShared>,
    heartbeat: Duration,
) {
    let mut last_send = Instant::now();
    loop {
        if shared.close.load(Ordering::SeqCst) {
            break;
        }
        let elapsed = last_send.elapsed();
        let wait = if elapsed >= heartbeat {
            Duration::from_millis(1)
        } else {
            heartbeat - elapsed
        };
        match receiver.recv_timeout(wait) {
            Ok(WriterMsg::Frame(envelope)) => {
                if connection.send(&envelope).is_err() {
                    shared.close_now(DisconnectReason::TransportError);
                    break;
                }
                last_send = Instant::now();
            }
            Ok(WriterMsg::Close) => break,
            Err(RecvTimeoutError::Timeout) => {
                let sequence = server.sequence();
                let beat = Envelope {
                    protocol_version: PROTOCOL_VERSION.to_string(),
                    request_id: None,
                    stream_id: 0,
                    sequence: Some(sequence),
                    trace_id: None,
                    timestamp_unix_millis: None,
                    body: Some(Body::Heartbeat(Heartbeat {
                        sequence: Some(sequence),
                    })),
                };
                if connection.send(&beat).is_err() {
                    shared.close_now(DisconnectReason::TransportError);
                    break;
                }
                last_send = Instant::now();
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    let _ = shared.stream.shutdown(Shutdown::Both);
}

/// Build a terminal-data envelope for fan-out to an attached stream.
pub(crate) fn terminal_output_envelope(stream_id: u64, stream_seq: u64, bytes: &[u8]) -> Envelope {
    Envelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        request_id: None,
        stream_id,
        sequence: None,
        trace_id: None,
        timestamp_unix_millis: None,
        body: Some(Body::TerminalData(relayable_protocol::v1::TerminalOutput {
            stream_id,
            stream_seq,
            bytes: bytes.to_vec(),
        })),
    }
}

fn major_version(version: &str) -> Option<u64> {
    let (major, _minor) = version.split_once('.')?;
    major.parse().ok()
}

/// Classify a frame-read error as an idle read timeout or a genuine transport
/// error.
///
/// A read timeout can surface **either** directly (`TransportError::Io`) **or**
/// wrapped by the frame codec (`TransportError::Codec(CodecError::Io)`, because
/// the length-prefixed reader wraps every `io::Error` it observes). Rather than
/// match the known variants — which silently misclassified the real reader path
/// as `transport_error` — walk the `std::error::Error` source chain to the
/// underlying `io::Error`. Any current or future wrapping layer that preserves
/// its source is therefore covered by this single decision point.
fn classify_read_error(error: &TransportError) -> DisconnectReason {
    match source_io_error(error) {
        Some(io) if is_read_timeout(io) => DisconnectReason::IdleTimeout,
        _ => DisconnectReason::TransportError,
    }
}

/// The first `io::Error` in an error's `source()` chain, if any.
fn source_io_error<'a>(error: &'a (dyn std::error::Error + 'static)) -> Option<&'a std::io::Error> {
    let mut current = Some(error);
    while let Some(error) = current {
        if let Some(io) = error.downcast_ref::<std::io::Error>() {
            return Some(io);
        }
        current = error.source();
    }
    None
}

fn is_read_timeout(error: &std::io::Error) -> bool {
    matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut)
}

/// The method string a request operation must carry (contract §3/§6).
pub(crate) fn expected_method(operation: &request::Operation) -> &'static str {
    match operation {
        request::Operation::SessionCreate(_) => relayable_protocol::methods::SESSION_CREATE,
        request::Operation::SessionList(_) => relayable_protocol::methods::SESSION_LIST,
        request::Operation::SessionView(_) => relayable_protocol::methods::SESSION_VIEW,
        request::Operation::SessionClose(_) => relayable_protocol::methods::SESSION_CLOSE,
        request::Operation::BlockAttach(_) => relayable_protocol::methods::BLOCK_ATTACH,
        request::Operation::BlockDetach(_) => relayable_protocol::methods::BLOCK_DETACH,
        request::Operation::TerminalInput(_) => relayable_protocol::methods::TERMINAL_INPUT,
        request::Operation::TerminalResize(_) => relayable_protocol::methods::TERMINAL_RESIZE,
        request::Operation::SessionResume(_) => relayable_protocol::methods::SESSION_RESUME,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use relayable_protocol::codec::CodecError;

    #[test]
    fn request_window_rejects_duplicates() {
        let mut window = RequestWindow::new(4);
        assert!(window.record(1));
        assert!(!window.record(1));
        assert!(window.record(2));
    }

    #[test]
    fn request_window_evicts_oldest() {
        let mut window = RequestWindow::new(3);
        for id in 1..=3 {
            assert!(window.record(id));
        }
        assert!(window.record(4));
        // Id 1 fell out of the window and is no longer a duplicate.
        assert!(window.record(1));
        // Id 4 is still a duplicate.
        assert!(!window.record(4));
    }

    #[test]
    fn major_version_parses() {
        assert_eq!(major_version("1.0"), Some(1));
        assert_eq!(major_version("2.5"), Some(2));
        assert_eq!(major_version("1"), None);
        assert_eq!(major_version("nope"), None);
    }

    #[test]
    fn read_timeouts_classify_as_idle_timeout_in_both_shapes() {
        // Direct shape: `TransportError::Io`.
        let direct = TransportError::Io(std::io::Error::from(ErrorKind::WouldBlock));
        assert_eq!(classify_read_error(&direct), DisconnectReason::IdleTimeout);

        // Real reader shape: the frame codec wraps the socket timeout.
        let wrapped =
            TransportError::Codec(CodecError::Io(std::io::Error::from(ErrorKind::TimedOut)));
        assert_eq!(classify_read_error(&wrapped), DisconnectReason::IdleTimeout);
    }

    #[test]
    fn other_read_errors_classify_as_transport_error() {
        let direct = TransportError::Io(std::io::Error::from(ErrorKind::ConnectionReset));
        assert_eq!(
            classify_read_error(&direct),
            DisconnectReason::TransportError
        );

        let wrapped =
            TransportError::Codec(CodecError::Io(std::io::Error::from(ErrorKind::BrokenPipe)));
        assert_eq!(
            classify_read_error(&wrapped),
            DisconnectReason::TransportError
        );

        // A non-I/O codec failure is never a timeout.
        assert_eq!(
            classify_read_error(&TransportError::Codec(CodecError::Truncated)),
            DisconnectReason::TransportError
        );
    }
}
