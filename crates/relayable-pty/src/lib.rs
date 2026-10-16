//! Relayable PTY/process boundary.
//!
//! ADR-001 places PTY ownership on the **server**: this crate is the only place
//! that allocates pseudo-terminals and drives child processes, and it depends on
//! **nothing in the Relayable domain** (`relayable-model`, `relayable-protocol`,
//! the server, the client, or the CLI). It intentionally exposes only opaque
//! handles: no `portable_pty` type appears in this crate's public API, so the
//! domain never learns which PTY implementation is used.
//!
//! PTY ownership is independent of any client connection. A [`PtyHandle`] keeps
//! the master side and the child process alive whether or not a client is
//! attached; output that no client consumes stays buffered by the kernel, and
//! detaching a client never signals or stops the child. Only an explicit
//! [`PtyHandle::shutdown`] / [`PtyHandle::kill`] terminates it.
#![deny(unsafe_code)]

mod handle;
mod size;
mod status;

pub use handle::{KillHandle, PtyHandle};
pub use size::PtySize;
pub use status::PtyExitStatus;
