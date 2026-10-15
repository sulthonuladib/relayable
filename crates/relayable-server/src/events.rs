//! Server-wide retained control-event log and live fan-out.
//!
//! This is the frozen CORE-002 `session.resume` source of truth
//! (`protocol/compatibility/core-002-method-set.md` §B/§C).
//!
//! # Locking discipline (binding)
//!
//! One [`EventBus`] owns a mutex over the retained event log, the head
//! sequence, and the live-subscriber registration list.
//!
//! - **Events are appended only while the bus lock is held.** [`EventBus::append`]
//!   assigns the next sequence and, in the same critical section, fans the new
//!   event out to every registered connection with a non-blocking
//!   [`ConnectionShared::enqueue_envelope`] (`SyncSender::try_send`). The log
//!   head is mirrored in an [`AtomicU64`] updated inside that same critical
//!   section, so lock-free readers (the heartbeat writer, `session.view`,
//!   `session.list`, snapshots) never take the bus lock. The retained floor is
//!   computed from the log under the lock (`EventState::floor`).
//! - **The bus lock is a leaf for every path except `session.resume`.** Every
//!   other path mutates state under the existing entity locks
//!   (`sessions`/`clients`/`connections`/`output`/`writer_lease`/PTY), releases
//!   them, and only then calls [`EventBus::append`]. No entity lock is held
//!   while the bus lock is held, and `close_now`, session teardown, thread
//!   joins, and entity locks are never taken while holding the bus lock.
//! - **Single exception: `session.resume`.** Its handler acquires the bus lock
//!   first, then, while holding it, reads `S = head`, validates the cursor,
//!   builds the replay-or-snapshot payload, enqueues the response frame into
//!   that connection's writer queue with a non-blocking send, and registers the
//!   connection as a live subscriber from `S + 1`. That is what makes the frozen
//!   "no gap, no duplicate across the snapshot/replay boundary" property true.
//!   Building the snapshot reads the sessions map (and the block records) under
//!   the same lock; no path takes those locks and then waits for the bus, so the
//!   order cannot deadlock. If the response queue is full the connection is
//!   dropped via `close_now` **after** the bus lock is released.
//! - **Events carry absolute post-change values.** Re-applying an event to a
//!   snapshot is idempotent; a client may replay retained events over any
//!   projection at or after their sequence without double-applying a change.
//! - **Terminal data is not an event.** PTY output is never appended here and
//!   `push_output` never takes the bus lock (contract §11: no terminal bytes in
//!   events).
//!
//! Event sequences are strictly increasing from `1`. `retained_floor_sequence`
//! is the oldest retained sequence, or `current_sequence + 1` when the log is
//! empty; retention is bounded by COUNT and evicts oldest-first.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use relayable_model::ClientId;
use relayable_protocol::v1::envelope::Body;
use relayable_protocol::v1::{
    event, ClientConnectionStateChanged, ClientDisconnected, Envelope, Event, SessionState,
    SessionStateChanged, TerminalChildExited, WriterLeaseChanged,
};
use relayable_protocol::PROTOCOL_VERSION;
use relayable_pty::PtyExitStatus;

use crate::connection::ConnectionShared;
use crate::lock;

/// Why a connection is being torn down. The reason is informational: it is
/// carried on `client_disconnected` and never changes session lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DisconnectReason {
    /// The reader hit the configured idle timeout.
    IdleTimeout,
    /// The peer closed the transport cleanly.
    PeerClosed,
    /// A transport or codec error, or a control-queue overflow.
    TransportError,
    /// The bounded terminal-output queue/credit window overflowed.
    FlowControlOverflow,
    /// The server is shutting down.
    ServerShutdown,
}

impl DisconnectReason {
    /// The frozen §D `client_disconnected.reason` spelling.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::IdleTimeout => "idle_timeout",
            Self::PeerClosed => "peer_closed",
            Self::TransportError => "transport_error",
            Self::FlowControlOverflow => "flow_control_overflow",
            Self::ServerShutdown => "server_shutdown",
        }
    }
}

/// The retained log plus the live subscriber list, guarded by the bus mutex.
pub(crate) struct EventState {
    pub(crate) events: VecDeque<Event>,
    pub(crate) next_sequence: u64,
    pub(crate) subscribers: Vec<Arc<ConnectionShared>>,
}

