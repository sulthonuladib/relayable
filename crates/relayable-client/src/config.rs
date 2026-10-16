use std::time::Duration;

#[cfg(doc)]
use crate::client::Client;
#[cfg(doc)]
use crate::error::ClientError;

/// Maximum time a control RPC (and the initial handshake) waits for its
/// response before failing with [`ClientError::Timeout`].
pub const CONTROL_RPC_TIMEOUT: Duration = Duration::from_secs(10);

/// Maximum number of received `Event` frames buffered inside a [`Client`] when
/// the caller has not drained them yet.
///
/// CORE-002 events are control-class frames the client may receive at any time;
/// [`Client::drain_events`], [`Client::wait_for_event`], and
/// [`Client::collect_events`] surface them. The buffer is bounded so a client
/// that never drains cannot grow without limit: when full, the **oldest**
/// buffered event is evicted (FIFO) before the newest is stored. Evicted events
/// are not lost from the server's retained event log, so a later
/// [`Client::resume_session`] from the last applied cursor can re-fetch them.
/// Terminal bytes never live in this buffer and are never evicted by this bound.
pub const MAX_PENDING_EVENTS: usize = 4096;

/// Static client identity sent in the handshake.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Client kind (for example `cli`).
    pub client_kind: String,
    /// Operating-system platform (`std::env::consts::OS`).
    pub platform: String,
    /// Client version (`CARGO_PKG_VERSION`).
    pub client_version: String,
}

impl ClientConfig {
    /// Build a config for `kind`, filling platform and version from the build.
    #[must_use]
    pub fn new(kind: &str) -> Self {
        Self {
            client_kind: kind.to_string(),
            platform: std::env::consts::OS.to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}
