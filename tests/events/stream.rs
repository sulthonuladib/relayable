//! Event-stream sequencing and subscription-scoping scenarios.

use std::time::Duration;

use anyhow::{Context, Result};
use relayable_client::{Client, ClientConfig};
use relayable_integration_tests::harness::{collect_until_quiet, step, TestServer, WAIT};
use relayable_model::SessionId;
use relayable_protocol::v1::{response, AttachMode};

use crate::frames::{
    raw_connect, raw_handshake, raw_response_recording_events, raw_resume, read_event_envelopes,
    resume_envelope, QUIET,
};

/// Requirement 1/§C/§D: a subscriber observes a strictly increasing, gap-free
/// control-event stream with well-formed envelopes.
#[test]
fn event_stream_is_sequenced_and_contiguous_for_a_subscriber() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut creator = Client::connect(&socket, ClientConfig::new("events-creator"))?;
    let created = creator.create_session(Some("events"))?;
    let session_id = created.session.session_id.clone();
    server.track_child(created.program_pid);

    // A raw subscriber: handshake, then `session.resume` with no cursor. Only
    // the successful resume subscribes it to live events.
    let mut raw = raw_connect(&socket);
    raw_handshake(&mut raw);
    let resumed = raw_resume(&mut raw, 2, &session_id, 0);
    assert!(resumed.snapshot_required, "no cursor is a snapshot");
    assert_eq!(resumed.snapshot_required_reason, "no_cursor");
    let s = resumed.snapshot_sequence;
    assert_eq!(resumed.current_sequence, s);

    // Real state changes from another client: connect/handshake, attach, drop.
    let mut other = Client::connect(&socket, ClientConfig::new("events-other"))?;
    let attached = other.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(attached.writer_lease, "the other client holds the lease");
    drop(other);

    let frames = read_event_envelopes(&mut raw, QUIET, WAIT);
    assert!(
        !frames.is_empty(),
        "the subscriber must observe live events"
    );
    assert_eq!(
        frames[0].1.sequence,
        s + 1,
        "the live stream starts immediately after the snapshot cursor"
    );

    let mut previous = s;
    for (envelope, event) in &frames {
        assert_eq!(
            envelope.sequence,
            Some(event.sequence),
            "Envelope.sequence must equal Event.sequence"
        );
        assert_eq!(
            envelope.stream_id, 0,
            "events are control-class on stream 0"
        );
        assert!(
            envelope.request_id.is_none(),
            "events carry no request_id, got {:?}",
            envelope.request_id
        );
        assert_eq!(
            event.sequence,
            previous + 1,
            "observed events must be strictly increasing with no gap"
        );
        previous = event.sequence;
    }
    step(
        "events-1",
        &format!(
            "{} live events contiguous from S+1={} (Envelope.sequence == Event.sequence, stream_id 0, no request_id)",
            frames.len(),
            s + 1
        ),
    );
    Ok(())
}

/// §C wire ordering: a `session.resume` response frame is read before any
/// `Event` frame whose sequence is greater than the captured head — even while
/// another connection is generating events concurrently.
#[test]
fn resume_response_frame_precedes_later_events_on_the_same_connection() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut setup = Client::connect(&socket, ClientConfig::new("order-setup"))?;
    let created = setup.create_session(Some("order"))?;
    let session_id = created.session.session_id.clone();
    server.track_child(created.program_pid);

    let mut raw = raw_connect(&socket);
    raw_handshake(&mut raw);

    // A separate connection generates real events while the raw stream is read.
    let generator_socket = socket.clone();
    let generator = std::thread::spawn(move || {
        for _ in 0..40 {
            let client = Client::connect(&generator_socket, ClientConfig::new("order-generator"))
                .expect("generator connect");
            drop(client);
        }
    });

    raw.send(&resume_envelope(2, &session_id, 0))
        .expect("send resume");
    let (result, before) = raw_response_recording_events(&mut raw, 2);
    let resume = match result {
        response::Result::SessionResume(response) => response,
        other => panic!("raw session.resume failed: {other:?}"),
    };
    assert!(
        before.is_empty(),
        "no Event frame may precede the resume response, saw sequences {:?}",
        before
            .iter()
            .map(|(sequence, _)| *sequence)
            .collect::<Vec<_>>()
    );

    // The response names the captured head S; live events all follow it and are
    // contiguous from S+1.
    let s = resume.snapshot_sequence;
    assert_eq!(resume.current_sequence, s);
    let after = read_event_envelopes(&mut raw, QUIET, Duration::from_secs(5));
    for (envelope, event) in &after {
        assert_eq!(envelope.sequence, Some(event.sequence));
        assert!(
            event.sequence > s,
            "a live event {} must be greater than the response head {s}",
            event.sequence
        );
    }
    if let Some((_, first)) = after.first() {
        assert_eq!(first.sequence, s + 1, "live delivery starts at S+1");
        let mut previous = first.sequence;
        for (_, event) in after.iter().skip(1) {
            assert_eq!(event.sequence, previous + 1, "contiguous live events");
            previous = event.sequence;
        }
    }
    generator.join().expect("generator thread");
    step(
        "events-10-order",
        &format!(
            "resume response (request 2) read first under concurrent generation; {} live events all > S={s}",
            after.len()
        ),
    );
    Ok(())
}

/// §C: a bare `session.view` does NOT subscribe the connection to live events.
#[test]
fn session_view_does_not_subscribe_to_the_event_stream() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut creator = Client::connect(&socket, ClientConfig::new("view-creator"))?;
    let created = creator.create_session(Some("view"))?;
    let session_id = created.session.session_id.clone();
    let parsed: SessionId = session_id.parse().context("session id")?;
    server.track_child(created.program_pid);

    // The viewer ONLY views; it must never call `resume_session`.
    let mut viewer = Client::connect(&socket, ClientConfig::new("view-viewer"))?;
    let view = viewer.session_snapshot(&parsed)?;
    assert_eq!(
        view.session.as_ref().context("session")?.session_id,
        session_id,
        "the view response projects the requested session"
    );
    assert!(
        view.blocks
            .iter()
            .any(|block| block.block_id == created.block_id.to_string()),
        "the view response projects the block"
    );

    // A control subscriber proves the generated events really are fanned out.
    let mut control = Client::connect(&socket, ClientConfig::new("view-control"))?;
    let _ = control.resume_session(&parsed, None)?;

    // Real state changes: connect/handshake, create, disconnect.
    let mut generator = Client::connect(&socket, ClientConfig::new("view-generator"))?;
    let generated = generator.create_session(Some("view-generated"))?;
    server.track_child(generated.program_pid);
    drop(generator);

    let control_events = collect_until_quiet(&mut control, QUIET, WAIT)?;
    assert!(
        !control_events.is_empty(),
        "the control subscriber must observe the generated events"
    );

    // If the viewer had been subscribed, the fanned-out events would already be
    // buffered on its socket; this 2 s window is far longer than the
    // generation-to-read gap (the control subscriber already saw them).
    let viewer_events = viewer.collect_events(Duration::from_secs(2))?;
    assert!(
        viewer_events.is_empty(),
        "session.view must not subscribe; the viewer saw event sequences {:?}",
        viewer_events
            .iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>()
    );
    step(
        "events-11-view",
        &format!(
            "view response correct; control observed {} events; viewer observed 0 in a 2 s window",
            control_events.len()
        ),
    );
    Ok(())
}
