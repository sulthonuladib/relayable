//! `capture_stream` — capture the RAW byte stream a real full-screen program
//! writes to a real pty, then decode which control sequences it actually emits.
//!
//! Usage:
//!   capture_stream --name NAME --out DIR [--cols N] [--rows N]
//!                   [--timeout-ms MS] [--send DELAY_MS:SPEC]... -- PROG [ARGS]...
//!
//! `SPEC` is decoded with `\n`, `\r`, `\t`, `\e` (= ESC) and `\xNN` escapes,
//! so keystrokes can be logged in the provenance header without embedding raw
//! control bytes in the command line.
//!
//! Outputs (under `--out`):
//!   NAME.bin  exact raw bytes received from the pty master
//!   NAME.esc  caret-escaped text companion
//!   NAME.seq  decoded sequence listing + category checklist
//!
//! ADR-003 claim: alt-screen / mouse / bracketed-paste / OSC / cursor-mode /
//! SGR support can be established with byte-level evidence against real
//! programs, independently of any terminal-emulation engine.

use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::thread;
use std::time::{Duration, Instant};

use relayable_terminal_engine_spike::{esc, pty};

const REQUIRED: &[&str] = &[
    "alt_screen_1049_enter",
    "alt_screen_1049_leave",
    "alt_screen_47_enter",
    "alt_screen_47_leave",
    "alt_screen_1047_enter",
    "alt_screen_1047_leave",
    "mouse_1000_enable",
    "mouse_1000_disable",
    "mouse_1002_enable",
    "mouse_1002_disable",
    "mouse_1003_enable",
    "mouse_1003_disable",
    "mouse_1005_enable",
    "mouse_1005_disable",
    "mouse_1006_enable",
    "mouse_1006_disable",
    "mouse_1015_enable",
    "mouse_1015_disable",
    "bracketed_paste_enable",
    "bracketed_paste_disable",
    "osc_title",
    "osc_hyperlink",
    "osc_color",
    "cursor_hide",
    "cursor_show",
    "decscusr",
    "sgr",
];

#[derive(Default)]
struct Args {
    name: String,
    out: PathBuf,
    cols: u16,
    rows: u16,
    timeout_ms: u64,
    sends: Vec<(u64, Vec<u8>)>,
    program: String,
    program_args: Vec<OsString>,
}

