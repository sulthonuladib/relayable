//! Server configuration.
//!
//! The only environment inputs are `RELAYABLE_SERVER` (socket path override),
//! `SHELL` (shell override), and `XDG_RUNTIME_DIR` (via
//! [`relayable_protocol::transport::default_socket_path`]). `REX_*` names are
//! reference evidence only and are never read (ADR-002 D14; DEC-006).

use std::ffi::OsString;
use std::path::PathBuf;

use relayable_model::TerminalSize;
use relayable_protocol::limits::DEFAULT_LIMITS;
use relayable_protocol::v1::Limits;

/// Configuration for a [`crate::Server`].
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Filesystem path of the local Unix-domain socket.
    pub socket_path: PathBuf,
    /// Shell program spawned for each session (`$SHELL`, else `/bin/sh`).
    pub shell: OsString,
    /// Initial terminal geometry for new sessions.
    pub initial_size: TerminalSize,
    /// Advertised protocol limits.
    pub limits: Limits,
    /// Maximum per-block retained output tail, in bytes.
    pub output_tail_bytes: usize,
}

impl ServerConfig {
    /// Build from the environment.
    ///
    /// The socket path is `$RELAYABLE_SERVER` when set and non-empty, else
    /// [`relayable_protocol::transport::default_socket_path`]. The shell is
    /// `$SHELL` when set and non-empty, else `/bin/sh`. Remaining fields take
    /// documented defaults (80×24, [`DEFAULT_LIMITS`], 32 KiB tail).
    #[must_use]
    pub fn from_env() -> Self {
        let shell = std::env::var_os("SHELL")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| OsString::from("/bin/sh"));
        let socket_path = std::env::var_os("RELAYABLE_SERVER")
            .filter(|value| !value.is_empty())
            .map_or_else(
                relayable_protocol::transport::default_socket_path,
                PathBuf::from,
            );
        Self {
            socket_path,
            shell,
            initial_size: TerminalSize::default(),
            limits: DEFAULT_LIMITS,
            output_tail_bytes: 32_768,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_documented() {
        let config = ServerConfig::from_env();
        assert!(!config.socket_path.as_os_str().is_empty());
        assert_eq!(config.initial_size, TerminalSize::default());
        assert_eq!(config.limits, DEFAULT_LIMITS);
        assert_eq!(config.output_tail_bytes, 32_768);
    }
}
