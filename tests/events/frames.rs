//! Raw-frame helpers shared by the `events` scenario modules.

use std::io::ErrorKind;
use std::path::Path;
use std::time::{Duration, Instant};

use relayable_integration_tests::harness::WAIT;
use relayable_protocol::codec::CodecError;
use relayable_protocol::methods;
use relayable_protocol::transport::{FrameConnection, TransportError};
use relayable_protocol::v1::event::Event as EventKind;
use relayable_protocol::v1::{self, envelope::Body, request, response, Envelope, Request};
use relayable_protocol::PROTOCOL_VERSION;

/// A quiet window long enough for a fan-out to land, short enough to keep the
/// suite fast.
pub(crate) const QUIET: Duration = Duration::from_millis(500);

// --- raw-frame helpers (result-level, used only where the client API cannot
// expose the envelope) -------------------------------------------------------

pub(crate) fn raw_envelope(request_id: Option<u64>, body: Body) -> Envelope {
    Envelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        request_id,
        stream_id: 0,
        sequence: None,
        trace_id: None,
        timestamp_unix_millis: None,
        body: Some(body),
    }
}

pub(crate) fn raw_connect(socket: &Path) -> FrameConnection {
    let deadline = Instant::now() + WAIT;
    loop {
        match FrameConnection::connect(socket) {
            Ok(connection) => return connection,
            Err(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            Err(error) => panic!("raw connect to {}: {error}", socket.display()),
        }
    }
}

pub(crate) fn is_timeout(error: &TransportError) -> bool {
    let io = match error {
        TransportError::Io(error) => error,
        TransportError::Codec(CodecError::Io(error)) => error,
        TransportError::Codec(_) => return false,
    };
    matches!(io.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut)
}

pub(crate) fn raw_handshake(connection: &mut FrameConnection) {
    connection
        .send(&raw_envelope(
            Some(1),
            Body::Handshake(v1::Handshake {
                client_kind: "events-raw".to_string(),
                platform: "linux".to_string(),
                client_version: "0".to_string(),
                protocol_version: PROTOCOL_VERSION.to_string(),
                capabilities: Vec::new(),
            }),
        ))
        .expect("send raw handshake");
    match raw_response(connection, 1) {
        response::Result::Hello(_) => {}
        other => panic!("raw handshake failed: {other:?}"),
    }
}

/// Read until the response for `request_id`. Heartbeats and events are ignored:
/// once a connection is subscribed, its own `ready` event may arrive before a
/// later response and must not be mistaken for the reply.
pub(crate) fn raw_response(connection: &mut FrameConnection, request_id: u64) -> response::Result {
    let deadline = Instant::now() + WAIT;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(
            !remaining.is_zero(),
            "no raw response for request {request_id}"
        );
        connection.set_read_timeout(Some(remaining)).ok();
        match connection.recv() {
            Ok(Some(envelope)) => match envelope.body {
                Some(Body::Response(response)) if response.request_id == request_id => {
                    return response.result.expect("response result");
                }
                Some(Body::Heartbeat(_) | Body::Event(_)) => {}
                other => panic!("unexpected frame awaiting {request_id}: {other:?}"),
            },
            Ok(None) => panic!("connection closed before response {request_id}"),
            Err(error) => {
                assert!(
                    is_timeout(&error),
                    "raw recv error for request {request_id}: {error}"
                );
            }
        }
    }
}

/// Read raw frames until the response for `request_id`, returning the response
/// and every `Event` frame seen **before** it (with its `Envelope.sequence`).
///
/// Used to assert the frozen §C wire ordering: a `session.resume` response is
/// enqueued before the connection is registered for live delivery, so no event
/// frame may precede it.
pub(crate) fn raw_response_recording_events(
    connection: &mut FrameConnection,
    request_id: u64,
) -> (response::Result, Vec<(u64, v1::Event)>) {
    let mut events = Vec::new();
    let deadline = Instant::now() + WAIT;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(
            !remaining.is_zero(),
            "no raw response for request {request_id}"
        );
        connection.set_read_timeout(Some(remaining)).ok();
        match connection.recv() {
            Ok(Some(envelope)) => match envelope.body {
                Some(Body::Response(response)) if response.request_id == request_id => {
                    return (response.result.expect("response result"), events);
                }
                Some(Body::Event(event)) => {
                    events.push((envelope.sequence.unwrap_or(0), event));
                }
                Some(Body::Heartbeat(_)) => {}
                other => panic!("unexpected frame awaiting {request_id}: {other:?}"),
            },
            Ok(None) => panic!("connection closed before response {request_id}"),
            Err(error) => {
                assert!(
                    is_timeout(&error),
                    "raw recv error for request {request_id}: {error}"
                );
            }
        }
    }
}

pub(crate) fn resume_envelope(request_id: u64, session_id: &str, cursor: u64) -> Envelope {
    raw_envelope(
        Some(request_id),
        Body::Request(Request {
            method: methods::SESSION_RESUME.to_string(),
            operation: Some(request::Operation::SessionResume(
                v1::SessionResumeRequest {
                    session_id: session_id.to_string(),
                    last_sequence_applied: cursor,
                },
            )),
        }),
    )
}

pub(crate) fn raw_resume(
    connection: &mut FrameConnection,
    request_id: u64,
    session_id: &str,
    cursor: u64,
) -> v1::SessionResumeResponse {
    connection
        .send(&resume_envelope(request_id, session_id, cursor))
        .expect("send raw session.resume");
    match raw_response(connection, request_id) {
        response::Result::SessionResume(response) => response,
        other => panic!("raw session.resume failed: {other:?}"),
    }
}

/// Collect raw `Event` envelopes until `quiet` passes with nothing new (or `cap`).
pub(crate) fn read_event_envelopes(
    connection: &mut FrameConnection,
    quiet: Duration,
    cap: Duration,
) -> Vec<(Envelope, v1::Event)> {
    let mut events = Vec::new();
    let deadline = Instant::now() + cap;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        connection.set_read_timeout(Some(remaining.min(quiet))).ok();
        match connection.recv() {
            Ok(Some(envelope)) => {
                if let Some(Body::Event(event)) = &envelope.body {
                    events.push((envelope.clone(), event.clone()));
                }
            }
            Ok(None) => break,
            Err(error) if is_timeout(&error) => break,
            Err(error) => panic!("raw event recv: {error}"),
        }
    }
    events
}

/// Locate the adjacent `client_connection_state_changed{Disconnected}` then
/// `client_disconnected{reason}` pair and return `(first_sequence, reason, client_id)`.
pub(crate) fn disconnect_pair(events: &[v1::Event]) -> Option<(u64, String, String)> {
    events.windows(2).find_map(|pair| {
        let state = match &pair[0].event {
            Some(EventKind::ClientConnectionStateChanged(state)) => state,
            _ => return None,
        };
        let disconnected = match &pair[1].event {
            Some(EventKind::ClientDisconnected(disconnected)) => disconnected,
            _ => return None,
        };
        if state.connection_state == "Disconnected" && state.client_id == disconnected.client_id {
            Some((
                pair[0].sequence,
                disconnected.reason.clone(),
                state.client_id.clone(),
            ))
        } else {
            None
        }
    })
}