fn decode_spec(spec: &str) -> Vec<u8> {
    let b = spec.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < b.len() {
        if b[i] == b'\\' && i + 1 < b.len() {
            match b[i + 1] {
                b'n' => {
                    out.push(b'\n');
                    i += 2;
                }
                b'r' => {
                    out.push(b'\r');
                    i += 2;
                }
                b't' => {
                    out.push(b'\t');
                    i += 2;
                }
                b'e' => {
                    out.push(0x1b);
                    i += 2;
                }
                b'\\' => {
                    out.push(b'\\');
                    i += 2;
                }
                b'x' if i + 3 < b.len() => {
                    let hi = std::str::from_utf8(&b[i + 2..i + 4])
                        .ok()
                        .and_then(|h| u8::from_str_radix(h, 16).ok());
                    if let Some(v) = hi {
                        out.push(v);
                        i += 4;
                    } else {
                        out.push(b[i]);
                        i += 1;
                    }
                }
                _ => {
                    out.push(b[i]);
                    i += 1;
                }
            }
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    out
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        cols: 80,
        rows: 24,
        timeout_ms: 4000,
        ..Args::default()
    };
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0usize;
    while i < argv.len() {
        match argv[i].as_str() {
            "--name" => {
                i += 1;
                a.name = argv.get(i).ok_or("--name needs a value")?.clone();
            }
            "--out" => {
                i += 1;
                a.out = PathBuf::from(argv.get(i).ok_or("--out needs a value")?);
            }
            "--cols" => {
                i += 1;
                a.cols = argv
                    .get(i)
                    .ok_or("--cols needs a value")?
                    .parse()
                    .map_err(|e| format!("--cols: {e}"))?;
            }
            "--rows" => {
                i += 1;
                a.rows = argv
                    .get(i)
                    .ok_or("--rows needs a value")?
                    .parse()
                    .map_err(|e| format!("--rows: {e}"))?;
            }
            "--timeout-ms" => {
                i += 1;
                a.timeout_ms = argv
                    .get(i)
                    .ok_or("--timeout-ms needs a value")?
                    .parse()
                    .map_err(|e| format!("--timeout-ms: {e}"))?;
            }
            "--send" => {
                i += 1;
                let v = argv.get(i).ok_or("--send needs a value")?;
                let (delay, spec) = v.split_once(':').ok_or("--send needs DELAY_MS:SPEC")?;
                let delay: u64 = delay.parse().map_err(|e| format!("--send delay: {e}"))?;
                a.sends.push((delay, decode_spec(spec)));
            }
            "--" => {
                let rest = &argv[i + 1..];
                if rest.is_empty() {
                    return Err("no program given after --".to_string());
                }
                a.program = rest[0].clone();
                a.program_args = rest[1..].iter().map(OsString::from).collect();
                return Ok(a);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        i += 1;
    }
    Err("missing `-- PROGRAM [ARGS...]`".to_string())
}

fn terminate(child: &mut std::process::Child) -> Option<ExitStatus> {
    if let Ok(Some(s)) = child.try_wait() {
        return Some(s);
    }
    let pid = child.id() as i32;
    let _ = pty::killpg(pid, libc::SIGTERM);
    for _ in 0..50 {
        if let Ok(Some(s)) = child.try_wait() {
            return Some(s);
        }
        thread::sleep(Duration::from_millis(20));
    }
    let _ = pty::killpg(pid, libc::SIGKILL);
    for _ in 0..50 {
        if let Ok(Some(s)) = child.try_wait() {
            return Some(s);
        }
        thread::sleep(Duration::from_millis(20));
    }
    None
}

fn main() {
    let a = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("capture_stream: {e}");
            std::process::exit(2);
        }
    };
    if a.name.is_empty() {
        eprintln!("capture_stream: --name is required");
        std::process::exit(2);
    }
    if let Err(e) = fs::create_dir_all(&a.out) {
        eprintln!("capture_stream: cannot create {}: {e}", a.out.display());
        std::process::exit(2);
    }

    println!("== capture_stream: {name} ==", name = a.name);
    println!("PROGRAM={}", a.program);
    println!("PROGRAM_ARGS={:?}", a.program_args);
    println!("PTY_SIZE={} rows x {} cols", a.rows, a.cols);
    println!("TERM={}", pty::TERM);
    println!("TIMEOUT_MS={}", a.timeout_ms);
    for (d, s) in &a.sends {
        println!("SEND_AT_MS={d} SPEC_ESCAPED={:?}", esc::caret(s));
    }

    let mut sends = a.sends.clone();
    sends.sort_by_key(|(d, _)| *d);

    let mut session = match pty::spawn(&a.program, &a.program_args, &[], a.rows, a.cols) {
        Ok(s) => s,
        Err(e) => {
            println!("SPAWN_ERROR={e}");
            println!("VERDICT=FAIL");
            std::process::exit(1);
        }
    };
    println!("CHILD_PID={}", session.pid());
    println!("SLAVE_DEVICE={}", session.spawn.slave_path);

    let start = Instant::now();
    let mut idx = 0usize;
    let mut exit_status: Option<ExitStatus> = None;
    loop {
        let elapsed = start.elapsed();
        if idx < sends.len() && elapsed >= Duration::from_millis(sends[idx].0) {
            if let Err(e) = session.spawn.master.write_all(&sends[idx].1) {
                println!("SEND_WRITE_ERROR={e}");
            }
            let _ = session.spawn.master.flush();
            idx += 1;
            continue;
        }
        match session.spawn.child.try_wait() {
            Ok(Some(s)) => {
                exit_status = Some(s);
                break;
            }
            Ok(None) => {}
            Err(e) => {
                println!("WAIT_ERROR={e}");
                break;
            }
        }
        if elapsed >= Duration::from_millis(a.timeout_ms) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    if exit_status.is_none() {
        println!(
            "NOTE=did not exit within {} ms; terminating process group",
            a.timeout_ms
        );
        exit_status = terminate(&mut session.spawn.child);
    }

    let raw = session.finish();
    println!("BYTES_RECEIVED={}", raw.len());
    match &exit_status {
        Some(s) => {
            use std::os::unix::process::ExitStatusExt;
            println!("CHILD_EXIT_CODE={:?}", s.code());
            println!("CHILD_EXIT_SIGNAL={:?}", s.signal());
        }
        None => println!("CHILD_EXIT_CODE=<unknown>"),
    }

    let bin = a.out.join(format!("{}.bin", a.name));
    let path_esc = a.out.join(format!("{}.esc", a.name));
    let path_seq = a.out.join(format!("{}.seq", a.name));
    if let Err(e) = fs::write(&bin, &raw) {
        println!("WRITE_ERROR={bin:?}: {e}");
    }

    let seqs = esc::scan(&raw);
    let counts = esc::summary(&seqs);

    let mut seq_out = String::new();
    seq_out.push_str(&format!("# decoded sequence listing for {}\n", a.name));
    seq_out.push_str(&format!("# total raw bytes: {}\n", raw.len()));
    seq_out.push_str("# category checklist:\n");
    for c in REQUIRED {
        match counts.get(c) {
            Some((n, off, len, detail)) => {
                let bytes = raw.get(*off..off + len).map(esc::hex).unwrap_or_default();
                seq_out.push_str(&format!(
                    "# CHECK {c} = OBSERVED count={n} first_offset={off} first_bytes=[{bytes}] {detail}\n"
                ));
            }
            None => seq_out.push_str(&format!("# CHECK {c} = NOT_OBSERVED\n")),
        }
    }
    seq_out.push_str("# all decoded sequences:\n");
    seq_out.push_str(&esc::listing(&seqs));
    let _ = fs::write(&path_seq, &seq_out);
    let _ = fs::write(&path_esc, esc::caret(&raw));

    println!();
    println!("CATEGORY_CHECKLIST");
    for c in REQUIRED {
        match counts.get(c) {
            Some((n, off, len, _)) => {
                let bytes = raw.get(*off..off + len).map(esc::hex).unwrap_or_default();
                println!("CHECK {c} = OBSERVED count={n} first_offset={off} first_bytes=[{bytes}]");
            }
            None => println!("CHECK {c} = NOT_OBSERVED"),
        }
    }
    println!();
    println!("WROTE={}", bin.display());
    println!("WROTE={}", path_esc.display());
    println!("WROTE={}", path_seq.display());
    println!("VERDICT={}", if raw.is_empty() { "FAIL" } else { "PASS" });

    if raw.is_empty() {
        std::process::exit(1);
    }
}
