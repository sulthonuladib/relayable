//! Local Unix-domain socket transport (ADR-002 D6; `full.md` §6.2, §7.1).
//!
//! This module is only compiled with the non-default `transport-unix` feature,
//! so a default build of `relayable-protocol` stays free of `std::os::unix`
//! and socket types and remains usable from the Android client (DEC-010).
//!
//! Peer-credential checks (`SO_PEERCRED`) are deliberately deferred: they would
//! require `libc` and `unsafe`, and are not part of the CORE-001 acceptance
//! list. The socket directory is created `0700` and the socket file `0600` as
//! the restrictive-ownership baseline required by `full.md` §6.2 / §11.

use std::io;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::codec::{read_envelope, write_envelope, CodecError};
use crate::limits::DEFAULT_LIMITS;
use crate::v1::Envelope;

/// Transport-layer failure.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// A socket operation failed.
    #[error("transport i/o error: {0}")]
    Io(#[from] io::Error),
    /// A frame could not be written or read.
    #[error("frame codec error: {0}")]
    Codec(#[from] CodecError),
}

/// Resolve the default local-socket path.
///
/// Order (contract §3, ADR-002 D6, `full.md` §7.1):
/// 1. `$XDG_RUNTIME_DIR/relayable/relayable.sock` when `XDG_RUNTIME_DIR` is a
///    non-empty existing directory;
/// 2. `/run/user/<uid>/relayable/relayable.sock` when that runtime root exists;
/// 3. `$TMPDIR/relayable-<uid>/relayable.sock`, falling back to
///    `/tmp/relayable-<uid>/relayable.sock`.
#[must_use]
pub fn default_socket_path() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        let dir = PathBuf::from(dir);
        if !dir.as_os_str().is_empty() && dir.is_dir() {
            return dir.join("relayable").join("relayable.sock");
        }
    }

    let uid = current_uid();
    let run_user = PathBuf::from(format!("/run/user/{uid}"));
    if run_user.is_dir() {
        return run_user.join("relayable").join("relayable.sock");
    }

    let tmp = std::env::var_os("TMPDIR")
        .filter(|value| !value.is_empty())
        .map_or_else(|| PathBuf::from("/tmp"), PathBuf::from);
    tmp.join(format!("relayable-{uid}")).join("relayable.sock")
}

/// The process's real uid, without `libc`.
///
/// Reads `/proc/self` ownership on Linux; falls back to the conventional `UID`
/// environment variable, then to `0`. Used only for the socket path, never for
/// authorization.
fn current_uid() -> u32 {
    use std::os::unix::fs::MetadataExt;
    if let Ok(meta) = std::fs::metadata("/proc/self") {
        return meta.uid();
    }
    if let Some(text) = std::env::var_os("UID").and_then(|v| v.to_str().map(str::to_owned)) {
        if let Ok(uid) = text.parse::<u32>() {
            return uid;
        }
    }
    0
}

/// Create the parent directory (`0700`), remove a stale socket file, bind, and
/// restrict the socket file to `0600`.
///
/// A socket file is only removed when connecting to it is refused, so a live
/// listener is never silently replaced.
///
/// # Errors
///
/// Returns an error if the directory cannot be created, a live listener already
/// owns `path`, or the bind fails.
pub fn bind_listener(path: &Path) -> io::Result<UnixListener> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
    }

    if path.exists() {
        match UnixStream::connect(path) {
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    format!("socket {} already has a live listener", path.display()),
                ));
            }
            Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
                std::fs::remove_file(path)?;
            }
            Err(e) => return Err(e),
        }
    }

    let listener = UnixListener::bind(path)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}

/// A framed connection over a Unix stream.
#[derive(Debug)]
pub struct FrameConnection {
    stream: UnixStream,
    max_frame_bytes: u32,
}

