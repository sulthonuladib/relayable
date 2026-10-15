//! `SIGINT`/`SIGTERM` handling.
//!
//! The installed handler is async-signal-safe: it does **not** lock, allocate,
//! or perform I/O, and only stores `true` in an [`AtomicBool`]. The accept loop
//! polls [`shutdown_requested`] between non-blocking `accept` calls.
//!
//! This module contains the crate's single documented `unsafe` block.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

/// Set by the signal handler; never cleared.
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

/// Ensures the handlers are installed at most once.
static INSTALL: Once = Once::new();

/// Install `SIGINT`/`SIGTERM` handlers. Idempotent.
pub(crate) fn install() {
    INSTALL.call_once(|| {
        // SAFETY: `libc::signal` is called with a valid signal number and a
        // valid `extern "C"` handler. The handler only performs an atomic store,
        // which is async-signal-safe. Installation happens once, before the
        // accept loop starts.
        let handler = handle_signal as extern "C" fn(libc::c_int);
        // Route through a raw pointer to avoid a `fn`-to-integer cast lint.
        let handler = handler as *const () as libc::sighandler_t;
        unsafe {
            libc::signal(libc::SIGINT, handler);
            libc::signal(libc::SIGTERM, handler);
        }
    });
}

/// Whether a shutdown signal has been observed.
pub(crate) fn shutdown_requested() -> bool {
    SHUTDOWN.load(Ordering::SeqCst)
}

extern "C" fn handle_signal(_signal: libc::c_int) {
    SHUTDOWN.store(true, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_is_idempotent_and_readable() {
        // Do not raise signals in-process (it would set the global flag for the
        // whole test binary); just prove install/read are safe to call.
        install();
        install();
        let _ = shutdown_requested();
    }
}
