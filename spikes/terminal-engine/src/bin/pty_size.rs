//! `pty_size` — real PTY window-size / SIGWINCH ground truth.
//!
//! Proves, with the real kernel API on this Linux host:
//!   1. a real pty is opened with a requested window size;
//!   2. a child started on the slave sees that size (`stty size`);
//!   3. a real `ioctl(TIOCSWINSZ)` on the master changes the child's view;
//!   4. the kernel delivers `SIGWINCH` to the child's foreground process group.
//!
//! ADR-003 claim: "resize" is observable and testable independently of any
//! terminal-emulation engine. See `results/README.md`.

use std::ffi::OsString;
use std::io::Write;
use std::os::fd::AsRawFd;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use relayable_terminal_engine_spike::{esc, pty};

const ROWS0: u16 = 24;
const COLS0: u16 = 80;
const ROWS1: u16 = 40;
const COLS1: u16 = 120;

const SCRIPT: &str = r#"printf 'INITIAL_STTY_SIZE=%s\n' "$(stty size)"
GOT=0
trap 'printf "SIGWINCH_STTY_SIZE=%s\n" "$(stty size)"; GOT=1' WINCH
i=0
while [ "$i" -lt 400 ] && [ "$GOT" -eq 0 ]; do sleep 0.05; i=$((i+1)); done
if [ "$GOT" -eq 0 ]; then printf 'TIMEOUT_NO_SIGWINCH\n'; exit 3; fi
sleep 0.5
exit 0
"#;

fn value_after(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        line.find(key)
            .map(|idx| line[idx + key.len()..].trim().to_string())
    })
}

fn ps_line(pid: u32) -> String {
    match Command::new("ps")
        .args([
            "-o",
            "pid=,ppid=,pgid=,sid=,tpgid=,stat=,comm=",
            "-p",
            &pid.to_string(),
        ])
        .output()
    {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(e) => format!("<ps failed: {e}>"),
    }
}

fn fmt_ws(ws: &nix::pty::Winsize) -> String {
    format!("{} {}", ws.ws_row, ws.ws_col)
}

fn main() {
    let mut failures: Vec<String> = Vec::new();

    println!("== pty_size: real PTY window-size / SIGWINCH probe ==");
    println!("TERM={}", pty::TERM);
    println!("REQUESTED_INITIAL_WINSIZE={ROWS0} rows x {COLS0} cols");

    let args: Vec<OsString> = vec!["-c".into(), SCRIPT.into()];
    let mut session = match pty::spawn("/bin/sh", &args, &[], ROWS0, COLS0) {
        Ok(s) => s,
        Err(e) => {
            println!("SPAWN_ERROR={e}");
            println!("VERDICT=FAIL");
            std::process::exit(1);
        }
    };
    println!("CHILD_PID={}", session.pid());
    println!("SLAVE_DEVICE={}", session.spawn.slave_path);
    println!("CHILD_PS_INITIAL={}", ps_line(session.pid() as u32));

    // Wait for the child's first `stty size`.
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut text = String::new();
    while Instant::now() < deadline {
        text = String::from_utf8_lossy(&session.snapshot()).into_owned();
        if text.contains("INITIAL_STTY_SIZE=") {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    let child_initial = value_after(&text, "INITIAL_STTY_SIZE=");
    let master_initial = pty::get_winsize(session.spawn.master.as_raw_fd());
    match (&master_initial, &child_initial) {
        (Ok(ws), Some(_)) => println!("MASTER_WINSIZE_INITIAL={}", fmt_ws(ws)),
        (Err(e), _) => println!("MASTER_WINSIZE_INITIAL_ERROR={e}"),
        _ => println!("MASTER_WINSIZE_INITIAL=<not read>"),
    }
    println!(
        "CHILD_STTY_SIZE_INITIAL={}",
        child_initial
            .clone()
            .unwrap_or_else(|| "<not observed>".to_string())
    );

    // Real resize.
    let ws1 = pty::winsize(ROWS1, COLS1);
    println!("SET_WINSIZE_CALL=ioctl(master, TIOCSWINSZ, {{ws_row:{ROWS1}, ws_col:{COLS1}}})");
    match pty::set_winsize(session.spawn.master.as_raw_fd(), &ws1) {
        Ok(()) => println!("SET_WINSIZE_RC=0"),
        Err(e) => {
            println!("SET_WINSIZE_RC=<error: {e}>");
            failures.push(format!("TIOCSWINSZ failed: {e}"));
        }
    }
    println!("CHILD_PS_AFTER_RESIZE={}", ps_line(session.pid() as u32));

    // Wait for SIGWINCH reaction / exit.
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut status = None;
    while Instant::now() < deadline {
        if let Ok(Some(s)) = session.spawn.child.try_wait() {
            status = Some(s);
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    if status.is_none() {
        let _ = pty::killpg(session.pid(), libc::SIGKILL);
        status = session.spawn.child.wait().ok();
        failures.push("child did not exit after resize".to_string());
    }

    let master_after = pty::get_winsize(session.spawn.master.as_raw_fd());
    match &master_after {
        Ok(ws) => println!("MASTER_WINSIZE_AFTER={}", fmt_ws(ws)),
        Err(e) => println!("MASTER_WINSIZE_AFTER_ERROR={e}"),
    }
    text = String::from_utf8_lossy(&session.snapshot()).into_owned();
    let child_after = value_after(&text, "SIGWINCH_STTY_SIZE=");
    println!(
        "CHILD_STTY_SIZE_AFTER={}",
        child_after
            .clone()
            .unwrap_or_else(|| "<not observed>".to_string())
    );
    println!(
        "SIGWINCH_OBSERVED={}",
        if child_after.is_some() {
            "true"
        } else {
            "false"
        }
    );
    if text.contains("TIMEOUT_NO_SIGWINCH") {
        println!("CHILD_TIMEOUT_NOTE=TIMEOUT_NO_SIGWINCH was printed");
    }

    match &status {
        Some(s) => {
            println!("CHILD_EXIT_CODE={:?}", s.code());
            println!("CHILD_EXIT_SUCCESS={}", s.success());
            if !s.success() {
                failures.push(format!("child exit not success: {s:?}"));
            }
        }
        None => println!("CHILD_EXIT_CODE=<unknown>"),
    }

    // Expected-value checks.
    let expect_initial = format!("{ROWS0} {COLS0}");
    let expect_after = format!("{ROWS1} {COLS1}");
    match &child_initial {
        Some(v) if *v == expect_initial => {}
        other => failures.push(format!(
            "initial child size {:?} != expected {:?}",
            other, expect_initial
        )),
    }
    match &child_after {
        Some(v) if *v == expect_after => {}
        other => failures.push(format!(
            "post-resize child size {:?} != expected {:?}",
            other, expect_after
        )),
    }
    if let Ok(ws) = &master_after {
        if format!("{} {}", ws.ws_row, ws.ws_col) != expect_after {
            failures.push("master winsize after resize does not reflect TIOCSWINSZ".to_string());
        }
    } else {
        failures.push("could not read master winsize after resize".to_string());
    }

    let raw = session.finish();
    println!();
    println!(
        "VERDICT={}",
        if failures.is_empty() { "PASS" } else { "FAIL" }
    );
    for f in &failures {
        println!("FAILURE={f}");
    }
    println!();
    println!(
        "== raw child output (caret-escaped, {} bytes) ==",
        raw.len()
    );
    print!("{}", esc::caret(&raw));
    let _ = std::io::stdout().flush();

    if failures.is_empty() {
        std::process::exit(0);
    }
    std::process::exit(1);
}
