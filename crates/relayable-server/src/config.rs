//! Server configuration.
//!
//! Environment inputs are `RELAYABLE_SERVER` (socket path override),
//! `RELAYABLE_EVENT_RETENTION_EVENTS`, `RELAYABLE_IDLE_TIMEOUT_MILLIS`,
//! `RELAYABLE_HEARTBEAT_INTERVAL_MILLIS`, `SHELL` (shell override),
//! `RELAYABLE_STATE_DIR` / `XDG_STATE_HOME` / `HOME` (durable state directory,
//! via [`relayable_store::resolve_state_dir`]),
//! `RELAYABLE_STORE_AUDIT_MAX_ROWS`, `RELAYABLE_STORE_AUDIT_MAX_AGE_MILLIS`,
//! `RELAYABLE_STORE_TOMBSTONES_MAX_ROWS`,
//! `RELAYABLE_STORE_TOMBSTONES_MAX_AGE_MILLIS`,
//! `RELAYABLE_STORE_SESSIONS_MAX_ROWS`, and
//! `RELAYABLE_STORE_SESSIONS_MAX_AGE_MILLIS` (durable store retention overrides,
//! read once at startup via [`relayable_store::PruneLimits::from_env`]), and
//! `XDG_RUNTIME_DIR` (via
//! [`relayable_protocol::transport::default_socket_path`]). `REX_*` names are
//! reference evidence only and are never read (ADR-002 D14; DEC-006).
//!
//! All numeric values are parameters, not invented constants. Invalid, empty,
//! and zero values are startup errors (non-zero exit); the heartbeat interval
//! must be strictly less than the idle timeout.

use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::Context;
use relayable_model::TerminalSize;
use relayable_protocol::limits::DEFAULT_LIMITS;
use relayable_protocol::v1::Limits;

/// Default retained-log count bound (`protocol/compatibility/core-002-method-set.md` §F).
pub const DEFAULT_EVENT_RETENTION_EVENTS: usize = 4096;
/// Default connection idle timeout, in milliseconds.
pub const DEFAULT_IDLE_TIMEOUT_MILLIS: u64 = 30_000;
/// Default server heartbeat interval, in milliseconds.
pub const DEFAULT_HEARTBEAT_INTERVAL_MILLIS: u64 = 5_000;

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
    /// Maximum number of retained control events (`RELAYABLE_EVENT_RETENTION_EVENTS`).
    pub event_retention_events: usize,
    /// Resolved durable state directory for the SQLite metadata/audit store, or
    /// `None` when the environment has no usable source (`ADR-004` D2.8).
    /// `None` is a legitimate "run without persistence" mode.
    pub state_dir: Option<PathBuf>,
}