impl EventState {
    /// Head (most recently assigned) sequence; `0` when the log is empty.
    pub(crate) fn current(&self) -> u64 {
        self.next_sequence - 1
    }

    /// Oldest retained sequence, or `current + 1` when the log is empty.
    pub(crate) fn floor(&self) -> u64 {
        self.events
            .front()
            .map_or(self.next_sequence, |event| event.sequence)
    }

    /// Number of retained events.
    pub(crate) fn count(&self) -> usize {
        self.events.len()
    }

    /// Register a connection for live delivery, once.
    pub(crate) fn subscribe(&mut self, connection: &Arc<ConnectionShared>) {
        if !self
            .subscribers
            .iter()
            .any(|existing| Arc::ptr_eq(existing, connection))
        {
            self.subscribers.push(Arc::clone(connection));
        }
    }
}

/// The server-wide retained control-event log and live fan-out.
pub(crate) struct EventBus {
    state: Mutex<EventState>,
    head: AtomicU64,
    capacity: usize,
}

impl EventBus {
    /// Create a bus retaining at most `capacity` events (minimum 1).
    pub(crate) fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            state: Mutex::new(EventState {
                events: VecDeque::new(),
                next_sequence: 1,
                subscribers: Vec::new(),
            }),
            head: AtomicU64::new(0),
            capacity,
        }
    }

    /// Lock-free head sequence, mirrored inside the append critical section.
    pub(crate) fn current_sequence(&self) -> u64 {
        self.head.load(Ordering::SeqCst)
    }

    /// Lock the bus mutex. Used only by `session.resume`, which must hold the
    /// lock across cursor validation, payload construction, response enqueue,
    /// and subscriber registration.
    pub(crate) fn lock_state(&self) -> MutexGuard<'_, EventState> {
        lock(&self.state)
    }

    /// Append one event: assign the next sequence and fan out under the lock.
    ///
    /// Returns the assigned sequence. A subscriber whose bounded queue is full
    /// is closed **after** the lock is released.
    pub(crate) fn append(&self, event: Event) -> u64 {
        self.append_many(vec![event])[0]
    }

    /// Append two events as one atomic unit and return their two sequences.
    ///
    /// Both sequences are assigned, both events are inserted into the log, and
    /// both frames are fanned out inside a **single** bus-lock acquisition, so
    /// the pair is guaranteed to occupy consecutive sequences even while other
    /// threads append concurrently. This is what makes the frozen §D
    /// "immediately followed by" / "two sequences, same cause, ordered"
    /// invariant for the disconnect pair deterministic rather than
    /// probabilistic. Fan-out stays non-blocking; a subscriber whose queue is
    /// full is closed after the lock is released.
    pub(crate) fn append_pair(&self, first: Event, second: Event) -> (u64, u64) {
        let sequences = self.append_many(vec![first, second]);
        (sequences[0], sequences[1])
    }

    /// Assign sequences to `events`, insert them into the log, and fan them out
    /// while holding the bus lock exactly once.
    ///
    /// The head mirror is updated inside the same critical section. Per
    /// subscriber, frames are enqueued in `events` order.
    fn append_many(&self, mut events: Vec<Event>) -> Vec<u64> {
        let (sequences, failed) = {
            let mut state = lock(&self.state);
            let mut sequences = Vec::with_capacity(events.len());
            let mut envelopes = Vec::with_capacity(events.len());
            for mut event in events.drain(..) {
                let sequence = state.next_sequence;
                state.next_sequence += 1;
                event.sequence = sequence;
                sequences.push(sequence);
                envelopes.push(event_envelope(&event));
                if state.events.len() >= self.capacity {
                    state.events.pop_front();
                }
                state.events.push_back(event);
            }
            if let Some(last) = sequences.last() {
                self.head.store(*last, Ordering::SeqCst);
            }

            let mut failed = Vec::new();
            for envelope in &envelopes {
                for subscriber in &state.subscribers {
                    if subscriber.enqueue_envelope(envelope.clone()).is_err() {
                        failed.push(Arc::clone(subscriber));
                    }
                }
            }
            (sequences, failed)
        };
        for connection in failed {
            connection.close_now(DisconnectReason::TransportError);
        }
        sequences
    }

    /// Drop a connection from live delivery (idempotent).
    pub(crate) fn unsubscribe(&self, connection: &Arc<ConnectionShared>) {
        lock(&self.state)
            .subscribers
            .retain(|existing| !Arc::ptr_eq(existing, connection));
    }
}

