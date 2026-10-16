//! Shared, bounded, leak-free harness for the CORE-001 integration tests.
//!
//! Everything here is synchronous and bounded: no test performs an unbounded
//! wait, and a failing test cannot leave a `relayable-server` or `sh` process
//! behind ([`TestServer`]'s `Drop` reaps the server and every tracked PTY child).
//!
//! The binaries are located relative to the running test executable rather than
//! a hard-coded `target/` path, so the harness works with `CARGO_TARGET_DIR`.
//!
//! The implementation is split by concern; this module only declares the
//! submodules and re-exports their public items under `harness::`.

mod binary;
mod cli;
mod common;
mod helpers;
mod server;

pub use binary::bin;
pub use cli::{cli, cli_json, CliOutput};
pub use common::{nonce, step, RETRY, WAIT};
pub use helpers::{
    assert_event_sequences_contiguous, assert_tagged_uuid_v7, collect_events_over,
    collect_until_quiet, contains, is_timeout, poll_until, proc_exists, proc_fd0, proc_ppid,
    socket_mode,
};
pub use server::TestServer;
