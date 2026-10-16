//! CLIENT-API-001 — end-to-end proof for `Client::next_output`, the public
//! incremental terminal-bytes API.
//!
//! The unit tests in `crates/relayable-client` pin the frame-level semantics
//! against a scripted server. This target drives the real `relayable-server`
//! binary and the real `relayable-client` library: a shell command produces more
//! than one credit window of output, the client consumes it with repeated
//! `next_output` calls, and the run must finish with the session still active,
//! the writer's socket still usable, no `flow_control_overflow`, and `Ok(None)`
//! (never `Err(Timeout)`) on a quiet stream. Every reported number is real
//! command output; nothing is faked.

use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use relayable_client::{AttachMode, Client, ClientConfig, ClientError};
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

/// `(start, len)` of the longest contiguous run of `byte` in `bytes`.
///
/// The shell echoes the command line (which contains the marker nonces and a
/// single literal `x`) before executing it, so a whole-buffer equality check
/// would not work; the longest `x` run isolates the newline-free filler the
/// command actually produces and makes duplication visible.
fn longest_run(bytes: &[u8], byte: u8) -> (usize, usize) {
    let mut best = (0usize, 0usize);
    let mut current_start = 0usize;
    let mut current_len = 0usize;
    for (index, &value) in bytes.iter().enumerate() {
        if value == byte {
            if current_len == 0 {
                current_start = index;
            }
            current_len += 1;
            if current_len > best.1 {
                best = (current_start, current_len);
            }
        } else {
            current_len = 0;
        }
    }
    best
}

