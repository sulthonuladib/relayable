//! Live server client for the UI: two internal connections, one non-blocking
//! event/command surface (GPUI-free).
//!
//! # Two planes
//!
//! * **Control plane** — [`relayable_client::Client`] on its own connection for
//!   `session.create` / `session.list` / `session.view` and for `session.resume`
//!   and `collect_events`, so reconnect visuals are driven by **real** server
//!   events. It never calls `block.attach`, so it never contends for a writer
//!   lease (multi-client is supported).
//! * **Data plane** — a **second** [`relayable_client::Client`] on its own
//!   connection: `attach_with_takeover` (WRITER, lease takeover so a
//!   post-reconnect re-attach wins the lease back),
//!   [`relayable_client::Client::next_output`] per
//!   attached stream, `send_input`, and `resize_with_pixels`. It never issues
//!   `session.*`, so the two planes share no state but the socket path.
//!
//! Both planes are the public client library: no module here speaks raw
//! `relayable_protocol` frames.
//!
//! # Threading
//!
//! [`ApiHandle::connect_live`] spawns one background thread per plane. The UI
//! thread only calls the non-blocking methods below; it never performs socket
//! I/O. Both planes are synchronous `std::io` over a Unix socket, matching
//! ADR-002 (no async runtime).
//!
//! # Flow control (delegated to the client, drained by this module)
//!
//! The server charges its per-stream credit window per **output payload byte**
//! (`Subscriber::deliver` / `try_consume`,
//! `crates/relayable-server/src/session/subscriber.rs`) and closes the **whole
//! connection** on `flow_control_overflow`.
//! [`relayable_client::Client::next_output`] acknowledges
//! exactly the chunk it returns, so the data plane never sends a `FlowControl`
//! frame itself; instead it drains **every** attached stream to empty before any
//! bounded blocking read and again after it (`data.rs::pump`). The data plane
//! holds many concurrent attachments (up to the server's advertised
//! `max_concurrent_streams`, default 8); the handshake's advertised limits are
//! logged at connect time.
//!
//! # Reconnect policy (client-side, Relayable-original)
//!
//! `session.resume` is core CORE-002 behavior, but a *reconnect policy* is
//! explicitly unimplemented in the core (`protocol/compatibility/core-002-method-set.md`
//! §G). This module implements **one bounded reconnect loop** with a fixed small
//! backoff; it emits `Reconnecting { attempt }` then `Connected`/`Error`. This
//! is a **client-side Relayable-original policy, not core protocol behavior**.
//!
//! # Mock
//!
//! [`ApiHandle::connect_mock`] is the **only** way to get synthetic data. It is
//! clearly labelled ([`ApiHandle::is_mock`] returns `true`) and never pretends
//! to be a live connection. Live is the default.

mod control;
mod data;
mod data_thread;
mod mock;
mod osc;
mod types;
mod wire;

pub use control::ApiHandle;
pub use types::{
    ApiCommand, ApiEvent, ConnectionState, SessionSummaryView, RECONNECT_BACKOFF,
    RECONNECT_MAX_ATTEMPTS,
};

#[cfg(test)]
mod tests;
