//! Integration tests for `relayable-pty` against a real PTY.
//!
//! These tests allocate real pseudo-terminals and spawn real `/bin/sh`
//! processes; nothing is mocked. Every read is bounded by a deadline so the
//! suite cannot hang.

use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use relayable_pty::{PtyHandle, PtySize};

const READ_DEADLINE: Duration = Duration::from_secs(10);

/// Collects every chunk the reader thread sends until EOF, the deadline, or an
/// optional `stop_at` marker. Returns the bytes and whether the deadline was
/// hit before EOF/marker.
fn read_until(
    rx: &mpsc::Receiver<Vec<u8>>,
    deadline: Instant,
    stop_at: Option<&str>,
) -> (Vec<u8>, bool) {
    let mut output = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return (output, true);
        }
        match rx.recv_timeout(remaining) {
            Ok(chunk) => {
                output.extend_from_slice(&chunk);
                if let Some(marker) = stop_at {
                    if String::from_utf8_lossy(&output).contains(marker) {
                        return (output, false);
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => return (output, true),
            Err(RecvTimeoutError::Disconnected) => return (output, false),
        }
    }
}

/// Spawns a reader thread that forwards PTY output over a channel.
fn spawn_reader(handle: &PtyHandle) -> mpsc::Receiver<Vec<u8>> {
    let mut reader = handle.reader().expect("clone pty reader");
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    rx
}

#[test]
fn short_lived_child_output_and_exit_code() {
    let mut handle = PtyHandle::spawn(
        OsStr::new("/bin/sh"),
        &[
            OsString::from("-c"),
            OsString::from("echo PTY_READY; stty size; tty; exit 7"),
        ],
        PtySize::DEFAULT,
        None,
        &[],
    )
    .expect("spawn short-lived child");

    let rx = spawn_reader(&handle);
    let (output, timed_out) = read_until(&rx, Instant::now() + READ_DEADLINE, None);
    if timed_out {
        let _ = handle.kill();
    }
    assert!(
        !timed_out,
        "reader deadline exceeded; output so far: {:?}",
        String::from_utf8_lossy(&output)
    );

    let status = handle.wait().expect("wait for child");
    let text = String::from_utf8_lossy(&output);

    assert!(text.contains("PTY_READY"), "missing PTY_READY in {text:?}");
    assert!(
        text.contains("24 80"),
        "stty size should report 24 rows x 80 cols: {text:?}"
    );
    assert!(
        text.contains("/dev/pts/"),
        "tty should be a pts device: {text:?}"
    );
    assert_eq!(status.exit_code, 7, "unexpected exit code: {status:?}");
    assert!(!status.success, "exit code 7 must not be a success");
}

#[test]
fn resize_reaches_child_and_shutdown_terminates_it() {
    let mut handle = PtyHandle::spawn(OsStr::new("/bin/sh"), &[], PtySize::DEFAULT, None, &[])
        .expect("spawn long-lived shell");

    handle.resize(PtySize::new(100, 40)).expect("resize pty");

    let mut writer = handle.take_writer().expect("take writer");
    writer.write_all(b"stty size\n").expect("write stty size");
    writer.flush().expect("flush stty size");

    let rx = spawn_reader(&handle);
    let (output, timed_out) = read_until(&rx, Instant::now() + READ_DEADLINE, Some("40 100"));
    assert!(
        !timed_out,
        "resize did not reach the child; output: {:?}",
        String::from_utf8_lossy(&output)
    );

    // Release the writer; its drop emits EOT, then shutdown makes sure the
    // child is gone within the grace period.
    drop(writer);
    let status = handle
        .shutdown(Duration::from_millis(500))
        .expect("shutdown child");
    assert!(status.is_some(), "shutdown must observe the child exiting");
    assert!(
        handle
            .try_wait()
            .expect("try_wait after shutdown")
            .is_some(),
        "child status must be reported after shutdown"
    );

    let pid = handle.process_id().expect("process id");
    let proc_path = format!("/proc/{pid}");
    assert!(
        !Path::new(&proc_path).exists(),
        "{proc_path} must not exist after the child is gone"
    );
}

#[test]
fn writer_can_only_be_taken_once() {
    let mut handle = PtyHandle::spawn(OsStr::new("/bin/sh"), &[], PtySize::DEFAULT, None, &[])
        .expect("spawn shell");

    let _writer = handle.take_writer().expect("first take_writer succeeds");
    let second = handle.take_writer();
    assert!(second.is_err(), "second take_writer must return Err");

    let _ = handle.kill();
    let _ = handle.wait();
}

#[test]
fn exposes_process_id_and_tty_name() {
    let mut handle = PtyHandle::spawn(OsStr::new("/bin/sh"), &[], PtySize::DEFAULT, None, &[])
        .expect("spawn shell");

    let pid = handle.process_id().expect("process id must be known");
    assert!(pid > 0, "process id must be positive");

    let tty = handle.tty_name().expect("tty name must be known");
    assert!(tty.starts_with("/dev/pts/"), "unexpected tty path: {tty:?}");
    assert!(Path::new(&tty).exists(), "tty path must exist: {tty:?}");

    let _ = handle.kill();
    let _ = handle.wait();
}
