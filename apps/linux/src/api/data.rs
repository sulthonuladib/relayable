use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use relayable_protocol::codec::CodecError;
use relayable_protocol::methods;
use relayable_protocol::transport::{FrameConnection, TransportError};
use relayable_protocol::v1::{
    envelope::Body, request::Operation, response::Result as ResponseResult, AttachMode,
    BlockAttachRequest, BlockDetachRequest, Envelope, FlowControl, Handshake, Request,
    TerminalInput, TerminalOutput, TerminalResize,
};
use relayable_protocol::{CONTROL_STREAM_ID, PROTOCOL_VERSION};
use vte::{Parser, Perform};

use super::control::{emit, translate_wire_event};
use super::types::{
    ApiCommand, ApiEvent, POLL_INTERVAL, RECONNECT_BACKOFF, RECONNECT_MAX_ATTEMPTS,
    REQUEST_TIMEOUT, SNAPSHOT_TAIL_BYTES,
};

// --- data plane ------------------------------------------------------------

/// One data-plane read outcome.
enum FrameOutcome {
    /// A complete frame arrived (boxed: `Envelope` is large).
    Frame(Box<Envelope>),
    /// The read deadline expired with no frame (not a disconnect).
    Idle,
    /// The peer closed the connection cleanly.
    Closed,
}

/// The private data-plane connection.
struct DataPlane {
    conn: FrameConnection,
    events: Sender<ApiEvent>,
    next_request_id: u64,
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
    fn connect(path: &std::path::Path, events: Sender<ApiEvent>) -> Result<Self, String> {
        let conn = FrameConnection::connect(path)
            .map_err(|error| format!("could not connect to {}: {error}", path.display()))?;
        conn.set_read_timeout(Some(POLL_INTERVAL))
            .map_err(|error| format!("could not set read timeout: {error}"))?;

        let mut plane = Self {
            conn,
            events,
            next_request_id: 1,
            stream_ids: BTreeSet::new(),
            titles: HashMap::new(),
        };
        plane.perform_handshake()?;
        Ok(plane)
    }

