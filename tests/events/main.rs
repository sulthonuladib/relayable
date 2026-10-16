//! CORE-002 assignment E — retained event log and `session.resume` proofs.
//!
//! These tests drive the real `relayable-server` and the real
//! `relayable-client`. The frozen semantics are
//! `protocol/compatibility/core-002-method-set.md`:
//!
//! - §B cursor rule: replayable iff `retained_floor_sequence <= cursor + 1` and
//!   `cursor <= current_sequence`; otherwise a fresh snapshot with a reason in
//!   `{no_cursor, cursor_below_floor, cursor_ahead}` and never a false replay;
//! - §C ordering: a `session.resume` response always precedes any live event
//!   with sequence greater than the captured head, and the snapshot/replay
//!   boundary has no gap and no duplicate;
//! - §D events: control-class frames on `stream_id 0`, no `request_id`, and
//!   `Envelope.sequence == Event.sequence`;
//! - §F configuration: retention/idle/heartbeat are per-process env parameters.
//!
//! The envelope-level assertions (stream id, request id, `Envelope.sequence`)
//! use the protocol crate's own `FrameConnection`, because the client API
//! intentionally surfaces only `Event`; every client-level operation uses the
//! frozen `relayable-client` API.

mod connection_state;
mod frames;
mod lifecycle;
mod replay;
mod retry;
mod stream;
mod terminal;
