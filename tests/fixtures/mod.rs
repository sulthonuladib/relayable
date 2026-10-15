//! Shared, bounded, leak-free harness for the CORE-001 integration tests.
//!
//! Everything here is synchronous and bounded: no test performs an unbounded
//! wait, and a failing test cannot leave a `relayable-server` or `sh` process
//! behind ([`TestServer`]'s `Drop` reaps the server and every tracked PTY child).
//!
//! The binaries are located relative to the running test executable rather than
//! a hard-coded `target/` path, so the harness works with `CARGO_TARGET_DIR`.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::Value;
use tempfile::TempDir;

/// Default generous timeout for a server to become ready or a response to arrive.
pub const WAIT: Duration = Duration::from_secs(10);
/// Shorter timeout for a cleanup/reconnect observation.
pub const RETRY: Duration = Duration::from_secs(3);

static NONCE: AtomicU64 = AtomicU64::new(0);

/// Print one auditable evidence line for the CORE-001 acceptance run.
pub fn step(label: &str, detail: &str) {
    eprintln!("[CORE-001] {label}: {detail}");
}

/// Resolve a workspace binary next to the running test executable.
///
/// **The acceptance tests require a `cargo build --workspace` first.** This
/// package intentionally does not depend on `relayable-server`/`relayable-cli`
/// (ADR-001: the tests package links only model/protocol/client), so `cargo test
/// -p relayable-integration-tests` does not build those binaries. To keep the
/// evidence honest, this function therefore fails loudly when the binary is
/// older than the newest crate source instead of silently running a stale
/// build (a stale-server run already produced misleading output once).
///
/// The test exe lives in `<target>/debug/deps/`, so pop the exe, pop `deps` when
/// present, and append `name`. Never hard-codes a target directory.
///
/// Panics if the binary is missing, not executable, or stale relative to the
/// newest `*.rs`/`*.toml` of the crates it is built from (see
/// [`dependency_crates`]).
#[must_use]
pub fn bin(name: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("current test executable path");
    let mut dir = exe
        .parent()
        .expect("test executable has a parent directory")
        .to_path_buf();
    if dir.file_name().and_then(|value| value.to_str()) == Some("deps") {
        dir = dir
            .parent()
            .expect("deps directory has a parent")
            .to_path_buf();
    }
    let path = dir.join(name);
    assert!(
        path.exists(),
        "binary {name:?} not found at {path:?}; run `cargo build --workspace` first"
    );
    let metadata =
        fs::metadata(&path).unwrap_or_else(|error| panic!("cannot stat binary {path:?}: {error}"));
    let mode = metadata.permissions().mode();
    assert!(
        mode & 0o111 != 0,
        "binary {path:?} is not executable (mode {:o})",
        mode & 0o777
    );
    guard_freshness(name, &path, &metadata);
    path
}

/// Pure staleness decision: `Some(reason)` when the newest source is strictly
/// newer than the binary, `None` when the binary is fresh. Kept free of I/O so
/// it can be unit-tested directly.
fn staleness(binary_mtime: SystemTime, newest_source_mtime: SystemTime) -> Option<String> {
    if newest_source_mtime > binary_mtime {
        Some(format!(
            "it is older (mtime {binary_mtime:?}) than the newest crate source (mtime {newest_source_mtime:?})"
        ))
    } else {
        None
    }
}

/// Fail loudly if `path` is older than the newest source it is built from,
/// naming that source.
fn guard_freshness(name: &str, path: &Path, metadata: &fs::Metadata) {
    let Ok(binary_mtime) = metadata.modified() else {
        return; // Cannot compare: do not block the run on an unsupported timestamp.
    };
    let Some((newest_mtime, newest_path)) = newest_dependency_source(name) else {
        return; // No sources found: nothing to compare against.
    };
    if let Some(reason) = staleness(binary_mtime, newest_mtime) {
        panic!(
            "refusing to run a stale {name:?} binary at {path:?}: {reason}. \
             The newest source it is built from is {newest_path:?}; run `cargo build --workspace` first."
        );
    }
}

