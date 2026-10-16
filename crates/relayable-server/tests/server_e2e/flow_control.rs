//! A blocking program must not wedge the connection, leak the writer lease, or
//! bypass input backpressure.

use std::thread;
use std::time::{Duration, Instant};

use relayable_protocol::methods;
use relayable_protocol::v1::{
    request, response, AttachMode, BlockAttachRequest, ErrorCode, TerminalInput,
};

use crate::harness::*;

/// M2: a program that never reads must not wedge the connection or leak the
/// writer lease; a full input queue answers `rate_limited`.
#[test]
fn blocking_program_does_not_wedge_the_connection() {
    let mut server = ServerGuard::spawn();
    let mut client = Client::new(server.connect());
    client.handshake();
    let created = create(&mut client, "m2");
    let session_id = created
        .session
        .as_ref()
        .expect("session")
        .session_id
        .clone();
    let block_id = created.block_id.clone();
    server.track(created.program_pid);
    let stream_id = attach(&mut client, &block_id, AttachMode::Writer).stream_id;

    // Foreground program that never reads, with echo disabled so the large
    // input is not echoed back as output (which would trip output backpressure).
    assert!(input(&mut client, stream_id, "stty -echo; sleep 300\n") > 0);
    thread::sleep(Duration::from_millis(100));

    // ~1 MiB across frames that each fit the 1 MiB frame limit.
    let frame = vec![b'x'; 128 * 1024 - 64];
    let mut accepted = 0usize;
    for _ in 0..8 {
        let started = Instant::now();
        match client.request(
            methods::TERMINAL_INPUT,
            request::Operation::TerminalInput(TerminalInput {
                stream_id,
                bytes: frame.clone(),
            }),
        ) {
            response::Result::TerminalInput(ok) => accepted += ok.accepted_bytes as usize,
            other => panic!("unexpected terminal.input result: {other:?}"),
        }
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "terminal.input response was slow (> 2 s)"
        );
    }
    assert!(
        accepted >= frame.len(),
        "at least one frame must be accepted"
    );

    // The connection thread is still responsive.
    let snapshot = view(&mut client, &session_id);
    assert_eq!(snapshot.blocks[0].block_id, block_id);

    // The writer lease is still held; a second connection cannot take it.
    let mut other = Client::new(server.connect());
    other.handshake();
    expect_error(
        other.request(
            methods::BLOCK_ATTACH,
            request::Operation::BlockAttach(BlockAttachRequest {
                block_id: block_id.clone(),
                mode: AttachMode::Writer as i32,
                takeover: false,
            }),
        ),
        ErrorCode::InputNotOwned,
    );

    // Keep enqueueing until the bounded queue reports `rate_limited`.
    let mut rate_limited = false;
    for _ in 0..200 {
        match client.request(
            methods::TERMINAL_INPUT,
            request::Operation::TerminalInput(TerminalInput {
                stream_id,
                bytes: frame.clone(),
            }),
        ) {
            response::Result::TerminalInput(_) => {}
            response::Result::Error(error) if error.code == ErrorCode::RateLimited as i32 => {
                assert!(error.retryable);
                rate_limited = true;
                break;
            }
            other => panic!("unexpected overflow result: {other:?}"),
        }
    }
    assert!(
        rate_limited,
        "the bounded input queue must report rate_limited"
    );

    // Still responsive after overflow.
    assert_eq!(view(&mut client, &session_id).blocks[0].block_id, block_id);

    // Detaching frees the lease; another writer can attach normally.
    detach(&mut client, stream_id);
    let mut third = Client::new(server.connect());
    third.handshake();
    let attached = attach(&mut third, &block_id, AttachMode::Writer);
    assert!(attached.writer_lease);
}
