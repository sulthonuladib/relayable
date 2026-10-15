//! Per-block PTY runtime and per-session runtime.
//!
//! A [`BlockRuntime`] owns the server-side PTY child for the lifetime of the
//! session, independently of any client connection: it holds the [`PtyHandle`],
//! a dedicated PTY-writer thread, the bounded output tail, the per-stream output
//! sequence, and the fan-out to attached streams. Dropping a client never
//! signals, reaps, or stops the child; only [`BlockRuntime::close`] does.

use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use relayable_model::{
    BlockId, ClientId, ProgramState, PtyProcessId, SessionId, TerminalSize, WindowId,
};
use relayable_protocol::v1::{BlockKind, BlockRecord, ProgramStatus, TerminalSnapshot};
use relayable_pty::{PtyExitStatus, PtyHandle, PtySize};

use crate::connection::{terminal_output_envelope, ConnectionShared, WriterMsg};
use crate::lock;

/// Maximum number of queued PTY-input messages per block.
const INPUT_QUEUE_MESSAGES: usize = 1024;
/// Maximum bytes of PTY input queued per block; memory is bounded by this.
const INPUT_QUEUE_BYTES: usize = 4 * 1024 * 1024;
/// How long `close` waits for each block thread before detaching it.
const THREAD_JOIN_TIMEOUT: Duration = Duration::from_secs(1);

/// Bounded per-block output tail plus the per-stream output sequence.
#[derive(Debug)]
pub(crate) struct OutputTail {
    bytes: VecDeque<u8>,
    capacity: usize,
    stream_seq: u64,
}

impl OutputTail {
    fn new(capacity: usize) -> Self {
        Self {
            bytes: VecDeque::new(),
            capacity: capacity.max(1),
            stream_seq: 0,
        }
    }

    /// Append a chunk, dropping the oldest bytes beyond the capacity, and
    /// advance the per-stream output sequence.
    fn push(&mut self, chunk: &[u8]) {
        self.stream_seq = self.stream_seq.wrapping_add(1);
        if chunk.len() >= self.capacity {
            self.bytes.clear();
            self.bytes
                .extend(chunk[chunk.len() - self.capacity..].iter().copied());
            return;
        }
        self.bytes.extend(chunk.iter().copied());
        while self.bytes.len() > self.capacity {
            self.bytes.pop_front();
        }
    }

    fn snapshot(&self) -> Vec<u8> {
        self.bytes.iter().copied().collect()
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.bytes.len()
    }
}

/// Normalized program status tracked by the block runtime.
#[derive(Debug, Clone)]
struct StoredStatus {
    state: ProgramState,
    exit_code: Option<u32>,
    signal: Option<String>,
}

impl StoredStatus {
    fn running() -> Self {
        Self {
            state: ProgramState::Running,
            exit_code: None,
            signal: None,
        }
    }

    fn exited(status: &PtyExitStatus) -> Self {
        Self {
            state: ProgramState::Exited,
            exit_code: Some(status.exit_code),
            signal: status.signal.clone(),
        }
    }

    fn to_wire(&self) -> ProgramStatus {
        ProgramStatus {
            state: match self.state {
                ProgramState::Running => relayable_protocol::v1::ProgramState::Running as i32,
                ProgramState::Exited => relayable_protocol::v1::ProgramState::Exited as i32,
            },
            exit_code: self.exit_code,
            signal: self.signal.clone(),
            pwd: None,
            title: None,
        }
    }
}

/// The single writer lease for a block, held by one `(client, stream)` pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WriterLease {
    client_id: ClientId,
    stream_id: u64,
}

/// A stream attached to a block, used for output fan-out and credit accounting.
pub(crate) struct Subscriber {
    pub(crate) stream_id: u64,
    pub(crate) client_id: ClientId,
    connection: Arc<ConnectionShared>,
    /// Remaining outstanding-byte window; starts at `credit_window_bytes`.
    credit: AtomicI64,
    overflowed: AtomicBool,
    window: i64,
}

impl Subscriber {
    pub(crate) fn new(
        stream_id: u64,
        client_id: ClientId,
        connection: Arc<ConnectionShared>,
        credit_window: u32,
    ) -> Self {
        let window = i64::from(credit_window);
        Self {
            stream_id,
            client_id,
            connection,
            credit: AtomicI64::new(window),
            overflowed: AtomicBool::new(false),
            window,
        }
    }

    /// Add client-returned credit, capped at the configured window.
    fn add_credit(&self, ack_bytes: u64) {
        let mut current = self.credit.load(Ordering::SeqCst);
        loop {
            let next = current.saturating_add(ack_bytes as i64).min(self.window);
            match self
                .credit
                .compare_exchange(current, next, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return,
                Err(observed) => current = observed,
            }
        }
    }

