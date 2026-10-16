//! The server-owned runtime for one session.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use relayable_model::{SessionId, WindowId};

use super::block::BlockRuntime;

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
