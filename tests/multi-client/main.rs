//! CORE-001 assignment E — multi-client input arbitration.
//!
//! Documents the implemented writer-lease contract: one writer per block, any
//! number of observers; an observer's `terminal.input` is rejected; the lease is
//! transferable after the holder detaches.

use std::time::Duration;

use anyhow::{bail, Context, Result};
use relayable_client::{Client, ClientConfig, ClientError};
use relayable_integration_tests::harness::{
    collect_until_quiet, contains, nonce, proc_exists, step, TestServer,
};
use relayable_model::SessionId;
use relayable_protocol::error_code::contract_str;
use relayable_protocol::v1::event::Event as EventKind;
use relayable_protocol::v1::{AttachMode, ErrorCode, SessionState};

const WAIT: Duration = Duration::from_secs(10);
/// A no-new-event window for collecting a live control-event stream.
const QUIET: Duration = Duration::from_millis(500);

fn input_not_owned(error: &ClientError) -> bool {
    error.server_error_code() == Some(ErrorCode::InputNotOwned)
}

#[test]
fn writer_arbitration_and_observer_fanout() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    // Client A creates the session and holds the writer lease.
    let mut a = Client::connect(&socket, ClientConfig::new("multi-a"))?;
    let created = a.create_session(Some("multi"))?;
    let session_id: SessionId = created.session.session_id.parse().context("session id")?;
    server.track_child(created.program_pid);
    let writer = a.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(writer.writer_lease, "A holds the writer lease");

    // Client B cannot take a second writer: exact InputNotOwned error code and
    // contract spelling.
    let mut b = Client::connect(&socket, ClientConfig::new("multi-b"))?;
    match b.attach_block(&created.block_id, AttachMode::Writer) {
        Err(ClientError::Server(error)) => {
            assert_eq!(
                error.code,
                ErrorCode::InputNotOwned as i32,
                "exact ErrorCode must be InputNotOwned"
            );
            let code = ErrorCode::try_from(error.code).context("known error code")?;
            assert_eq!(contract_str(code), "input_not_owned");
        }
        other => bail!("expected input_not_owned for a second writer, got {other:?}"),
    }

    // B can observe, and receives A's live output.
    let observer = b.attach_block(&created.block_id, AttachMode::Observer)?;
    assert!(!observer.writer_lease, "an observer has no writer lease");
    let marker = nonce("fanout");
    a.send_input(writer.stream_id, format!("echo {marker}\n").as_bytes())?;
    let seen = b.wait_for_output(observer.stream_id, marker.as_bytes(), WAIT)?;
    assert!(
        contains(&seen, marker.as_bytes()),
        "the observer must receive the writer's output"
    );

    // B's input is rejected while A holds the lease.
    match b.send_input(observer.stream_id, b"whoami\n") {
        Err(ref error) if input_not_owned(error) => {}
        other => bail!("observed terminal.input must be input_not_owned, got {other:?}"),
    }

    // A detaches; B can then acquire the writer lease.
    a.detach_block(writer.stream_id)?;
    let b_writer = b.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(b_writer.writer_lease, "B acquires the freed lease");
    b.send_input(b_writer.stream_id, b"echo writer-b\n")?;
    let seen = b.wait_for_output(b_writer.stream_id, b"writer-b", WAIT)?;
    assert!(
        contains(&seen, b"writer-b"),
        "B can write after the transfer"
    );
    step(
        "multi-client-arbitration",
        &format!(
            "second writer rejected (input_not_owned), observer fanout marked {marker}, observer input rejected, lease transferred to B"
        ),
    );

    b.close_session(&session_id)?;
    Ok(())
}

/// Requirement 5: an observer receives terminal output while the writer holds
/// the lease (CORE-002 keeps the CORE-001 fan-out contract).
#[test]
fn observer_receives_terminal_output_while_the_writer_holds_the_lease() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut writer = Client::connect(&socket, ClientConfig::new("fanout-writer"))?;
    let created = writer.create_session(Some("fanout"))?;
    let session_id: SessionId = created.session.session_id.parse().context("session id")?;
    server.track_child(created.program_pid);
    let writer_stream = writer.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(writer_stream.writer_lease, "the writer holds the lease");

    let mut observer = Client::connect(&socket, ClientConfig::new("fanout-observer"))?;
    // A successful resume subscribes the connection to live control events; the
    // observer attach registers it for terminal fan-out.
    let _ = observer.resume_session(&session_id, None)?;
    let observer_stream = observer.attach_block(&created.block_id, AttachMode::Observer)?;
    assert!(!observer_stream.writer_lease, "an observer has no lease");

    let marker = nonce("fanout");
    writer.send_input(
        writer_stream.stream_id,
        format!("echo {marker}\n").as_bytes(),
    )?;
    let seen = observer.wait_for_output(observer_stream.stream_id, marker.as_bytes(), WAIT)?;
    assert!(
        contains(&seen, marker.as_bytes()),
        "the observer must receive output while the writer holds the lease"
    );
    step(
        "multi-10",
        &format!("observer received writer output {marker} with the lease held"),
    );
    Ok(())
}

