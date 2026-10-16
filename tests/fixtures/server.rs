//! The foreground `relayable-server` test harness.

use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use tempfile::TempDir;

use super::binary::bin;
use super::common::{child_path, WAIT};
use super::helpers::poll_until;

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
        Self::spawn_with_env(&[])
    }

    /// Spawn `relayable-server run` with explicit extra environment values.
    ///
    /// `extra` is applied *after* the harness defaults (`PATH`,
    /// `RELAYABLE_SERVER`, `SHELL`, `TERM`), so it is the supported way for a
    /// test to set the CORE-002 server parameters
    /// (`RELAYABLE_EVENT_RETENTION_EVENTS`, `RELAYABLE_IDLE_TIMEOUT_MILLIS`,
    /// `RELAYABLE_HEARTBEAT_INTERVAL_MILLIS`) for one spawned process. Each call
    /// still gets its own temp dir and socket path, so tests stay parallel-safe.
    pub fn spawn_with_env(extra: &[(&str, &str)]) -> Self {
        let dir = tempfile::Builder::new()
            .prefix("relayable-it-")
            .tempdir()
            .expect("create temp dir");
        let socket = dir.path().join("relayable.sock");
        let mut command = Command::new(bin("relayable-server"));
        command
            .arg("run")
            .env_clear()
            .env("PATH", child_path())
            .env("RELAYABLE_SERVER", &socket)
            .env("SHELL", "/bin/sh")
            .env("TERM", "xterm-256color");
        for (key, value) in extra {
            command.env(key, value);
        }
        let child = command
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