/// The in-repo crates each workspace binary is built from. This mirrors the
/// frozen ADR-001 dependency graph; keep it in step with the manifests.
///
/// The freshness comparison must be scoped to a binary's real dependencies:
/// Cargo does **not** rebuild `relayable-cli` when only `relayable-server`
/// changes (and vice versa), so comparing against every crate under `crates/**`
/// would falsely flag a freshly built binary after an unrelated crate edit and
/// make the `cargo build --workspace` guidance useless.
fn dependency_crates(binary: &str) -> &'static [&'static str] {
    const SERVER: &[&str] = &[
        "relayable-server",
        "relayable-pty",
        "relayable-protocol",
        "relayable-model",
    ];
    const CLI: &[&str] = &[
        "relayable-cli",
        "relayable-client",
        "relayable-protocol",
        "relayable-model",
    ];
    const ALL: &[&str] = &[
        "relayable-model",
        "relayable-protocol",
        "relayable-pty",
        "relayable-server",
        "relayable-client",
        "relayable-cli",
    ];
    match binary {
        "relayable-server" => SERVER,
        "relayable" => CLI,
        // Conservative fallback for an unknown binary: compare against all crates.
        _ => ALL,
    }
}

/// The newest `*.rs`/`*.toml` mtime across the crates `binary` is built from.
fn newest_dependency_source(binary: &str) -> Option<(SystemTime, PathBuf)> {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?;
    let crates = repo.join("crates");
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    for crate_name in dependency_crates(binary) {
        if let Some((time, path)) = newest_source_mtime_in(&crates.join(crate_name)) {
            if newest.as_ref().is_none_or(|(current, _)| time > *current) {
                newest = Some((time, path));
            }
        }
    }
    newest
}

/// Walk `root` recursively and return the newest `(mtime, path)` of a `*.rs` or
/// `*.toml` file, skipping any `target` directory. Bounded and tiny (the crate
/// tree only).
fn newest_source_mtime_in(root: &Path) -> Option<(SystemTime, PathBuf)> {
    let mut newest: Option<(SystemTime, PathBuf)> = None;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => continue,
            };
            if file_type.is_dir() {
                if path.file_name().and_then(|value| value.to_str()) == Some("target") {
                    continue;
                }
                stack.push(path);
            } else if is_source_file(&path) {
                if let Ok(modified) = entry.metadata().and_then(|meta| meta.modified()) {
                    if newest.as_ref().is_none_or(|(time, _)| modified > *time) {
                        newest = Some((modified, path));
                    }
                }
            }
        }
    }
    newest
}

fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("rs" | "toml")
    )
}

