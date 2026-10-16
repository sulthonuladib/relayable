//! GPUI-free **live proof** for the Linux app's terminal data plane.
//!
//! Drives the app's own [`ApiHandle`] against a real `relayable-server`: creates
//! two sessions, attaches both blocks as WRITER through the data plane (two
//! concurrent streams on one connection) and proves
//!
//! * the deterministic command output and the initial `stty size` arrive;
//! * `terminal.resize` — columns/rows plus the pane's **real pixel size** — is
//!   accepted and takes effect, proven by the child's own `stty size`;
//! * a burst larger than the server's per-stream credit window
//!   (`credit_window_bytes`, 256 KiB) on the second attached stream is delivered
//!   in full and the connection survives, which it cannot do if a chunk is left
//!   unacknowledged (`Subscriber::overflow` closes the whole connection);
//! * the first stream is still usable afterwards.
//!
//! Exit status is non-zero if any of those is not observed. No output is
//! fabricated: everything printed comes from the live server.
//!
//! Socket resolution: `$RELAYABLE_SERVER` if set and non-empty, else
//! `relayable_protocol::transport::default_socket_path()`.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use relayable_linux::api::{ApiCommand, ApiEvent, ApiHandle, ConnectionState};
use relayable_linux::term::{TerminalModel, TerminalSize};

/// Initial model/server size (matches the server's `TerminalSize::DEFAULT_SIZE`).
const INITIAL_COLS: u16 = 80;
const INITIAL_ROWS: u16 = 24;
/// Requested post-resize size; must differ from the initial one.
const RESIZED_COLS: u16 = 120;
const RESIZED_ROWS: u16 = 40;
/// Pane pixel size sent with the resize — the app reports its measured pane box
/// (`apps/linux/src/app/resize.rs`).
const PANE_PIXEL_WIDTH: u32 = 1200;
const PANE_PIXEL_HEIGHT: u32 = 800;
/// Output bytes requested on the second stream. Larger than the server's default
/// per-stream credit window (262 144 bytes), so a chunk left unacknowledged would
/// exhaust the window and close the connection before the burst completes.
const BURST_BYTES: u64 = 400_000;
/// Overall deadline for each wait step.
const STEP_TIMEOUT: Duration = Duration::from_secs(20);
/// Longer deadline for the flow-control burst.
const BURST_TIMEOUT: Duration = Duration::from_secs(60);

fn resolve_socket() -> PathBuf {
    match std::env::var_os("RELAYABLE_SERVER") {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => relayable_protocol::transport::default_socket_path(),
    }
}

struct Probe {
    api: ApiHandle,
    /// Screen model for the **first** attached stream (the probe is not a
    /// multi-pane UI; later streams are counted, not rendered).
    model: TerminalModel,
    /// Every output byte received, in arrival order, across all streams.
    transcript: Vec<u8>,
    /// Live output bytes per stream id.
    bytes_by_stream: BTreeMap<u64, u64>,
    /// Number of `ApiEvent::Output` events per stream id (fan-out granularity).
    chunks_by_stream: BTreeMap<u64, u64>,
    /// Stream ids in attach order.
    attached: Vec<u64>,
    /// `(session_id, block_id)` in `session.create` order.
    created: Vec<(String, String)>,
    resize_accepted: Option<String>,
    /// The flow-control burst: which stream, when it was requested, and how long
    /// it took to deliver [`BURST_BYTES`].
    burst: Option<Burst>,
}

/// One flow-control burst measurement.
struct Burst {
    stream_id: u64,
    started: Instant,
    elapsed: Option<Duration>,
}

impl Probe {
    fn new(api: ApiHandle) -> Self {
        Self {
            api,
            model: TerminalModel::new(TerminalSize::cells(INITIAL_COLS, INITIAL_ROWS)),
            transcript: Vec::new(),
            bytes_by_stream: BTreeMap::new(),
            chunks_by_stream: BTreeMap::new(),
            attached: Vec::new(),
            created: Vec::new(),
            resize_accepted: None,
            burst: None,
        }
    }

