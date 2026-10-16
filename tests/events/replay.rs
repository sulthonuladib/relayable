//! `session.resume` replay and cursor scenarios.

use std::collections::BTreeSet;

use anyhow::{Context, Result};
use relayable_client::{Client, ClientConfig};
use relayable_integration_tests::harness::{
    assert_event_sequences_contiguous, collect_until_quiet, step, TestServer, WAIT,
};
use relayable_model::SessionId;
use relayable_protocol::v1::AttachMode;

use crate::frames::QUIET;

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
