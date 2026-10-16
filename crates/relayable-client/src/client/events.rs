use std::time::{Duration, Instant};

#[cfg(doc)]
use relayable_protocol::v1::Limits;
use relayable_protocol::v1::{envelope::Body, Event};

use crate::client::Client;
use crate::config::MAX_PENDING_EVENTS;
use crate::error::ClientError;

impl Client {
    /// Return every `Event` frame already buffered inside the client, in
    /// arrival order, without reading the socket.
    ///
    /// Events are buffered by [`Client::wait_for_event`],
    /// [`Client::collect_events`], and whenever a control RPC drains a frame it
    /// is not waiting for. The buffer is bounded by [`MAX_PENDING_EVENTS`]; see
    /// that constant for the overflow policy.
    #[must_use]
    pub fn drain_events(&mut self) -> Vec<Event> {
        self.pending_events.drain(..).collect()
    }

    /// Block until the next `Event` frame arrives, or `timeout` elapses.
    ///
    /// Returns `Ok(None)` if no event arrived before the deadline. While
    /// waiting the method sends a `Heartbeat` whenever its own send cadence
    /// ([`Limits::heartbeat_interval_millis`], measured from the last heartbeat
    /// it *sent*) elapses. Inbound frames — including the server's own
    /// `Heartbeat` frames — never reset that timer, and the blocking read is
    /// capped so the loop wakes often enough to send on schedule. Non-event
    /// frames are retained (moved to the internal pending queue), never
    /// dropped.
    ///
    /// # Errors
    ///
    /// [`ClientError::Disconnected`] if the peer closes the connection, and any
    /// other [`ClientError`] from the transport.
    pub fn wait_for_event(&mut self, timeout: Duration) -> Result<Option<Event>, ClientError> {
        if let Some(event) = self.pending_events.pop_front() {
            return Ok(Some(event));
        }
        let heartbeat = self.heartbeat_interval();
        let start = Instant::now();
        let deadline = start.checked_add(timeout).unwrap_or(start);
        loop {
            if Instant::now() >= deadline {
                return Ok(None);
            }
            self.heartbeat_if_due(heartbeat)?;
            if Instant::now() >= deadline {
                return Ok(None);
            }
            let read_timeout = self.next_read_timeout(deadline, heartbeat);
            if read_timeout.is_zero() {
                continue;
            }
            if let Some(envelope) = self.read_event_frame(read_timeout)? {
                match envelope.body {
                    Some(Body::Event(event)) => return Ok(Some(event)),
                    _ => self.pending.push_back(envelope),
                }
            }
        }
    }

    /// Read `Event` frames until `timeout` elapses, returning every event
    /// received in that window in arrival order.
    ///
    /// Events already buffered inside the client are returned first. The method
    /// heartbeats and retains non-event frames exactly like
    /// [`Client::wait_for_event`]: the outbound cadence depends only on the last
    /// heartbeat this client sent, never on inbound frames. An empty vector
    /// means the deadline expired with no event, not an error.
    ///
    /// # Errors
    ///
    /// [`ClientError::Disconnected`] if the peer closes the connection, and any
    /// other [`ClientError`] from the transport.
    pub fn collect_events(&mut self, timeout: Duration) -> Result<Vec<Event>, ClientError> {
        let mut events: Vec<Event> = self.pending_events.drain(..).collect();
        let heartbeat = self.heartbeat_interval();
        let start = Instant::now();
        let deadline = start.checked_add(timeout).unwrap_or(start);
        loop {
            if Instant::now() >= deadline {
                return Ok(events);
            }
            self.heartbeat_if_due(heartbeat)?;
            if Instant::now() >= deadline {
                return Ok(events);
            }
            let read_timeout = self.next_read_timeout(deadline, heartbeat);
            if read_timeout.is_zero() {
                continue;
            }
            if let Some(envelope) = self.read_event_frame(read_timeout)? {
                match envelope.body {
                    Some(Body::Event(event)) => events.push(event),
                    _ => self.pending.push_back(envelope),
                }
            }
        }
    }

    /// Store a received event in the bounded pending list, evicting the oldest
    /// buffered event when full (see [`MAX_PENDING_EVENTS`]).
    pub(crate) fn enqueue_event(&mut self, event: Event) {
        if self.pending_events.len() >= MAX_PENDING_EVENTS {
            self.pending_events.pop_front();
        }
        self.pending_events.push_back(event);
    }
}
