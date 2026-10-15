//! CORE-001 assignment E — focused reconnect test driven by `relayable-client`
//! only (no CLI). Stricter than the lifecycle test: it proves the re-attach
//! snapshot advances past the pre-disconnect stream position and that the
//! dropped client's record is removed server-side, and that a second session
//! does not disturb the first.

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use relayable_client::{Client, ClientConfig, ClientError};
use relayable_integration_tests::harness::{
    assert_event_sequences_contiguous, collect_until_quiet, contains, nonce, poll_until,
    proc_exists, step, TestServer,
};
use relayable_model::SessionId;
use relayable_protocol::v1::{AttachMode, ErrorCode, SessionState};

const WAIT: Duration = Duration::from_secs(10);
/// A no-new-event window for collecting a live control-event stream.
const QUIET: Duration = Duration::from_millis(500);

fn attach_writer(
    client: &mut Client,
    block_id: &relayable_model::BlockId,
) -> Result<relayable_client::AttachOutcome> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match client.attach_block(block_id, AttachMode::Writer) {
            Ok(outcome) => return Ok(outcome),
            Err(ClientError::Server(ref error))
                if error.code == ErrorCode::InputNotOwned as i32 && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(error) => return Err(error.into()),
        }
    }
}

#[test]
fn reconnect_preserves_identity_and_advances_the_stream() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    // Client A creates the session and takes the writer lease.
    let mut a = Client::connect(&socket, ClientConfig::new("reconnect"))?;
    let created = a.create_session(Some("reconnect"))?;
    let session_id: SessionId = created.session.session_id.parse().context("session id")?;
    server.track_child(created.program_pid);

    let before = a.session_snapshot(&session_id)?;
    let client_a_id = before
        .clients
        .iter()
        .find(|client| client.kind == "reconnect")
        .map(|client| client.client_id.clone())
        .context("client A record in the snapshot")?;

    let attached = a.attach_block(&created.block_id, AttachMode::Writer)?;
    let stream = attached.stream_id;
    assert!(attached.writer_lease, "A holds the writer lease");
    assert_eq!(attached.snapshot.cols, 80);
    assert_eq!(attached.snapshot.rows, 24);
    let first_snapshot_seq = attached.snapshot.snapshot_seq;

    // Produce output, then drop A abruptly (socket close).
    let marker = nonce("reconnect");
    a.send_input(stream, format!("echo {marker}\n").as_bytes())?;
    let echoed = a.wait_for_output(stream, marker.as_bytes(), WAIT)?;
    assert!(
        contains(&echoed, marker.as_bytes()),
        "marker echoed before drop"
    );
    drop(a);

    // A fresh client B observes that A's record is gone and the child survives.
    let mut b = Client::connect(&socket, ClientConfig::new("reconnect-b"))?;
    assert!(
        poll_until(Duration::from_secs(5), Duration::from_millis(30), || {
            match b.session_snapshot(&session_id) {
                Ok(snapshot) => snapshot
                    .clients
                    .iter()
                    .all(|client| client.client_id != client_a_id),
                Err(_) => false,
            }
        }),
        "the dropped client record must be removed from the snapshot"
    );
    assert!(
        proc_exists(created.program_pid),
        "child survives the client disconnect"
    );

    // Re-attach: the snapshot must carry the retained output tail and a stream
    // sequence at or beyond the pre-disconnect position.
    let reattached = attach_writer(&mut b, &created.block_id)?;
    assert_eq!(reattached.stream_id, 1, "stream ids are per connection");
    assert!(reattached.writer_lease, "B acquires the freed writer lease");
    assert!(
        reattached.snapshot.snapshot_seq >= first_snapshot_seq,
        "reattach snapshot_seq {} must be >= pre-disconnect {}",
        reattached.snapshot.snapshot_seq,
        first_snapshot_seq
    );
    assert!(
        !reattached.snapshot.screen_bytes.is_empty(),
        "reattach snapshot must carry the retained output tail"
    );
    assert!(
        contains(&reattached.snapshot.screen_bytes, marker.as_bytes()),
        "the retained tail must contain the echoed marker"
    );

    // Identity is unchanged.
    let after = b.session_snapshot(&session_id)?;
    assert_eq!(
        after.session.as_ref().context("session")?.session_id,
        created.session.session_id
    );
    let block = after.blocks.first().context("block")?;
    assert_eq!(block.block_id, created.block_id.to_string());
    assert_eq!(block.pty_process_id, created.pty_process_id.to_string());
    assert_eq!(block.program_pid, created.program_pid);
    step(
        "reconnect-identity",
        &format!(
            "session={} block={} pty={} pid={} snapshot_seq {}->{} tail={} bytes",
            created.session.session_id,
            created.block_id,
            created.pty_process_id,
            created.program_pid,
            first_snapshot_seq,
            reattached.snapshot.snapshot_seq,
            reattached.snapshot.screen_bytes.len()
        ),
    );

    // A second session must not disturb the first.
    let second = b.create_session(Some("second"))?;
    server.track_child(second.program_pid);
    assert_ne!(
        second.program_pid, created.program_pid,
        "the two sessions have distinct children"
    );
    assert_ne!(
        second.session.session_id, created.session.session_id,
        "the two sessions have distinct ids"
    );
    assert!(proc_exists(created.program_pid), "first child still alive");
    assert!(proc_exists(second.program_pid), "second child alive");
    let first_after_second = b.session_snapshot(&session_id)?;
    assert_eq!(
        first_after_second
            .blocks
            .first()
            .context("first block")?
            .program_pid,
        created.program_pid,
        "the first session is untouched by the second"
    );
    step(
        "coexistence",
        &format!(
            "first pid={} second pid={} both alive",
            created.program_pid, second.program_pid
        ),
    );

    // Clean up both sessions and prove all children are reaped.
    b.close_session(&session_id)?;
    let second_id: SessionId = second.session.session_id.parse().context("second id")?;
    b.close_session(&second_id)?;
    assert!(
        poll_until(Duration::from_secs(3), Duration::from_millis(20), || {
            !proc_exists(created.program_pid) && !proc_exists(second.program_pid)
        }),
        "closing both sessions reaps both children"
    );
    Ok(())
}

