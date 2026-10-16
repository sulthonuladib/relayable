//! Relayable client library.
//!
//! This crate owns the connection lifecycle, the synchronous request/response
//! RPC surface, and terminal-stream reads over the local Unix-domain socket
//! transport. It depends on `relayable-model` and `relayable-protocol` only and
//! never on server internals (ADR-001 §7).
//!
//! # Concurrency
//!
//! The client is **synchronous and blocking**: it uses `std::io` over
//! `std::os::unix::net`, with no async runtime (`docs/adr/ADR-002-protocol-schema-and-transport.md`;
//! `full.md` §5.2). A dropped socket is surfaced as [`ClientError::Disconnected`]
//! on the next call; no I/O or protocol path panics.
//!
//! # Handshake (frozen by the CORE-001 slice note)
//!
//! The first frame on a connection is
//! `Envelope{ protocol_version: "1.0", request_id: Some(1), stream_id: 0,
//! body: Handshake{..} }`. The server echoes `request_id = 1` with
//! `Response.result = Hello`. Ordinary request ids then start at `2` and
//! increment monotonically per connection. Every control request uses
//! `stream_id = 0` (`relayable_protocol::CONTROL_STREAM_ID`).
//!
//! # Control RPC deadlines
//!
//! Every control RPC (and the initial handshake) waits at most
//! [`CONTROL_RPC_TIMEOUT`] for its response and then fails with
//! [`ClientError::Timeout`] instead of blocking forever. [`Client::wait_for_output`]
//! instead uses its caller-supplied timeout.
//!
//! # Re-exports
//!
//! Protocol types that appear in this crate's public API are re-exported so a
//! consumer such as the future `relayable-automation` (ADR-001
//! `automation -> {model, client}`) does not need a direct
//! `relayable-protocol` dependency:
//!
//! ```
//! use relayable_client::{
//!     AttachMode, Error, ErrorCode, Limits, ServerHello, SessionRecord,
//!     SessionSnapshotResponse, SessionSummary, TerminalResizeAccepted, TerminalSnapshot,
//!     TransportError,
//! };
//! let _ = AttachMode::Writer;
//! let _ = ErrorCode::Internal;
//! let _ = ServerHello::default();
//! let _ = SessionRecord::default();
//! let _ = SessionSnapshotResponse::default();
//! let _ = SessionSummary::default();
//! let _ = TerminalResizeAccepted::default();
//! let _ = TerminalSnapshot::default();
//! let _ = Limits::default();
//! let _: Option<Error> = None;
//! fn _accepts_transport(_: TransportError) {}
//! ```

#![deny(unsafe_code)]

mod client;
mod config;
mod error;
mod outcome;
mod rpc;

pub use client::Client;
pub use config::{ClientConfig, CONTROL_RPC_TIMEOUT, MAX_PENDING_EVENTS};
pub use error::ClientError;
pub use outcome::{AttachOutcome, CreatedSession};

/// Protocol types that appear in this crate's public API, re-exported so
/// consumers (`relayable-automation`: ADR-001 `automation -> {model, client}`)
/// do not need a direct `relayable-protocol` dependency.
pub use relayable_protocol::transport::TransportError;
pub use relayable_protocol::v1::{
    AttachMode, BlockAttachResponse, Error, ErrorCode, Event, Limits, ServerHello, SessionRecord,
    SessionResumeResponse, SessionSnapshotResponse, SessionSummary, TerminalResizeAccepted,
    TerminalSnapshot,
};

#[cfg(test)]
mod tests;
