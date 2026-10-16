use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::sync::mpsc::Sender;
use std::time::Duration;

use relayable_client::{AttachMode, Client, ClientConfig, ClientError};
use relayable_model::{BlockId, SessionId};
use vte::Parser;

use super::types::{ApiEvent, DATA_READ_WINDOW, SNAPSHOT_TAIL_BYTES};
use super::wire::{emit, translate_wire_event};

pub(super) use super::data_thread::data_thread;
#[cfg(test)]
pub(super) use super::osc::strip_file_uri;
pub(super) use super::osc::{OscPerformer, TitleTracker};

// --- data plane ------------------------------------------------------------

/// The private data-plane connection: its own [`Client`], so the data plane and
/// the control plane never share a socket or a writer lease.
///
/// Every wire concern the old hand-rolled frame client owned is now the client
/// library's: the handshake, `block.attach` / `block.detach` / `terminal.input`
/// / `terminal.resize` round trips, and **flow control** — [`Client::next_output`]
/// acknowledges exactly the chunk it returns. The server charges its per-stream
/// credit window by output payload byte and closes the *whole connection* when a
/// subscriber exhausts it (`crates/relayable-server/src/session/subscriber.rs`,
/// `deliver` / `overflow`), so this module sends no `FlowControl` frame itself and
/// instead guarantees the window is refilled by draining every attached stream
/// (see [`DataPlane::pump`]).
pub(super) struct DataPlane {
    client: Client,
    events: Sender<ApiEvent>,
    /// Every stream this connection currently holds (`block.attach` succeeded and
    /// no `block.detach` has completed). N concurrent live attachments.
    stream_ids: BTreeSet<u64>,
    /// Per-stream OSC parser + title/cwd state, so interleaved output from
    /// different streams cannot corrupt each other's escape parsing.
    titles: HashMap<u64, StreamTitle>,
}

/// Per-stream OSC 0/2 title and OSC 7 cwd state.
#[derive(Default)]
struct StreamTitle {
    parser: Parser,
    tracker: TitleTracker,
}

impl DataPlane {
    pub(super) fn connect(path: &Path, events: Sender<ApiEvent>) -> Result<Self, String> {
        let client = Client::connect(path, ClientConfig::new("ui"))
            .map_err(|error| format!("could not connect to {}: {error}", path.display()))?;
        let limits = client.limits();
        eprintln!(
            "relayable-data: server advertised max_concurrent_streams={} credit_window_bytes={}",
            limits.max_concurrent_streams, limits.credit_window_bytes
        );

        Ok(Self {
            client,
            events,
            stream_ids: BTreeSet::new(),
            titles: HashMap::new(),
        })
    }

    /// Attach `block_id` to this connection as **WRITER**.
    ///
    /// `takeover` asks the server to transfer the single writer lease from its
    /// current holder; `app/streams.rs` (`attach_pane`) passes `true` so a
    /// re-attach after a data-plane-only reconnect wins the lease back from its
    /// own dead connection. That path needs `attach_with_takeover`:
    /// `Client::attach_block` hardcodes `takeover: false`.
    ///
    /// `session_id` is the wire session id of the block's session. `block.attach`
    /// is addressed by `block_id` alone; the client takes the session for
    /// call-site parity (`crates/relayable-client/src/client/terminal.rs`), and
    /// passing it keeps the command's identity explicit for the caller that has
    /// both ids.
    pub(super) fn attach(
        &mut self,
        session_id: &str,
        block_id: &str,
        takeover: bool,
    ) -> Result<(), String> {
        let session = session_id
            .parse::<SessionId>()
            .map_err(|error| format!("bad session id {session_id:?}: {error}"))?;
        let block = block_id
            .parse::<BlockId>()
            .map_err(|error| format!("bad block id {block_id:?}: {error}"))?;

        let response = self
            .client
            .attach_with_takeover(&session, &block, AttachMode::Writer, takeover)
            .map_err(|error| format!("block.attach failed: {error}"))?;

        let snapshot = response.snapshot.unwrap_or_default();
        let cols = clamp_dimension(snapshot.cols);
        let rows = clamp_dimension(snapshot.rows);
        let screen_bytes = tail(&snapshot.screen_bytes, SNAPSHOT_TAIL_BYTES);
        self.stream_ids.insert(response.stream_id);
        // Emit `Attached` before the snapshot's OSC title/cwd so a listener can
        // attribute the following `PaneTitleChanged` to the stream that was just
        // attached (the event carries no id itself).
        emit(
            &self.events,
            ApiEvent::Attached {
                stream_id: response.stream_id,
                cols,
                rows,
                screen_bytes: screen_bytes.clone(),
            },
        );
        self.observe(response.stream_id, &screen_bytes);
        Ok(())
    }