impl FrameConnection {
    /// Connect to `path` using the default frame limit.
    ///
    /// # Errors
    ///
    /// Returns the underlying connect error.
    pub fn connect(path: &Path) -> io::Result<Self> {
        Ok(Self {
            stream: UnixStream::connect(path)?,
            max_frame_bytes: DEFAULT_LIMITS.max_frame_bytes,
        })
    }

    /// Wrap an accepted/connected stream with an explicit frame limit.
    #[must_use]
    pub fn from_stream(stream: UnixStream, max_frame_bytes: u32) -> Self {
        Self {
            stream,
            max_frame_bytes,
        }
    }

    /// Duplicate the underlying stream (for a reader/writer split).
    ///
    /// # Errors
    ///
    /// Returns the underlying clone error.
    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            stream: self.stream.try_clone()?,
            max_frame_bytes: self.max_frame_bytes,
        })
    }

    /// The configured maximum accepted frame length.
    #[must_use]
    pub fn max_frame_bytes(&self) -> u32 {
        self.max_frame_bytes
    }

    /// Write one envelope.
    ///
    /// # Errors
    ///
    /// See [`TransportError`].
    pub fn send(&mut self, env: &Envelope) -> Result<(), TransportError> {
        write_envelope(&mut self.stream, env)?;
        Ok(())
    }

    /// Read one envelope, or `Ok(None)` at a clean EOF.
    ///
    /// # Errors
    ///
    /// See [`TransportError`].
    pub fn recv(&mut self) -> Result<Option<Envelope>, TransportError> {
        Ok(read_envelope(&mut self.stream, self.max_frame_bytes)?)
    }

    /// Set the read timeout.
    ///
    /// # Errors
    ///
    /// Returns the underlying error.
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.stream.set_read_timeout(timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::{envelope::Body, Heartbeat};
    use std::sync::Mutex;

    /// Environment variables are process-global; serialize tests that mutate.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn unique_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "relayable-proto-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn default_socket_path_uses_xdg_runtime_dir() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = unique_dir("xdg");
        let previous = std::env::var_os("XDG_RUNTIME_DIR");
        std::env::set_var("XDG_RUNTIME_DIR", &dir);

        let path = default_socket_path();

        match previous {
            Some(value) => std::env::set_var("XDG_RUNTIME_DIR", value),
            None => std::env::remove_var("XDG_RUNTIME_DIR"),
        }
        let _ = std::fs::remove_dir_all(&dir);

        assert!(path.starts_with(&dir));
        assert!(path.ends_with("relayable/relayable.sock"));
    }

    #[test]
    fn default_socket_path_ignores_empty_xdg_runtime_dir() {
        let _guard = ENV_LOCK.lock().unwrap();
        let previous = std::env::var_os("XDG_RUNTIME_DIR");
        std::env::set_var("XDG_RUNTIME_DIR", "");

        let path = default_socket_path();

        match previous {
            Some(value) => std::env::set_var("XDG_RUNTIME_DIR", value),
            None => std::env::remove_var("XDG_RUNTIME_DIR"),
        }

        assert!(path.ends_with("relayable/relayable.sock"));
    }

    #[test]
    fn bind_listener_creates_a_0600_socket_and_frames_round_trip() {
        let dir = unique_dir("bind");
        let path = dir.join("relayable.sock");
        let listener = bind_listener(&path).unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);

        let client = FrameConnection::connect(&path).unwrap();
        let (server_stream, _) = listener.accept().unwrap();
        let mut server = FrameConnection::from_stream(server_stream, 4096);
        let mut client = client;

        let env = Envelope {
            protocol_version: "1.0".to_string(),
            stream_id: 0,
            body: Some(Body::Heartbeat(Heartbeat { sequence: Some(1) })),
            ..Envelope::default()
        };
        client.send(&env).unwrap();
        assert_eq!(server.recv().unwrap(), Some(env));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bind_listener_refuses_to_replace_a_live_listener() {
        let dir = unique_dir("live");
        let path = dir.join("relayable.sock");
        let listener = bind_listener(&path).unwrap();
        let _keep = listener;

        let err = bind_listener(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AddrInUse);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