/// The explicit `PATH` handed to child processes. The harness is environment-
/// hermetic (`env_clear`), so the one variable a shell actually needs is set
/// from the parent if present, else from a conventional default.
fn child_path() -> std::ffi::OsString {
    std::env::var_os("PATH").unwrap_or_else(|| {
        std::ffi::OsString::from("/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
    })
}

/// A unique, non-`rand` nonce: process id + atomic counter + wall-clock nanos.
pub fn nonce(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let counter = NONCE.fetch_add(1, Ordering::SeqCst);
    format!("{prefix}-{:x}-{nanos:x}-{counter:x}", std::process::id())
}

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

/// A foreground `relayable-server run` subprocess plus the PTY children it
/// created. Dropping it terminates everything, even on a test failure.
pub struct TestServer {
    child: Option<Child>,
    socket: PathBuf,
    _dir: TempDir,
    pids: Vec<u32>,
}

impl TestServer {
    /// Spawn `relayable-server run` with a private socket under a temp dir.
    pub fn spawn() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("relayable-it-")
            .tempdir()
            .expect("create temp dir");
        let socket = dir.path().join("relayable.sock");
        let child = Command::new(bin("relayable-server"))
            .arg("run")
            .env_clear()
            .env("PATH", child_path())
            .env("RELAYABLE_SERVER", &socket)
            .env("SHELL", "/bin/sh")
            .env("TERM", "xterm-256color")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn relayable-server");
        let server = Self {
            child: Some(child),
            socket,
            _dir: dir,
            pids: Vec::new(),
        };
        assert!(
            server.wait_ready(WAIT),
            "server never became ready on {:?}",
            server.socket
        );
        server
    }

    /// The socket path the server is listening on.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket
    }

    /// The server process id (0 once reaped).
    #[must_use]
    pub fn pid(&self) -> u32 {
        self.child.as_ref().map_or(0, Child::id)
    }

    /// Register a server-owned PTY child pid for cleanup.
    pub fn track_child(&mut self, pid: u32) {
        assert!(pid > 0, "tracked pid must be nonzero");
        self.pids.push(pid);
    }

    /// Bounded poll until the socket exists and accepts a connection.
    pub fn wait_ready(&self, timeout: Duration) -> bool {
        poll_until(timeout, Duration::from_millis(20), || {
            self.socket.exists() && UnixStream::connect(&self.socket).is_ok()
        })
    }

    /// Send `SIGTERM` and wait (bounded) for graceful exit; returns the status.
    pub fn shutdown(&mut self) -> Option<ExitStatus> {
        let mut child = self.child.take()?;
        let pid = child.id();
        let _ = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match child.try_wait() {
                Ok(Some(status)) => return Some(status),
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
            }
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let pid = child.id();
            let _ = Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let deadline = Instant::now() + Duration::from_secs(3);
            let mut exited = false;
            while Instant::now() < deadline {
                if matches!(child.try_wait(), Ok(Some(_))) {
                    exited = true;
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            if !exited {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
        for pid in &self.pids {
            let _ = Command::new("kill")
                .arg("-KILL")
                .arg(pid.to_string())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
}

/// Captured result of one `relayable` CLI invocation.
pub struct CliOutput {
    /// Process exit code (`-1` if killed by a signal).
    pub status: i32,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
}

impl CliOutput {
    /// The exit code as `i64` for equality checks against literals.
    #[must_use]
    pub fn code(&self) -> i64 {
        i64::from(self.status)
    }
}

/// Run the `relayable` binary with `args` and `env` (for example
/// `[("RELAYABLE_SERVER", sock)]`).
///
/// The child environment is cleared first, so a developer's `RELAYABLE_SESSION`
/// or `XDG_RUNTIME_DIR` cannot leak in; only the caller's explicit pairs (plus
/// `PATH`) reach the CLI.
pub fn cli(args: &[&str], env: &[(&str, &str)]) -> CliOutput {
    let mut command = Command::new(bin("relayable"));
    command.args(args).env_clear().env("PATH", child_path());
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command.output().expect("run relayable");
    CliOutput {
        status: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// Run `relayable` with `args`/`env`, assert exit 0, and parse stdout as JSON.
pub fn cli_json(args: &[&str], env: &[(&str, &str)]) -> Value {
    let output = cli(args, env);
    assert_eq!(
        output.status, 0,
        "relayable {args:?} failed: status={} stdout={:?} stderr={:?}",
        output.status, output.stdout, output.stderr
    );
    serde_json::from_str(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "relayable {args:?} stdout is not JSON ({error}): {:?}",
            output.stdout
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staleness_flags_only_an_older_binary() {
        let base = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000);
        let newer_source = base + Duration::from_secs(1);

        assert!(staleness(base, base).is_none(), "equal mtimes are fresh");
        assert!(
            staleness(newer_source, base).is_none(),
            "a binary newer than its sources is fresh"
        );
        let reason = staleness(base, newer_source).expect("an older binary is stale");
        assert!(
            reason.contains("older"),
            "the staleness reason should explain the condition: {reason}"
        );
    }

    #[test]
    fn source_scan_tolerates_a_missing_root() {
        assert!(newest_source_mtime_in(Path::new("/nonexistent-relayable-root")).is_none());
    }
}