    pub(super) fn detach(&mut self, stream_id: u64) -> Result<(), String> {
        self.client
            .detach_block(stream_id)
            .map_err(|error| format!("block.detach failed: {error}"))?;
        self.stream_ids.remove(&stream_id);
        self.titles.remove(&stream_id);
        Ok(())
    }

    pub(super) fn send_input(&mut self, stream_id: u64, bytes: Vec<u8>) -> Result<(), String> {
        let accepted = self
            .client
            .send_input(stream_id, &bytes)
            .map_err(|error| format!("terminal.input failed: {error}"))?;
        emit(
            &self.events,
            ApiEvent::ServerEvent {
                kind: "TerminalInputAccepted".to_string(),
                detail: format!("stream_id={stream_id} accepted_bytes={accepted}"),
            },
        );
        Ok(())
    }

    /// Resize the pane and report back the geometry the **server** accepted.
    ///
    /// The pixel dimensions are forwarded for real: `terminal.resize` carries
    /// `optional uint32 pixel_width/pixel_height`, the server applies them to the
    /// block's PTY window size, and it rejects a resize whose pixel dimensions do
    /// not fit 16 bits (`crates/relayable-server/src/dispatch.rs`,
    /// `invalid_resize`) — which is why [`Client::resize_with_pixels`] takes
    /// `Option<u16>`. The request carries the measured pane size as
    /// `Option<u32>` (`app/resize.rs`); an out-of-range value saturates to
    /// `u16::MAX` (an accepted value) rather than silently dropping the
    /// dimension, and `None` stays `None` (the server substitutes `0`).
    ///
    /// [`relayable_client::TerminalResizeAccepted`] carries only
    /// `stream_id`/`cols`/`rows`, so those are what is reported: never the
    /// requested values as if the server had confirmed them.
    pub(super) fn send_resize(
        &mut self,
        stream_id: u64,
        cols: u16,
        rows: u16,
        pixel_width: Option<u32>,
        pixel_height: Option<u32>,
    ) -> Result<(), String> {
        let accepted = self
            .client
            .resize_with_pixels(
                stream_id,
                cols,
                rows,
                pixel_width.map(clamp_dimension),
                pixel_height.map(clamp_dimension),
            )
            .map_err(|error| format!("terminal.resize failed: {error}"))?;

        emit(
            &self.events,
            ApiEvent::Resized {
                stream_id: accepted.stream_id,
                cols: clamp_dimension(accepted.cols),
                rows: clamp_dimension(accepted.rows),
            },
        );
        emit(
            &self.events,
            ApiEvent::ServerEvent {
                kind: "TerminalResizeAccepted".to_string(),
                detail: format!(
                    "stream_id={} cols={} rows={}",
                    accepted.stream_id, accepted.cols, accepted.rows
                ),
            },
        );
        Ok(())
    }

