//! The server-owned runtime for one terminal block.
//!
//! [`BlockRuntime`] owns the PTY handle, the dedicated PTY-writer thread, the
//! bounded output tail, the per-stream output sequence, and the lease state for
//! one terminal block, independently of any client connection.

use std::io::{ErrorKind, Read};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use relayable_model::{BlockId, ClientId, PtyProcessId, SessionId, TerminalSize, WindowId};
use relayable_protocol::v1::{BlockKind, BlockRecord, TerminalSnapshot};
use relayable_pty::{PtyExitStatus, PtyHandle, PtySize};

use crate::events::{terminal_child_exited_event, EventBus};
use crate::lock;

use super::output::{OutputState, OutputTail};
use super::process::{owns_process_group, session_of, signal_group, signal_session};
use super::status::{StoredStatus, WriterLease};
use super::subscriber::Subscriber;
use super::threads::{
    input_loop, join_bounded, INPUT_QUEUE_BYTES, INPUT_QUEUE_MESSAGES, THREAD_JOIN_TIMEOUT,
};

/// The outcome of [`BlockRuntime::close`].
pub(crate) struct CloseOutcome {
    /// The observed exit status, when the PTY handle could report one.
    pub(crate) status: Option<PtyExitStatus>,
    /// Whether the direct child was confirmed reaped within the bound.
    pub(crate) reaped: bool,
    /// The writer lease this close cleared, as `(client_id, stream_id)`.
    pub(crate) released_lease: Option<(ClientId, u64)>,
}

/// The result of enqueueing terminal input.
pub(crate) enum InputError {
    /// The bounded queue (or byte budget) is full.
    Full,
    /// The block is closing or dead.
    Closed,
}

/// The result of trying to take the single writer lease.
pub(crate) enum WriterAcquire {
    /// The lease was free and is now held by the caller's stream.
    Acquired,
    /// The lease is held by another stream and `takeover` was not requested.
    Busy,
    /// `takeover` was requested and the lease moved from the previous holder.
    Taken { previous_client_id: ClientId },
}

/// Construction inputs for a [`BlockRuntime`] that are not entity identities.
pub(crate) struct BlockInit {
    /// Initial terminal geometry.
    pub(crate) initial_size: TerminalSize,
    /// Maximum retained output tail, in bytes.
    pub(crate) output_tail_bytes: usize,
    /// The server's retained event log, or `None` in isolated unit tests.
    pub(crate) events: Option<Arc<EventBus>>,
}

/// The server-owned runtime for one terminal block.
pub(crate) struct BlockRuntime {
    pub(crate) id: BlockId,
    pub(crate) window_id: WindowId,
    pub(crate) session_id: SessionId,
    pub(crate) pty_process_id: PtyProcessId,
    pub(crate) program_pid: Option<u32>,
    revision: AtomicU64,
    size: Mutex<TerminalSize>,
    program: Mutex<StoredStatus>,
    output: Mutex<OutputState>,
    input_tx: Mutex<Option<SyncSender<Vec<u8>>>>,
    input_bytes: Arc<AtomicUsize>,
    writer_lease: Mutex<Option<WriterLease>>,
    pty: Mutex<PtyHandle>,
    stop: Arc<AtomicBool>,
    dead: AtomicBool,
    close_lock: Mutex<()>,
    reader_thread: Mutex<Option<JoinHandle<()>>>,
    reaper_thread: Mutex<Option<JoinHandle<()>>>,
    input_thread: Mutex<Option<JoinHandle<()>>>,
    /// The retained event log, when this block is owned by a server. `None` in
    /// isolated unit tests, where no event is emitted.
    events: Option<Arc<EventBus>>,
    /// Ensures `terminal_child_exited` is emitted at most once.
    exit_emitted: AtomicBool,
    #[cfg(test)]
    force_unreaped: AtomicBool,
}

