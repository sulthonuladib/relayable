//! GPUI-free **live proof** for UI-002 stage A.
//!
//! Connects to a real `relayable-server`, creates a session, attaches its
//! terminal block as WRITER through the data plane, feeds the attach snapshot
//! and live output into a [`TerminalModel`], runs a deterministic harmless
//! command, resizes the terminal, and proves via the child's own `stty size`
//! that the resize took effect.
//!
//! Exit status is non-zero if either the command output or the post-resize size
//! is not observed. No output is fabricated: everything printed comes from the
//! live server.
//!
//! Socket resolution: `$RELAYABLE_SERVER` if set and non-empty, else
//! `relayable_protocol::transport::default_socket_path()`.

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
/// Overall deadline for each wait step.
const STEP_TIMEOUT: Duration = Duration::from_secs(20);

fn resolve_socket() -> PathBuf {
    match std::env::var_os("RELAYABLE_SERVER") {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => relayable_protocol::transport::default_socket_path(),
    }
}

struct Probe {
    api: ApiHandle,
    model: TerminalModel,
    transcript: Vec<u8>,
    output_bytes: u64,
    stream_id: Option<u64>,
    block_id: Option<String>,
    resize_accepted: Option<String>,
}

impl Probe {
    fn new(api: ApiHandle) -> Self {
        Self {
            api,
            model: TerminalModel::new(TerminalSize::cells(INITIAL_COLS, INITIAL_ROWS)),
            transcript: Vec::new(),
            output_bytes: 0,
            stream_id: None,
            block_id: None,
            resize_accepted: None,
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
                    self.stream_id = Some(*stream_id);
                    self.model = TerminalModel::new(TerminalSize::cells(*cols, *rows));
                    self.model.process(screen_bytes);
                }
                ApiEvent::Output { bytes, .. } => {
                    self.output_bytes += bytes.len() as u64;
                    self.transcript.extend_from_slice(bytes);
                    self.model.process(bytes);
                }
                ApiEvent::Resized { cols, rows, .. } => {
                    self.model.resize(TerminalSize::cells(*cols, *rows));
                }
                ApiEvent::SessionCreated { block_id, .. } => {
                    self.block_id = Some(block_id.clone());
                }
                ApiEvent::ServerEvent { kind, detail } if kind == "TerminalResizeAccepted" => {
                    self.resize_accepted = Some(detail.clone());
                }
                _ => {}
            }
        }
        events
    }

    /// Poll until `ready`, failing on `Error` events or the deadline.
    fn wait_for<F>(&mut self, what: &str, mut ready: F) -> Result<(), String>
    where
        F: FnMut(&Probe) -> bool,
    {
        let deadline = Instant::now() + STEP_TIMEOUT;
        loop {
            for event in self.drain() {
                if let ApiEvent::Error { message } = event {
                    return Err(format!("while waiting for {what}: {message}"));
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
}

fn run() -> Result<(), String> {
    let socket = resolve_socket();
    println!("live-probe: socket = {}", socket.display());

    let api = ApiHandle::connect_live(socket.clone());
    let mut probe = Probe::new(api);

    // 1. Connected.
    probe.wait_for("control connection", |p| {
        matches!(p.api.state(), ConnectionState::Connected)
    })?;
    println!("live-probe: connected");

    // 2. Create a session.
    probe.api.send(ApiCommand::CreateSession {
        label: Some("live-probe".to_string()),
    });
    probe.wait_for("session.create", |p| p.block_id.is_some())?;
    let block_id = probe
        .block_id
        .clone()
        .ok_or_else(|| "session.create returned no block".to_string())?;
    println!("live-probe: created block {block_id}");

    // 3. Attach the block as WRITER on the data plane.
    probe.api.send(ApiCommand::AttachBlock {
        block_id: block_id.clone(),
        takeover: false,
    });
    probe.wait_for("block.attach", |p| p.stream_id.is_some())?;
    let stream_id = probe
        .stream_id
        .ok_or_else(|| "block.attach returned no stream".to_string())?;
    println!("live-probe: attached stream_id = {stream_id}");

    // 4. Run the deterministic command and read the initial size.
    probe.api.send(ApiCommand::Input {
        stream_id,
        bytes: b"printf 'RELAYABLE-PROBE-%s\\n' OK; stty size\n".to_vec(),
    });
    probe.wait_for("command output + initial stty size", |p| {
        p.contains(b"RELAYABLE-PROBE-OK") && p.contains(b"24 80")
    })?;
    println!("live-probe: observed RELAYABLE-PROBE-OK and initial size 24 80");

    // 5. Resize and prove it took effect.
    probe.api.send(ApiCommand::Resize {
        stream_id,
        cols: RESIZED_COLS,
        rows: RESIZED_ROWS,
        pixel_width: None,
        pixel_height: None,
    });
    probe.wait_for("TerminalResizeAccepted", |p| p.resize_accepted.is_some())?;
    probe.api.send(ApiCommand::Input {
        stream_id,
        bytes: b"stty size\n".to_vec(),
    });
    probe.wait_for("post-resize stty size", |p| p.contains(b"40 120"))?;

    // 6. Report.
    println!("live-probe: received output bytes = {}", probe.output_bytes);
    println!("live-probe: stream_id = {stream_id}");
    match &probe.resize_accepted {
        Some(detail) => println!("live-probe: TerminalResizeAccepted = {detail}"),
        None => return Err("no TerminalResizeAccepted confirmation observed".to_string()),
    }

    let snapshot = probe.model.snapshot();
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
