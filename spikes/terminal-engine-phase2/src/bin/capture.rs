//! `capture` — drive a real program on a real PTY (portable-pty 0.9.0), inject
//! timed input / resizes, and persist the exact byte stream.
//!
//! ```text
//! capture --name NAME [--out DIR] [--cols N] [--rows N] [--duration-ms MS]
//!         [--send DELAY_MS:SPEC]... [--resize-ms MS:COLSxROWS]...
//!         [--desc TEXT] [--no-version] -- PROGRAM [ARGS...]
//! ```
//!
//! Outputs under `--out` (default `results/raw/streams`):
//! * `NAME.bin`  — exact bytes read from the PTY master;
//! * `NAME.meta` — provenance, argv, geometry, TERM/COLORTERM, duration,
//!   program `--version` (best effort), byte count and SHA-256.

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use relayable_terminal_engine_phase2_spike::pty::{
    self, PtyRun, ResizeSpec, SendSpec, COLORTERM, TERM,
};
use relayable_terminal_engine_phase2_spike::sha256::sha256_hex;

#[derive(Debug, Default)]
struct Args {
    name: String,
    out: PathBuf,
    cols: u16,
    rows: u16,
    duration_ms: u64,
    desc: String,
    no_version: bool,
    sends: Vec<(u64, String, Vec<u8>)>,
    resizes: Vec<(u64, u16, u16)>,
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
        out: PathBuf::from("results/raw/streams"),
        cols: 80,
        rows: 24,
        duration_ms: 4000,
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
            "--name" => {
                i += 1;
                a.name = val(i)?.clone();
            }
            "--out" => {
                i += 1;
                a.out = PathBuf::from(val(i)?);
            }
            "--cols" => {
                i += 1;
                a.cols = val(i)?.parse().map_err(|e| format!("--cols: {e}"))?;
            }
            "--rows" => {
                i += 1;
                a.rows = val(i)?.parse().map_err(|e| format!("--rows: {e}"))?;
            }
            "--duration-ms" => {
                i += 1;
                a.duration_ms = val(i)?.parse().map_err(|e| format!("--duration-ms: {e}"))?;
            }
            "--desc" => {
                i += 1;
                a.desc = val(i)?.clone();
            }
            "--no-version" => a.no_version = true,
            "--send" => {
                i += 1;
                let v = val(i)?;
                let (delay, spec) = v.split_once(':').ok_or("--send needs DELAY_MS:SPEC")?;
                let delay: u64 = delay.parse().map_err(|e| format!("--send delay: {e}"))?;
                a.sends.push((delay, spec.to_string(), decode_spec(spec)));
            }
            "--resize-ms" => {
                i += 1;
                let v = val(i)?;
                let (ms, dims) = v.split_once(':').ok_or("--resize-ms needs MS:COLSxROWS")?;
                let ms: u64 = ms.parse().map_err(|e| format!("--resize-ms ms: {e}"))?;
                let (cols, rows) = dims.split_once('x').ok_or("--resize-ms needs COLSxROWS")?;
                a.resizes.push((
                    ms,
                    cols.parse().map_err(|e| format!("--resize-ms cols: {e}"))?,
                    rows.parse().map_err(|e| format!("--resize-ms rows: {e}"))?,
                ));
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

fn main() {
    let a = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("capture: {e}");
            std::process::exit(2);
        }
    };
    if a.name.is_empty() {
        eprintln!("capture: --name is required");
        std::process::exit(2);
    }
    if let Err(e) = fs::create_dir_all(&a.out) {
        eprintln!("capture: cannot create {}: {e}", a.out.display());
        std::process::exit(2);
    }

    let run = PtyRun {
        program: a.program.clone(),
        args: a.program_args.clone(),
        cols: a.cols,
        rows: a.rows,
        duration: Duration::from_millis(a.duration_ms),
        sends: a
            .sends
            .iter()
            .map(|(ms, _, bytes)| SendSpec {
                at: Duration::from_millis(*ms),
                bytes: bytes.clone(),
            })
            .collect(),
        resizes: a
            .resizes
            .iter()
            .map(|(ms, cols, rows)| ResizeSpec {
                at: Duration::from_millis(*ms),
                cols: *cols,
                rows: *rows,
            })
            .collect(),
        env: Vec::new(),
        cwd: None,
    };