    fn perform_handshake(&mut self) -> Result<(), String> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: Some(request_id),
            stream_id: CONTROL_STREAM_ID,
            body: Some(Body::Handshake(Handshake {
                client_kind: "ui".to_string(),
                platform: std::env::consts::OS.to_string(),
                client_version: env!("CARGO_PKG_VERSION").to_string(),
                protocol_version: PROTOCOL_VERSION.to_string(),
                capabilities: Vec::new(),
            })),
            ..Envelope::default()
        };
        self.conn
            .send(&envelope)
            .map_err(|error| format!("handshake send failed: {error}"))?;
        match self.await_result(request_id)? {
            ResponseResult::Hello(hello) => {
                if let Some(limits) = hello.limits.as_ref() {
                    eprintln!(
                        "relayable-data: server advertised max_concurrent_streams={} credit_window_bytes={}",
                        limits.max_concurrent_streams, limits.credit_window_bytes
                    );
                }
                Ok(())
            }
            ResponseResult::Error(error) => Err(format!("handshake rejected: {}", error.message)),
            other => Err(format!("unexpected handshake response: {other:?}")),
        }
    }

    fn attach(&mut self, block_id: &str, takeover: bool) -> Result<(), String> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        let envelope = self.request_envelope(
            request_id,
            methods::BLOCK_ATTACH,
            Operation::BlockAttach(BlockAttachRequest {
                block_id: block_id.to_string(),
                mode: AttachMode::Writer as i32,
                takeover,
            }),
        );
        self.conn
            .send(&envelope)
            .map_err(|error| format!("block.attach send failed: {error}"))?;

        match self.await_result(request_id)? {
            ResponseResult::BlockAttach(response) => {
                let snapshot = response.snapshot.unwrap_or_default();
                let cols = clamp_dimension(snapshot.cols);
                let rows = clamp_dimension(snapshot.rows);
                let screen_bytes = tail(&snapshot.screen_bytes, SNAPSHOT_TAIL_BYTES);
                self.stream_ids.insert(response.stream_id);
                // Emit `Attached` before the snapshot's OSC title/cwd so a
                // listener can attribute the following `PaneTitleChanged` to the
                // stream that was just attached (the event carries no id itself).
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
            ResponseResult::Error(error) => Err(format!("block.attach failed: {}", error.message)),
            other => Err(format!("unexpected block.attach response: {other:?}")),
        }
    }

    fn detach(&mut self, stream_id: u64) -> Result<(), String> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        let envelope = self.request_envelope(
            request_id,
            methods::BLOCK_DETACH,
            Operation::BlockDetach(BlockDetachRequest { stream_id }),
        );
        self.conn
            .send(&envelope)
            .map_err(|error| format!("block.detach send failed: {error}"))?;
        match self.await_result(request_id)? {
            ResponseResult::BlockDetach(_) => {
                self.stream_ids.remove(&stream_id);
                self.titles.remove(&stream_id);
                Ok(())
            }
            ResponseResult::Error(error) => Err(format!("block.detach failed: {}", error.message)),
            other => Err(format!("unexpected block.detach response: {other:?}")),
        }
    }

    fn send_input(&mut self, stream_id: u64, bytes: Vec<u8>) -> Result<(), String> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        let envelope = self.request_envelope(
            request_id,
            methods::TERMINAL_INPUT,
            Operation::TerminalInput(TerminalInput {
                stream_id,
                bytes: bytes.clone(),
            }),
        );
        self.conn
            .send(&envelope)
            .map_err(|error| format!("terminal.input send failed: {error}"))?;
        match self.await_result(request_id)? {
            ResponseResult::TerminalInput(accepted) => {
                emit(
                    &self.events,
                    ApiEvent::ServerEvent {
                        kind: "TerminalInputAccepted".to_string(),
                        detail: format!(
                            "stream_id={} accepted_bytes={}",
                            accepted.stream_id, accepted.accepted_bytes
                        ),
                    },
                );
                Ok(())
            }
            ResponseResult::Error(error) => {
                Err(format!("terminal.input failed: {}", error.message))
            }
            other => Err(format!("unexpected terminal.input response: {other:?}")),
        }
    }

    fn send_resize(
        &mut self,
        stream_id: u64,
        cols: u16,
        rows: u16,
        pixel_width: Option<u32>,
        pixel_height: Option<u32>,
    ) -> Result<(), String> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        let envelope = self.request_envelope(
            request_id,
            methods::TERMINAL_RESIZE,
            Operation::TerminalResize(TerminalResize {
                stream_id,
                cols: u32::from(cols),
                rows: u32::from(rows),
                pixel_width,
                pixel_height,
            }),
        );
        self.conn
            .send(&envelope)
            .map_err(|error| format!("terminal.resize send failed: {error}"))?;
        match self.await_result(request_id)? {
            ResponseResult::TerminalResize(accepted) => {
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
            ResponseResult::Error(error) => {
                Err(format!("terminal.resize failed: {}", error.message))
            }
            other => Err(format!("unexpected terminal.resize response: {other:?}")),
        }
    }

    fn request_envelope(&self, request_id: u64, method: &str, operation: Operation) -> Envelope {
        Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: Some(request_id),
            stream_id: CONTROL_STREAM_ID,
            body: Some(Body::Request(Request {
                method: method.to_string(),
                operation: Some(operation),
            })),
            ..Envelope::default()
        }
    }

    /// Read until the response for `request_id`, handling output frames on the
    /// way. Returns the response's `result`.
    fn await_result(&mut self, request_id: u64) -> Result<ResponseResult, String> {
        let deadline = Instant::now() + REQUEST_TIMEOUT;
        loop {
            if Instant::now() >= deadline {
                return Err("timed out waiting for a server response".to_string());
            }
            match self.recv_frame()? {
                FrameOutcome::Frame(envelope) => match envelope.body {
                    Some(Body::Response(response)) if response.request_id == request_id => {
                        return response
                            .result
                            .ok_or_else(|| "response carried no result".to_string());
                    }
                    Some(Body::TerminalData(output)) => self.handle_output(output),
                    Some(Body::Response(_) | Body::Heartbeat(_) | Body::FlowControl(_)) => {}
                    Some(Body::Event(event)) => {
                        emit(&self.events, translate_wire_event(event));
                    }
                    _ => {}
                },
                FrameOutcome::Idle => {}
                FrameOutcome::Closed => return Err("server closed the connection".to_string()),
            }
        }
    }

    /// One output pump; a read timeout is not an error.
    fn pump(&mut self) -> Result<(), String> {
        match self.recv_frame()? {
            FrameOutcome::Frame(envelope) => {
                match envelope.body {
                    Some(Body::TerminalData(output)) => self.handle_output(output),
                    Some(Body::Event(event)) => {
                        emit(&self.events, translate_wire_event(event));
                    }
                    Some(Body::Heartbeat(_) | Body::FlowControl(_) | Body::Response(_)) => {}
                    _ => {}
                }
                Ok(())
            }
            FrameOutcome::Idle => Ok(()),
            FrameOutcome::Closed => Err("server closed the connection".to_string()),
        }
    }

    /// One read. A read-deadline expiry is [`FrameOutcome::Idle`]; a clean EOF is
    /// [`FrameOutcome::Closed`]; anything else is an error.
    fn recv_frame(&mut self) -> Result<FrameOutcome, String> {
        match self.conn.recv() {
            Ok(Some(envelope)) => Ok(FrameOutcome::Frame(Box::new(envelope))),
            Ok(None) => Ok(FrameOutcome::Closed),
            Err(TransportError::Io(error)) if is_timeout(&error) => Ok(FrameOutcome::Idle),
            Err(TransportError::Codec(CodecError::Io(error))) if is_timeout(&error) => {
                Ok(FrameOutcome::Idle)
            }
            Err(error) => Err(error.to_string()),
        }
    }

    fn handle_output(&mut self, output: TerminalOutput) {
        let TerminalOutput {
            stream_id,
            stream_seq,
            bytes,
        } = output;
        let ack = bytes.len() as u64;
        // Emit the output first so the UI can associate the following
        // `PaneTitleChanged` (which carries no stream id) with this stream.
        emit(
            &self.events,
            ApiEvent::Output {
                stream_id,
                stream_seq,
                bytes: bytes.clone(),
            },
        );
        self.observe(stream_id, &bytes);
        if should_ack(&self.stream_ids, stream_id) && ack > 0 {
            // The server charges its credit window per output payload byte.
            let envelope = Envelope {
                protocol_version: PROTOCOL_VERSION.to_string(),
                stream_id,
                body: Some(Body::FlowControl(FlowControl {
                    stream_id,
                    ack_bytes: ack,
                })),
                ..Envelope::default()
            };
            let _ = self.conn.send(&envelope);
        }
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

/// Connect the data plane and, on success, tell the UI it is usable again.
///
/// The signal is a [`ApiEvent::ServerEvent`] (not a new [`ApiEvent`] variant,
/// which is frozen) so the UI can re-attach panes whose streams were invalidated
/// by a data-plane-only disconnect.
fn connect_data_plane(
    path: &std::path::Path,
    events: &Sender<ApiEvent>,
) -> Result<DataPlane, String> {
    let plane = DataPlane::connect(path, events.clone())?;
    emit(
        events,
        ApiEvent::ServerEvent {
            kind: "data_plane_connected".to_string(),
            detail: path.display().to_string(),
        },
    );
    Ok(plane)
}

pub(super) fn data_thread(path: PathBuf, commands: Receiver<ApiCommand>, events: Sender<ApiEvent>) {
    let mut plane = match connect_data_plane(&path, &events) {
        Ok(plane) => Some(plane),
        Err(error) => {
            emit(&events, ApiEvent::Error { message: error });
            None
        }
    };
    let mut attempts = 0u32;

    loop {
        if plane.is_none() {
            if attempts >= RECONNECT_MAX_ATTEMPTS {
                std::thread::sleep(POLL_INTERVAL);
                // Still drain commands so `Shutdown` is honoured.
                while let Ok(command) = commands.try_recv() {
                    if matches!(command, ApiCommand::Shutdown) {
                        return;
                    }
                }
                continue;
            }
            attempts += 1;
            match connect_data_plane(&path, &events) {
                Ok(connected) => {
                    attempts = 0;
                    plane = Some(connected);
                }
                Err(error) => {
                    emit(
                        &events,
                        ApiEvent::Error {
                            message: format!(
                                "data-plane connect attempt {attempts} failed: {error}"
                            ),
                        },
                    );
                    std::thread::sleep(RECONNECT_BACKOFF);
                    continue;
                }
            }
        }

        let mut shutdown = false;
        while let Ok(command) = commands.try_recv() {
            match command {
                ApiCommand::Shutdown => {
                    shutdown = true;
                    break;
                }
                ApiCommand::AttachBlock { block_id, takeover } => {
                    if let Some(plane) = plane.as_mut() {
                        if let Err(error) = plane.attach(&block_id, takeover) {
                            emit(&events, ApiEvent::Error { message: error });
                        }
                    }
                }
                ApiCommand::Detach { stream_id } => {
                    if let Some(plane) = plane.as_mut() {
                        if let Err(error) = plane.detach(stream_id) {
                            emit(&events, ApiEvent::Error { message: error });
                        }
                    }
                }
                ApiCommand::Input { stream_id, bytes } => {
                    if let Some(plane) = plane.as_mut() {
                        if let Err(error) = plane.send_input(stream_id, bytes) {
                            emit(&events, ApiEvent::Error { message: error });
                        }
                    }
                }
                ApiCommand::Resize {
                    stream_id,
                    cols,
                    rows,
                    pixel_width,
                    pixel_height,
                } => {
                    if let Some(plane) = plane.as_mut() {
                        if let Err(error) =
                            plane.send_resize(stream_id, cols, rows, pixel_width, pixel_height)
                        {
                            emit(&events, ApiEvent::Error { message: error });
                        }
                    }
                }
                _ => {}
            }
        }
        if shutdown {
            return;
        }

        if let Some(active_plane) = plane.as_mut() {
            if let Err(error) = active_plane.pump() {
                emit(
                    &events,
                    ApiEvent::Disconnected {
                        reason: format!("data connection lost: {error}"),
                    },
                );
                plane = None;
            }
        }
    }
}

// --- OSC title/cwd tracking ------------------------------------------------

/// The latest OSC 0/2 title and OSC 7 cwd seen on the output stream.
#[derive(Debug, Default, Clone)]
pub(super) struct TitleTracker {
    pub(super) title: Option<String>,
    pub(super) cwd: Option<String>,
}

/// A `vte` performer that only records OSC 0/2 (title) and OSC 7 (cwd).
pub(super) struct OscPerformer<'a> {
    pub(super) tracker: &'a mut TitleTracker,
    pub(super) changed: bool,
}

