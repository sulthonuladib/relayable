//! CLIENT-API-001 — end-to-end proof for `Client::next_output`, the public
//! incremental terminal-bytes API.
//!
//! The unit tests in `crates/relayable-client` pin the frame-level semantics
//! against a scripted server. This target drives the real `relayable-server`
//! binary and the real `relayable-client` library: a shell command produces more
//! than one credit window of output, the client consumes it with repeated
//! `next_output` calls, and the run must finish with the session still active,
//! no `flow_control_overflow`, and `Ok(None)` (never `Err(Timeout)`) on a quiet
//! stream. Every reported number is real command output; nothing is faked.

use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use relayable_client::{AttachMode, Client, ClientConfig};
use relayable_integration_tests::harness::{
    collect_until_quiet, contains, nonce, step, TestServer, WAIT,
};
use relayable_model::SessionId;
use relayable_protocol::v1::event::Event as EventKind;
use relayable_protocol::v1::SessionState;

/// One `next_output` wait slice for the consumption loop.
const SLICE: Duration = Duration::from_millis(100);
/// A quiet window for the "no disconnect event" observation.
const QUIET: Duration = Duration::from_millis(500);

/// CLIENT-API-001: `next_output` carries a payload larger than the protocol
/// credit window incrementally, keeps the client acked, and timeouts as
/// `Ok(None)`.
#[test]
fn next_output_delivers_incremental_bytes_across_credit_windows() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut client = Client::connect(&socket, ClientConfig::new("client-output"))?;
    let created = client.create_session(Some("client-output"))?;
    let session_id: SessionId = created.session.session_id.parse().context("session id")?;
    server.track_child(created.program_pid);

    let attached = client.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(attached.writer_lease, "the client holds the writer lease");
    let stream = attached.stream_id;

    // Capture this client's identity so the disconnect check is about *this*
    // client, not any other.
    let snapshot = client.session_snapshot(&session_id)?;
    let client_id = snapshot
        .clients
        .iter()
        .find(|record| record.kind == "client-output")
        .map(|record| record.client_id.clone())
        .context("the client is listed live after attaching")?;

    let window = u64::from(client.limits().credit_window_bytes);
    assert!(window > 0, "the credit window is a real bound");

    let start_marker = nonce("client-output-start");
    let sentinel = nonce("client-output-done");
    // Two credit windows (>= 400 KiB for the protocol default) so exceeding the
    // window proves the client re-acked as it consumed.
    let payload_bytes = window * 2;
    // A single shell line: start marker, newline-free filler (ONLCR would
    // otherwise double every `\n`), then a unique sentinel line. `head`/`tr`
    // stream the filler so the server reads it in PTY-sized chunks.
    let command = format!(
        "printf '%s\\n' '{start_marker}'; head -c {payload_bytes} /dev/zero | tr '\\0' x; printf '\\n%s\\n' '{sentinel}'\n"
    );
    client.send_input(stream, command.as_bytes())?;

    // Consume continuously. `Ok(None)` means "no frame in this slice", not an
    // error: keep going until the sentinel plus the expected minimum have
    // arrived and one quiet slice confirms the tail is complete.
    let mut collected: Vec<u8> = Vec::new();
    let mut calls = 0usize;
    let deadline = Instant::now() + WAIT;
    while Instant::now() < deadline {
        match client.next_output(stream, SLICE)? {
            Some(chunk) => {
                calls += 1;
                collected.extend_from_slice(&chunk);
            }
            None => {
                if contains(&collected, sentinel.as_bytes())
                    && collected.len() >= payload_bytes as usize
                {
                    break;
                }
            }
        }
    }

    assert!(
        contains(&collected, start_marker.as_bytes()),
        "the start marker must have been emitted"
    );
    assert!(
        contains(&collected, sentinel.as_bytes()),
        "the sentinel must have arrived within {WAIT:?}"
    );
    assert!(
        collected.len() >= payload_bytes as usize,
        "expected at least {payload_bytes}B of output, saw {}B",
        collected.len()
    );
    assert!(
        (collected.len() as u64) > window,
        "output must span more than one credit window ({window}B), saw {}B",
        collected.len()
    );
    assert!(
        calls >= 2,
        "next_output must be exercised across multiple frames, got {calls}"
    );
    step(
        "client-output-1",
        &format!(
            "stream={stream} window={window}B payload={payload_bytes}B calls={calls} received={}B",
            collected.len()
        ),
    );

    // Assertion 2: the session survived continuous consumption.
    let after = client.session_snapshot(&session_id)?;
    assert_eq!(
        after.session.as_ref().context("session record")?.state,
        SessionState::Active as i32,
        "the session stays active after the incremental read"
    );

    // Assertion 3: no flow-control disconnect for this client.
    let events = collect_until_quiet(&mut client, QUIET, WAIT)?;
    assert!(
        events.iter().all(|event| !matches!(
            &event.event,
            Some(EventKind::ClientDisconnected(disconnected))
                if disconnected.client_id == client_id
                    && disconnected.reason == "flow_control_overflow"
        )),
        "this client must not be flow-control disconnected"
    );
    assert!(
        events.iter().all(|event| !matches!(
            &event.event,
            Some(EventKind::ClientConnectionStateChanged(change))
                if change.client_id == client_id && change.connection_state == "Disconnected"
        )),
        "this client must not be marked Disconnected"
    );
    step(
        "client-output-2",
        &format!("session active; no flow_control_overflow for {client_id}"),
    );

    // Drain any trailing shell prompt so assertion 4 is about the timeout
    // contract, not a racing prompt byte.
    let drain_deadline = Instant::now() + QUIET;
    while Instant::now() < drain_deadline {
        match client.next_output(stream, Duration::from_millis(50))? {
            Some(_) => {}
            None => break,
        }
    }

    // Assertion 4: a quiet stream returns `Ok(None)`, never `Err(Timeout)`.
    let extra = client.next_output(stream, Duration::from_millis(100))?;
    assert!(
        extra.is_none(),
        "a quiet stream must return Ok(None), got {extra:?}"
    );

    client.close_session(&session_id)?;
    Ok(())
}
