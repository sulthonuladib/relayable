//! Bounded per-block output tail and the tail/subscriber state behind one lock.

use std::collections::VecDeque;
use std::sync::Arc;

use super::subscriber::Subscriber;

/// Bounded per-block output tail plus the per-stream output sequence.
#[derive(Debug)]
pub(crate) struct OutputTail {
    bytes: VecDeque<u8>,
    capacity: usize,
    pub(super) stream_seq: u64,
}

impl OutputTail {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            bytes: VecDeque::new(),
            capacity: capacity.max(1),
            stream_seq: 0,
        }
    }

    /// Append a chunk, dropping the oldest bytes beyond the capacity, and
    /// advance the per-stream output sequence.
    pub(super) fn push(&mut self, chunk: &[u8]) {
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

    pub(super) fn snapshot(&self) -> Vec<u8> {
        self.bytes.iter().copied().collect()
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.bytes.len()
    }
}

/// The tail buffer and subscriber list, behind one lock.
///
/// The PTY reader appends to the tail, advances `stream_seq`, and fans out while
/// holding this lock, and `attach_stream` captures the tail and registers the
/// new subscriber while holding the same lock. That makes attach atomic: for a
/// given stream, `screen_bytes` plus the delivered frames are a contiguous,
/// non-duplicated suffix of the block output.
pub(super) struct OutputState {
    pub(super) tail: OutputTail,
    pub(super) subscribers: Vec<Arc<Subscriber>>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
