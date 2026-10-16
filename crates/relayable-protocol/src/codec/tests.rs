use super::*;
use crate::limits::DEFAULT_LIMITS;
use crate::v1::{
    event, ClientDisconnected, Heartbeat, Response, SessionResumeResponse, SessionSnapshotResponse,
    TerminalChildExited, WriterLeaseChanged,
};

fn sample() -> Envelope {
    Envelope {
        protocol_version: "1.0".to_string(),
        request_id: Some(7),
        stream_id: 0,
        sequence: Some(11),
        trace_id: Some("trace-1".to_string()),
        timestamp_unix_millis: Some(1_700_000_000_000),
        body: Some(crate::v1::envelope::Body::Heartbeat(Heartbeat {
            sequence: Some(3),
        })),
    }
}

#[test]
fn round_trips_an_envelope() {
    let env = sample();
    let mut buf = Vec::new();
    write_envelope(&mut buf, &env).unwrap();
    // 4-byte prefix + non-empty body.
    assert!(buf.len() > 4);

    let mut cursor = std::io::Cursor::new(buf);
    let decoded = read_envelope(&mut cursor, DEFAULT_LIMITS.max_frame_bytes).unwrap();
    assert_eq!(decoded, Some(env));
}

#[test]
fn clean_eof_yields_none() {
    let mut cursor = std::io::Cursor::new(Vec::<u8>::new());
    assert_eq!(
        read_envelope(&mut cursor, DEFAULT_LIMITS.max_frame_bytes).unwrap(),
        None
    );
}

#[test]
fn zero_length_frame_is_rejected() {
    let mut cursor = std::io::Cursor::new(0u32.to_le_bytes().to_vec());
    assert!(matches!(
        read_envelope(&mut cursor, DEFAULT_LIMITS.max_frame_bytes),
        Err(CodecError::ZeroLengthFrame)
    ));
}

#[test]
fn oversized_frame_is_rejected_before_body_read() {
    let max = 16u32;
    let mut cursor = std::io::Cursor::new((max + 1).to_le_bytes().to_vec());
    match read_envelope(&mut cursor, max) {
        Err(CodecError::FrameTooLarge { len, max: m }) => {
            assert_eq!(len, 17);
            assert_eq!(m, 16);
        }
        other => panic!("expected FrameTooLarge, got {other:?}"),
    }
}

#[test]
fn truncated_body_is_rejected() {
    let mut bytes = 10u32.to_le_bytes().to_vec();
    bytes.extend_from_slice(&[1, 2, 3]);
    let mut cursor = std::io::Cursor::new(bytes);
    assert!(matches!(
        read_envelope(&mut cursor, DEFAULT_LIMITS.max_frame_bytes),
        Err(CodecError::Truncated)
    ));
}

#[test]
fn truncated_prefix_is_rejected() {
    let mut cursor = std::io::Cursor::new(vec![1u8, 2]);
    assert!(matches!(
        read_envelope(&mut cursor, DEFAULT_LIMITS.max_frame_bytes),
        Err(CodecError::Truncated)
    ));
}

#[test]
fn malformed_body_is_decode_error() {
    // A 1-byte body `0xFF` decodes as an unknown field tag start: prost
    // reports a decode error.
    let mut bytes = 1u32.to_le_bytes().to_vec();
    bytes.push(0xFF);
    let mut cursor = std::io::Cursor::new(bytes);
    assert!(matches!(
        read_envelope(&mut cursor, DEFAULT_LIMITS.max_frame_bytes),
        Err(CodecError::Decode(_))
    ));
}

#[test]
fn writing_an_empty_envelope_is_rejected() {
    let mut buf = Vec::new();
    assert!(matches!(
        write_envelope(&mut buf, &Envelope::default()),
        Err(CodecError::ZeroLengthFrame)
    ));
}

