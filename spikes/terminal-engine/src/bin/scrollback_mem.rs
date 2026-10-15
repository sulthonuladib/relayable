//! `scrollback_mem` — bounded-memory / scrollback OBSERVATION harness.
//!
//! This is an observation, not a benchmark and not a memory target (full.md
//! §0.3.7, §12: "measure before setting hard memory targets"). It:
//!   1. emits a precisely quantified volume of fixed-size numbered lines
//!      (`--mib M` -> M MiB of payload bytes) from a child on a real pty;
//!   2. reads all of it from the pty master under one retention policy:
//!      - `discard` — read and drop (baseline)
//!      - `unbounded` — retain every byte
//!      - `bounded` — retain only the last `--cap-lines` lines
//!   3. samples this process's RSS from `/proc/self/status` VmRSS at a fixed
//!      interval and records VmHWM at the end, plus the child's peak VmRSS.
//!
//! The child mode (`--emit-lines N`) is the same binary, re-executed via
//! `/proc/self/exe`, so no external generator is required.

use std::collections::VecDeque;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::process;
use std::thread;
use std::time::{Duration, Instant};

use relayable_terminal_engine_spike::pty;

/// 40 ASCII bytes.
const PAYLOAD: &str = "abcdefghijklmnopqrstuvwxyz0123456789ABCD";
/// `line ` (5) + 8 digits + ` ` (1) + payload (40) + `\n` (1)
const LINE_LEN: usize = 55;

fn emit_lines(n: u64) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = BufWriter::with_capacity(1 << 20, stdout.lock());
    for i in 0..n {
        writeln!(out, "line {:08} {}", i, PAYLOAD)?;
    }
    out.flush()
}

fn read_status_field(pid: u32, field: &str) -> Option<u64> {
    let s = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix(field) {
            let v = rest.trim().trim_end_matches("kB").trim();
            return v.parse().ok();
        }
    }
    None
}

fn rss_kb(pid: u32) -> Option<u64> {
    read_status_field(pid, "VmRSS:")
}

fn hwm_kb(pid: u32) -> Option<u64> {
    read_status_field(pid, "VmHWM:")
}

enum Retention {
    Discard,
    Unbounded(Vec<u8>),
    Bounded { q: VecDeque<Vec<u8>>, cap: usize },
}

struct Stats {
    bytes_read: usize,
    retained_lines: usize,
    retained_bytes: usize,
}

fn read_loop(mut r: File, mut retention: Retention) -> Stats {
    let mut buf = [0u8; 65536];
    let mut bytes_read = 0usize;
    let mut retained_bytes = 0usize;
    let mut retained_lines = 0usize;
    let mut pending: Vec<u8> = Vec::new();
    loop {
        match r.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                bytes_read += n;
                match &mut retention {
                    Retention::Discard => {}
                    Retention::Unbounded(v) => {
                        v.extend_from_slice(&buf[..n]);
                        retained_bytes = v.len();
                    }
                    Retention::Bounded { q, cap } => {
                        pending.extend_from_slice(&buf[..n]);
                        let mut start = 0usize;
                        for i in 0..pending.len() {
                            if pending[i] == b'\n' {
                                let line = pending[start..=i].to_vec();
                                retained_bytes += line.len();
                                retained_lines += 1;
                                q.push_back(line);
                                start = i + 1;
                                if q.len() > *cap {
                                    if let Some(old) = q.pop_front() {
                                        retained_bytes -= old.len();
                                        retained_lines -= 1;
                                    }
                                }
                            }
                        }
                        if start > 0 {
                            pending.drain(..start);
                        }
                    }
                }
            }
            Err(ref e) if e.raw_os_error() == Some(libc::EIO) => break,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    Stats {
        bytes_read,
        retained_lines,
        retained_bytes,
    }
}