    /// Drain and apply every pending API event, returning them.
    fn drain(&mut self) -> Vec<ApiEvent> {
        let verbose = std::env::var_os("RELAYABLE_PROBE_VERBOSE").is_some();
        let events = self.api.drain();
        for event in &events {
            if verbose {
                eprintln!("live-probe: event {event:?}");
            }
            match event {
                ApiEvent::Attached {
                    stream_id,
                    cols,
                    rows,
                    screen_bytes,
                } => {
                    if self.attached.is_empty() {
                        self.model = TerminalModel::new(TerminalSize::cells(*cols, *rows));
                        self.model.process(screen_bytes);
                    }
                    self.attached.push(*stream_id);
                }
                ApiEvent::Output { stream_id, bytes } => {
                    *self.bytes_by_stream.entry(*stream_id).or_insert(0) += bytes.len() as u64;
                    *self.chunks_by_stream.entry(*stream_id).or_insert(0) += 1;
                    self.transcript.extend_from_slice(bytes);
                    if Some(*stream_id) == self.attached.first().copied() {
                        self.model.process(bytes);
                    }
                    let delivered = self.bytes_by_stream.get(stream_id).copied().unwrap_or(0);
                    if let Some(burst) = self.burst.as_mut() {
                        if burst.stream_id == *stream_id
                            && burst.elapsed.is_none()
                            && delivered >= BURST_BYTES
                        {
                            burst.elapsed = Some(burst.started.elapsed());
                        }
                    }
                }
                ApiEvent::Resized { cols, rows, .. } => {
                    self.model.resize(TerminalSize::cells(*cols, *rows));
                }
                ApiEvent::SessionCreated {
                    session_id,
                    block_id,
                    ..
                } => self.created.push((session_id.clone(), block_id.clone())),
                ApiEvent::ServerEvent { kind, detail } if kind == "TerminalResizeAccepted" => {
                    self.resize_accepted = Some(detail.clone());
                }
                _ => {}
            }
        }
        events
    }

