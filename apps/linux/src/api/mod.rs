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
//! * **Data plane** — a private [`relayable_protocol::transport::FrameConnection`]
//!   that mirrors the client library's handshake byte shape exactly, then
//!   `block.attach` (WRITER), then reads `TerminalOutput` frames, sending the
//!   mandatory per-frame `FlowControl` acknowledgement the server's credit
//!   window requires (`crates/relayable-server/src/session.rs`).
//!
//! # Threading
//!
//! [`ApiHandle::connect_live`] spawns one background thread per plane. The UI
//! thread only calls the non-blocking methods below; it never performs socket
//! I/O. Both planes are synchronous `std::io` over a Unix socket, matching
//! ADR-002 (no async runtime).
//!
//! # Flow-control accounting (interpretation)
//!
//! The server charges the credit window per **output payload byte**
//! (`Subscriber::try_consume(bytes)`, `session.rs`; `add_credit(ack_bytes)`), and
//! the reference client acknowledges `output.bytes.len()`
//! (`relayable-client/src/lib.rs`, `wait_for_output_inner`). This module
//! therefore acknowledges the exact `TerminalOutput.bytes` length — the byte
//! length of the terminal-data frame's payload. Envelope/framing overhead is
//! **not** acked; acking it would permanently pin the window at its cap.
//!
//! The data plane holds **many** concurrent attachments, so it acks output for
//! every attached stream it currently holds, not just one. The server permits up
//! to its advertised `max_concurrent_streams` (default 8) streams per connection
//! (`crates/relayable-protocol/src/limits.rs`, enforced in
//! `crates/relayable-server/src/dispatch.rs`); the handshake's advertised limit
//! is logged at connect time.
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

pub use control::ApiHandle;
pub use types::{
    ApiCommand, ApiEvent, ConnectionState, SessionSummaryView, RECONNECT_BACKOFF,
    RECONNECT_MAX_ATTEMPTS,
};

#[cfg(test)]
mod tests;
