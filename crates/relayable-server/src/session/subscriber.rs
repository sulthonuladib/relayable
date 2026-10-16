//! A stream attached to a block: output fan-out and credit accounting.

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::mpsc::TrySendError;
use std::sync::Arc;

use relayable_model::ClientId;

use crate::connection::{terminal_output_envelope, ConnectionShared, WriterMsg};

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
    pub(super) fn add_credit(&self, ack_bytes: u64) {
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
    pub(super) fn deliver(&self, stream_seq: u64, chunk: &[u8]) {
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
            self.connection
                .close_now(crate::events::DisconnectReason::FlowControlOverflow);
        }
    }
}
