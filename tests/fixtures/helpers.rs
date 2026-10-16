//! Bounded polling, process/socket/byte helpers, and event assertions.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Bounded polling: call `probe` every `interval` until it returns `true` or
/// `timeout` elapses.
pub fn poll_until(timeout: Duration, interval: Duration, mut probe: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if probe() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(interval);
    }
}

/// Whether `/proc/<pid>` exists.
pub fn proc_exists(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).is_dir()
}

/// The parent pid from `/proc/<pid>/stat`, robust to `comm` containing spaces
/// and parentheses.
pub fn proc_ppid(pid: u32) -> Option<u32> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let rest = &stat[stat.rfind(')')? + 1..];
    let mut fields = rest.split_whitespace();
    let _state = fields.next()?;
    fields.next()?.parse().ok()
}

/// The target of `/proc/<pid>/fd/0` (a PTY child's stdin is the PTY slave).
pub fn proc_fd0(pid: u32) -> Option<PathBuf> {
    fs::read_link(format!("/proc/{pid}/fd/0")).ok()
}

/// The permission bits (`0o600` etc.) of `path`.
pub fn socket_mode(path: &Path) -> u32 {
    fs::metadata(path)
        .map(|metadata| metadata.permissions().mode() & 0o777)
        .unwrap_or(0)
}

/// True for a socket read timeout (as opposed to a real disconnect).
pub fn is_timeout(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
    )
}

/// Substring search over bytes.
pub fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack.len() >= needle.len()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

/// Collect [`relayable_client::Event`] frames until `quiet` elapses with no new
/// event, or `cap` elapses. Events already buffered inside the client are
/// returned first. [`relayable_client::Client::wait_for_event`] heartbeats
/// while idle, so this never trips the server idle timeout.
pub fn collect_until_quiet(
    client: &mut relayable_client::Client,
    quiet: Duration,
    cap: Duration,
) -> Result<Vec<relayable_client::Event>, relayable_client::ClientError> {
    let mut events = Vec::new();
    let deadline = Instant::now() + cap;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        match client.wait_for_event(remaining.min(quiet))? {
            Some(event) => events.push(event),
            None => break,
        }
    }
    Ok(events)
}

/// Pump [`relayable_client::Client::collect_events`] in short `slice` windows
/// until `total` elapses, accumulating events in arrival order.
///
/// The client sends a `Heartbeat` whenever a read deadline expires. A single
/// long `collect_events` call can receive the server's own heartbeats on the
/// same cadence and therefore never reach its own deadline; short slices force
/// the documented keep-alive path so an observer stays alive across a server
/// idle-timeout window.
pub fn collect_events_over(
    client: &mut relayable_client::Client,
    slice: Duration,
    total: Duration,
) -> Result<Vec<relayable_client::Event>, relayable_client::ClientError> {
    let mut events = Vec::new();
    let deadline = Instant::now() + total;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        events.extend(client.collect_events(remaining.min(slice))?);
    }
    Ok(events)
}

/// Assert event sequences are strictly increasing by exactly one (no gap, no
/// duplicate) across the slice.
pub fn assert_event_sequences_contiguous(events: &[relayable_client::Event]) {
    for pair in events.windows(2) {
        assert_eq!(
            pair[1].sequence,
            pair[0].sequence + 1,
            "event sequences must be contiguous with no gap or duplicate: {} -> {}",
            pair[0].sequence,
            pair[1].sequence
        );
    }
}

/// Assert a `<kind>:<lowercase uuidv7>` tagged id: 36-char hyphenated canonical
/// lowercase UUID, version nibble `7`.
pub fn assert_tagged_uuid_v7(text: &str, kind: &str) {
    let prefix = format!("{kind}:");
    let rest = text
        .strip_prefix(&prefix)
        .unwrap_or_else(|| panic!("{text:?} does not use the {prefix:?} prefix"));
    assert_eq!(rest.len(), 36, "uuid length for {text:?}");
    for index in [8usize, 13, 18, 23] {
        assert_eq!(
            rest.as_bytes()[index],
            b'-',
            "hyphen at {index} in {rest:?}"
        );
    }
    let hex: Vec<char> = rest.chars().filter(|c| *c != '-').collect();
    assert_eq!(hex.len(), 32, "hex digits in {rest:?}");
    assert!(
        hex.iter()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "lowercase hex in {rest:?}"
    );
    assert_eq!(hex[12], '7', "UUIDv7 version nibble in {rest:?}");
}
