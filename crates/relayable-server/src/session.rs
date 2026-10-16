//! Per-block PTY runtime and per-session runtime.
//!
//! A [`BlockRuntime`] owns the server-side PTY child for the lifetime of the
//! session, independently of any client connection: it holds the
//! [`relayable_pty::PtyHandle`], a dedicated PTY-writer thread, the bounded
//! output tail, the per-stream output sequence, and the fan-out to attached
//! streams. Dropping a client never signals, reaps, or stops the child; only
//! [`BlockRuntime::close`] does.
//!
//! This file is the module's thin entry point: it declares the responsibility
//! submodules and re-exports the crate-visible API, so `crate::session::…` paths
//! are unchanged. The submodules are [`output`] (bounded tail and subscriber
//! list), [`subscriber`] (per-stream fan-out and credit), [`status`] (stored
//! program status and the writer lease), [`block`] (the block runtime),
//! [`process`] (POSIX process-group/session helpers), [`threads`] (block thread
//! helpers), and [`runtime`] (the per-session runtime).

mod block;
mod output;
mod process;
mod runtime;
mod status;
mod subscriber;
mod threads;

pub(crate) use block::{BlockInit, BlockRuntime, InputError, WriterAcquire};
pub(crate) use runtime::SessionRuntime;
pub(crate) use subscriber::Subscriber;
