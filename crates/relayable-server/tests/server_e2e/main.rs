//! End-to-end test: a real `relayable-server` subprocess driven by raw protocol
//! frames.
//!
//! Nothing is mocked. The test spawns the compiled binary, talks to it over a
//! real Unix socket using `relayable_protocol`'s `FrameConnection` (the
//! `relayable-client` crate is intentionally not a dependency), spawns a real
//! `/bin/sh` PTY child, and cleans up every process it started.
//!
//! The suite is split by scenario family: `harness` holds the shared
//! `ServerGuard`/`Client` helpers, and each sibling module covers one family.

mod block_validity;
mod end_to_end;
mod flow_control;
mod harness;
mod heartbeat;
mod lifecycle;
mod resume;
mod snapshot;
