//! Real-PTY driver built on `portable-pty 0.9.0`.
//!
//! The *approach* mirrors the phase-1 harness (`spikes/terminal-engine/src/pty.rs`
//! and `src/bin/capture_stream.rs`): spawn a real program on a fresh PTY, capture
//! every byte on a reader thread, optionally inject timed keystrokes and resize
//! the master mid-run, and always kill/reap the child. Phase 1 used `nix`/`libc`
//! directly; phase 2 uses the pinned `portable-pty` crate as required.
//!
//! ```text
//! PtyRun { program, args, cols, rows, duration, sends, resizes, env, cwd }
//!   -> openpty -> spawn (setsid + controlling tty) -> reader thread
//!   -> timed send/resize -> kill|wait -> bounded reader join -> Capture
//! ```
//!
//! No hang and no orphan process is guaranteed by:
//! 1. `portable-pty` spawns the child via `setsid()` (verified in
//!    `portable-pty-0.9.0/src/unix.rs:257`), so the child is its own process
//!    group leader;
//! 2. on the way out we kill the whole process group via `/bin/kill -KILL -- -PGID`
//!    (falling back to `Child::kill`), then reap with `wait`;
//! 3. the reader join is bounded by a timeout, so a stray grandchild holding the
//!    slave open cannot wedge the harness.

use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use portable_pty::{native_pty_system, CommandBuilder, ExitStatus, PtySize};

/// `TERM` for every spawned program (matches phase 1).
pub const TERM: &str = "xterm-256color";
/// `COLORTERM`, enabling 24-bit colour.
pub const COLORTERM: &str = "truecolor";

/// How long to wait for the reader thread after the child has been killed.
const READER_JOIN_TIMEOUT: Duration = Duration::from_secs(5);

/// A timed keystroke injection.
#[derive(Debug, Clone)]
pub struct SendSpec {
    pub at: Duration,
    /// Exact bytes written to the master.
    pub bytes: Vec<u8>,
}

/// A timed PTY resize.
#[derive(Debug, Clone)]
pub struct ResizeSpec {
    pub at: Duration,
    pub cols: u16,
    pub rows: u16,
}

/// Everything needed to run one real-PTY session.
#[derive(Debug, Clone)]
pub struct PtyRun {
    pub program: String,
    pub args: Vec<OsString>,
    pub cols: u16,
    pub rows: u16,
    pub duration: Duration,
    pub sends: Vec<SendSpec>,
    pub resizes: Vec<ResizeSpec>,
    pub env: Vec<(String, String)>,
    pub cwd: Option<PathBuf>,
}

/// Result of a real-PTY session.
#[derive(Debug, Clone)]
pub struct Capture {
    /// Exact bytes read from the master, in order.
    pub bytes: Vec<u8>,
    pub requested_cols: u16,
    pub requested_rows: u16,
    /// Applied resizes as `(elapsed_ms, cols, rows)`.
    pub applied_resizes: Vec<(u64, u16, u16)>,
    pub slave_path: Option<PathBuf>,
    pub child_pid: Option<u32>,
    pub pgid: Option<i32>,
    pub exit: String,
    pub exit_code: Option<u32>,
    pub killed: bool,
    pub timed_out: bool,
    pub reader_stalled: bool,
}

fn describe_exit(status: &ExitStatus) -> (String, Option<u32>) {
    let code = status.exit_code();
    let text = if let Some(sig) = status.signal() {
        format!("signaled:{sig}")
    } else if status.success() {
        "exited:0".to_string()
    } else {
        format!("exited:{code}")
    };
    (text, Some(code))
}