impl BlockRuntime {
    /// Wrap an already-spawned PTY and start its dedicated input-writer thread.
    pub(crate) fn new(
        id: BlockId,
        window_id: WindowId,
        session_id: SessionId,
        pty_process_id: PtyProcessId,
        mut pty: PtyHandle,
        init: BlockInit,
    ) -> Result<Self> {
        let BlockInit {
            initial_size,
            output_tail_bytes,
            events,
        } = init;
        let writer = pty.take_writer().context("failed to take the pty writer")?;
        let program_pid = pty.process_id();

        let (input_tx, input_rx) = mpsc::sync_channel::<Vec<u8>>(INPUT_QUEUE_MESSAGES);
        let input_bytes = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let input_bytes_for_thread = Arc::clone(&input_bytes);
        let stop_for_thread = Arc::clone(&stop);
        let input_thread = thread::spawn(move || {
            input_loop(input_rx, writer, input_bytes_for_thread, stop_for_thread)
        });

        Ok(Self {
            id,
            window_id,
            session_id,
            pty_process_id,
            program_pid,
            revision: AtomicU64::new(1),
            size: Mutex::new(initial_size),
            program: Mutex::new(StoredStatus::running()),
            output: Mutex::new(OutputState {
                tail: OutputTail::new(output_tail_bytes),
                subscribers: Vec::new(),
            }),
            input_tx: Mutex::new(Some(input_tx)),
            input_bytes,
            writer_lease: Mutex::new(None),
            pty: Mutex::new(pty),
            stop,
            dead: AtomicBool::new(false),
            close_lock: Mutex::new(()),
            reader_thread: Mutex::new(None),
            reaper_thread: Mutex::new(None),
            input_thread: Mutex::new(Some(input_thread)),
            events,
            exit_emitted: AtomicBool::new(false),
            #[cfg(test)]
            force_unreaped: AtomicBool::new(false),
        })
    }

    /// Start the output-reader and child-reaper threads.
    pub(crate) fn start_threads(self: &Arc<Self>) -> Result<()> {
        let reader = {
            let pty = lock(&self.pty);
            pty.reader().context("failed to clone the pty reader")?
        };
        let this = Arc::clone(self);
        let reader_handle = thread::spawn(move || this.reader_loop(reader));
        *lock(&self.reader_thread) = Some(reader_handle);

        let this = Arc::clone(self);
        let reaper_handle = thread::spawn(move || this.reaper_loop());
        *lock(&self.reaper_thread) = Some(reaper_handle);
        Ok(())
    }

    pub(crate) fn is_dead(&self) -> bool {
        self.dead.load(Ordering::SeqCst)
    }

