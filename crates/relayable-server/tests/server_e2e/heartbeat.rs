//! Heartbeats and the idle-timeout disconnect reason reported to a subscriber.

use std::time::{Duration, Instant};

use relayable_protocol::v1::{envelope::Body, Envelope};
use relayable_protocol::PROTOCOL_VERSION;

use crate::harness::*;

/// A client-to-server heartbeat envelope; it carries no application data but
/// resets the server reader's idle deadline.
fn heartbeat_envelope() -> Envelope {
    Envelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        request_id: None,
        stream_id: 0,
        sequence: None,
        trace_id: None,
        timestamp_unix_millis: None,
        body: Some(Body::Heartbeat(relayable_protocol::v1::Heartbeat {
            sequence: None,
        })),
    }
}

/// The real reader path: a silent client must be classified `idle_timeout`
/// (never `transport_error`), while a heartbeating observer stays connected and
/// receives the disconnect event pair. This exercises the codec-wrapped timeout
/// shape over a live socket.
#[test]
fn idle_timeout_reason_is_reported_to_a_subscriber() {
    let mut server = ServerGuard::spawn_with_env(&[
        ("RELAYABLE_IDLE_TIMEOUT_MILLIS", "700"),
        ("RELAYABLE_HEARTBEAT_INTERVAL_MILLIS", "200"),
    ]);

    let mut observer = Client::new(server.connect());
    observer.handshake_as("e2e-observer");
    let created = create(&mut observer, "idle");
    let session_id = created
        .session
        .as_ref()
        .expect("session")
        .session_id
        .clone();
    let program_pid = created.program_pid;
    server.track(program_pid);
    // Subscribe the observer to live control events.
    let _ = resume(&mut observer, &session_id, 0);

    // A second client handshakes and is then deliberately silent.
    let mut victim = Client::new(server.connect());
    victim.handshake_as("e2e-victim");
    let victim_id = view(&mut observer, &session_id)
        .clients
        .iter()
        .find(|client| client.kind == "e2e-victim")
        .map(|client| client.client_id.clone())
        .expect("the victim is listed live");

    // Keep the observer alive with heartbeats well inside the idle window,
    // while the silent victim idles out.
    let deadline = Instant::now() + Duration::from_secs(4);
    let mut reason = None;
    'outer: while Instant::now() < deadline {
        observer.send(&heartbeat_envelope());
        let read_until = Instant::now() + Duration::from_millis(150);
        while let Some(envelope) = observer.recv_frame(read_until) {
            if let Some(Body::Event(event)) = envelope.body {
                if let Some(relayable_protocol::v1::event::Event::ClientDisconnected(
                    disconnected,
                )) = event.event
                {
                    if disconnected.client_id == victim_id {
                        reason = Some(disconnected.reason);
                        break 'outer;
                    }
                }
            }
        }
    }

    assert_eq!(reason.as_deref(), Some("idle_timeout"));
    assert!(
        proc_exists(program_pid),
        "the session and child must survive the idle disconnect"
    );
    // The victim is gone from the snapshot; the observer is still listed.
    let snapshot = view(&mut observer, &session_id);
    assert!(
        snapshot
            .clients
            .iter()
            .all(|client| client.client_id != victim_id),
        "the victim record must be removed"
    );
    assert!(
        snapshot
            .clients
            .iter()
            .any(|client| client.kind == "e2e-observer"),
        "the heartbeating observer stays connected"
    );
}
