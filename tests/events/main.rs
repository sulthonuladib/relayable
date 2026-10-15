//! CORE-002 assignment E — retained event log and `session.resume` proofs.
//!
//! These tests drive the real `relayable-server` and the real
//! `relayable-client`. The frozen semantics are
//! `protocol/compatibility/core-002-method-set.md`:
//!
//! - §B cursor rule: replayable iff `retained_floor_sequence <= cursor + 1` and
//!   `cursor <= current_sequence`; otherwise a fresh snapshot with a reason in
//!   `{no_cursor, cursor_below_floor, cursor_ahead}` and never a false replay;
//! - §C ordering: a `session.resume` response always precedes any live event
//!   with sequence greater than the captured head, and the snapshot/replay
//!   boundary has no gap and no duplicate;
//! - §D events: control-class frames on `stream_id 0`, no `request_id`, and
//!   `Envelope.sequence == Event.sequence`;
//! - §F configuration: retention/idle/heartbeat are per-process env parameters.
//!
//! The envelope-level assertions (stream id, request id, `Envelope.sequence`)
//! use the protocol crate's own `FrameConnection`, because the client API
//! intentionally surfaces only `Event`; every client-level operation uses the
//! frozen `relayable-client` API.

use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use relayable_client::{Client, ClientConfig};
use relayable_integration_tests::harness::{
    assert_event_sequences_contiguous, collect_events_over, collect_until_quiet, step, TestServer,
    WAIT,
};
use relayable_model::SessionId;
use relayable_protocol::codec::CodecError;
use relayable_protocol::methods;
use relayable_protocol::transport::{FrameConnection, TransportError};
use relayable_protocol::v1::event::Event as EventKind;
use relayable_protocol::v1::{
    self, envelope::Body, request, response, AttachMode, Envelope, ErrorCode, Request,
};
use relayable_protocol::PROTOCOL_VERSION;

/// A quiet window long enough for a fan-out to land, short enough to keep the
/// suite fast.
const QUIET: Duration = Duration::from_millis(500);

// --- raw-frame helpers (result-level, used only where the client API cannot
// expose the envelope) -------------------------------------------------------