/// CLIENT-API-001: `next_output` carries a payload larger than the protocol
/// credit window incrementally, keeps the client acked, and timeouts as
/// `Ok(None)`.
#[test]
fn next_output_delivers_incremental_bytes_across_credit_windows() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    // --- Writer: creates the session and consumes the terminal output.
    let mut client = Client::connect(&socket, ClientConfig::new("client-output"))?;
    let created = client.create_session(Some("client-output"))?;
    let session_id: SessionId = created.session.session_id.parse().context("session id")?;
    server.track_child(created.program_pid);

    // The writer's own identity, captured before the observer subscribes.
    let writer_snapshot = client.session_snapshot(&session_id)?;
    let writer_id = writer_snapshot
        .clients
        .iter()
        .find(|record| record.kind == "client-output")
        .map(|record| record.client_id.clone())
        .context("the writer is listed live after creating the session")?;

    // --- Observer: subscribes to the session event stream *before* the writer
    // attaches. Event-bus subscription is what `session.resume` does
    // (`crates/relayable-server/src/registry.rs:401`), and the bus is
    // server-wide, so this connection receives later events for every client.
    // The observer deliberately does not attach to the block yet: an early
    // observer attach would register a terminal subscriber that never acks and
    // would itself overflow on the 512 KiB transfer.
    let mut observer = Client::connect(&socket, ClientConfig::new("client-output-observer"))?;
    let _ = observer.resume_session(&session_id, None)?;

    // The writer takes the lease; this broadcasts
    // `writer_lease_changed{new_client_id: writer_id}` to the observer and is
    // the positive control that the observer's stream covers the writer.
    let attached = client.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(attached.writer_lease, "the writer holds the writer lease");
    let stream = attached.stream_id;

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
                    && longest_run(&collected, b'x').1 >= payload_bytes as usize
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
    // The filler must appear exactly once, in full: a duplicated or truncated
    // payload fails this rather than merely shrinking the total.
    let (filler_start, filler_len) = longest_run(&collected, b'x');
    assert_eq!(
        filler_len, payload_bytes as usize,
        "the filler run must be exactly the scripted {payload_bytes} bytes"
    );
    // The sentinel must follow the real filler output, not merely appear in the
    // echoed command line (which also contains the sentinel literal).
    assert!(
        contains(&collected[filler_start + filler_len..], sentinel.as_bytes()),
        "the sentinel must follow the {payload_bytes}-byte filler output"
    );
    assert!(
        (collected.len() as u64) > window,
        "output must span more than one credit window ({window}B), saw {}B",
        collected.len()
    );
    // The only bytes beyond the payload are the observed echo, marker, prompt,
    // and ONLCR overhead. The bound is far below 2x payload so a duplicated
    // stream cannot pass.
    let overhead_bound = command.len() + start_marker.len() + sentinel.len() + 512;
    assert!(
        collected.len() >= payload_bytes as usize
            && collected.len() <= payload_bytes as usize + overhead_bound,
        "expected payload {payload_bytes}B plus <= {overhead_bound}B overhead, saw {}B",
        collected.len()
    );
    assert!(
        calls >= 2,
        "next_output must be exercised across multiple frames, got {calls}"
    );
    step(
        "client-output-1",
        &format!(
            "stream={stream} window={window}B payload={payload_bytes}B calls={calls} received={}B filler={filler_len}B",
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

    // Drain any trailing shell prompt so the timeout check is about the
    // contract, not a racing prompt byte.
    let drain_deadline = Instant::now() + QUIET;
    while Instant::now() < drain_deadline {
        match client.next_output(stream, Duration::from_millis(50))? {
            Some(_) => {}
            None => break,
        }
    }

    // Assertion 3 (primary overflow detector): an overflow calls
    // `close_now(FlowControlOverflow)` which shuts the writer's socket down
    // (`crates/relayable-server/src/session.rs:229-236`,
    // `crates/relayable-server/src/connection.rs:89`). A shut-down socket makes
    // the next call fail with `Disconnected`/transport error, so this and the
    // following control RPC fail on a real overflow. The quiet-stream
    // `Ok(None)` is only meaningful because the socket demonstrably still works.
    let quiet = client.next_output(stream, Duration::from_millis(100));
    match &quiet {
        Ok(None) => {}
        Ok(Some(bytes)) => panic!(
            "a quiet stream returned {} unexpected bytes; expected Ok(None)",
            bytes.len()
        ),
        Err(ClientError::Disconnected) => {
            panic!("the writer was disconnected: an overflow would have closed its socket")
        }
        Err(err) => panic!("the writer connection must stay usable, got {err:?}"),
    }
    let following = client.session_snapshot(&session_id)?;
    assert_eq!(
        following
            .session
            .as_ref()
            .context("session record after the quiet check")?
            .state,
        SessionState::Active as i32,
        "a control RPC after the quiet check must still succeed"
    );

    // The observer attaches as an observer only now, after the transfer, so it
    // never receives the bulk output. It must not take the writer lease.
    let observer_attached = observer.attach_block(&created.block_id, AttachMode::Observer)?;
    assert!(
        !observer_attached.writer_lease,
        "the observer must not hold the writer lease"
    );

    // Assertion 4: an independent subscriber sees the writer's events but no
    // `flow_control_overflow` disconnect for it. Because `client_disconnected`
    // broadcasts the ordered pair to every remaining bus subscriber
    // (`crates/relayable-server/src/events.rs:180-238`; the closing connection is
    // unsubscribed first at `crates/relayable-server/src/connection.rs:314`), a
    // real writer overflow would appear here. The positive control below proves
    // this observer's stream actually carries the writer's identity, so the
    // absence check cannot pass vacuously.
    let events = collect_until_quiet(&mut observer, QUIET, WAIT)?;
    let saw_writer = events.iter().any(|event| {
        matches!(
            &event.event,
            Some(EventKind::WriterLeaseChanged(lease))
                if lease.new_client_id == writer_id && lease.lease_held
        )
    });
    assert!(
        saw_writer,
        "positive control: the observer must have seen the writer {writer_id}'s \
         writer_lease_changed; without it the no-overflow check would be vacuous"
    );
    assert!(
        events.iter().all(|event| !matches!(
            &event.event,
            Some(EventKind::ClientDisconnected(disconnected))
                if disconnected.client_id == writer_id
                    && disconnected.reason == "flow_control_overflow"
        )),
        "the observer must not see a flow_control_overflow disconnect for the writer"
    );
    assert!(
        events.iter().all(|event| !matches!(
            &event.event,
            Some(EventKind::ClientConnectionStateChanged(change))
                if change.client_id == writer_id && change.connection_state == "Disconnected"
        )),
        "the observer must not see the writer marked Disconnected"
    );
    step(
        "client-output-2",
        &format!(
            "session active, writer socket usable; observer saw writer {writer_id} lease event and no overflow ({} events)",
            events.len()
        ),
    );

    client.close_session(&session_id)?;
    Ok(())
}