#[derive(Default)]
struct Args {
    emit_lines: Option<u64>,
    mib: u64,
    lines: u64,
    retain: String,
    cap_lines: usize,
    interval_ms: u64,
    rows: u16,
    cols: u16,
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        retain: "discard".to_string(),
        cap_lines: 1000,
        interval_ms: 25,
        rows: 24,
        cols: 80,
        ..Args::default()
    };
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0usize;
    while i < argv.len() {
        let val = |i: usize| -> Result<&String, String> {
            argv.get(i)
                .ok_or_else(|| format!("{} needs a value", argv[i - 1]))
        };
        match argv[i].as_str() {
            "--emit-lines" => {
                i += 1;
                a.emit_lines = Some(val(i)?.parse().map_err(|e| format!("{e}"))?);
            }
            "--mib" => {
                i += 1;
                a.mib = val(i)?.parse().map_err(|e| format!("{e}"))?;
            }
            "--lines" => {
                i += 1;
                a.lines = val(i)?.parse().map_err(|e| format!("{e}"))?;
            }
            "--retain" => {
                i += 1;
                a.retain = val(i)?.clone();
            }
            "--cap-lines" => {
                i += 1;
                a.cap_lines = val(i)?.parse().map_err(|e| format!("{e}"))?;
            }
            "--interval-ms" => {
                i += 1;
                a.interval_ms = val(i)?.parse().map_err(|e| format!("{e}"))?;
            }
            "--rows" => {
                i += 1;
                a.rows = val(i)?.parse().map_err(|e| format!("{e}"))?;
            }
            "--cols" => {
                i += 1;
                a.cols = val(i)?.parse().map_err(|e| format!("{e}"))?;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        i += 1;
    }
    Ok(a)
}

fn main() {
    let a = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("scrollback_mem: {e}");
            process::exit(2);
        }
    };

    if let Some(n) = a.emit_lines {
        if let Err(e) = emit_lines(n) {
            eprintln!("scrollback_mem: emit failed: {e}");
            process::exit(1);
        }
        return;
    }

    let lines = if a.lines > 0 {
        a.lines
    } else if a.mib > 0 {
        (a.mib * 1024 * 1024) / LINE_LEN as u64
    } else {
        eprintln!("scrollback_mem: need --lines or --mib");
        process::exit(2);
    };
    let payload_bytes = lines * LINE_LEN as u64;

    let me = process::id();
    let (retention, mode_desc) = match a.retain.as_str() {
        "discard" => (Retention::Discard, "discard".to_string()),
        "unbounded" => (Retention::Unbounded(Vec::new()), "unbounded".to_string()),
        "bounded" => (
            Retention::Bounded {
                q: VecDeque::new(),
                cap: a.cap_lines,
            },
            format!("bounded cap_lines={}", a.cap_lines),
        ),
        other => {
            eprintln!("scrollback_mem: unknown --retain {other}");
            process::exit(2);
        }
    };

    println!("== scrollback_mem: PTY output volume vs process RSS observation ==");
    println!("RETENTION_MODE={mode_desc}");
    println!("TARGET_MIB={}", a.mib);
    println!("LINES={lines}");
    println!("LINE_BYTES={LINE_LEN}");
    println!("EXPECTED_PAYLOAD_BYTES={payload_bytes}");
    println!("PTY_SIZE={} rows x {} cols", a.rows, a.cols);
    println!("TERM={}", pty::TERM);
    println!(
        "RSS_SAMPLING=/proc/<pid>/status VmRSS every {} ms; VmHWM at end",
        a.interval_ms
    );
    println!("RSS_LIMITATION=RSS includes the allocator, reader buffers, and binary; it is not an engine baseline and no memory target is asserted.");

    let exe =
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("scrollback_mem"));
    let args: Vec<OsString> = vec!["--emit-lines".into(), lines.to_string().into()];

    let mut start_rss = rss_kb(me);
    let spawn = match pty::spawn_only(&exe.to_string_lossy(), &args, &[], a.rows, a.cols) {
        Ok(s) => s,
        Err(e) => {
            println!("SPAWN_ERROR={e}");
            println!("VERDICT=FAIL");
            process::exit(1);
        }
    };
    let child_pid = spawn.child.id();
    let read_end = match spawn.master.try_clone() {
        Ok(f) => f,
        Err(e) => {
            println!("CLONE_ERROR={e}");
            println!("VERDICT=FAIL");
            process::exit(1);
        }
    };
    start_rss = start_rss.or_else(|| rss_kb(me));
    println!(
        "RSS_BASELINE_AFTER_SPAWN_KB={}",
        start_rss
            .map(|v| v.to_string())
            .unwrap_or_else(|| "<unavailable>".into())
    );

    let reader = thread::spawn(move || read_loop(read_end, retention));

    let mut child = spawn.child;
    let interval = Duration::from_millis(a.interval_ms.max(1));
    let started = Instant::now();
    let mut samples: Vec<u64> = Vec::new();
    if let Some(v) = rss_kb(me) {
        samples.push(v);
    }
    let mut child_peak: u64 = 0;
    let mut status = None;
    loop {
        match child.try_wait() {
            Ok(Some(s)) => {
                status = Some(s);
                break;
            }
            Ok(None) => {}
            Err(e) => {
                println!("WAIT_ERROR={e}");
                break;
            }
        }
        thread::sleep(interval);
        if let Some(v) = rss_kb(me) {
            samples.push(v);
        }
        if let Some(v) = rss_kb(child_pid) {
            child_peak = child_peak.max(v);
        }
        if started.elapsed() > Duration::from_secs(180) {
            println!("NOTE=180s budget exhausted; killing child");
            let _ = pty::killpg(child_pid as i32, libc::SIGKILL);
        }
    }
    if status.is_none() {
        status = child.wait().ok();
    }

    let stats = match reader.join() {
        Ok(s) => s,
        Err(_) => {
            println!("READER_PANIC=true");
            Stats {
                bytes_read: 0,
                retained_lines: 0,
                retained_bytes: 0,
            }
        }
    };
    let wall_ms = started.elapsed().as_millis() as u64;
    if let Some(v) = rss_kb(me) {
        samples.push(v);
    }

    let peak_sampled = samples.iter().copied().max().unwrap_or(0);
    let end_rss = samples.last().copied().unwrap_or(0);
    let hwm = hwm_kb(me).unwrap_or(0);

    println!("BYTES_READ_FROM_PTY={}", stats.bytes_read);
    println!(
        "BYTES_DELTA_VS_PAYLOAD={}",
        stats.bytes_read as i64 - payload_bytes as i64
    );
    println!("BYTES_DELTA_NOTE=the slave applies OPOST|ONLCR, so each \\n arrives as \\r\\n (+1 byte/line)");
    println!("RETAINED_LINES={}", stats.retained_lines);
    println!("RETAINED_BYTES={}", stats.retained_bytes);
    println!("RSS_SAMPLE_COUNT={}", samples.len());
    println!("RSS_START_KB={}", samples.first().copied().unwrap_or(0));
    println!("RSS_PEAK_SAMPLED_KB={peak_sampled}");
    println!("RSS_END_KB={end_rss}");
    println!("RSS_VMHWM_KB={hwm}");
    println!("CHILD_RSS_PEAK_KB={child_peak}");
    println!("WALL_MS={wall_ms}");
    match &status {
        Some(s) => {
            println!("CHILD_EXIT_CODE={:?}", s.code());
            println!("CHILD_EXIT_SUCCESS={}", s.success());
        }
        None => println!("CHILD_EXIT_CODE=<unknown>"),
    }
    println!(
        "OBSERVATION_RETAINED_MINUS_START_KB={}",
        end_rss as i64 - samples.first().copied().unwrap_or(0) as i64
    );
    println!(
        "VERDICT={}",
        if stats.bytes_read > 0 { "PASS" } else { "FAIL" }
    );

    // Keep the pty master fd alive until here (it is dropped at end of scope).
    drop(spawn.master);

    if stats.bytes_read == 0 {
        process::exit(1);
    }
}