fn raw_envelope(request_id: Option<u64>, body: Body) -> Envelope {
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

fn raw_connect(socket: &Path) -> FrameConnection {
    let deadline = Instant::now() + WAIT;
    loop {
        match FrameConnection::connect(socket) {
            Ok(connection) => return connection,
            Err(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            Err(error) => panic!("raw connect to {}: {error}", socket.display()),
        }
    }
}

fn is_timeout(error: &TransportError) -> bool {
    let io = match error {
        TransportError::Io(error) => error,
        TransportError::Codec(CodecError::Io(error)) => error,
        TransportError::Codec(_) => return false,
    };
    matches!(io.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut)
}

fn raw_handshake(connection: &mut FrameConnection) {
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
fn raw_response(connection: &mut FrameConnection, request_id: u64) -> response::Result {
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
fn raw_response_recording_events(
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

fn resume_envelope(request_id: u64, session_id: &str, cursor: u64) -> Envelope {
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

fn raw_resume(
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
fn read_event_envelopes(
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
fn disconnect_pair(events: &[v1::Event]) -> Option<(u64, String, String)> {
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

// --- tests ------------------------------------------------------------------

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

/// Requirement 4/§C: a cursor equal to the snapshot cursor replays exactly the
/// subscriber's observed suffix with no gap and no duplicate, and live delivery
/// then continues from the last replayed sequence.
#[test]
fn resume_replays_a_contiguous_suffix_with_no_gap_and_no_duplicate() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut a = Client::connect(&socket, ClientConfig::new("replay-a"))?;
    let created = a.create_session(Some("replay"))?;
    let session_id = created.session.session_id.clone();
    let parsed: SessionId = session_id.parse().context("session id")?;
    server.track_child(created.program_pid);

    // A resumes at S (no cursor) and is subscribed from S+1.
    let resumed = a.resume_session(&parsed, None)?;
    assert!(resumed.snapshot_required, "no cursor is a snapshot");
    let s = resumed.snapshot_sequence;

    // Generate live events.
    let mut c = Client::connect(&socket, ClientConfig::new("replay-c"))?;
    let writer = c.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(writer.writer_lease);
    drop(c);

    let observed = collect_until_quiet(&mut a, QUIET, WAIT)?;
    assert!(!observed.is_empty(), "A must observe live events");
    assert_eq!(observed[0].sequence, s + 1);
    assert_event_sequences_contiguous(&observed);

    // B's own handshake is one more event A must observe before B resumes.
    let mut b = Client::connect(&socket, ClientConfig::new("replay-b"))?;
    let tail = collect_until_quiet(&mut a, QUIET, WAIT)?;
    let mut a_stream = observed.clone();
    a_stream.extend(tail);

    let replay = b.resume_session(&parsed, Some(s))?;
    assert!(!replay.snapshot_required, "S must be replayable");
    assert!(replay.snapshot.is_none(), "a replay carries no snapshot");
    assert_eq!(replay.snapshot_sequence, s);
    assert_eq!(
        replay.events, a_stream,
        "B's replay must equal A's observed suffix S+1.. in the same order"
    );
    assert_event_sequences_contiguous(&replay.events);
    let unique: BTreeSet<u64> = replay.events.iter().map(|event| event.sequence).collect();
    assert_eq!(
        unique.len(),
        replay.events.len(),
        "the replay must contain no duplicate sequences"
    );

    // B's subsequent live stream continues from the last replayed sequence.
    let live = collect_until_quiet(&mut b, QUIET, WAIT)?;
    let last = replay.events.last().map_or(s, |event| event.sequence);
    assert_eq!(
        live.first().map(|event| event.sequence),
        Some(last + 1),
        "live delivery must continue with no gap after the replay"
    );
    assert_event_sequences_contiguous(&live);
    step(
        "events-2",
        &format!(
            "replayed {} events S+1={}..={last}, live continued at {}",
            replay.events.len(),
            s + 1,
            last + 1
        ),
    );
    Ok(())
}

/// Requirement 2/§B.3/§B.4: a cursor below the retained floor cannot be
/// replayed, so the server demands a snapshot and returns no events.
#[test]
fn cursor_below_the_retained_floor_yields_snapshot_required_with_no_replay() -> Result<()> {
    let mut server = TestServer::spawn_with_env(&[("RELAYABLE_EVENT_RETENTION_EVENTS", "4")]);
    let socket = server.socket_path().to_path_buf();

    let mut a = Client::connect(&socket, ClientConfig::new("retention"))?;
    let created = a.create_session(Some("retention"))?;
    let session_id = created.session.session_id.clone();
    let parsed: SessionId = session_id.parse().context("session id")?;
    server.track_child(created.program_pid);

    // Generate more events than the retention bound so the oldest are evicted.
    let mut fillers = Vec::new();
    for _ in 0..8 {
        fillers.push(Client::connect(&socket, ClientConfig::new("filler"))?);
    }

    let old_cursor = 1u64;
    let resumed = a.resume_session(&parsed, Some(old_cursor))?;
    assert!(
        resumed.snapshot_required,
        "a below-floor cursor needs a snapshot"
    );
    assert_eq!(resumed.snapshot_required_reason, "cursor_below_floor");
    assert!(resumed.events.is_empty(), "never a false replay");
    assert!(
        resumed.retained_floor_sequence > old_cursor + 1,
        "floor {} must exceed cursor+1 {}",
        resumed.retained_floor_sequence,
        old_cursor + 1
    );
    assert!(
        resumed.retained_event_count <= 4,
        "retention bound is 4, saw {}",
        resumed.retained_event_count
    );
    let snapshot = resumed.snapshot.as_ref().expect("snapshot payload");
    assert_eq!(snapshot.sequence, resumed.snapshot_sequence);
    assert_eq!(resumed.snapshot_sequence, resumed.current_sequence);
    step(
        "events-3",
        &format!(
            "old_cursor={old_cursor} floor={} current={} retained={} reason=cursor_below_floor, snapshot.sequence={}",
            resumed.retained_floor_sequence,
            resumed.current_sequence,
            resumed.retained_event_count,
            snapshot.sequence
        ),
    );
    drop(fillers);
    Ok(())
}

/// Requirement 2/§B.4: a cursor ahead of the server cannot be replayed.
#[test]
fn cursor_ahead_of_the_server_yields_snapshot_required() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut a = Client::connect(&socket, ClientConfig::new("ahead"))?;
    let created = a.create_session(Some("ahead"))?;
    let session_id = created.session.session_id.clone();
    let parsed: SessionId = session_id.parse().context("session id")?;
    server.track_child(created.program_pid);

    let head = a.resume_session(&parsed, None)?;
    let ahead = a.resume_session(&parsed, Some(head.current_sequence + 1000))?;
    assert!(ahead.snapshot_required);
    assert_eq!(ahead.snapshot_required_reason, "cursor_ahead");
    assert!(ahead.events.is_empty(), "never a false replay");
    let snapshot = ahead.snapshot.as_ref().expect("snapshot payload");
    assert_eq!(snapshot.sequence, ahead.snapshot_sequence);
    assert_eq!(ahead.snapshot_sequence, ahead.current_sequence);
    step(
        "events-4",
        &format!(
            "cursor={} > current={}, reason=cursor_ahead, snapshot.sequence={}",
            head.current_sequence + 1000,
            ahead.current_sequence,
            snapshot.sequence
        ),
    );
    Ok(())
}

/// Requirement 2/§B.4: no cursor is the documented "snapshot, never a false
/// replay" fallback with reason `no_cursor`.
#[test]
fn no_cursor_yields_snapshot_required_with_a_fresh_snapshot() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut a = Client::connect(&socket, ClientConfig::new("no-cursor"))?;
    let created = a.create_session(Some("no-cursor"))?;
    let session_id = created.session.session_id.clone();
    let parsed: SessionId = session_id.parse().context("session id")?;
    server.track_child(created.program_pid);

    let resumed = a.resume_session(&parsed, None)?;
    assert!(resumed.snapshot_required);
    assert_eq!(resumed.snapshot_required_reason, "no_cursor");
    assert!(resumed.events.is_empty(), "never a false replay");
    let snapshot = resumed.snapshot.as_ref().expect("snapshot payload");
    assert_eq!(snapshot.sequence, resumed.snapshot_sequence);
    assert_eq!(resumed.snapshot_sequence, resumed.current_sequence);
    assert!(
        snapshot
            .blocks
            .iter()
            .any(|block| block.block_id == created.block_id.to_string()),
        "the fresh snapshot projects the live block"
    );
    step(
        "events-5",
        &format!(
            "no cursor: snapshot_required=true reason=no_cursor snapshot.sequence={} == current",
            snapshot.sequence
        ),
    );
    Ok(())
}

/// Requirement 3/§D: an idle victim is marked `Disconnected` and immediately
/// followed (adjacent sequences) by `client_disconnected{idle_timeout}` while
/// the heartbeating observer stays connected.
#[test]
fn idle_timeout_marks_the_client_disconnected() -> Result<()> {
    let mut server = TestServer::spawn_with_env(&[
        ("RELAYABLE_IDLE_TIMEOUT_MILLIS", "700"),
        ("RELAYABLE_HEARTBEAT_INTERVAL_MILLIS", "200"),
    ]);
    let socket = server.socket_path().to_path_buf();

    let mut observer = Client::connect(&socket, ClientConfig::new("idle-observer"))?;
    let created = observer.create_session(Some("idle"))?;
    let session_id = created.session.session_id.clone();
    let parsed: SessionId = session_id.parse().context("session id")?;
    server.track_child(created.program_pid);
    let _ = observer.resume_session(&parsed, None)?;

    // The victim handshakes and is then deliberately never touched for the
    // whole idle window.
    let _victim = Client::connect(&socket, ClientConfig::new("idle-victim"))?;
    let snapshot = observer.session_snapshot(&parsed)?;
    let victim_id = snapshot
        .clients
        .iter()
        .find(|client| client.kind == "idle-victim")
        .map(|client| client.client_id.clone())
        .context("the victim record is live at handshake time")?;

    // `collect_events` in short slices heartbeats while idle, so the observer
    // survives the idle window that expires the silent victim.
    let events = collect_events_over(
        &mut observer,
        Duration::from_millis(50),
        Duration::from_secs(2),
    )?;
    let (first, reason, client_id) =
        disconnect_pair(&events).context("an idle-timeout disconnect pair")?;
    assert_eq!(reason, "idle_timeout");
    assert_eq!(client_id, victim_id, "the victim, not the observer");
    let second = events
        .iter()
        .find(|event| event.sequence == first + 1)
        .context("the following sequence")?;
    assert!(
        matches!(&second.event, Some(EventKind::ClientDisconnected(_))),
        "the disconnect event must be adjacent to the state change"
    );
    assert!(
        events
            .iter()
            .all(|event| !matches!(&event.event, Some(EventKind::ClientDisconnected(d)) if d.client_id != victim_id)),
        "only the victim may be disconnected"
    );
    step(
        "events-6a",
        &format!(
            "victim {victim_id}: Disconnected at seq {first} then client_disconnected{{idle_timeout}} at seq {}",
            first + 1
        ),
    );
    Ok(())
}

/// Requirement 3/§D: a dropped transport is marked `Disconnected` and
/// immediately followed by an ordered `client_disconnected`.
///
/// Classification note: an abrupt local transport drop races two server
/// observations — the reader's clean EOF (`Ok(None)` -> `PeerClosed` =
/// `peer_closed`) and a writer heartbeat send failing on the just-closed socket
/// (`TransportError` = `transport_error`). The frozen §D reason set covers
/// both, so this test asserts the disjunction and records which the
/// implementation observed; the ordered `Disconnected` then `client_disconnected`
/// pair and the victim identity are the load-bearing assertions.
#[test]
fn transport_drop_marks_the_client_disconnected() -> Result<()> {
    let mut server = TestServer::spawn_with_env(&[
        ("RELAYABLE_IDLE_TIMEOUT_MILLIS", "700"),
        ("RELAYABLE_HEARTBEAT_INTERVAL_MILLIS", "200"),
    ]);
    let socket = server.socket_path().to_path_buf();

    let mut observer = Client::connect(&socket, ClientConfig::new("drop-observer"))?;
    let created = observer.create_session(Some("drop"))?;
    let session_id = created.session.session_id.clone();
    let parsed: SessionId = session_id.parse().context("session id")?;
    server.track_child(created.program_pid);
    let _ = observer.resume_session(&parsed, None)?;

    let victim = Client::connect(&socket, ClientConfig::new("drop-victim"))?;
    let snapshot = observer.session_snapshot(&parsed)?;
    let victim_id = snapshot
        .clients
        .iter()
        .find(|client| client.kind == "drop-victim")
        .map(|client| client.client_id.clone())
        .context("the victim record is live at handshake time")?;

    // Abrupt transport drop while the observer is alive.
    drop(victim);

    let events = collect_events_over(
        &mut observer,
        Duration::from_millis(50),
        Duration::from_secs(2),
    )?;
    let (first, reason, client_id) =
        disconnect_pair(&events).context("a transport-drop disconnect pair")?;
    assert!(
        reason == "peer_closed" || reason == "transport_error",
        "a dropped transport must be classified peer_closed or transport_error, got {reason:?}"
    );
    assert_eq!(client_id, victim_id);
    let second = events
        .iter()
        .find(|event| event.sequence == first + 1)
        .context("the following sequence")?;
    assert!(
        matches!(&second.event, Some(EventKind::ClientDisconnected(_))),
        "the disconnect event must be adjacent to the state change"
    );
    step(
        "events-6b",
        &format!(
            "victim {victim_id}: Disconnected at seq {first} then client_disconnected{{{reason}}} at seq {}",
            first + 1
        ),
    );
    Ok(())
}

/// A waiting client must not be dropped merely for waiting: ONE single
/// `collect_events` call that spans several server idle deadlines stays
/// connected, while a genuinely silent victim is disconnected. This is the
/// integration-level proof of the client's outbound-heartbeat baseline plus the
/// server's idle classification.
#[test]
fn single_long_wait_does_not_trip_the_server_idle_timeout() -> Result<()> {
    let mut server = TestServer::spawn_with_env(&[
        ("RELAYABLE_IDLE_TIMEOUT_MILLIS", "700"),
        ("RELAYABLE_HEARTBEAT_INTERVAL_MILLIS", "200"),
    ]);
    let socket = server.socket_path().to_path_buf();

    let mut observer = Client::connect(&socket, ClientConfig::new("long-wait-observer"))?;
    let created = observer.create_session(Some("long-wait"))?;
    let session_id = created.session.session_id.clone();
    let parsed: SessionId = session_id.parse().context("session id")?;
    server.track_child(created.program_pid);
    let _ = observer.resume_session(&parsed, None)?;

    // A genuinely silent victim: handshake only, then never touched again.
    let _victim = Client::connect(&socket, ClientConfig::new("long-wait-victim"))?;
    let snapshot = observer.session_snapshot(&parsed)?;
    let victim_id = snapshot
        .clients
        .iter()
        .find(|client| client.kind == "long-wait-victim")
        .map(|client| client.client_id.clone())
        .context("the victim record is live at handshake time")?;

    // ONE long wait spanning several 700 ms idle deadlines, with no other API
    // call in between. It must return Ok, not ClientError::Disconnected.
    let events = observer
        .collect_events(Duration::from_millis(2500))
        .expect("a single long wait must not trip the observer's idle timeout");
    assert!(
        events.iter().any(|event| matches!(
            &event.event,
            Some(EventKind::ClientDisconnected(disconnected))
                if disconnected.client_id == victim_id
        )),
        "the long wait must still deliver the victim's client_disconnected"
    );
    let (first, reason, client_id) =
        disconnect_pair(&events).context("the silent victim's idle-timeout pair")?;
    assert_eq!(reason, "idle_timeout");
    assert_eq!(client_id, victim_id, "the victim, not the observer");
    let second = events
        .iter()
        .find(|event| event.sequence == first + 1)
        .context("the following sequence")?;
    assert!(
        matches!(&second.event, Some(EventKind::ClientDisconnected(_))),
        "the disconnect event must be adjacent to the state change"
    );
    step(
        "events-6c",
        &format!(
            "single collect_events(2500ms) stayed connected; victim {victim_id} Disconnected at seq {first} then client_disconnected{{idle_timeout}} at seq {}",
            first + 1
        ),
    );
    Ok(())
}

/// §D/D8 retry-safety: replaying the exact same `session.resume` request id is
/// answered with `duplicate_request` (not re-run), and the connection's stream
/// stays healthy afterwards.
#[test]
fn replayed_resume_request_returns_the_original_outcome() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut setup = Client::connect(&socket, ClientConfig::new("dup-setup"))?;
    let created = setup.create_session(Some("dup"))?;
    let session_id = created.session.session_id.clone();
    server.track_child(created.program_pid);

    let mut conn = raw_connect(&socket);
    raw_handshake(&mut conn);

    // First resume (request id 2) succeeds and subscribes the connection.
    conn.send(&resume_envelope(2, &session_id, 1))
        .expect("first resume");
    let first = raw_response(&mut conn, 2);
    assert!(
        matches!(first, response::Result::SessionResume(_)),
        "the first resume must succeed, got {first:?}"
    );

    // The exact same request id replayed: the dedup path must answer
    // duplicate_request instead of running resume again.
    conn.send(&resume_envelope(2, &session_id, 1))
        .expect("replayed resume");
    let second = raw_response(&mut conn, 2);
    match second {
        response::Result::Error(error) => assert_eq!(
            error.code,
            ErrorCode::DuplicateRequest as i32,
            "second frame must be duplicate_request, got {error:?}"
        ),
        other => panic!("a replayed request id must be duplicate_request, got {other:?}"),
    }

    // The server did not corrupt the connection: a fresh request id answers.
    conn.send(&raw_envelope(
        Some(3),
        Body::Request(Request {
            method: methods::SESSION_LIST.to_string(),
            operation: Some(request::Operation::SessionList(v1::SessionListRequest {})),
        }),
    ))
    .expect("follow-up session.list");
    match raw_response(&mut conn, 3) {
        response::Result::SessionList(_) => {}
        other => panic!("the stream was corrupted after a duplicate resume: {other:?}"),
    }
    step(
        "events-7",
        "same resume request id twice -> duplicate_request; follow-up session.list still answers",
    );
    Ok(())
}

/// §D: the reaper emits `terminal_child_exited` for a real observed child exit,
/// with the server's actual exit code/signal, at most once.
#[test]
fn terminal_child_exited_is_emitted_when_the_program_exits() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut a = Client::connect(&socket, ClientConfig::new("exit-subscriber"))?;
    let created = a.create_session(Some("exit"))?;
    let session_id = created.session.session_id.clone();
    let parsed: SessionId = session_id.parse().context("session id")?;
    server.track_child(created.program_pid);
    let _ = a.resume_session(&parsed, None)?;
    let attached = a.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(
        attached.writer_lease,
        "the subscriber holds the writer lease"
    );

    // Make the interactive shell exit with a deterministic status.
    a.send_input(attached.stream_id, b"exit 7\n")?;

    let events = collect_until_quiet(&mut a, Duration::from_millis(700), Duration::from_secs(5))?;
    let exits: Vec<v1::TerminalChildExited> = events
        .iter()
        .filter_map(|event| match &event.event {
            Some(EventKind::TerminalChildExited(child))
                if child.block_id == created.block_id.to_string() =>
            {
                Some(child.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        exits.len(),
        1,
        "exactly one TerminalChildExited for the block, saw {exits:?}"
    );
    let child = &exits[0];
    assert_eq!(
        child.session_id, session_id,
        "the exit event names the session"
    );
    assert_eq!(child.block_id, created.block_id.to_string());
    assert!(child.exited, "the reaper observed an exited program");
    assert_eq!(child.exit_code, 7, "the real observed exit code");
    assert_eq!(
        child.signal, "",
        "a normal `exit 7` reports no terminating signal"
    );

    // At most once: a further bounded window must not yield a second event.
    let later = collect_until_quiet(&mut a, Duration::from_millis(700), Duration::from_secs(2))?;
    assert!(
        later.iter().all(|event| !matches!(
            &event.event,
            Some(EventKind::TerminalChildExited(child))
                if child.block_id == created.block_id.to_string()
        )),
        "terminal_child_exited must be emitted at most once"
    );
    step(
        "events-8-exit",
        &format!(
            "TerminalChildExited session={} block={} exit_code={} signal={:?} exited={}, exactly once",
            child.session_id, child.block_id, child.exit_code, child.signal, child.exited
        ),
    );
    Ok(())
}

/// §D: handshake emits `Negotiated` and a completed `resume` emits `Ready`, in
/// that order, for a subscribed observer.
#[test]
fn connection_state_events_are_emitted_for_negotiated_and_ready() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut subscriber = Client::connect(&socket, ClientConfig::new("state-subscriber"))?;
    let created = subscriber.create_session(Some("state"))?;
    let session_id = created.session.session_id.clone();
    let parsed: SessionId = session_id.parse().context("session id")?;
    server.track_child(created.program_pid);
    let _ = subscriber.resume_session(&parsed, None)?;

    // B is handshake-only; C handshakes and then resumes.
    let b = Client::connect(&socket, ClientConfig::new("state-b"))?;
    let mut c = Client::connect(&socket, ClientConfig::new("state-c"))?;
    let _ = c.resume_session(&parsed, None)?;

    let snapshot = subscriber.session_snapshot(&parsed)?;
    let b_id = snapshot
        .clients
        .iter()
        .find(|client| client.kind == "state-b")
        .map(|client| client.client_id.clone())
        .context("client B record")?;
    let c_id = snapshot
        .clients
        .iter()
        .find(|client| client.kind == "state-c")
        .map(|client| client.client_id.clone())
        .context("client C record")?;

    let events = collect_until_quiet(&mut subscriber, Duration::from_millis(700), WAIT)?;
    let states_for = |client_id: &str| -> Vec<String> {
        events
            .iter()
            .filter_map(|event| match &event.event {
                Some(EventKind::ClientConnectionStateChanged(change))
                    if change.client_id == client_id =>
                {
                    Some(change.connection_state.clone())
                }
                _ => None,
            })
            .collect()
    };
    let b_states = states_for(&b_id);
    let c_states = states_for(&c_id);
    assert_eq!(
        b_states,
        vec!["Negotiated".to_string()],
        "a handshake-only client is Negotiated and never Ready"
    );
    assert_eq!(
        c_states,
        vec!["Negotiated".to_string(), "Ready".to_string()],
        "a resumed client is Negotiated then Ready, in that order"
    );
    step(
        "events-9-states",
        &format!("B({b_id})={b_states:?} C({c_id})={c_states:?}"),
    );
    drop(b);
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