/// Kill a whole process group, tolerating absence of the external `kill`.
fn kill_group(pgid: i32) {
    let arg = format!("-{pgid}");
    let _ = std::process::Command::new("/bin/kill")
        .args(["-KILL", &arg])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

/// Run `run` to completion (or timeout) and return the captured bytes.
pub fn run(run: &PtyRun) -> Result<Capture, String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: run.rows,
            cols: run.cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("openpty: {e}"))?;

    let mut cmd = CommandBuilder::new(&run.program);
    cmd.args(run.args.iter().map(|s| s.as_os_str()));
    cmd.env("TERM", TERM);
    cmd.env("COLORTERM", COLORTERM);
    for (k, v) in &run.env {
        cmd.env(k, v);
    }
    if let Some(cwd) = &run.cwd {
        cmd.cwd(cwd);
    }
    // Default is true; set explicitly so SIGWINCH reaches the child.
    cmd.set_controlling_tty(true);

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("spawn {}: {e}", run.program))?;
    drop(pair.slave);

    let slave_path = pair.master.tty_name();
    let child_pid = child.process_id();
    let pgid = pair.master.process_group_leader();

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("clone reader: {e}"))?;
    let mut writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("take writer: {e}"))?;

    let output: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&output);
    let reader_handle = thread::spawn(move || {
        let mut buf = [0u8; 65536];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let mut guard = sink.lock().unwrap_or_else(|p| p.into_inner());
                    guard.extend_from_slice(&buf[..n]);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
    });

    let mut sends = run.sends.clone();
    sends.sort_by_key(|s| s.at);
    let mut resizes = run.resizes.clone();
    resizes.sort_by_key(|r| r.at);

    let start = Instant::now();
    let mut send_idx = 0usize;
    let mut resize_idx = 0usize;
    let mut applied_resizes = Vec::new();
    let mut exit: Option<ExitStatus> = None;
    let mut timed_out = false;

    loop {
        let elapsed = start.elapsed();
        while resize_idx < resizes.len() && elapsed >= resizes[resize_idx].at {
            let r = &resizes[resize_idx];
            if let Err(e) = pair.master.resize(PtySize {
                rows: r.rows,
                cols: r.cols,
                pixel_width: 0,
                pixel_height: 0,
            }) {
                return Err(format!("resize to {}x{}: {e}", r.cols, r.rows));
            }
            applied_resizes.push((elapsed.as_millis() as u64, r.cols, r.rows));
            resize_idx += 1;
        }
        while send_idx < sends.len() && elapsed >= sends[send_idx].at {
            let bytes = &sends[send_idx].bytes;
            writer
                .write_all(bytes)
                .map_err(|e| format!("write send at index {send_idx}: {e}"))?;
            writer.flush().map_err(|e| format!("flush: {e}"))?;
            send_idx += 1;
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                exit = Some(status);
                break;
            }
            Ok(None) => {}
            Err(e) => return Err(format!("try_wait: {e}")),
        }
        if elapsed >= run.duration {
            timed_out = true;
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }

    let mut killed = false;
    if exit.is_none() {
        // Group-kill first so grandchildren cannot keep the slave open.
        if let Some(group) = pgid {
            kill_group(group);
        }
        let _ = child.kill();
        match child.wait() {
            Ok(status) => exit = Some(status),
            Err(e) => return Err(format!("wait after kill: {e}")),
        }
        killed = true;
    }

    // Drop the writer (EOF/newline semantics) and the master so the reader sees
    // the slave close; then join with a timeout.
    drop(writer);
    drop(pair.master);

    let join_deadline = Instant::now() + READER_JOIN_TIMEOUT;
    while !reader_handle.is_finished() && Instant::now() < join_deadline {
        thread::sleep(Duration::from_millis(10));
    }
    let reader_stalled = !reader_handle.is_finished();
    if !reader_stalled {
        let _ = reader_handle.join();
    }

    let bytes = output.lock().unwrap_or_else(|p| p.into_inner()).clone();
    let (exit_text, exit_code) = match &exit {
        Some(status) => describe_exit(status),
        None => ("unknown".to_string(), None),
    };

    Ok(Capture {
        bytes,
        requested_cols: run.cols,
        requested_rows: run.rows,
        applied_resizes,
        slave_path,
        child_pid,
        pgid,
        exit: exit_text,
        exit_code,
        killed,
        timed_out,
        reader_stalled,
    })
}

/// Best-effort `PROGRAM --version` probe with a timeout; never panics or hangs.
/// Returns a one-line summary including the first stdout/stderr line.
pub fn program_version(program: &str, timeout: Duration) -> String {
    use std::sync::mpsc;

    let child = match std::process::Command::new(program)
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return format!("<--version spawn failed: {e}>"),
    };
    let pid = child.id();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });
    let first_line = |bytes: &[u8]| -> String {
        String::from_utf8_lossy(bytes)
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string()
    };
    match rx.recv_timeout(timeout) {
        Ok(Ok(out)) => {
            let stdout = first_line(&out.stdout);
            let stderr = first_line(&out.stderr);
            format!(
                "exit={:?} stdout={stdout:?} stderr={stderr:?}",
                out.status.code()
            )
        }
        Ok(Err(e)) => format!("<--version wait failed: {e}>"),
        Err(_) => {
            // Timeout: kill the straggler and try briefly to collect it.
            let _ = std::process::Command::new("/bin/kill")
                .args(["-KILL", &pid.to_string()])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            match rx.recv_timeout(Duration::from_secs(2)) {
                Ok(Ok(out)) => {
                    format!(
                        "<--version timed out; killed pid {pid}; exit={:?}>",
                        out.status.code()
                    )
                }
                _ => format!("<--version timed out; killed pid {pid}>"),
            }
        }
    }
}