/// The `session.resume` replayability decision, extracted so it is unit-tested
/// without a live server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResumeDecision {
    /// The retained events `last + 1 ..= current` are contiguous and complete.
    Replay,
    /// A fresh snapshot is required; the reason is one of `no_cursor`,
    /// `cursor_below_floor`, `cursor_ahead`.
    Snapshot(&'static str),
}

/// Frozen §B rule. `last == 0` is always "no cursor" (never a false replay).
pub(crate) fn decide_resume(last: u64, current: u64, floor: u64) -> ResumeDecision {
    if last == 0 {
        return ResumeDecision::Snapshot("no_cursor");
    }
    if last > current {
        return ResumeDecision::Snapshot("cursor_ahead");
    }
    if floor > last + 1 {
        return ResumeDecision::Snapshot("cursor_below_floor");
    }
    ResumeDecision::Replay
}

/// Build the control-class event envelope.
pub(crate) fn event_envelope(event: &Event) -> Envelope {
    Envelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        request_id: None,
        stream_id: 0,
        sequence: Some(event.sequence),
        trace_id: None,
        timestamp_unix_millis: None,
        body: Some(Body::Event(event.clone())),
    }
}

/// `client_connection_state_changed`. The state string is the contract §4
/// spelling (`relayable_model::ConnectionState::as_str`).
pub(crate) fn client_connection_state_event(client_id: &ClientId, state: &str) -> Event {
    Event {
        sequence: 0,
        revision: 0,
        session_id: String::new(),
        event: Some(event::Event::ClientConnectionStateChanged(
            ClientConnectionStateChanged {
                client_id: client_id.to_string(),
                connection_state: state.to_string(),
            },
        )),
    }
}

/// `client_disconnected`, emitted immediately after the disconnected state
/// change (two sequences, same cause, ordered).
pub(crate) fn client_disconnected_event(client_id: &ClientId, reason: DisconnectReason) -> Event {
    Event {
        sequence: 0,
        revision: 0,
        session_id: String::new(),
        event: Some(event::Event::ClientDisconnected(ClientDisconnected {
            client_id: client_id.to_string(),
            reason: reason.as_str().to_string(),
        })),
    }
}

/// `session_state_changed` (ACTIVE on create, CLOSED on confirmed close).
pub(crate) fn session_state_event(session_id: &str, revision: u64, state: SessionState) -> Event {
    Event {
        sequence: 0,
        revision,
        session_id: session_id.to_string(),
        event: Some(event::Event::SessionStateChanged(SessionStateChanged {
            session_id: session_id.to_string(),
            state: state as i32,
        })),
    }
}

/// `writer_lease_changed`. A release with no new holder passes an empty
/// `new_client_id` and `held = false`. `revision` is the affected block's
/// current revision (the lease change does not itself bump it).
pub(crate) fn writer_lease_event(
    session_id: &str,
    block_id: &str,
    stream_id: u64,
    revision: u64,
    previous_client_id: &str,
    new_client_id: &str,
    lease_held: bool,
) -> Event {
    Event {
        sequence: 0,
        revision,
        session_id: session_id.to_string(),
        event: Some(event::Event::WriterLeaseChanged(WriterLeaseChanged {
            session_id: session_id.to_string(),
            block_id: block_id.to_string(),
            stream_id: u32::try_from(stream_id).unwrap_or(u32::MAX),
            previous_client_id: previous_client_id.to_string(),
            new_client_id: new_client_id.to_string(),
            lease_held,
        })),
    }
}