    fn reader_loop(self: Arc<Self>, mut reader: Box<dyn Read + Send>) {
        let mut buffer = vec![0u8; 8192];
        while !self.stop.load(Ordering::SeqCst) {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => self.push_output(&buffer[..read]),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                // On Linux a PTY master read after the child exits reports EIO.
                Err(error) if error.raw_os_error() == Some(libc::EIO) => break,
                Err(_) => break,
            }
        }
    }

    fn reaper_loop(self: Arc<Self>) {
        loop {
            if self.stop.load(Ordering::SeqCst) {
                break;
            }
            let status = {
                let mut pty = lock(&self.pty);
                pty.try_wait()
            };
            match status {
                Ok(Some(status)) => {
                    *lock(&self.program) = StoredStatus::exited(&status);
                    // Emit after the pty lock is released; the atomic guard
                    // makes a close-observed status emit at most once too.
                    self.emit_child_exited(&status);
                    break;
                }
                Ok(None) => thread::sleep(Duration::from_millis(50)),
                Err(_) => break,
            }
        }
    }

    /// Append output to the bounded tail and fan it out to attached streams.
    ///
    /// The tail append, `stream_seq` increment, and fan-out happen under the
    /// single output lock so that [`BlockRuntime::attach_stream`] cannot observe
    /// a subscriber registered at a different sequence than its tail snapshot.
    fn push_output(&self, chunk: &[u8]) {
        let mut state = lock(&self.output);
        state.tail.push(chunk);
        let stream_seq = state.tail.stream_seq;
        for subscriber in &state.subscribers {
            subscriber.deliver(stream_seq, chunk);
        }
    }

    /// Atomically capture the current tail and register a new subscriber.
    ///
    /// Invariant: for the returned stream, `screen_bytes` and every subsequently
    /// delivered `TerminalOutput` frame are a contiguous, non-duplicated suffix
    /// of the block's total output (no gap at the attach boundary).
    pub(crate) fn attach_stream(
        &self,
        stream_id: u64,
        subscriber: Arc<Subscriber>,
    ) -> TerminalSnapshot {
        let size = self.current_size();
        let mut state = lock(&self.output);
        let snapshot_seq = state.tail.stream_seq;
        let screen_bytes = state.tail.snapshot();
        state.subscribers.push(subscriber);
        TerminalSnapshot {
            stream_id,
            snapshot_seq,
            cols: u32::from(size.cols),
            rows: u32::from(size.rows),
            screen_bytes,
            scrollback_bytes: Vec::new(),
        }
    }

    pub(crate) fn remove_subscriber(&self, client_id: ClientId, stream_id: u64) {
        lock(&self.output)
            .subscribers
            .retain(|s| !(s.client_id == client_id && s.stream_id == stream_id));
    }

    pub(crate) fn add_credit(&self, client_id: ClientId, stream_id: u64, ack_bytes: u64) {
        let state = lock(&self.output);
        for subscriber in &state.subscribers {
            if subscriber.client_id == client_id && subscriber.stream_id == stream_id {
                subscriber.add_credit(ack_bytes);
            }
        }
    }

    /// Enqueue terminal input; never performs a blocking PTY write on the caller.
    ///
    /// `Ok(n)` means `n` bytes were accepted into the bounded queue (not that
    /// they were written). `Err(Full)` means the queue/byte budget is exhausted.
    pub(crate) fn enqueue_input(&self, bytes: Vec<u8>) -> Result<usize, InputError> {
        let length = bytes.len();
        if length == 0 {
            return Ok(0);
        }
        if self.dead.load(Ordering::SeqCst) {
            return Err(InputError::Closed);
        }
        if !self.reserve_input(length) {
            return Err(InputError::Full);
        }
        let sender = lock(&self.input_tx).clone();
        let Some(sender) = sender else {
            self.release_input(length);
            return Err(InputError::Closed);
        };
        match sender.try_send(bytes) {
            Ok(()) => Ok(length),
            Err(TrySendError::Full(_)) => {
                self.release_input(length);
                Err(InputError::Full)
            }
            Err(TrySendError::Disconnected(_)) => {
                self.release_input(length);
                Err(InputError::Closed)
            }
        }
    }

    fn reserve_input(&self, length: usize) -> bool {
        let mut current = self.input_bytes.load(Ordering::SeqCst);
        loop {
            if current + length > INPUT_QUEUE_BYTES {
                return false;
            }
            match self.input_bytes.compare_exchange(
                current,
                current + length,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return true,
                Err(observed) => current = observed,
            }
        }
    }

    fn release_input(&self, length: usize) {
        self.input_bytes.fetch_sub(length, Ordering::SeqCst);
    }

    /// Try to take the single writer lease for `(client_id, stream_id)`.
    ///
    /// `takeover = false` preserves CORE-001 behavior (a held lease is never
    /// stolen). `takeover = true` moves the lease and reports the previous
    /// holder so the caller can emit `writer_lease_changed`.
    pub(crate) fn acquire_writer(
        &self,
        client_id: ClientId,
        stream_id: u64,
        takeover: bool,
    ) -> WriterAcquire {
        let mut lease = lock(&self.writer_lease);
        match *lease {
            None => {
                *lease = Some(WriterLease {
                    client_id,
                    stream_id,
                });
                WriterAcquire::Acquired
            }
            Some(current) if takeover => {
                *lease = Some(WriterLease {
                    client_id,
                    stream_id,
                });
                WriterAcquire::Taken {
                    previous_client_id: current.client_id,
                }
            }
            Some(_) => WriterAcquire::Busy,
        }
    }

    /// Release the lease only if this `(client_id, stream_id)` still holds it.
    /// Returns whether a release actually happened.
    pub(crate) fn release_writer(&self, client_id: ClientId, stream_id: u64) -> bool {
        let mut lease = lock(&self.writer_lease);
        if *lease
            == Some(WriterLease {
                client_id,
                stream_id,
            })
        {
            *lease = None;
            true
        } else {
            false
        }
    }

    pub(crate) fn holds_writer(&self, client_id: ClientId, stream_id: u64) -> bool {
        *lock(&self.writer_lease)
            == Some(WriterLease {
                client_id,
                stream_id,
            })
    }

    fn writer_attached(&self) -> bool {
        lock(&self.writer_lease).is_some()
    }

    pub(crate) fn current_size(&self) -> TerminalSize {
        *lock(&self.size)
    }

    /// The block's current optimistic-concurrency revision.
    pub(crate) fn revision(&self) -> u64 {
        self.revision.load(Ordering::SeqCst)
    }

    /// Apply a resize to the PTY, the block record, and the block revision.
    pub(crate) fn apply_resize(&self, size: TerminalSize) -> Result<()> {
        {
            let pty = lock(&self.pty);
            pty.resize(PtySize {
                cols: size.cols,
                rows: size.rows,
                pixel_width: size.pixel_width,
                pixel_height: size.pixel_height,
            })
            .context("failed to resize the pty")?;
        }
        *lock(&self.size) = size;
        self.revision.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// Project this block into a contract §2 `BlockRecord`.
    pub(crate) fn record(&self) -> BlockRecord {
        let size = self.current_size();
        let program = lock(&self.program).clone();
        BlockRecord {
            block_id: self.id.to_string(),
            window_id: self.window_id.to_string(),
            session_id: self.session_id.to_string(),
            kind: BlockKind::Terminal as i32,
            label: None,
            revision: self.revision.load(Ordering::SeqCst),
            cols: u32::from(size.cols),
            rows: u32::from(size.rows),
            program_status: Some(program.to_wire()),
            pty_process_id: self.pty_process_id.to_string(),
            program_pid: self.program_pid.unwrap_or(0),
            writer_attached: self.writer_attached(),
        }
    }

    /// Terminate the session tree and join the block threads (bounded).
    ///
    /// The direct child is killed by [`PtyHandle::shutdown`]/[`PtyHandle::kill`]
    /// (SIGHUP, poll, SIGKILL). Because an interactive shell enables job control,
    /// a background descendant such as `sleep 30 &` gets its own process group
    /// but shares the child's **session**; therefore we also signal the child's
    /// process group (when it is its own group leader, as `portable-pty`'s
    /// `setsid()` guarantees) and every process in the child's session. Without
    /// that, a descendant holding the PTY slave keeps the reader blocked.
    ///
    /// Returns the observed status and whether the direct child was confirmed
    /// reaped. When it was not confirmed, the block is left intact so the caller
    /// can keep the session ACTIVE.
    pub(crate) fn close(&self, grace: Duration) -> CloseOutcome {
        let _serialize = lock(&self.close_lock);

        #[cfg(test)]
        if self.force_unreaped.load(Ordering::SeqCst) {
            return CloseOutcome {
                status: None,
                reaped: false,
                released_lease: None,
            };
        }

        // Only a group leader (portable-pty's `setsid()` child) can be
        // group/session signalled. Otherwise fall back to the direct child only.
        let leader_pid = self.program_pid.filter(|pid| owns_process_group(*pid));
        let group_leader = leader_pid.and_then(|pid| i32::try_from(pid).ok());
        let session = leader_pid.and_then(session_of);
        if let Some(group) = group_leader {
            signal_group(group, libc::SIGHUP);
            if let Some(session) = session {
                signal_session(session, libc::SIGHUP);
            }
        }

        // `shutdown` waits `grace`, then escalates on the direct child only.
        let first = {
            let mut pty = lock(&self.pty);
            pty.shutdown(grace)
        };
        let mut observed = match first {
            Ok(Some(status)) => Some(status),
            _ => None,
        };

        // Hard-kill the group and every session member so descendants that
        // inherited the slave go away too.
        if let Some(group) = group_leader {
            signal_group(group, libc::SIGKILL);
            if let Some(session) = session {
                signal_session(session, libc::SIGKILL);
            }
        }

        if observed.is_none() {
            let deadline = Instant::now() + Duration::from_secs(2);
            while Instant::now() < deadline {
                let current = {
                    let mut pty = lock(&self.pty);
                    pty.try_wait()
                };
                match current {
                    Ok(Some(status)) => {
                        observed = Some(status);
                        break;
                    }
                    Ok(None) => thread::sleep(Duration::from_millis(10)),
                    Err(_) => break,
                }
            }
        }

        let reaped = observed.is_some();
        let mut released_lease = None;
        if reaped {
            if let Some(status) = &observed {
                *lock(&self.program) = StoredStatus::exited(status);
            }
            // Detach every stream and free the lease, then mark the block dead
            // so later input/detach on stale stream entries is rejected. The
            // lease is taken (not just cleared) so a concurrent release cannot
            // double-report the same change.
            lock(&self.output).subscribers.clear();
            released_lease = lock(&self.writer_lease)
                .take()
                .map(|lease| (lease.client_id, lease.stream_id));
            self.dead.store(true, Ordering::SeqCst);
            self.stop.store(true, Ordering::SeqCst);
            // Dropping the sender lets the input thread observe Disconnected.
            let _ = lock(&self.input_tx).take();
            let _ = join_bounded(lock(&self.reaper_thread).take(), THREAD_JOIN_TIMEOUT);
            let _ = join_bounded(lock(&self.reader_thread).take(), THREAD_JOIN_TIMEOUT);
            let _ = join_bounded(lock(&self.input_thread).take(), THREAD_JOIN_TIMEOUT);
        }

        CloseOutcome {
            status: observed,
            reaped,
            released_lease,
        }
    }

    /// Emit `terminal_child_exited` once, when the reaper or a confirmed close
    /// observed the exit status. Does nothing in isolated tests (no bus).
    pub(crate) fn emit_child_exited(&self, status: &PtyExitStatus) {
        if self.exit_emitted.swap(true, Ordering::SeqCst) {
            return;
        }
        let Some(events) = &self.events else {
            return;
        };
        events.append(terminal_child_exited_event(
            &self.session_id.to_string(),
            &self.id.to_string(),
            self.revision.load(Ordering::SeqCst),
            status,
        ));
    }

    /// Test-only hook: make [`BlockRuntime::close`] report an unconfirmed exit
    /// without terminating the child, so the registry's "cannot confirm" branch
    /// can be exercised.
    #[cfg(test)]
    pub(crate) fn set_force_unreaped(&self, value: bool) {
        self.force_unreaped.store(value, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn resize_updates_size_and_revision() {
        let block = test_block();
        assert_eq!(block.current_size(), TerminalSize::DEFAULT_SIZE);
        assert_eq!(block.record().revision, 1);

        block
            .apply_resize(TerminalSize {
                cols: 100,
                rows: 40,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("resize");

        let record = block.record();
        assert_eq!(record.cols, 100);
        assert_eq!(record.rows, 40);
        assert_eq!(record.revision, 2);
        block.close(Duration::from_millis(100));
    }

    fn test_block() -> Arc<BlockRuntime> {
        let pty = PtyHandle::spawn(
            std::ffi::OsStr::new("/bin/sh"),
            &[OsString::from("-c"), OsString::from("sleep 30")],
            PtySize::new(80, 24),
            None,
            &[],
        )
        .expect("spawn test pty");
        let block = Arc::new(
            BlockRuntime::new(
                BlockId::new(),
                WindowId::new(),
                SessionId::new(),
                PtyProcessId::new(),
                pty,
                BlockInit {
                    initial_size: TerminalSize::DEFAULT_SIZE,
                    output_tail_bytes: 32_768,
                    events: None,
                },
            )
            .expect("block"),
        );
        block.start_threads().expect("threads");
        block
    }
}