/// Requirement 4: the retained event log can reconstruct exactly the events a
/// dropped connection missed, with no gap and no duplicate, and the child PTY
/// survives the reconnect so a new writer can continue.
#[test]
fn snapshot_at_s_plus_events_has_no_gap_no_duplicate_and_the_child_pid_survives() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    // A creates the session and subscribes at the snapshot cursor S.
    let mut a = Client::connect(&socket, ClientConfig::new("reconnect-a"))?;
    let created = a.create_session(Some("reconnect-events"))?;
    let session_id: SessionId = created.session.session_id.parse().context("session id")?;
    server.track_child(created.program_pid);

    let resumed = a.resume_session(&session_id, None)?;
    assert!(resumed.snapshot_required, "no cursor is a snapshot");
    let s = resumed.snapshot_sequence;
    assert_eq!(resumed.current_sequence, s);

    // Record A's client id (for the removal wait) before holding the lease.
    let before = a.session_snapshot(&session_id)?;
    let a_id = before
        .clients
        .iter()
        .find(|client| client.kind == "reconnect-a")
        .map(|client| client.client_id.clone())
        .context("client A record")?;

    // A holds the writer lease and produces live output.
    let attached = a.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(attached.writer_lease, "A holds the writer lease");
    let marker = nonce("reconnect-events");
    a.send_input(attached.stream_id, format!("echo {marker}\n").as_bytes())?;
    let echoed = a.wait_for_output(attached.stream_id, marker.as_bytes(), WAIT)?;
    assert!(contains(&echoed, marker.as_bytes()));

    // A's live event stream from S+1.
    let a_events = collect_until_quiet(&mut a, QUIET, WAIT)?;
    assert!(!a_events.is_empty(), "A must observe its live events");
    assert_eq!(a_events[0].sequence, s + 1);
    assert_event_sequences_contiguous(&a_events);

    // Drop A entirely. The child must survive and A's disconnect events must
    // be durable before B resumes.
    drop(a);
    let mut probe = Client::connect(&socket, ClientConfig::new("reconnect-probe"))?;
    assert!(
        poll_until(Duration::from_secs(5), Duration::from_millis(30), || {
            match probe.session_snapshot(&session_id) {
                Ok(snapshot) => snapshot
                    .clients
                    .iter()
                    .all(|client| client.client_id != a_id),
                Err(_) => false,
            }
        }),
        "A's client record must be removed after the disconnect pair"
    );
    assert!(
        proc_exists(created.program_pid),
        "the same child pid must survive A's disconnect"
    );

    // B reconnects and replays the retained suffix from S.
    let mut b = Client::connect(&socket, ClientConfig::new("reconnect-b"))?;
    let replay = b.resume_session(&session_id, Some(s))?;
    assert!(!replay.snapshot_required, "S must be replayable");
    assert!(replay.snapshot.is_none(), "a replay carries no snapshot");
    assert_eq!(replay.snapshot_sequence, s);
    let replayed = &replay.events;
    assert_eq!(replayed.first().map(|event| event.sequence), Some(s + 1));
    assert_eq!(
        replayed.last().map(|event| event.sequence),
        Some(replay.current_sequence)
    );
    assert_event_sequences_contiguous(replayed);
    let unique: BTreeSet<u64> = replayed.iter().map(|event| event.sequence).collect();
    assert_eq!(unique.len(), replayed.len(), "no duplicate sequences");
    // A's live stream is exactly the prefix of B's replay: the union is
    // contiguous S+1.. with no gap and no duplicate.
    assert!(replayed.len() >= a_events.len());
    assert_eq!(&replayed[..a_events.len()], a_events.as_slice());

    // The session is still ACTIVE and the same pid is alive.
    let view = b.session_snapshot(&session_id)?;
    assert_eq!(
        view.session.as_ref().context("session")?.state,
        SessionState::Active as i32,
        "the session survives the reconnect"
    );
    assert!(proc_exists(created.program_pid));

    // B re-acquires the freed writer lease and still gets terminal output.
    let writer_b = b.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(writer_b.writer_lease, "B re-acquires the freed lease");
    let marker_b = nonce("reconnect-events-b");
    b.send_input(writer_b.stream_id, format!("echo {marker_b}\n").as_bytes())?;
    let seen = b.wait_for_output(writer_b.stream_id, marker_b.as_bytes(), WAIT)?;
    assert!(contains(&seen, marker_b.as_bytes()));
    assert!(proc_exists(created.program_pid));
    step(
        "reconnect-8",
        &format!(
            "S={s} A_live={} replayed={} pid={} alive and re-acquired, marker {marker_b}",
            a_events.len(),
            replayed.len(),
            created.program_pid
        ),
    );

    b.close_session(&session_id)?;
    assert!(
        poll_until(Duration::from_secs(3), Duration::from_millis(20), || {
            !proc_exists(created.program_pid)
        }),
        "closing the session reaps the child"
    );
    Ok(())
}

