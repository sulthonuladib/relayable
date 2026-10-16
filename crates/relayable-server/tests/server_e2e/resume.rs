//! `session.resume` replay/snapshot shapes and writer-lease takeover.

use std::time::Instant;

use relayable_protocol::methods;
use relayable_protocol::v1::{
    request, response, AttachMode, ErrorCode, SessionCloseRequest, TerminalInput,
};

use crate::harness::*;

/// CORE-002: `session.resume` replay/snapshot shapes and writer-lease takeover.
#[test]
fn resume_shapes_and_writer_takeover() {
    let mut server = ServerGuard::spawn();
    let mut a = Client::new(server.connect());
    a.handshake();
    let created = create(&mut a, "resume");
    let session_id = created
        .session
        .as_ref()
        .expect("session")
        .session_id
        .clone();
    let block_id = created.block_id.clone();
    server.track(created.program_pid);

    // First view completes synchronization (Ready) and emits the `ready` event.
    let _ = view(&mut a, &session_id);
    // A second view observes the log head with no further state change.
    let head = view(&mut a, &session_id).sequence;

    // No cursor: snapshot fallback, reason `no_cursor`, events EMPTY.
    let no_cursor = resume(&mut a, &session_id, 0);
    assert!(no_cursor.snapshot_required);
    assert_eq!(no_cursor.snapshot_required_reason, "no_cursor");
    assert!(no_cursor.events.is_empty(), "never a false replay");
    let snapshot = no_cursor.snapshot.as_ref().expect("snapshot payload");
    assert_eq!(no_cursor.snapshot_sequence, no_cursor.current_sequence);
    assert_eq!(snapshot.sequence, no_cursor.snapshot_sequence);

    // Cursor at the head: replay with an empty event range and no snapshot.
    let at_head = resume(&mut a, &session_id, head);
    assert!(!at_head.snapshot_required);
    assert!(at_head.snapshot.is_none());
    assert!(at_head.events.is_empty());
    assert_eq!(at_head.snapshot_sequence, head);
    assert_eq!(at_head.current_sequence, head);

    // Cursor ahead of the server: snapshot fallback, reason `cursor_ahead`.
    let ahead = resume(&mut a, &session_id, head + 100);
    assert!(ahead.snapshot_required);
    assert_eq!(ahead.snapshot_required_reason, "cursor_ahead");
    assert!(ahead.events.is_empty());

    // Cursor at the floor: the contiguous retained suffix replays in order.
    let from_floor = resume(&mut a, &session_id, 1);
    assert!(!from_floor.snapshot_required);
    assert!(
        !from_floor.events.is_empty(),
        "session create and lease events are retained"
    );
    assert_eq!(from_floor.events.first().map(|e| e.sequence), Some(2));
    assert_eq!(from_floor.events.last().map(|e| e.sequence), Some(head));
    for pair in from_floor.events.windows(2) {
        assert_eq!(
            pair[1].sequence,
            pair[0].sequence + 1,
            "replayed events must be contiguous with no duplicates"
        );
    }

    // Takeover: a second writer takes the single lease, and the previously
    // subscribed client A receives `writer_lease_changed`.
    let stream_a = attach(&mut a, &block_id, AttachMode::Writer).stream_id;
    let mut b = Client::new(server.connect());
    b.handshake();
    let taken = attach_takeover(&mut b, &block_id);
    assert!(taken.writer_lease, "B now holds the lease");

    let deadline = Instant::now() + WAIT;
    let event = loop {
        let event = next_event(&mut a, deadline).expect("a control event");
        if matches!(
            event.event,
            Some(relayable_protocol::v1::event::Event::WriterLeaseChanged(_))
        ) {
            break event;
        }
    };
    match event.event {
        Some(relayable_protocol::v1::event::Event::WriterLeaseChanged(change)) => {
            assert!(change.lease_held, "the lease is held by the new writer");
            assert_eq!(change.block_id, block_id);
            assert!(
                !change.previous_client_id.is_empty(),
                "a takeover names the previous holder"
            );
            assert!(!change.new_client_id.is_empty());
        }
        other => panic!("expected writer_lease_changed, got {other:?}"),
    }

    // The demoted holder's input is rejected, never silently dropped.
    expect_error(
        a.request(
            methods::TERMINAL_INPUT,
            request::Operation::TerminalInput(TerminalInput {
                stream_id: stream_a,
                bytes: b"should be rejected\n".to_vec(),
            }),
        ),
        ErrorCode::InputNotOwned,
    );
    // B holds the lease and can write.
    assert!(input(&mut b, taken.stream_id, "echo takeover\n") > 0);

    match b.request(
        methods::SESSION_CLOSE,
        request::Operation::SessionClose(SessionCloseRequest {
            session_id: session_id.clone(),
        }),
    ) {
        response::Result::SessionClose(_) => {}
        other => panic!("cleanup close failed: {other:?}"),
    }
}