/// Encode then decode one envelope through the real frame codec.
fn round_trip(env: &Envelope) -> Envelope {
    let mut buf = Vec::new();
    write_envelope(&mut buf, env).unwrap();
    let mut cursor = std::io::Cursor::new(buf);
    read_envelope(&mut cursor, DEFAULT_LIMITS.max_frame_bytes)
        .unwrap()
        .expect("non-empty frame decodes to an envelope")
}

fn event_envelope(sequence: u64, session_id: &str, ev: event::Event) -> Envelope {
    Envelope {
        protocol_version: "1.0".to_string(),
        stream_id: 0,
        sequence: Some(sequence),
        body: Some(crate::v1::envelope::Body::Event(crate::v1::Event {
            sequence,
            revision: 0,
            session_id: session_id.to_string(),
            event: Some(ev),
        })),
        ..Envelope::default()
    }
}

#[test]
fn core_002_event_variants_round_trip_through_the_codec() {
    let variants = [
        event::Event::ClientDisconnected(ClientDisconnected {
            client_id: "client:1".to_string(),
            reason: "peer_closed".to_string(),
        }),
        event::Event::WriterLeaseChanged(WriterLeaseChanged {
            session_id: "session:1".to_string(),
            block_id: "block:1".to_string(),
            stream_id: 3,
            previous_client_id: "client:1".to_string(),
            new_client_id: "client:2".to_string(),
            lease_held: true,
        }),
        event::Event::TerminalChildExited(TerminalChildExited {
            session_id: "session:1".to_string(),
            block_id: "block:1".to_string(),
            exit_code: 130,
            signal: "SIGINT".to_string(),
            exited: true,
        }),
    ];

    for (offset, ev) in variants.into_iter().enumerate() {
        let sequence = 100 + offset as u64;
        let env = event_envelope(sequence, "session:1", ev.clone());
        let decoded = round_trip(&env);
        // Full byte-for-byte field equality, then an explicit check of the
        // fields the CORE-002 contract leans on.
        assert_eq!(decoded, env);
        match decoded.body {
            Some(crate::v1::envelope::Body::Event(e)) => {
                assert_eq!(e.sequence, sequence);
                assert_eq!(e.session_id, "session:1");
                assert_eq!(e.event, Some(ev));
            }
            other => panic!("expected Event body, got {other:?}"),
        }
    }
}

#[test]
fn snapshot_required_resume_response_round_trips() {
    let response = SessionResumeResponse {
        snapshot_required: true,
        snapshot_required_reason: "cursor_below_floor".to_string(),
        snapshot: Some(SessionSnapshotResponse {
            sequence: 42,
            ..SessionSnapshotResponse::default()
        }),
        events: Vec::new(),
        snapshot_sequence: 42,
        current_sequence: 42,
        retained_floor_sequence: 30,
        retained_event_count: 12,
    };
    let env = Envelope {
        protocol_version: "1.0".to_string(),
        request_id: Some(5),
        stream_id: 0,
        body: Some(crate::v1::envelope::Body::Response(Response {
            request_id: 5,
            result: Some(crate::v1::response::Result::SessionResume(response)),
        })),
        ..Envelope::default()
    };

    let decoded = round_trip(&env);
    assert_eq!(decoded, env);
    match decoded.body {
        Some(crate::v1::envelope::Body::Response(resp)) => match resp.result {
            Some(crate::v1::response::Result::SessionResume(r)) => {
                assert!(r.snapshot_required);
                assert_eq!(r.snapshot_required_reason, "cursor_below_floor");
                assert_eq!(r.snapshot.as_ref().map(|s| s.sequence), Some(42));
                assert_eq!(r.snapshot_sequence, 42);
                assert_eq!(r.current_sequence, 42);
                assert_eq!(r.retained_floor_sequence, 30);
                assert_eq!(r.retained_event_count, 12);
                assert!(
                    r.events.is_empty(),
                    "snapshot fallback must not fake a replay"
                );
            }
            other => panic!("expected session_resume result, got {other:?}"),
        },
        other => panic!("expected Response body, got {other:?}"),
    }
}