impl Perform for OscPerformer<'_> {
    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.len() < 2 {
            return;
        }
        match params[0] {
            b"0" | b"2" => {
                let title = params[1..]
                    .iter()
                    .map(|part| String::from_utf8_lossy(part))
                    .collect::<Vec<_>>()
                    .join(";");
                self.tracker.title = Some(title.trim().to_string());
                self.changed = true;
            }
            b"7" => {
                let raw = String::from_utf8_lossy(params[1]).to_string();
                self.tracker.cwd = Some(strip_file_uri(&raw));
                self.changed = true;
            }
            _ => {}
        }
    }
}

/// Turn `file://host/path` into `/path`; leave anything else untouched.
pub(super) fn strip_file_uri(raw: &str) -> String {
    let Some(rest) = raw.strip_prefix("file://") else {
        return raw.to_string();
    };
    match rest.find('/') {
        Some(index) => rest[index..].to_string(),
        None => rest.to_string(),
    }
}

// --- shared helpers --------------------------------------------------------

fn is_timeout(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
    )
}

/// Whether an output frame's stream is one this connection currently holds and
/// therefore must credit-ack. Pure so it can be unit-tested without a socket.
pub(super) fn should_ack(attached: &BTreeSet<u64>, stream_id: u64) -> bool {
    attached.contains(&stream_id)
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