/// `terminal_child_exited`, emitted when the existing reaper path observes the
/// child's exit status (never fabricated).
pub(crate) fn terminal_child_exited_event(
    session_id: &str,
    block_id: &str,
    revision: u64,
    status: &PtyExitStatus,
) -> Event {
    Event {
        sequence: 0,
        revision,
        session_id: session_id.to_string(),
        event: Some(event::Event::TerminalChildExited(TerminalChildExited {
            session_id: session_id.to_string(),
            block_id: block_id.to_string(),
            exit_code: status.exit_code,
            signal: status.signal.clone().unwrap_or_default(),
            exited: true,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn append_active(bus: &EventBus) -> u64 {
        bus.append(session_state_event("session:test", 1, SessionState::Active))
    }

    #[test]
    fn empty_log_head_is_zero_and_floor_is_one() {
        let bus = EventBus::new(4);
        assert_eq!(bus.current_sequence(), 0);
        assert_eq!(bus.lock_state().floor(), 1);
        assert_eq!(bus.lock_state().count(), 0);
    }

    #[test]
    fn retention_evicts_oldest_and_tracks_floor() {
        let bus = EventBus::new(2);
        assert_eq!(append_active(&bus), 1);
        assert_eq!(append_active(&bus), 2);
        assert_eq!(bus.current_sequence(), 2);
        assert_eq!(bus.lock_state().floor(), 1);
        assert_eq!(bus.lock_state().count(), 2);

        // A third event evicts sequence 1.
        assert_eq!(append_active(&bus), 3);
        assert_eq!(bus.current_sequence(), 3);
        assert_eq!(bus.lock_state().floor(), 2);
        assert_eq!(bus.lock_state().count(), 2);

        let state = bus.lock_state();
        assert_eq!(state.events.front().map(|e| e.sequence), Some(2));
        assert_eq!(state.events.back().map(|e| e.sequence), Some(3));
    }

    #[test]
    fn replayability_decision_table() {
        // No cursor is never a replay, even on an empty log.
        assert_eq!(
            decide_resume(0, 0, 1),
            ResumeDecision::Snapshot("no_cursor")
        );
        assert_eq!(
            decide_resume(0, 10, 3),
            ResumeDecision::Snapshot("no_cursor")
        );
        // Cursor ahead of the server is a snapshot.
        assert_eq!(
            decide_resume(11, 10, 3),
            ResumeDecision::Snapshot("cursor_ahead")
        );
        assert_eq!(
            decide_resume(1, 0, 1),
            ResumeDecision::Snapshot("cursor_ahead")
        );
        // Cursor below the retained floor is a snapshot.
        assert_eq!(
            decide_resume(3, 10, 5),
            ResumeDecision::Snapshot("cursor_below_floor")
        );
        // The boundary just at the floor replays.
        assert_eq!(decide_resume(4, 10, 5), ResumeDecision::Replay);
        // Cursor at the head replays with an empty event range.
        assert_eq!(decide_resume(10, 10, 5), ResumeDecision::Replay);
        // A mid-range cursor replays.
        assert_eq!(decide_resume(7, 10, 5), ResumeDecision::Replay);
    }

    #[test]
    fn disconnect_reason_spellings_are_frozen() {
        assert_eq!(DisconnectReason::IdleTimeout.as_str(), "idle_timeout");
        assert_eq!(DisconnectReason::PeerClosed.as_str(), "peer_closed");
        assert_eq!(DisconnectReason::TransportError.as_str(), "transport_error");
        assert_eq!(
            DisconnectReason::FlowControlOverflow.as_str(),
            "flow_control_overflow"
        );
        assert_eq!(DisconnectReason::ServerShutdown.as_str(), "server_shutdown");
    }

    /// The disconnect pair must be exactly consecutive even while another
    /// thread appends single events concurrently. With two separate `append`
    /// calls this assertion fails; with `append_pair` it cannot.
    #[test]
    fn append_pair_sequences_are_consecutive_under_contention() {
        use std::sync::atomic::AtomicBool;

        let bus = Arc::new(EventBus::new(1 << 16));
        let stop = Arc::new(AtomicBool::new(false));
        let contender = {
            let bus = Arc::clone(&bus);
            let stop = Arc::clone(&stop);
            std::thread::spawn(move || {
                while !stop.load(Ordering::SeqCst) {
                    bus.append(session_state_event("session:test", 1, SessionState::Active));
                }
            })
        };

        for _ in 0..2000 {
            let (first, second) = bus.append_pair(
                client_connection_state_event(&ClientId::new(), "Disconnected"),
                client_disconnected_event(&ClientId::new(), DisconnectReason::PeerClosed),
            );
            assert_eq!(
                second,
                first + 1,
                "append_pair must assign consecutive sequences"
            );
        }

        stop.store(true, Ordering::SeqCst);
        contender.join().expect("contender thread");

        // The whole retained log is still gap-free and duplicate-free.
        let state = bus.lock_state();
        let sequences: Vec<u64> = state.events.iter().map(|event| event.sequence).collect();
        assert!(sequences.len() > 2000);
        for pair in sequences.windows(2) {
            assert_eq!(pair[1], pair[0] + 1, "log must be contiguous");
        }
    }
}
