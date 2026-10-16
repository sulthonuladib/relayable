use std::time::{Duration, Instant};

use relayable_model::BlockId;
use relayable_protocol::v1::{envelope::Body, AttachMode};

use super::harness::*;

/// CLIENT-API-001 rule 1: repeated `next_output` calls return the target
/// stream's frames exactly once each, in arrival order, with no byte loss.
#[test]
fn next_output_returns_successive_target_frames_without_loss() {
    let target = 7u64;
    let (mut client, server, rx) = spawn_output_server(
        vec![
            terminal_data(target, 1, b"first-"),
            terminal_data(target, 2, b"second"),
        ],
        Duration::from_millis(500),
    );
    client
        .attach_block(&BlockId::new(), AttachMode::Writer)
        .unwrap();

    let first = client
        .next_output(target, Duration::from_millis(500))
        .unwrap()
        .expect("first target frame");
    let second = client
        .next_output(target, Duration::from_millis(500))
        .unwrap()
        .expect("second target frame");
    assert_eq!(first, b"first-");
    assert_eq!(second, b"second");
    let mut joined = first;
    joined.extend_from_slice(&second);
    assert_eq!(joined, b"first-second", "no byte may be lost between calls");

    drop(client);
    let _ = server.join();
    drop(rx);
}

/// CLIENT-API-001 rule 3: a normal timeout is `Ok(None)`, never
/// [`ClientError::Timeout`].
#[test]
fn next_output_returns_none_on_timeout() {
    let target = 7u64;
    let (mut client, server, rx) = spawn_output_server(Vec::new(), Duration::from_millis(300));
    client
        .attach_block(&BlockId::new(), AttachMode::Writer)
        .unwrap();

    let result = client
        .next_output(target, Duration::from_millis(50))
        .expect("a timeout is not an error");
    assert!(result.is_none(), "expected Ok(None), got {result:?}");

    drop(client);
    let _ = server.join();
    drop(rx);
}

/// CLIENT-API-001 rule 5: `Duration::ZERO` serves frames already buffered in
/// `pending` (deferred by an earlier call) and returns `Ok(None)` promptly
/// with an empty queue, never reading the socket.
#[test]
fn zero_timeout_serves_buffered_frames_without_blocking() {
    let target = 7u64;
    let (mut client, server, rx) = spawn_output_server(
        vec![
            terminal_data(target, 1, b"buffered-bytes"),
            event_envelope(21),
        ],
        Duration::from_millis(500),
    );
    client
        .attach_block(&BlockId::new(), AttachMode::Writer)
        .unwrap();

    // `wait_for_event` reads the target data frame first (deferring it into
    // `pending`) and then the event, so the bytes are only reachable through
    // public API.
    let event = client
        .wait_for_event(Duration::from_millis(500))
        .unwrap()
        .expect("buffered event");
    assert_eq!(event.sequence, 21);

    let started = Instant::now();
    let bytes = client
        .next_output(target, Duration::ZERO)
        .unwrap()
        .expect("buffered target frame");
    assert_eq!(bytes, b"buffered-bytes");
    assert!(
        started.elapsed() < Duration::from_millis(250),
        "a zero-timeout drain must not block: {:?}",
        started.elapsed()
    );

    // With an empty queue the same call returns promptly without a read.
    let started = Instant::now();
    let nothing = client.next_output(target, Duration::ZERO).unwrap();
    assert!(nothing.is_none());
    assert!(
        started.elapsed() < Duration::from_millis(250),
        "an empty zero-timeout call must not block: {:?}",
        started.elapsed()
    );

    drop(client);
    let _ = server.join();
    drop(rx);
}

/// CLIENT-API-001 rule 2: every returned chunk is acknowledged exactly once
/// with `ack_bytes == chunk.len()`, and the totals agree.
#[test]
fn one_flow_control_ack_per_returned_chunk() {
    let target = 7u64;
    let chunks: &[&[u8]] = &[b"aaaa", b"bb", b"cccccc"];
    let script = chunks
        .iter()
        .enumerate()
        .map(|(index, bytes)| terminal_data(target, index as u64 + 1, bytes))
        .collect();
    let (mut client, server, rx) = spawn_output_server(script, Duration::from_millis(500));
    client
        .attach_block(&BlockId::new(), AttachMode::Writer)
        .unwrap();

    let mut returned: Vec<Vec<u8>> = Vec::new();
    for _ in 0..chunks.len() {
        returned.push(
            client
                .next_output(target, Duration::from_millis(500))
                .unwrap()
                .expect("scripted frame"),
        );
    }
    drop(client);

    let mut acks = Vec::new();
    while let Ok(envelope) = rx.recv_timeout(Duration::from_millis(500)) {
        if let Some(Body::FlowControl(flow)) = envelope.body {
            acks.push(flow);
        }
    }
    let _ = server.join();

    assert_eq!(
        acks.len(),
        chunks.len(),
        "exactly one ack per returned chunk"
    );
    for (chunk, ack) in returned.iter().zip(&acks) {
        assert_eq!(ack.stream_id, target);
        assert_eq!(ack.ack_bytes, chunk.len() as u64);
    }
    let returned_total: u64 = returned.iter().map(|chunk| chunk.len() as u64).sum();
    let acked_total: u64 = acks.iter().map(|ack| ack.ack_bytes).sum();
    assert_eq!(acked_total, returned_total);
}

/// CLIENT-API-001 rule 2 for a zero-length frame: the protocol layer does
/// not reject `ack_bytes == 0` (nothing validates a `FlowControl` body; the
/// server's `add_credit` is a no-op for zero), so the ack is still sent
/// exactly once and the "one ack per returned chunk" invariant holds.
#[test]
fn zero_length_chunk_is_acked_exactly_once() {
    let target = 7u64;
    let (mut client, server, rx) = spawn_output_server(
        vec![terminal_data(target, 1, b"")],
        Duration::from_millis(500),
    );
    client
        .attach_block(&BlockId::new(), AttachMode::Writer)
        .unwrap();

    let chunk = client
        .next_output(target, Duration::from_millis(500))
        .unwrap()
        .expect("zero-length frame");
    assert!(chunk.is_empty());
    drop(client);

    let mut acks = Vec::new();
    while let Ok(envelope) = rx.recv_timeout(Duration::from_millis(500)) {
        if let Some(Body::FlowControl(flow)) = envelope.body {
            acks.push(flow);
        }
    }
    let _ = server.join();
    assert_eq!(acks.len(), 1, "exactly one ack for the empty chunk");
    assert_eq!(acks[0].stream_id, target);
    assert_eq!(acks[0].ack_bytes, 0);
}