/// Requirement 4: resuming with the cursor accumulated by the previous
/// connection takes the REPLAY path (`snapshot_required == false`) and the
/// continuation is contiguous with what that connection observed.
#[test]
fn resume_without_a_snapshot_continues_the_event_stream() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut a = Client::connect(&socket, ClientConfig::new("continuation-a"))?;
    let created = a.create_session(Some("continuation"))?;
    let session_id: SessionId = created.session.session_id.parse().context("session id")?;
    server.track_child(created.program_pid);

    let first = a.resume_session(&session_id, None)?;
    assert!(first.snapshot_required);
    let s = first.snapshot_sequence;

    let view = a.session_snapshot(&session_id)?;
    let a_id = view
        .clients
        .iter()
        .find(|client| client.kind == "continuation-a")
        .map(|client| client.client_id.clone())
        .context("client A record")?;

    // Generate live events through a separate client.
    let mut c = Client::connect(&socket, ClientConfig::new("continuation-c"))?;
    let writer = c.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(writer.writer_lease);
    drop(c);

    let observed = collect_until_quiet(&mut a, QUIET, WAIT)?;
    assert!(!observed.is_empty());
    assert_event_sequences_contiguous(&observed);
    let cursor = observed.last().context("an observed event")?.sequence;
    assert!(cursor > s, "the stream advanced past the snapshot cursor");

    // Drop A and wait for its disconnect events to be durable.
    drop(a);
    let mut probe = Client::connect(&socket, ClientConfig::new("continuation-probe"))?;
    assert!(
        poll_until(Duration::from_secs(5), Duration::from_millis(30), || {
            match probe.session_snapshot(&session_id) {
                Ok(snapshot) => snapshot
                    .clients
                    .iter()
                    .all(|client| client.client_id != a_id),
                Err(_) => false,
            }
        }),
        "A's client record must be removed"
    );

    // B resumes from A's cursor: this must be a replay, not a snapshot fallback.
    let mut b = Client::connect(&socket, ClientConfig::new("continuation-b"))?;
    let resumed = b.resume_session(&session_id, Some(cursor))?;
    assert!(
        !resumed.snapshot_required,
        "the accumulated cursor must replay, not fall back to a snapshot"
    );
    assert!(resumed.snapshot.is_none());
    assert_eq!(resumed.snapshot_sequence, cursor);
    assert_eq!(
        resumed.events.first().map(|event| event.sequence),
        Some(cursor + 1),
        "the continuation starts immediately after the cursor"
    );
    assert_eq!(
        resumed.events.last().map(|event| event.sequence),
        Some(resumed.current_sequence)
    );
    assert_event_sequences_contiguous(&resumed.events);

    // B's live stream then continues with no gap after the replay.
    let live = collect_until_quiet(&mut b, QUIET, WAIT)?;
    assert_eq!(
        live.first().map(|event| event.sequence),
        Some(resumed.current_sequence + 1)
    );
    step(
        "reconnect-9",
        &format!(
            "cursor={cursor} replayed={} current={} live from {}",
            resumed.events.len(),
            resumed.current_sequence,
            resumed.current_sequence + 1
        ),
    );

    b.close_session(&session_id)?;
    Ok(())
}
