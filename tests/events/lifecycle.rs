//! Client lifecycle scenarios: idle timeout, transport drop, long wait.

use std::time::Duration;

use anyhow::{Context, Result};
use relayable_client::{Client, ClientConfig};
use relayable_integration_tests::harness::{collect_events_over, step, TestServer};
use relayable_model::SessionId;
use relayable_protocol::v1::event::Event as EventKind;

use crate::frames::disconnect_pair;

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