    /// Atomically reserve `bytes` of the window; `false` when exhausted.
    fn try_consume(&self, bytes: i64) -> bool {
        let mut current = self.credit.load(Ordering::SeqCst);
        loop {
            if current < bytes {
                return false;
            }
            match self.credit.compare_exchange(
                current,
                current - bytes,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return true,
                Err(observed) => current = observed,
            }
        }
    }

    /// Fan out one output chunk to this stream.
    ///
    /// A slow client that exhausts its window or whose bounded outbound queue is
    /// full has its connection closed (a transport drop); the session and child
    /// stay alive.
    fn deliver(&self, stream_seq: u64, chunk: &[u8]) {
        if self.overflowed.load(Ordering::SeqCst) {
            return;
        }
        let bytes = chunk.len() as i64;
        if !self.try_consume(bytes) {
            eprintln!(
                "relayable-server: subscriber client {} stream {} credit exhausted (want {bytes}, have {})",
                self.client_id,
                self.stream_id,
                self.credit.load(Ordering::SeqCst)
            );
            self.overflow();
            return;
        }
        let envelope = terminal_output_envelope(self.stream_id, stream_seq, chunk);
        match self
            .connection
            .tx
            .try_send(WriterMsg::Frame(Box::new(envelope)))
        {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                self.credit.fetch_add(bytes, Ordering::SeqCst);
                eprintln!(
                    "relayable-server: subscriber client {} stream {} outbound channel full",
                    self.client_id, self.stream_id
                );
                self.overflow();
            }
            Err(TrySendError::Disconnected(_)) => {
                self.credit.fetch_add(bytes, Ordering::SeqCst);
                self.overflowed.store(true, Ordering::SeqCst);
            }
        }
    }

    fn overflow(&self) {
        if !self.overflowed.swap(true, Ordering::SeqCst) {
            eprintln!(
                "relayable-server: stream {} overflowed its bounded output queue; closing connection",
                self.stream_id
            );
            self.connection.close_now();
        }
    }
}

/// The tail buffer and subscriber list, behind one lock.
///
/// The PTY reader appends to the tail, advances `stream_seq`, and fans out while
/// holding this lock, and `attach_stream` captures the tail and registers the
/// new subscriber while holding the same lock. That makes attach atomic: for a
/// given stream, `screen_bytes` plus the delivered frames are a contiguous,
/// non-duplicated suffix of the block output.
struct OutputState {
    tail: OutputTail,
    subscribers: Vec<Arc<Subscriber>>,
}

/// The outcome of [`BlockRuntime::close`].
pub(crate) struct CloseOutcome {
    /// The observed exit status, when the PTY handle could report one.
    pub(crate) status: Option<PtyExitStatus>,
    /// Whether the direct child was confirmed reaped within the bound.
    pub(crate) reaped: bool,
}

/// The result of enqueueing terminal input.
pub(crate) enum InputError {
    /// The bounded queue (or byte budget) is full.
    Full,
    /// The block is closing or dead.
    Closed,
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
        initial_size: TerminalSize,
        mut pty: PtyHandle,
        output_tail_bytes: usize,
    ) -> Result<Self> {
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
    pub(crate) fn acquire_writer(&self, client_id: ClientId, stream_id: u64) -> bool {
        let mut lease = lock(&self.writer_lease);
        if lease.is_some() {
            return false;
        }
        *lease = Some(WriterLease {
            client_id,
            stream_id,
        });
        true
    }

    pub(crate) fn release_writer(&self, client_id: ClientId, stream_id: u64) {
        let mut lease = lock(&self.writer_lease);
        if *lease
            == Some(WriterLease {
                client_id,
                stream_id,
            })
        {
            *lease = None;
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
        if reaped {
            if let Some(status) = &observed {
                *lock(&self.program) = StoredStatus::exited(status);
            }
            // Detach every stream and free the lease, then mark the block dead
            // so later input/detach on stale stream entries is rejected.
            lock(&self.output).subscribers.clear();
            *lock(&self.writer_lease) = None;
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
        }
    }

    /// Test-only hook: make [`BlockRuntime::close`] report an unconfirmed exit
    /// without terminating the child, so the registry's "cannot confirm" branch
    /// can be exercised.
    #[cfg(test)]
    pub(crate) fn set_force_unreaped(&self, value: bool) {
        self.force_unreaped.store(value, Ordering::SeqCst);
    }
}

/// Drain queued input into the PTY from a dedicated thread.
///
/// The lock is never held while writing: the thread alone owns the writer, and
/// the connection thread only enqueues. Byte reservations are released after the
/// write attempt so a blocked write applies backpressure via the byte budget.
fn input_loop(
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
fn join_bounded(handle: Option<JoinHandle<()>>, timeout: Duration) -> bool {
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

/// Whether `pid` is its own process-group leader (portable-pty's `setsid()`).
fn owns_process_group(pid: u32) -> bool {
    // SAFETY: `getpgid` is a read-only query on a pid this server created.
    let pgid = unsafe { libc::getpgid(pid as libc::pid_t) };
    pgid >= 0 && pgid == pid as libc::pid_t
}

fn session_of(pid: u32) -> Option<libc::pid_t> {
    // SAFETY: `getsid` is a read-only query on a pid this server created.
    let sid = unsafe { libc::getsid(pid as libc::pid_t) };
    if sid >= 0 {
        Some(sid)
    } else {
        None
    }
}

/// Signal one process group (`kill(-pgid, ...)`).
fn signal_group(group: libc::pid_t, signal: libc::c_int) {
    // SAFETY: best-effort signal to a process group created by this server.
    unsafe {
        libc::kill(-group, signal);
    }
}

/// Signal every process whose session id is `session`.
///
/// Job control puts background jobs in their own process groups but keeps them
/// in the child's session, so a group kill alone can miss a descendant holding
/// the PTY slave.
fn signal_session(session: libc::pid_t, signal: libc::c_int) {
    for pid in session_members(session) {
        // SAFETY: best-effort signal to a process in this server's session tree.
        unsafe {
            libc::kill(pid, signal);
        }
    }
}

/// Enumerate the pids whose session id is `session`, by reading `/proc`.
fn session_members(session: libc::pid_t) -> Vec<libc::pid_t> {
    let mut members = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return members;
    };
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<libc::pid_t>().ok())
        else {
            continue;
        };
        if let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) {
            if stat_session(&stat) == Some(session) {
                members.push(pid);
            }
        }
    }
    members
}