impl ServerConfig {
    /// Build from the environment.
    ///
    /// The socket path is `$RELAYABLE_SERVER` when set and non-empty, else
    /// [`relayable_protocol::transport::default_socket_path`]. The shell is
    /// `$SHELL` when set and non-empty, else `/bin/sh`. The state directory is
    /// resolved by [`relayable_store::resolve_state_dir`]
    /// (`RELAYABLE_STATE_DIR` -> `$XDG_STATE_HOME/relayable` ->
    /// `$HOME/.local/state/relayable` -> `None`). The event-retention bound,
    /// idle timeout, and heartbeat interval take their documented defaults when
    /// unset.
    ///
    /// An unresolvable state directory is **not** an error: the server then runs
    /// with in-memory metadata only and logs a warning at startup.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid or zero `RELAYABLE_EVENT_RETENTION_EVENTS`,
    /// `RELAYABLE_IDLE_TIMEOUT_MILLIS`, or `RELAYABLE_HEARTBEAT_INTERVAL_MILLIS`,
    /// or when the heartbeat interval is not strictly less than the idle
    /// timeout. Callers exit non-zero.
    pub fn from_env() -> anyhow::Result<Self> {
        let shell = std::env::var_os("SHELL")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| OsString::from("/bin/sh"));
        let socket_path = std::env::var_os("RELAYABLE_SERVER")
            .filter(|value| !value.is_empty())
            .map_or_else(
                relayable_protocol::transport::default_socket_path,
                PathBuf::from,
            );
        let retention = std::env::var("RELAYABLE_EVENT_RETENTION_EVENTS").ok();
        let idle = std::env::var("RELAYABLE_IDLE_TIMEOUT_MILLIS").ok();
        let heartbeat = std::env::var("RELAYABLE_HEARTBEAT_INTERVAL_MILLIS").ok();
        let state_dir = relayable_store::resolve_state_dir().map(|(path, _source)| path);
        Self::from_parts(
            socket_path,
            shell,
            retention.as_deref(),
            idle.as_deref(),
            heartbeat.as_deref(),
            state_dir,
        )
    }

    /// Pure constructor used by [`ServerConfig::from_env`] and the unit tests so
    /// config validation is hermetic.
    fn from_parts(
        socket_path: PathBuf,
        shell: OsString,
        retention: Option<&str>,
        idle: Option<&str>,
        heartbeat: Option<&str>,
        state_dir: Option<PathBuf>,
    ) -> anyhow::Result<Self> {
        let event_retention_events = usize::try_from(parse_positive(
            "RELAYABLE_EVENT_RETENTION_EVENTS",
            retention,
            DEFAULT_EVENT_RETENTION_EVENTS as u64,
        )?)
        .context("RELAYABLE_EVENT_RETENTION_EVENTS does not fit in usize")?;
        let idle_timeout_millis = parse_positive(
            "RELAYABLE_IDLE_TIMEOUT_MILLIS",
            idle,
            DEFAULT_IDLE_TIMEOUT_MILLIS,
        )?;
        let heartbeat_interval_millis = parse_positive(
            "RELAYABLE_HEARTBEAT_INTERVAL_MILLIS",
            heartbeat,
            DEFAULT_HEARTBEAT_INTERVAL_MILLIS,
        )?;
        if heartbeat_interval_millis >= idle_timeout_millis {
            anyhow::bail!(
                "RELAYABLE_HEARTBEAT_INTERVAL_MILLIS ({heartbeat_interval_millis}) must be \
                 strictly less than RELAYABLE_IDLE_TIMEOUT_MILLIS ({idle_timeout_millis})"
            );
        }
        let limits = Limits {
            heartbeat_interval_millis: u32::try_from(heartbeat_interval_millis)
                .context("RELAYABLE_HEARTBEAT_INTERVAL_MILLIS does not fit in u32")?,
            idle_timeout_millis: u32::try_from(idle_timeout_millis)
                .context("RELAYABLE_IDLE_TIMEOUT_MILLIS does not fit in u32")?,
            ..DEFAULT_LIMITS
        };
        Ok(Self {
            socket_path,
            shell,
            initial_size: TerminalSize::default(),
            limits,
            output_tail_bytes: 32_768,
            event_retention_events,
            state_dir,
        })
    }
}

/// Parse a positive `u64` from an optional environment value.
fn parse_positive(name: &str, raw: Option<&str>, default: u64) -> anyhow::Result<u64> {
    let Some(raw) = raw else {
        return Ok(default);
    };
    let value = raw
        .trim()
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("{name} must be a positive integer, got {raw:?}"))?;
    if value == 0 {
        anyhow::bail!("{name} must be greater than zero");
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(
        retention: Option<&str>,
        idle: Option<&str>,
        heartbeat: Option<&str>,
    ) -> anyhow::Result<ServerConfig> {
        ServerConfig::from_parts(
            PathBuf::from("/tmp/relayable-test.sock"),
            OsString::from("/bin/sh"),
            retention,
            idle,
            heartbeat,
            None,
        )
    }

    #[test]
    fn defaults_are_documented() {
        let config = parts(None, None, None).expect("defaults are valid");
        assert_eq!(config.initial_size, TerminalSize::default());
        assert_eq!(config.limits, DEFAULT_LIMITS);
        assert_eq!(config.output_tail_bytes, 32_768);
        assert_eq!(config.event_retention_events, 4096);
        assert_eq!(config.limits.idle_timeout_millis, 30_000);
        assert_eq!(config.limits.heartbeat_interval_millis, 5_000);
    }

    #[test]
    fn explicit_values_are_applied() {
        let config = parts(Some("128"), Some("4000"), Some("1000")).expect("valid overrides");
        assert_eq!(config.event_retention_events, 128);
        assert_eq!(config.limits.idle_timeout_millis, 4000);
        assert_eq!(config.limits.heartbeat_interval_millis, 1000);
    }

    #[test]
    fn zero_and_invalid_are_rejected() {
        assert!(parts(Some("0"), None, None).is_err());
        assert!(parts(Some("nope"), None, None).is_err());
        assert!(parts(None, Some("0"), None).is_err());
        assert!(parts(None, Some("-1"), None).is_err());
        assert!(parts(None, None, Some("0")).is_err());
        assert!(parts(None, None, Some("later")).is_err());
    }

    #[test]
    fn heartbeat_must_be_strictly_less_than_idle() {
        assert!(parts(None, Some("1000"), Some("1000")).is_err());
        assert!(parts(None, Some("1000"), Some("2000")).is_err());
        assert!(parts(None, Some("2000"), Some("1000")).is_ok());
    }
}
