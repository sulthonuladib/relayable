use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use relayable_protocol::transport::FrameConnection;
use relayable_protocol::v1::{Envelope, Event, Limits, ServerHello};

use crate::config::ClientConfig;

mod connection;
mod control;
mod events;
mod terminal;

/// A synchronous Relayable control client.
pub struct Client {
    conn: FrameConnection,
    socket_path: PathBuf,
    hello: ServerHello,
    limits: Limits,
    config: ClientConfig,
    next_request_id: u64,
    pending: VecDeque<Envelope>,
    pending_events: VecDeque<Event>,
    /// When this client last actually sent a `Heartbeat`.
    ///
    /// Persisted across calls on purpose: repeated short waits must accumulate
    /// toward the outbound cadence instead of each call resetting it. Only
    /// [`Client::heartbeat_if_due`] advances it, and only after a heartbeat is
    /// sent.
    last_heartbeat_sent: Instant,
    pub(crate) control_rpc_timeout: Duration,
}