    println!("== capture: {} ==", a.name);
    println!("PROGRAM={}", a.program);
    println!("PROGRAM_ARGS={:?}", a.program_args);
    println!("PTY_SIZE={} rows x {} cols", a.rows, a.cols);
    println!("TERM={TERM}");
    println!("COLORTERM={COLORTERM}");
    println!("DURATION_MS={}", a.duration_ms);
    for (ms, spec, _) in &a.sends {
        println!("SEND_AT_MS={ms} SPEC={spec:?}");
    }
    for (ms, cols, rows) in &a.resizes {
        println!("RESIZE_AT_MS={ms} TO={cols}x{rows}");
    }

    let capture = match pty::run(&run) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("capture: {e}");
            println!("VERDICT=FAIL");
            std::process::exit(1);
        }
    };

    let bin_path = a.out.join(format!("{}.bin", a.name));
    let meta_path = a.out.join(format!("{}.meta", a.name));
    if let Err(e) = fs::write(&bin_path, &capture.bytes) {
        eprintln!("capture: write {}: {e}", bin_path.display());
        std::process::exit(1);
    }
    let version = if a.no_version {
        "<skipped --no-version>".to_string()
    } else {
        pty::program_version(&a.program, Duration::from_secs(3))
    };

    let mut meta = String::new();
    meta.push_str("kind=real-program\n");
    meta.push_str(&format!("name={}\n", a.name));
    meta.push_str(&format!("description={}\n", a.desc));
    meta.push_str(&format!("program={}\n", a.program));
    meta.push_str(&format!("program_args={:?}\n", a.program_args));
    meta.push_str(&format!("program_version={version}\n"));
    meta.push_str(&format!("cols={}\n", a.cols));
    meta.push_str(&format!("rows={}\n", a.rows));
    meta.push_str(&format!("TERM={TERM}\n"));
    meta.push_str(&format!("COLORTERM={COLORTERM}\n"));
    meta.push_str(&format!("duration_ms={}\n", a.duration_ms));
    meta.push_str(&format!("send_count={}\n", a.sends.len()));
    for (ms, spec, _) in &a.sends {
        meta.push_str(&format!("send={ms}:{spec}\n"));
    }
    meta.push_str(&format!("resize_count={}\n", capture.applied_resizes.len()));
    for (ms, cols, rows) in &capture.applied_resizes {
        meta.push_str(&format!("resize={ms}:{cols}x{rows}\n"));
    }
    meta.push_str(&format!("bytes={}\n", capture.bytes.len()));
    meta.push_str(&format!("sha256={}\n", sha256_hex(&capture.bytes)));
    meta.push_str(&format!("exit={}\n", capture.exit));
    meta.push_str(&format!("exit_code={:?}\n", capture.exit_code));
    meta.push_str(&format!("killed={}\n", capture.killed));
    meta.push_str(&format!("timed_out={}\n", capture.timed_out));
    meta.push_str(&format!("reader_stalled={}\n", capture.reader_stalled));
    meta.push_str(&format!("slave={:?}\n", capture.slave_path));
    meta.push_str(&format!("child_pid={:?}\n", capture.child_pid));
    meta.push_str(&format!("pgid={:?}\n", capture.pgid));
    meta.push_str(&format!(
        "utc_timestamp={}\n",
        relayable_terminal_engine_phase2_spike::report::utc_now()
    ));
    if let Err(e) = fs::write(&meta_path, meta) {
        eprintln!("capture: write {}: {e}", meta_path.display());
        std::process::exit(1);
    }

    println!("BYTES_RECEIVED={}", capture.bytes.len());
    println!("SHA256={}", sha256_hex(&capture.bytes));
    println!("CHILD_EXIT={}", capture.exit);
    println!("CHILD_EXIT_CODE={:?}", capture.exit_code);
    println!("KILLED={}", capture.killed);
    println!("TIMED_OUT={}", capture.timed_out);
    println!("READER_STALLED={}", capture.reader_stalled);
    println!("WROTE={}", bin_path.display());
    println!("WROTE={}", meta_path.display());
    if capture.bytes.is_empty() {
        println!("VERDICT=FAIL (empty capture)");
        std::process::exit(1);
    }
    println!("VERDICT=PASS");
}
