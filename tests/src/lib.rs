//! Relayable cross-crate integration test harness.
//!
//! The shared helpers live in `tests/fixtures/mod.rs` and are re-exported here
//! so each integration-test crate can `use relayable_integration_tests::harness::*`.
//! Contract tests per area live under `tests/{protocol,lifecycle,reconnect,multi-client}`
//! (ADR-001). This crate owns no production behavior; it only spawns the real
//! `relayable-server` / `relayable` binaries and drives the real client library.

#[path = "../fixtures/mod.rs"]
pub mod harness;
