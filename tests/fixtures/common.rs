//! Constants and environment-hermeticity helpers for the integration harness.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Default generous timeout for a server to become ready or a response to arrive.
pub const WAIT: Duration = Duration::from_secs(10);
/// Shorter timeout for a cleanup/reconnect observation.
pub const RETRY: Duration = Duration::from_secs(3);

static NONCE: AtomicU64 = AtomicU64::new(0);

/// Print one auditable evidence line for the CORE-001 acceptance run.
pub fn step(label: &str, detail: &str) {
    eprintln!("[CORE-001] {label}: {detail}");
}

/// The explicit `PATH` handed to child processes. The harness is environment-
/// hermetic (`env_clear`), so the one variable a shell actually needs is set
/// from the parent if present, else from a conventional default.
pub(crate) fn child_path() -> std::ffi::OsString {
    std::env::var_os("PATH").unwrap_or_else(|| {
        std::ffi::OsString::from("/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
    })
}

/// A unique, non-`rand` nonce: process id + atomic counter + wall-clock nanos.
pub fn nonce(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let counter = NONCE.fetch_add(1, Ordering::SeqCst);
    format!("{prefix}-{:x}-{nanos:x}-{counter:x}", std::process::id())
}
