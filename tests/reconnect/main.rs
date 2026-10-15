//! CORE-001 assignment E — focused reconnect test driven by `relayable-client`
//! only (no CLI). Stricter than the lifecycle test: it proves the re-attach
//! snapshot advances past the pre-disconnect stream position and that the
//! dropped client's record is removed server-side, and that a second session
//! does not disturb the first.

use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use relayable_client::{Client, ClientConfig, ClientError};
use relayable_integration_tests::harness::{
    contains, nonce, poll_until, proc_exists, step, TestServer,
};
use relayable_model::SessionId;
use relayable_protocol::v1::{AttachMode, ErrorCode};

const WAIT: Duration = Duration::from_secs(10);

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
