//! The block runtime's dedicated threads and their constants.

use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Maximum number of queued PTY-input messages per block.
pub(super) const INPUT_QUEUE_MESSAGES: usize = 1024;
/// Maximum bytes of PTY input queued per block; memory is bounded by this.
pub(super) const INPUT_QUEUE_BYTES: usize = 4 * 1024 * 1024;
/// How long `close` waits for each block thread before detaching it.
pub(super) const THREAD_JOIN_TIMEOUT: Duration = Duration::from_secs(1);

/// Drain queued input into the PTY from a dedicated thread.
///
/// The lock is never held while writing: the thread alone owns the writer, and
/// the connection thread only enqueues. Byte reservations are released after the
/// write attempt so a blocked write applies backpressure via the byte budget.
pub(super) fn input_loop(
    receiver: Receiver<Vec<u8>>,
    mut writer: Box<dyn Write + Send>,
    reserved: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::SeqCst) {
        match receiver.recv_timeout(Duration::from_millis(25)) {
            Ok(chunk) => {
                let length = chunk.len();
                let outcome = writer.write_all(&chunk).and_then(|()| writer.flush());
                reserved.fetch_sub(length, Ordering::SeqCst);
                if outcome.is_err() {
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}

/// Join a block thread, detaching it (without joining) if it does not finish.
pub(super) fn join_bounded(handle: Option<JoinHandle<()>>, timeout: Duration) -> bool {
    let Some(handle) = handle else {
        return true;
    };
    let deadline = Instant::now() + timeout;
    while !handle.is_finished() {
        if Instant::now() >= deadline {
            // Dropping the handle detaches the thread rather than blocking.
            return false;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let _ = handle.join();
    true
}