/// Parse the session id from a `/proc/<pid>/stat` line.
///
/// The `comm` field can contain spaces and parentheses, so parsing resumes
/// after the final `)`: `state ppid pgrp session ...`.
fn stat_session(stat: &str) -> Option<libc::pid_t> {
    let rest = stat.rsplit_once(") ")?.1;
    let mut fields = rest.split_whitespace();
    let _state = fields.next()?;
    let _ppid = fields.next()?;
    let _pgrp = fields.next()?;
    fields.next()?.parse().ok()
}

/// The server-owned runtime for one session: a default window and one terminal
/// block. Closed sessions are retained as tombstones (ADR-001).
pub(crate) struct SessionRuntime {
    pub(crate) id: SessionId,
    pub(crate) label: Option<String>,
    pub(crate) created_at_millis: u64,
    pub(crate) window_id: WindowId,
    pub(crate) block: Arc<BlockRuntime>,
    revision: AtomicU64,
    window_revision: AtomicU64,
    closed: AtomicBool,
}

impl SessionRuntime {
    pub(crate) fn new(
        id: SessionId,
        label: Option<String>,
        created_at_millis: u64,
        window_id: WindowId,
        block: Arc<BlockRuntime>,
    ) -> Self {
        Self {
            id,
            label,
            created_at_millis,
            window_id,
            block,
            revision: AtomicU64::new(1),
            window_revision: AtomicU64::new(1),
            closed: AtomicBool::new(false),
        }
    }

    pub(crate) fn revision(&self) -> u64 {
        self.revision.load(Ordering::SeqCst)
    }

    pub(crate) fn window_revision(&self) -> u64 {
        self.window_revision.load(Ordering::SeqCst)
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    pub(crate) fn mark_closed(&self) {
        self.closed.store(true, Ordering::SeqCst);
        self.revision.fetch_add(1, Ordering::SeqCst);
        self.window_revision.fetch_add(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn output_tail_drops_oldest_bytes() {
        let mut tail = OutputTail::new(8);
        tail.push(b"abc");
        assert_eq!(tail.len(), 3);
        tail.push(b"defghij");
        // Capacity is 8: the newest 8 bytes win and the oldest are dropped.
        assert_eq!(tail.len(), 8);
        assert_eq!(tail.snapshot(), b"cdefghij");
        // A chunk larger than the capacity keeps only its own tail.
        tail.push(b"0123456789");
        assert_eq!(tail.len(), 8);
        assert_eq!(tail.snapshot(), b"23456789");
    }

    #[test]
    fn output_tail_advances_sequence_per_chunk() {
        let mut tail = OutputTail::new(16);
        assert_eq!(tail.stream_seq, 0);
        tail.push(b"a");
        tail.push(b"b");
        assert_eq!(tail.stream_seq, 2);
    }

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

    #[test]
    fn stat_session_parses_after_comm_with_spaces() {
        let stat = "42 (weird ) name) S 1 42 42 0 -1 4194560";
        assert_eq!(stat_session(stat), Some(42));
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
                TerminalSize::DEFAULT_SIZE,
                pty,
                32_768,
            )
            .expect("block"),
        );
        block.start_threads().expect("threads");
        block
    }
}
