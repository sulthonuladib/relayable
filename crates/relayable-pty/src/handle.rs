use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use portable_pty::{native_pty_system, Child, ChildKiller, CommandBuilder, MasterPty};

use crate::size::PtySize;
use crate::status::PtyExitStatus;

/// A thread-transferable handle that can signal a child independently of the
/// thread blocked in [`PtyHandle::wait`].
///
/// On Unix this performs a best-effort `SIGHUP` delivery (see
/// [`PtyHandle::clone_killer`] for the exact semantics).
pub trait KillHandle: Send {
    /// Signals the child process to terminate.
    fn kill(&mut self) -> std::io::Result<()>;
}

struct ChildKillHandle {
    inner: Box<dyn ChildKiller + Send + Sync>,
}

impl KillHandle for ChildKillHandle {
    fn kill(&mut self) -> std::io::Result<()> {
        self.inner.kill()
    }
}

/// An owned pseudo-terminal plus its child process.
///
/// The handle is opaque: it owns the PTY master fd and the child, and is the
/// authority for reading output, writing input, resizing, observing exit, and
/// terminating the child. It is not `Clone`; a second reader can be obtained
/// with [`PtyHandle::reader`] (repeatable) and an independent killer with
/// [`PtyHandle::clone_killer`].
pub struct PtyHandle {
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    writer_taken: bool,
    status: Option<PtyExitStatus>,
}

// `MasterPty` is not `Debug`, so the derive cannot be used; this reports the
// observable identity of the handle without exposing the implementation type.
impl std::fmt::Debug for PtyHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtyHandle")
            .field("process_id", &self.child.process_id())
            .field("tty_name", &self.master.tty_name())
            .field("writer_taken", &self.writer_taken)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl PtyHandle {
    /// Spawns `program` with `args` attached to a **new** PTY of `size`.
    ///
    /// `envs` are added on top of the inherited environment. When `cwd` is
    /// given, the child starts there. The child becomes the session leader of
    /// the new PTY, so it never inherits a controlling terminal from the
    /// server.
    pub fn spawn(
        program: &OsStr,
        args: &[OsString],
        size: PtySize,
        cwd: Option<&Path>,
        envs: &[(OsString, OsString)],
    ) -> Result<PtyHandle> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(size.to_raw())
            .context("failed to allocate a pty")?;

        let mut builder = CommandBuilder::new(program);
        builder.args(args.iter());
        if let Some(cwd) = cwd {
            builder.cwd(cwd);
        }
        for (key, value) in envs {
            builder.env(key, value);
        }
        // Give the child the freshly allocated PTY as its controlling terminal.
        // portable-pty calls `setsid()` before `TIOCSCTTY`, so this is a new
        // session: the server's controlling terminal is never inherited.
        builder.set_controlling_tty(true);

        let child = pair
            .slave
            .spawn_command(builder)
            .context("failed to spawn child in pty")?;

        Ok(PtyHandle {
            master: pair.master,
            child,
            writer_taken: false,
            status: None,
        })
    }

    /// Returns an independent readable handle for the PTY master.
    ///
    /// This may be called more than once; each call clones the underlying fd.
    pub fn reader(&self) -> Result<Box<dyn Read + Send>> {
        self.master
            .try_clone_reader()
            .context("failed to clone pty reader")
    }

    /// Returns the writable handle for the PTY master.
    ///
    /// This may be called only once; a second call returns an error.
    pub fn take_writer(&mut self) -> Result<Box<dyn Write + Send>> {
        let writer = self
            .master
            .take_writer()
            .context("pty writer has already been taken")?;
        self.writer_taken = true;
        Ok(writer)
    }

    /// Informs the kernel and the child that the window size changed.
    pub fn resize(&self, size: PtySize) -> Result<()> {
        self.master
            .resize(size.to_raw())
            .context("failed to resize pty")
    }

    /// Reads the PTY geometry currently known by the kernel.
    pub fn size(&self) -> Result<PtySize> {
        self.master
            .get_size()
            .map(PtySize::from_raw)
            .context("failed to read pty size")
    }

    /// Returns the child process id, when the platform reports one.
    pub fn process_id(&self) -> Option<u32> {
        self.child.process_id()
    }

    /// Returns the child's tty device path (for example `/dev/pts/3`).
    pub fn tty_name(&self) -> Option<PathBuf> {
        self.master.tty_name()
    }

    /// Polls the child, returning its status once it has exited.
    ///
    /// The observed status is cached, so repeated calls after exit keep
    /// reporting it.
    pub fn try_wait(&mut self) -> Result<Option<PtyExitStatus>> {
        if let Some(status) = &self.status {
            return Ok(Some(status.clone()));
        }
        match self.child.try_wait().context("failed to poll pty child")? {
            Some(status) => {
                let status = convert_status(status);
                self.status = Some(status.clone());
                Ok(Some(status))
            }
            None => Ok(None),
        }
    }

    /// Waits for the child to exit and returns its status.
    ///
    /// The observed status is cached, so repeated calls after exit keep
    /// reporting it.
    pub fn wait(&mut self) -> Result<PtyExitStatus> {
        if let Some(status) = &self.status {
            return Ok(status.clone());
        }
        let status = convert_status(self.child.wait().context("failed to wait for pty child")?);
        self.status = Some(status.clone());
        Ok(status)
    }

    /// Terminates the child.
    ///
    /// On Unix this is `portable_pty`'s `ChildKiller for std::process::Child`
    /// implementation: it sends `SIGHUP` to the child pid, polls `try_wait` for
    /// up to ~200ms, and then falls back to `std::process::Child::kill` (a
    /// `SIGKILL` to the child pid) if it is still alive. It targets the child
    /// pid only, not the whole process group.
    pub fn kill(&mut self) -> Result<()> {
        self.child.kill().context("failed to signal pty child")
    }

    /// Returns a killer that can be used from another thread.
    ///
    /// On Unix this wraps `portable_pty`'s `ProcessSignaller`, which sends a
    /// best-effort `SIGHUP` to the child pid and nothing else: it does **not**
    /// escalate to `SIGKILL`. Use [`PtyHandle::kill`] when guaranteed
    /// termination is required.
    pub fn clone_killer(&self) -> Box<dyn KillHandle> {
        Box::new(ChildKillHandle {
            inner: self.child.clone_killer(),
        })
    }

    /// Best-effort graceful stop.
    ///
    /// If the writer has not been taken, writes `exit\n` to the child's tty.
    /// Then waits up to `grace` for the child to exit; if it is still running,
    /// kills it. Returns the observed status, or `None` if the child could not
    /// be observed to exit even after the kill.
    pub fn shutdown(&mut self, grace: Duration) -> Result<Option<PtyExitStatus>> {
        if let Some(status) = self.try_wait()? {
            return Ok(Some(status));
        }

        if !self.writer_taken {
            if let Ok(mut writer) = self.master.take_writer() {
                let _ = writer.write_all(b"exit\n");
                let _ = writer.flush();
                self.writer_taken = true;
                // Dropping the writer also emits EOT, giving an interactive
                // shell a second chance to observe end-of-input.
            }
        }

        let deadline = Instant::now() + grace;
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(Some(status));
            }
            if Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        self.kill()?;
        let kill_deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(Some(status));
            }
            if Instant::now() >= kill_deadline {
                return Ok(None);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

fn convert_status(status: portable_pty::ExitStatus) -> PtyExitStatus {
    PtyExitStatus {
        exit_code: status.exit_code(),
        signal: status.signal().map(str::to_owned),
        success: status.success(),
    }
}
