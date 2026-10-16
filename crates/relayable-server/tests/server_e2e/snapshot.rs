//! The attach snapshot and the subsequently delivered frames are contiguous.

use std::thread;
use std::time::{Duration, Instant};

use relayable_protocol::methods;
use relayable_protocol::v1::{request, response, AttachMode, SessionCloseRequest};

use crate::harness::*;

/// M4: the attach snapshot and subscriber registration are atomic, so
/// `screen_bytes` plus the delivered frames contain an unbroken, non-duplicated
/// marker sequence.
#[test]
fn attach_snapshot_and_frames_are_contiguous() {
    let mut server = ServerGuard::spawn();
    let mut producer = Client::new(server.connect());
    producer.handshake();
    let created = create(&mut producer, "m4");
    let session_id = created
        .session
        .as_ref()
        .expect("session")
        .session_id
        .clone();
    let block_id = created.block_id.clone();
    server.track(created.program_pid);
    let stream_a = attach(&mut producer, &block_id, AttachMode::Writer).stream_id;

    // A slow, continuous marker stream so the observer attaches mid-stream.
    input(
        &mut producer,
        stream_a,
        "i=0; while [ $i -lt 5000 ]; do printf 'MARK-%08d\\n' \"$i\"; i=$((i+1)); if [ $((i % 500)) -eq 0 ]; then sleep 0.05; fi; done\n",
    );
    thread::sleep(Duration::from_millis(150));

    let mut observer = Client::new(server.connect());
    observer.handshake();
    let attached = attach(&mut observer, &block_id, AttachMode::Observer);
    let screen = attached.snapshot.expect("snapshot").screen_bytes;

    observer.read_until("MARK-00004999", Instant::now() + Duration::from_secs(15));
    let mut combined = screen;
    combined.extend_from_slice(&observer.output);

    let markers = extract_markers(&combined);
    assert!(
        markers.len() > 100,
        "expected a substantial marker run, got {}",
        markers.len()
    );
    for pair in markers.windows(2) {
        assert_eq!(
            pair[1],
            pair[0] + 1,
            "gap or duplicate in the marker stream: {} then {}",
            pair[0],
            pair[1]
        );
    }

    match producer.request(
        methods::SESSION_CLOSE,
        request::Operation::SessionClose(SessionCloseRequest { session_id }),
    ) {
        response::Result::SessionClose(_) => {}
        other => panic!("cleanup close failed: {other:?}"),
    }
}
