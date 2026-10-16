//! Connection-state (`Negotiated`/`Ready`) event scenario.

use std::time::Duration;

use anyhow::{Context, Result};
use relayable_client::{Client, ClientConfig};
use relayable_integration_tests::harness::{collect_until_quiet, step, TestServer, WAIT};
use relayable_model::SessionId;
use relayable_protocol::v1::event::Event as EventKind;

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