/// Requirement 5/§E: `takeover = true` transfers the single writer lease, is
/// announced to a subscribed observer, demotes the previous holder (its input is
/// rejected with `input_not_owned`, never silently dropped), and leaves a
/// non-takeover attempt failing the CORE-001 way.
#[test]
fn writer_lease_takeover_is_announced_and_demotes_the_previous_holder() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut c1 = Client::connect(&socket, ClientConfig::new("takeover-1"))?;
    let created = c1.create_session(Some("takeover"))?;
    let session_id: SessionId = created.session.session_id.parse().context("session id")?;
    server.track_child(created.program_pid);
    let w1 = c1.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(w1.writer_lease, "client 1 holds the lease");
    let c1_id = c1
        .session_snapshot(&session_id)?
        .clients
        .iter()
        .find(|client| client.kind == "takeover-1")
        .map(|client| client.client_id.clone())
        .context("client 1 record")?;

    // Client 2 is subscribed to live events and attached for terminal fan-out.
    let mut c2 = Client::connect(&socket, ClientConfig::new("takeover-2"))?;
    let _ = c2.resume_session(&session_id, None)?;
    let o2 = c2.attach_block(&created.block_id, AttachMode::Observer)?;
    assert!(!o2.writer_lease);

    // Client 3 takes the lease over.
    let mut c3 = Client::connect(&socket, ClientConfig::new("takeover-3"))?;
    let c3_id = c2
        .session_snapshot(&session_id)?
        .clients
        .iter()
        .find(|client| client.kind == "takeover-3")
        .map(|client| client.client_id.clone())
        .context("client 3 record")?;
    let w3 = c3.attach_with_takeover(&session_id, &created.block_id, AttachMode::Writer, true)?;
    assert!(w3.writer_lease, "client 3 now holds the lease");

    let events = collect_until_quiet(&mut c2, QUIET, WAIT)?;
    let change = events
        .iter()
        .find_map(|event| match &event.event {
            Some(EventKind::WriterLeaseChanged(change)) => Some(change.clone()),
            _ => None,
        })
        .context("the observer receives writer_lease_changed on takeover")?;
    assert!(change.lease_held, "the lease is held by the new writer");
    assert_eq!(change.previous_client_id, c1_id);
    assert_eq!(change.new_client_id, c3_id);
    assert_eq!(change.block_id, created.block_id.to_string());

    // The demoted holder is rejected with the exact contract code, never
    // silently dropped.
    match c1.send_input(w1.stream_id, b"echo demoted-should-not-write\n") {
        Err(ClientError::Server(error)) => assert_eq!(
            error.code,
            ErrorCode::InputNotOwned as i32,
            "the demoted writer must see input_not_owned, got {error:?}"
        ),
        other => bail!("the demoted writer's input must be input_not_owned, got {other:?}"),
    }
    // A non-takeover writer attempt still fails the CORE-001 way.
    match c1.attach_block(&created.block_id, AttachMode::Writer) {
        Err(ClientError::Server(error)) => assert_eq!(
            error.code,
            ErrorCode::InputNotOwned as i32,
            "a second non-takeover writer must be input_not_owned, got {error:?}"
        ),
        other => bail!("a second non-takeover writer must be input_not_owned, got {other:?}"),
    }

    // The new writer works and the observer still receives output.
    let marker = nonce("takeover");
    c3.send_input(w3.stream_id, format!("echo {marker}\n").as_bytes())?;
    let seen = c2.wait_for_output(o2.stream_id, marker.as_bytes(), WAIT)?;
    assert!(
        contains(&seen, marker.as_bytes()),
        "the observer must still receive the new writer's output"
    );
    step(
        "multi-11",
        &format!(
            "takeover {c1_id} -> {c3_id}; demoted input input_not_owned; observer saw {marker}"
        ),
    );
    Ok(())
}

/// Requirement 5/§D: the writer disconnecting releases the lease, the release
/// is announced (`lease_held == false`, empty `new_client_id`), the lease is
/// acquirable again, and the session/child survived.
#[test]
fn lease_release_on_disconnect_is_announced() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut writer = Client::connect(&socket, ClientConfig::new("release-writer"))?;
    let created = writer.create_session(Some("release"))?;
    let session_id: SessionId = created.session.session_id.parse().context("session id")?;
    server.track_child(created.program_pid);
    let w = writer.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(w.writer_lease);
    let writer_id = writer
        .session_snapshot(&session_id)?
        .clients
        .iter()
        .find(|client| client.kind == "release-writer")
        .map(|client| client.client_id.clone())
        .context("writer record")?;

    // The observer only needs the live event subscription for this proof.
    let mut observer = Client::connect(&socket, ClientConfig::new("release-observer"))?;
    let _ = observer.resume_session(&session_id, None)?;

    drop(writer);

    let events = collect_until_quiet(&mut observer, QUIET, WAIT)?;
    let change = events
        .iter()
        .find_map(|event| match &event.event {
            Some(EventKind::WriterLeaseChanged(change)) => Some(change.clone()),
            _ => None,
        })
        .context("the observer receives writer_lease_changed on release")?;
    assert!(
        !change.lease_held,
        "a release with no new holder is not held"
    );
    assert!(
        change.new_client_id.is_empty(),
        "a release with no new holder names nobody, got {:?}",
        change.new_client_id
    );
    assert_eq!(change.previous_client_id, writer_id);

    // The lease is acquirable again and the session/child survived.
    let reacquired = observer.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(reacquired.writer_lease, "the freed lease is re-acquirable");
    assert!(
        proc_exists(created.program_pid),
        "the child survives the writer's disconnect"
    );
    let view = observer.session_snapshot(&session_id)?;
    assert_eq!(
        view.session.as_ref().context("session")?.state,
        SessionState::Active as i32,
        "the session is still active"
    );
    observer.send_input(reacquired.stream_id, b"echo release-ok\n")?;
    let seen = observer.wait_for_output(reacquired.stream_id, b"release-ok", WAIT)?;
    assert!(contains(&seen, b"release-ok"), "the new writer can write");
    step(
        "multi-12",
        &format!(
            "writer {writer_id} disconnected: release announced, lease re-acquired, child {} alive",
            created.program_pid
        ),
    );
    Ok(())
}
