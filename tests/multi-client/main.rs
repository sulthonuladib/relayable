//! CORE-001 assignment E — multi-client input arbitration.
//!
//! Documents the implemented writer-lease contract: one writer per block, any
//! number of observers; an observer's `terminal.input` is rejected; the lease is
//! transferable after the holder detaches.

use std::time::Duration;

use anyhow::{bail, Context, Result};
use relayable_client::{Client, ClientConfig, ClientError};
use relayable_integration_tests::harness::{contains, nonce, step, TestServer};
use relayable_model::SessionId;
use relayable_protocol::error_code::contract_str;
use relayable_protocol::v1::{AttachMode, ErrorCode};

const WAIT: Duration = Duration::from_secs(10);

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