    /// Poll until `ready`, failing on `Error`/`Disconnected` events or the
    /// deadline.
    fn wait_for<F>(&mut self, what: &str, timeout: Duration, mut ready: F) -> Result<(), String>
    where
        F: FnMut(&Probe) -> bool,
    {
        let deadline = Instant::now() + timeout;
        loop {
            for event in self.drain() {
                match event {
                    ApiEvent::Error { message } => {
                        return Err(format!("while waiting for {what}: {message}"));
                    }
                    ApiEvent::Disconnected { reason } => {
                        return Err(format!("while waiting for {what}: disconnected: {reason}"));
                    }
                    _ => {}
                }
            }
            if ready(self) {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(format!(
                    "timed out waiting for {what}; transcript tail:\n{}",
                    transcript_tail(&self.transcript)
                ));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn contains(&self, needle: &[u8]) -> bool {
        contains_bytes(&self.transcript, needle)
    }

    fn stream_bytes(&self, stream_id: u64) -> u64 {
        self.bytes_by_stream.get(&stream_id).copied().unwrap_or(0)
    }

    /// Print the counters and the screen model. Always called — on success and on
    /// failure — so a failing run still reports what really arrived.
    fn report(&self) {
        let total: u64 = self.bytes_by_stream.values().sum();
        println!("live-probe: received output bytes = {total}");
        println!("live-probe: attached stream_ids = {:?}", self.attached);
        println!("live-probe: bytes_by_stream = {:?}", self.bytes_by_stream);
        println!("live-probe: chunks_by_stream = {:?}", self.chunks_by_stream);
        match &self.resize_accepted {
            Some(detail) => println!("live-probe: TerminalResizeAccepted = {detail}"),
            None => println!("live-probe: TerminalResizeAccepted = <none observed>"),
        }
        match &self.burst {
            Some(burst) => println!(
                "live-probe: burst stream_id={} bytes={} chunks={} elapsed_ms={} delivered_ms={:?}",
                burst.stream_id,
                self.stream_bytes(burst.stream_id),
                self.chunks_by_stream
                    .get(&burst.stream_id)
                    .copied()
                    .unwrap_or(0),
                burst.started.elapsed().as_millis(),
                burst.elapsed.map(|elapsed| elapsed.as_millis()),
            ),
            None => println!("live-probe: burst = <none requested>"),
        }

        let snapshot = self.model.snapshot();
        println!(
            "live-probe: rendered screen ({}x{}), first 24 rows:",
            snapshot.cols, snapshot.rows
        );
        for (index, row) in snapshot
            .cells
            .chunks(usize::from(snapshot.cols))
            .enumerate()
        {
            if index >= 24 {
                break;
            }
            let text: String = row.iter().map(|cell| cell.ch).collect();
            println!("|{}|", text.trim_end());
        }
    }
}

fn run() -> Result<(), String> {
    let socket = resolve_socket();
    println!("live-probe: socket = {}", socket.display());

    let api = ApiHandle::connect_live(socket.clone());
    let mut probe = Probe::new(api);
    let outcome = drive(&mut probe);
    probe.report();
    outcome
}

fn drive(probe: &mut Probe) -> Result<(), String> {
    // 1. Connected.
    probe.wait_for("control connection", STEP_TIMEOUT, |p| {
        matches!(p.api.state(), ConnectionState::Connected)
    })?;
    println!("live-probe: connected");

    // 2. Create a session.
    probe.api.send(ApiCommand::CreateSession {
        label: Some("live-probe".to_string()),
    });
    probe.wait_for("session.create", STEP_TIMEOUT, |p| !p.created.is_empty())?;
    let (session_id, block_id) = probe
        .created
        .first()
        .cloned()
        .ok_or_else(|| "session.create returned no session".to_string())?;
    println!("live-probe: created session {session_id} block {block_id}");

    // 3. Attach the block as WRITER on the data plane.
    probe.api.send(ApiCommand::AttachBlock {
        session_id,
        block_id,
        takeover: false,
    });
    probe.wait_for("block.attach", STEP_TIMEOUT, |p| !p.attached.is_empty())?;
    let stream_id = probe.attached[0];
    println!("live-probe: attached stream_id = {stream_id}");

    // 4. Run the deterministic command and read the initial size.
    probe.api.send(ApiCommand::Input {
        stream_id,
        bytes: b"printf 'RELAYABLE-PROBE-%s\\n' OK; stty size\n".to_vec(),
    });
    probe.wait_for("command output + initial stty size", STEP_TIMEOUT, |p| {
        p.contains(b"RELAYABLE-PROBE-OK") && p.contains(b"24 80")
    })?;
    println!("live-probe: observed RELAYABLE-PROBE-OK and initial size 24 80");

    // 5. Resize with the pane's real pixel size and prove it took effect.
    probe.api.send(ApiCommand::Resize {
        stream_id,
        cols: RESIZED_COLS,
        rows: RESIZED_ROWS,
        pixel_width: Some(PANE_PIXEL_WIDTH),
        pixel_height: Some(PANE_PIXEL_HEIGHT),
    });
    probe.wait_for("TerminalResizeAccepted", STEP_TIMEOUT, |p| {
        p.resize_accepted.is_some()
    })?;
    probe.api.send(ApiCommand::Input {
        stream_id,
        bytes: b"stty size\n".to_vec(),
    });
    probe.wait_for("post-resize stty size", STEP_TIMEOUT, |p| {
        p.contains(b"40 120")
    })?;

    // 6. A second session, attached on the same data-plane connection.
    probe.api.send(ApiCommand::CreateSession {
        label: Some("live-probe-burst".to_string()),
    });
    probe.wait_for("second session.create", STEP_TIMEOUT, |p| {
        p.created.len() >= 2
    })?;
    let (burst_session, burst_block) = probe.created[1].clone();
    println!("live-probe: created second session {burst_session} block {burst_block}");
    probe.api.send(ApiCommand::AttachBlock {
        session_id: burst_session,
        block_id: burst_block,
        takeover: false,
    });
    probe.wait_for("second block.attach", STEP_TIMEOUT, |p| {
        p.attached.len() >= 2
    })?;
    let burst_stream = probe.attached[1];
    println!("live-probe: attached second stream_id = {burst_stream}");

    // 7. Marker on the first stream, then a burst on the second: the burst is
    //    larger than the server's per-stream credit window (`credit_window_bytes`,
    //    256 KiB), and it writes in multi-KiB blocks (a real command's output
    //    granularity), so the client must acknowledge ~1.5 windows' worth of
    //    output. `Subscriber::overflow` closes the whole connection if any chunk
    //    is left unacknowledged, so a surviving connection plus the marker below
    //    is the flow-control proof. The separate, harsher probe for a flood of
    //    2-byte writes is recorded in `evidence/ui-003-flow-control.log`.
    probe.api.send(ApiCommand::Input {
        stream_id,
        bytes: b"printf 'RELAYABLE-PROBE-%s\\n' TARGET\n".to_vec(),
    });
    probe.wait_for("first stream marker before the burst", STEP_TIMEOUT, |p| {
        p.contains(b"RELAYABLE-PROBE-TARGET")
    })?;
    probe.burst = Some(Burst {
        stream_id: burst_stream,
        started: Instant::now(),
        elapsed: None,
    });
    probe.api.send(ApiCommand::Input {
        stream_id: burst_stream,
        bytes: format!("head -c {BURST_BYTES} /dev/zero | tr '\\0' x; printf 'RELAYABLE-PROBE-%s\\n' BURST-OK\n")
            .into_bytes(),
    });
    probe.wait_for("flow-control burst marker", BURST_TIMEOUT, |p| {
        p.contains(b"RELAYABLE-PROBE-BURST-OK")
    })?;
    let burst_bytes = probe.stream_bytes(burst_stream);
    let elapsed = probe
        .burst
        .as_ref()
        .and_then(|burst| burst.elapsed)
        .ok_or_else(|| "burst delivery time was not recorded".to_string())?;
    let rate_mb_s = (burst_bytes as f64 / 1_048_576.0) / elapsed.as_secs_f64();
    println!(
        "live-probe: flow-control burst stream_id={burst_stream} bytes={burst_bytes} delivered_in_ms={} rate_mb_s={rate_mb_s:.1}",
        elapsed.as_millis()
    );
    if burst_bytes < BURST_BYTES {
        return Err(format!(
            "burst delivered {burst_bytes} bytes, expected at least {BURST_BYTES}"
        ));
    }

    // 8. The connection must still be usable after the burst.
    probe.api.send(ApiCommand::Input {
        stream_id,
        bytes: b"printf 'RELAYABLE-PROBE-%s\\n' ALIVE\n".to_vec(),
    });
    probe.wait_for("first stream alive after the burst", BURST_TIMEOUT, |p| {
        p.contains(b"RELAYABLE-PROBE-ALIVE")
    })?;

    // 9. The resize confirmation must have been observed.
    if probe.resize_accepted.is_none() {
        return Err("no TerminalResizeAccepted confirmation observed".to_string());
    }

    probe.api.send(ApiCommand::Shutdown);
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("live-probe: FAILED: {message}");
            ExitCode::from(1)
        }
    }
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack.len() >= needle.len()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn transcript_tail(bytes: &[u8]) -> String {
    let start = bytes.len().saturating_sub(2048);
    String::from_utf8_lossy(&bytes[start..]).to_string()
}
