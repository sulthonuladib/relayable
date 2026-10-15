//! Minimal, verified PTY ownership for the spike.
//!
//! Every API used here was checked against the real `nix` 0.31.3 and `libc`
//! sources before use (see `spikes/terminal-engine/README.md` for the audit
//! note). No terminal-engine or PTY-wrapper crate is involved.

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use nix::errno::Errno;
use nix::fcntl::{open, OFlag};
use nix::pty::{grantpt, posix_openpt, ptsname_r, unlockpt, Winsize};
use nix::sys::stat::Mode;

/// Term set for every spike child so full-screen programs pick a modern tty
/// capability set. Recorded in each result file.
pub const TERM: &str = "xterm-256color";

pub fn errno_to_io(e: Errno) -> io::Error {
    io::Error::from_raw_os_error(e as i32)
}

pub fn winsize(rows: u16, cols: u16) -> Winsize {
    Winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    }
}

/// TIOCGWINSZ on either end of the pty (both ends share the same window size).
pub fn get_winsize(fd: i32) -> io::Result<Winsize> {
    let mut ws = winsize(0, 0);
    let rc = unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) };
    if rc < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ws)
    }
}

/// TIOCSWINSZ on either end of the pty. The kernel delivers SIGWINCH to the
/// foreground process group of the slave side.
pub fn set_winsize(fd: i32, ws: &Winsize) -> io::Result<()> {
    let rc = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ, ws) };
    if rc < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn killpg(pid: i32, signal: i32) -> io::Result<()> {
    let rc = unsafe { libc::killpg(pid, signal) };
    if rc < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub struct PtySpawn {
    /// Master side. Kept open by the parent for the whole session.
    pub master: File,
    pub child: Child,
    /// e.g. "/dev/pts/4"
    pub slave_path: String,
}

/// A running PTY child plus a background thread that accumulates every byte the
/// child writes. The reader stops on EOF or EIO (both mean the slave side is
/// fully closed).
pub struct PtySession {
    pub spawn: PtySpawn,
    pub output: Arc<Mutex<Vec<u8>>>,
    reader: JoinHandle<()>,
}

impl PtySession {
    pub fn pid(&self) -> i32 {
        self.spawn.child.id() as i32
    }

    /// Clone of everything read from the pty so far.
    pub fn snapshot(&self) -> Vec<u8> {
        self.output.lock().unwrap().clone()
    }

    /// Block until the background reader finishes (child + all its slave fds
    /// exited). Consumes self.
    pub fn finish(self) -> Vec<u8> {
        let PtySession {
            spawn: _,
            output,
            reader,
        } = self;
        let _ = reader.join();
        Arc::try_unwrap(output)
            .map(|m| m.into_inner().unwrap())
            .unwrap_or_else(|_| Vec::new())
    }
}

/// Open the master, unlock the slave, and open the slave device.
fn open_pair(rows: u16, cols: u16) -> io::Result<(File, String, OwnedFd)> {
    let master = posix_openpt(OFlag::O_RDWR | OFlag::O_NOCTTY).map_err(errno_to_io)?;
    grantpt(&master).map_err(errno_to_io)?;
    unlockpt(&master).map_err(errno_to_io)?;
    let name = ptsname_r(&master).map_err(errno_to_io)?;
    let slave = open(
        Path::new(&name),
        OFlag::O_RDWR | OFlag::O_NOCTTY,
        Mode::empty(),
    )
    .map_err(errno_to_io)?;
    // Set the requested window size before the child starts, so the child's
    // very first TIOCGWINSZ (e.g. via `stty size`) sees it.
    set_winsize(master.as_raw_fd(), &winsize(rows, cols))?;
    let master: OwnedFd = master.into();
    Ok((File::from(master), name, slave))
}

/// Spawn `program args...` as a new session leader with the pty slave as its
/// controlling terminal and stdio, WITHOUT starting a reader thread. The caller
/// owns the returned master and decides how (and how much) to read, which is
/// what the scrollback/RSS observation needs.
pub fn spawn_only(
    program: &str,
    args: &[OsString],
    envs: &[(&str, &str)],
    rows: u16,
    cols: u16,
) -> io::Result<PtySpawn> {
    let (master, slave_path, slave_fd) = open_pair(rows, cols)?;
    let slave = File::from(slave_fd);

    let stdin = Stdio::from(slave.try_clone()?);
    let stdout = Stdio::from(slave.try_clone()?);
    let stderr = Stdio::from(slave);

    let mut cmd = Command::new(program);
    cmd.args(args)
        .env("TERM", TERM)
        .stdin(stdin)
        .stdout(stdout)
        .stderr(stderr);
    for (k, v) in envs {
        cmd.env(k, v);
    }

    // Runs in the forked child after stdio has been set up and immediately
    // before exec. `setsid` makes the child a session leader (so it is not a
    // process-group leader); TIOCSCTTY on fd 0 (the slave) then makes the pty
    // its controlling terminal, which is what routes SIGWINCH to it.
    unsafe {
        cmd.pre_exec(|| {
            if libc::setsid() < 0 {
                return Err(io::Error::last_os_error());
            }
            if libc::ioctl(0, libc::TIOCSCTTY, 0) < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }

    let child = cmd.spawn()?;

    Ok(PtySpawn {
        master,
        child,
        slave_path,
    })
}

/// Spawn and start the accumulating reader thread.
pub fn spawn(
    program: &str,
    args: &[OsString],
    envs: &[(&str, &str)],
    rows: u16,
    cols: u16,
) -> io::Result<PtySession> {
    let spawn = spawn_only(program, args, envs, rows, cols)?;
    let output = Arc::new(Mutex::new(Vec::new()));
    let mut read_end = spawn.master.try_clone()?;
    let sink = Arc::clone(&output);
    let reader = thread::spawn(move || {
        let mut buf = [0u8; 16384];
        loop {
            match read_end.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => sink.lock().unwrap().extend_from_slice(&buf[..n]),
                Err(ref e) if e.raw_os_error() == Some(libc::EIO) => break,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
    });

    Ok(PtySession {
        spawn,
        output,
        reader,
    })
}