    /// One data-plane step: drain every attached stream to empty, one bounded
    /// blocking read, drain again, then hand buffered events to the UI.
    ///
    /// # Why the drains sandwich the read
    ///
    /// The server charges its per-stream credit window by output payload byte
    /// (`credit_window_bytes`, 256 KiB by default) and closes the **whole
    /// connection** when a subscriber exhausts it (`Subscriber::overflow`,
    /// `crates/relayable-server/src/session/subscriber.rs`) — the UI would see
    /// that as a data-plane disconnect with every pane detached. Only
    /// [`Client::next_output`] refunds that credit, and only for the single chunk
    /// it returns, so every byte this plane receives must be handed back through
    /// a `next_output` call:
    ///
    /// 1. **drain every attached stream to empty** with `Duration::ZERO` (serve
    ///    what is already buffered; a zero timeout performs no socket read)
    ///    *before* waiting for more;
    /// 2. one bounded blocking read — [`Client::collect_events`] for
    ///    [`DATA_READ_WINDOW`]. It pumps the socket for that whole window,
    ///    moving output frames of **every** stream into the client's pending
    ///    queue and returning only the events it saw. Reading this way, rather
    ///    than targeting one stream with `next_output(stream, window)`, is what
    ///    makes step 3 complete: a stream-targeted read returns after a single
    ///    chunk of that stream and leaves every other stream's chunks in the
    ///    pending queue, so their acknowledgement would depend on which stream
    ///    the read happened to target. This read has no such preference.
    ///
    /// 3. **drain every attached stream again**, which is what actually
    ///    acknowledges everything step 2 pulled — one cheap `next_output` per
    ///    chunk, and no chunk is ever acknowledged without being delivered as
    ///    [`ApiEvent::Output`] in the same call.
    ///
    /// The unacknowledged volume of a stream is therefore bounded by one window's
    /// production (server rate x [`DATA_READ_WINDOW`]), not by how long the loop
    /// runs; see that constant for the numbers and for the residual limit this
    /// structure cannot remove.
    ///
    /// Both drain loops terminate: the client's pending queue only shrinks as
    /// `next_output` consumes it, and a drained stream can only be refilled by the
    /// server, whose per-stream credit this very call is refunding.
    ///
    /// # Errors
    ///
    /// A lost connection (`ClientError::Disconnected` / `ClientError::Transport`)
    /// is an `Err`, which `data_thread` turns into [`ApiEvent::Disconnected`] plus
    /// a reconnect. A read deadline is **not** an error: `collect_events` returns
    /// the events it saw and `next_output` reports `Ok(None)`, and neither returns
    /// [`ClientError::Timeout`] for an expired window.
    pub(super) fn pump(&mut self) -> Result<(), String> {
        self.drain_attached()?;

        let events = self
            .client
            .collect_events(DATA_READ_WINDOW)
            .map_err(describe_client_error)?;

        self.drain_attached()?;
        for event in events {
            emit(&self.events, translate_wire_event(event));
        }

        for event in self.client.drain_events() {
            emit(&self.events, translate_wire_event(event));
        }
        Ok(())
    }

    /// Drain every attached stream down to an empty buffer.
    ///
    /// `next_output(stream, Duration::ZERO)` is the acknowledgement path: it
    /// returns one already-buffered chunk per call and acks exactly that chunk.
    /// Finishing with `Ok(None)` means every chunk the client holds for that
    /// stream has been delivered and refunded.
    fn drain_attached(&mut self) -> Result<(), String> {
        // Snapshot the ids: `deliver` needs `&mut self`, and the set cannot
        // change while this loop runs (attaches/detaches are separate calls).
        for stream_id in self.stream_ids.clone() {
            while let Some(bytes) = self.next_chunk(stream_id, Duration::ZERO)? {
                self.deliver(stream_id, bytes);
            }
        }
        Ok(())
    }

    /// One output chunk: emit it, then feed it to the stream's OSC tracker.
    ///
    /// The chunk has already been acknowledged by `next_output`; nothing here
    /// acks again. The emit happens first so the UI can associate the following
    /// `PaneTitleChanged` (which carries no stream id) with this stream.
    fn deliver(&mut self, stream_id: u64, bytes: Vec<u8>) {
        emit(
            &self.events,
            ApiEvent::Output {
                stream_id,
                bytes: bytes.clone(),
            },
        );
        self.observe(stream_id, &bytes);
    }

    fn next_chunk(&mut self, stream_id: u64, timeout: Duration) -> Result<Option<Vec<u8>>, String> {
        self.client
            .next_output(stream_id, timeout)
            .map_err(describe_client_error)
    }

    /// Feed bytes to the lightweight OSC title/cwd tracker for one stream.
    fn observe(&mut self, stream_id: u64, bytes: &[u8]) {
        let entry = self.titles.entry(stream_id).or_default();
        let mut performer = OscPerformer {
            tracker: &mut entry.tracker,
            changed: false,
        };
        entry.parser.advance(&mut performer, bytes);
        if performer.changed {
            let program = entry.tracker.title.clone();
            let cwd = entry.tracker.cwd.clone();
            emit(&self.events, ApiEvent::PaneTitleChanged { program, cwd });
        }
    }
}

// --- data-plane helpers ----------------------------------------------------

/// Render a client failure for the data thread.
///
/// The peer-closed case keeps the wording the previous raw transport produced, so
/// `data_thread`'s `ApiEvent::Disconnected { reason }` text is unchanged; every
/// other failure keeps the client library's own message.
fn describe_client_error(error: ClientError) -> String {
    match error {
        ClientError::Disconnected => "server closed the connection".to_string(),
        other => other.to_string(),
    }
}

fn clamp_dimension(value: u32) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

pub(super) fn tail(bytes: &[u8], max: usize) -> Vec<u8> {
    if bytes.len() <= max {
        return bytes.to_vec();
    }
    bytes[bytes.len() - max..].to_vec()
}
