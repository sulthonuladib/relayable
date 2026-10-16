//! QA-FLOW-001 — the slow-client output-overflow / backpressure path.
//!
//! CORE-001 implemented the bounded per-stream credit window but left the
//! *slow-client* path unasserted (`protocol/compatibility/core-001-method-set.md`
//! §"Known tested gap"; `docs/agent-loop/TASKS.md` QA-FLOW-001). This test
//! proves the behavior from *outside* `crates/**` using only public APIs: the
//! real `relayable-server` binary, the real `relayable` CLI, and the real
//! `relayable-client` library.
//!
//! Two tests share this file:
//!
//! * `slow_client_output_overflow_detaches_with_flow_control_overflow` drives
//!   the client API and asserts the observed disconnect events and survivor
//!   state.
//! * `slow_client_receives_at_most_credit_window_bytes_before_overflow` uses a
//!   raw `FrameConnection` victim that never sends a `flow_control` ack and
//!   counts the `terminal_data` bytes actually delivered, so the credit window
//!   is a load-bearing bound rather than an inference.
//!
//! # Implemented semantics asserted here (verified in the source)
//!
//! * The PTY reader never pauses on a slow subscriber: it reads chunks of at
//!   most 8192 bytes and fans each one out (`crates/relayable-server/src/session.rs`,
//!   `reader_loop`/`push_output`).
//! * Each subscriber starts with `credit_window_bytes` of output credit
//!   (`server::session::Subscriber::new`); every delivered chunk consumes its
//!   length, and a `FlowControl` ack from the client is the only thing that
//!   refills it. The client only acks inside `wait_for_output`, so a client that
//!   never reads never acks.
//! * When the credit window is exhausted — or the per-connection bounded
//!   outbound queue (`OUTBOUND_QUEUE_MESSAGES = 8192`) is full — the server
//!   calls `close_now(DisconnectReason::FlowControlOverflow)`
//!   (`Subscriber::deliver`/`overflow`). The connection is closed and the server
//!   emits, in order and with adjacent sequences,
//!   `client_connection_state_changed{connection_state: "Disconnected"}` then
//!   `client_disconnected{reason: "flow_control_overflow"}` to subscribers
//!   (`registry::ServerShared::client_disconnected`).
//! * The session, block, PTY, and server-owned child process all survive; the
//!   reader lease is released (`ConnectionCtx::release_streams`).
//! * There is **no** pause, coalescing, or separate detach protocol message:
//!   "detach" here is exactly the connection close plus the reason event.
//!
//! The advertised window is `relayable_protocol::limits::DEFAULT_LIMITS.credit_window_bytes`
//! (`262_144`); the server never overrides it via environment
//! (`crates/relayable-server/src/config.rs` uses `..DEFAULT_LIMITS`). The test
//! derives the workload from the server's advertised limits rather than a bare
//! magic number.
//!
//! # What is *not* asserted, and why
//!
//! Unbounded memory growth is bounded **by construction** (per-stream credit
//! window + 8192-message outbound queue + `output_tail_bytes = 32_768` snapshot
//! tail), and those bounds are the asserted behavior's source. This test does
//! **not** assert process RSS or any absolute memory figure: `full.md` §12 says
//! regression thresholds must follow a measured, stable baseline, and no such
//! baseline exists in this repository. The honest observable proxy for "bounded"
//! is therefore a bounded-time detach (no unbounded buffering that would let the
//! client stay connected forever) plus continued server functionality and a
//! surviving child. The raw-victim test separately asserts the *delivered-byte*
//! bound (window + one PTY chunk), which is the observable consequence of the
//! credit accounting without measuring memory. No measurement is faked or
//! estimated.

use std::io::ErrorKind;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use relayable_client::{AttachMode, Client, ClientConfig, ClientError};
use relayable_integration_tests::harness::{
    assert_event_sequences_contiguous, cli_json, nonce, proc_exists, TestServer, WAIT,
};
use relayable_model::{BlockId, SessionId};
use relayable_protocol::codec::CodecError;
use relayable_protocol::limits::DEFAULT_LIMITS;
use relayable_protocol::methods;
use relayable_protocol::transport::{FrameConnection, TransportError};
use relayable_protocol::v1::event::Event as EventKind;
use relayable_protocol::v1::{
    envelope::Body, request, response, AttachMode as WireAttachMode, BlockAttachRequest, Envelope,
    Handshake, Limits, Request, SessionState, TerminalInput,
};
use relayable_protocol::PROTOCOL_VERSION;

/// One short collect slice: long enough to fan in queued frames, short enough to
/// stay responsive to the bounded overall deadline.
const SLICE: Duration = Duration::from_millis(50);

/// The server's PTY read-chunk bound: `reader_loop` reads into a buffer of this
/// size (`crates/relayable-server/src/session.rs` `reader_loop`), so one
/// delivered `terminal_data` chunk carries at most this many bytes.
const MAX_CHUNK_U32: u32 = 8_192;
/// [`MAX_CHUNK_U32`] as a byte count for the raw-victim arithmetic.
const MAX_CHUNK: u64 = MAX_CHUNK_U32 as u64;

/// Find the ordered `client_connection_state_changed{Disconnected}` then
/// `client_disconnected{reason}` pair for `client_id`, returning
/// `(first_sequence, reason)`.
///
/// The two events are appended under one bus-lock acquisition
/// (`EventBus::append_pair`), so they occupy adjacent sequences and are fanned
/// out in order; this asserts that adjacency on the observer's live stream.
fn disconnect_pair_for(
    events: &[relayable_client::Event],
    client_id: &str,
) -> Option<(u64, String)> {
    events.windows(2).find_map(|pair| {
        let state = match &pair[0].event {
            Some(EventKind::ClientConnectionStateChanged(state)) => state,
            _ => return None,
        };
        let disconnected = match &pair[1].event {
            Some(EventKind::ClientDisconnected(disconnected)) => disconnected,
            _ => return None,
        };
        if state.connection_state == "Disconnected"
            && state.client_id == client_id
            && disconnected.client_id == client_id
        {
            Some((pair[0].sequence, disconnected.reason.clone()))
        } else {
            None
        }
    })
}

/// QA-FLOW-001: a subscribed observer watches a victim client overflow its
/// bounded output queue and receive `flow_control_overflow`, while the session,
/// PTY, and child survive and the server keeps serving new clients.
#[test]
fn slow_client_output_overflow_detaches_with_flow_control_overflow() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();
    let socket_text = socket.to_str().context("socket path is UTF-8")?.to_string();

    // --- 1. Create a session through the frozen CLI.
    let created = cli_json(
        &["new", "--json"],
        &[("RELAYABLE_SERVER", socket_text.as_str())],
    );
    let session_id_text = created["session_id"]
        .as_str()
        .context("new.session_id")?
        .to_string();
    let session_id: SessionId = session_id_text.parse().context("parse session id")?;
    let block_id: BlockId = created["block_id"]
        .as_str()
        .context("new.block_id")?
        .parse()
        .context("parse block id")?;
    let program_pid = u32::try_from(created["program_pid"].as_u64().context("new.program_pid")?)
        .context("program_pid fits u32")?;
    assert!(program_pid > 0, "program_pid must be a real pid");
    server.track_child(program_pid);

    // --- 2. Observer: subscribe to the live control-event stream.
    let mut observer = Client::connect(&socket, ClientConfig::new("flow-observer"))?;
    let _ = observer.resume_session(&session_id, None)?;

    // --- 3. Victim: connect, take the writer lease, then derive the workload
    //        from the server-advertised credit window.
    let mut victim = Client::connect(&socket, ClientConfig::new("flow-victim"))?;
    let window = victim.limits().credit_window_bytes;
    assert_eq!(
        window, DEFAULT_LIMITS.credit_window_bytes,
        "the foreground server must advertise the protocol default credit window"
    );
    assert!(window > 0, "the credit window is a real bound");
    // One and a half windows of output is clearly more than the credit window.
    // The workload is deliberately newline-free (`tr '\0' 'x'`): the PTY expands
    // each output `\n` to `\r\n` (ONLCR), so a line-oriented command would make
    // the server observe ~1.5x the requested bytes and blow past every bound
    // below. The server consumes *more than* `window - max_chunk` bytes into the
    // victim's window before the tripping chunk: `try_consume` fails only when
    // the remaining credit is smaller than the incoming chunk, and one PTY read
    // yields at most `max_chunk` bytes. So keeping `payload < 2 * window` keeps
    // the residue that can still reach a *fresh* subscriber afterwards under one
    // full window. That is what makes ASSERT D deterministic: the fresh client's
    // `block.attach` / `terminal.input` reads do not acknowledge, so a residue
    // larger than its window would overflow the fresh client as well.
    let payload = window + window / 2;
    // The PTY read buffer bounds each chunk, so the consumed prefix is at least
    // `window - max_chunk`; the residue is `payload - (window - max_chunk)` at
    // worst. Require it to fit in one fresh window.
    let max_chunk = MAX_CHUNK_U32;
    let residue_bound = payload.saturating_sub(window.saturating_sub(max_chunk));
    assert!(
        residue_bound <= window,
        "post-overflow residue ({residue_bound}B) must fit a fresh subscriber's window ({window}B)"
    );

    // Capture identities before the overflow: a snapshot lists live clients.
    let snapshot = observer.session_snapshot(&session_id)?;
    let victim_id = snapshot
        .clients
        .iter()
        .find(|client| client.kind == "flow-victim")
        .map(|client| client.client_id.clone())
        .context("the victim is listed live at handshake time")?;
    let observer_id = snapshot
        .clients
        .iter()
        .find(|client| client.kind == "flow-observer")
        .map(|client| client.client_id.clone())
        .context("the observer is listed live")?;
    assert_ne!(victim_id, observer_id, "distinct client identities");

    let attached = victim.attach_block(&block_id, AttachMode::Writer)?;
    assert!(attached.writer_lease, "the victim holds the writer lease");
    let victim_stream = attached.stream_id;

    // Send a command that produces more than the credit window. The victim then
    // performs no further reads, so it never sends a `flow_control` ack.
    let command = format!("head -c {payload} /dev/zero | tr '\\0' 'x'\n");
    // The overflow may race the `terminal.input` response itself, so the call is
    // allowed to succeed or to observe the connection already closed.
    match victim.send_input(victim_stream, command.as_bytes()) {
        Ok(_) | Err(ClientError::Disconnected) | Err(ClientError::Transport(_)) => {}
        Err(other) => panic!("unexpected send_input error on the slow-client path: {other:?}"),
    }
    eprintln!(
        "[QA-FLOW-001] flow-control-1: victim client_id={victim_id} stream={victim_stream} work={payload}B from advertised credit_window_bytes={window}"
    );

    // --- 4. ASSERT A: the observer sees the victim's ordered disconnect pair
    //        with the exact `flow_control_overflow` reason, within a bounded wait.
    //        The bounded wait is itself evidence the server detaches rather than
    //        buffering the slow client without limit.
    let deadline = Instant::now() + WAIT;
    let mut events: Vec<relayable_client::Event> = Vec::new();
    let mut found: Option<(u64, String)> = None;
    while Instant::now() < deadline && found.is_none() {
        events.extend(observer.collect_events(SLICE)?);
        found = disconnect_pair_for(&events, &victim_id);
    }
    let (first_sequence, reason) =
        found.context("observer must see the victim's flow-control disconnect within WAIT")?;
    assert_eq!(
        reason, "flow_control_overflow",
        "the slow-client overflow reason is the frozen §D spelling"
    );
    // `disconnect_pair_for` matched the two events as a `windows(2)` pair and
    // `assert_event_sequences_contiguous` (below) requires consecutive
    // sequences, so the pair is adjacent on the wire as well; no separate
    // companion lookup is needed.
    assert!(
        events.iter().all(|event| !matches!(
            &event.event,
            Some(EventKind::ClientDisconnected(disconnected))
                if disconnected.client_id != victim_id
        )),
        "only the overflowing victim may be disconnected; the observer stays connected"
    );
    assert_event_sequences_contiguous(&events);
    eprintln!(
        "[QA-FLOW-001] flow-control-2: observer saw victim {victim_id}: Disconnected at seq {first_sequence} then client_disconnected{{flow_control_overflow}} at seq {}",
        first_sequence + 1
    );

    // --- 5. ASSERT B: the victim's transport is really closed.
    let victim_error = victim
        .list_sessions()
        .expect_err("the overflowing victim's connection must be closed");
    assert!(
        matches!(
            victim_error,
            ClientError::Disconnected | ClientError::Transport(_)
        ),
        "a closed connection surfaces as Disconnected or Transport, got {victim_error:?}"
    );

    // --- 6. ASSERT C: the session, block, PTY, and child survive.
    let mut probe = Client::connect(&socket, ClientConfig::new("flow-probe"))?;
    let after = probe
        .session_snapshot(&session_id)
        .context("fresh client session.view after overflow")?;
    let session = after.session.as_ref().context("session record survives")?;
    assert_eq!(
        session.state,
        SessionState::Active as i32,
        "the session stays active after the client overflow"
    );
    assert_eq!(
        after
            .blocks
            .first()
            .context("the block survives")?
            .program_pid,
        program_pid,
        "the same server-owned child is still attached"
    );
    assert!(
        proc_exists(program_pid),
        "the child process must survive the client overflow"
    );
    let listed = probe.list_sessions().context("fresh client session.list")?;
    let summary = listed
        .iter()
        .find(|entry| entry.session_id == session_id_text)
        .context("the session is still listed")?;
    assert_eq!(
        summary.state,
        SessionState::Active as i32,
        "the listed session is still active (not closed)"
    );
    eprintln!(
        "[QA-FLOW-001] flow-control-3: session {session_id_text} state=active, program_pid={program_pid} alive after overflow"
    );

    // --- 7. ASSERT D: the server is not wedged — a fresh client can take the
    //        writer lease (the victim's lease was released) and round-trip
    //        input -> output.
    let reattached = probe
        .attach_block(&block_id, AttachMode::Writer)
        .context("fresh client block.attach(Writer) after overflow")?;
    assert!(
        reattached.writer_lease,
        "the released writer lease is available to a fresh client"
    );
    let probe_stream = reattached.stream_id;
    let marker = nonce("flow-marker");
    probe
        .send_input(probe_stream, format!("echo {marker}\n").as_bytes())
        .context("fresh client terminal.input after overflow")?;
    // `wait_for_output` returns `Ok` only once the marker is present in the
    // accumulated output, so an extra `contains` check here would assert nothing.
    let _ = probe.wait_for_output(probe_stream, marker.as_bytes(), WAIT)?;
    assert!(
        proc_exists(program_pid),
        "the child is still alive after the recovery round-trip"
    );
    eprintln!(
        "[QA-FLOW-001] flow-control-4: fresh client attached stream={probe_stream}, echoed marker={marker}, child alive"
    );

    Ok(())
}

// --- Raw-frame victim ------------------------------------------------------
//
// The test above proves the *client-visible* overflow semantics. The test below
// closes the remaining gap: it uses a raw `FrameConnection` victim that never
// sends a `flow_control` ack and counts the `terminal_data` payload bytes the
// server actually hands it before closing. That makes the credit **bound**
// load-bearing: the test fails if the server streams far past the window and
// only then detaches.

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

/// A read timeout is not a close; the caller keeps polling until its deadline.
fn raw_is_timeout(error: &TransportError) -> bool {
    let io = match error {
        TransportError::Io(error) => error,
        TransportError::Codec(CodecError::Io(error)) => error,
        TransportError::Codec(_) => return false,
    };
    matches!(io.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut)
}

/// Read until the response for `request_id`, ignoring any event, heartbeat, or
/// terminal-data frame that arrives first.
fn raw_recv_response(
    connection: &mut FrameConnection,
    request_id: u64,
) -> Result<response::Result> {
    let deadline = Instant::now() + WAIT;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            bail!("no raw response for request {request_id} within WAIT");
        }
        connection.set_read_timeout(Some(remaining)).ok();
        match connection.recv() {
            Ok(Some(envelope)) => match envelope.body {
                Some(Body::Response(response)) if response.request_id == request_id => {
                    return response.result.context("response result missing");
                }
                Some(Body::Heartbeat(_) | Body::Event(_) | Body::TerminalData(_)) => {}
                other => bail!("unexpected frame awaiting response {request_id}: {other:?}"),
            },
            Ok(None) => bail!("connection closed before raw response {request_id}"),
            Err(error) => {
                if !raw_is_timeout(&error) {
                    bail!("raw recv error for request {request_id}: {error}");
                }
            }
        }
    }
}

/// Handshake and return the connection plus the server-advertised limits.
fn raw_handshake(socket: &Path) -> Result<(FrameConnection, Limits)> {
    let mut connection = FrameConnection::connect(socket)
        .with_context(|| format!("raw connect to {}", socket.display()))?;
    connection
        .send(&raw_envelope(
            Some(1),
            Body::Handshake(Handshake {
                client_kind: "flow-raw-victim".to_string(),
                platform: "linux".to_string(),
                client_version: "0".to_string(),
                protocol_version: PROTOCOL_VERSION.to_string(),
                capabilities: Vec::new(),
            }),
        ))
        .context("send raw handshake")?;
    match raw_recv_response(&mut connection, 1)? {
        response::Result::Hello(hello) => {
            let limits = hello.limits.context("server hello advertises limits")?;
            Ok((connection, limits))
        }
        other => bail!("raw handshake failed: {other:?}"),
    }
}

/// Attach `block_id` as the writer on a raw connection; returns the stream id.
fn raw_attach(connection: &mut FrameConnection, block_id: &BlockId) -> Result<u64> {
    connection
        .send(&raw_envelope(
            Some(2),
            Body::Request(Request {
                method: methods::BLOCK_ATTACH.to_string(),
                operation: Some(request::Operation::BlockAttach(BlockAttachRequest {
                    block_id: block_id.to_string(),
                    mode: WireAttachMode::Writer as i32,
                    takeover: false,
                })),
            }),
        ))
        .context("send raw block.attach")?;
    match raw_recv_response(connection, 2)? {
        response::Result::BlockAttach(attach) => Ok(attach.stream_id),
        other => bail!("raw block.attach failed: {other:?}"),
    }
}

/// Send raw `terminal.input` (no `flow_control` ack is ever sent by this client).
fn raw_send_input(connection: &mut FrameConnection, stream_id: u64, bytes: &[u8]) -> Result<()> {
    connection
        .send(&raw_envelope(
            Some(3),
            Body::Request(Request {
                method: methods::TERMINAL_INPUT.to_string(),
                operation: Some(request::Operation::TerminalInput(TerminalInput {
                    stream_id,
                    bytes: bytes.to_vec(),
                })),
            }),
        ))
        .context("send raw terminal.input")
}

/// QA-FLOW-001: a raw victim that never acknowledges reads at most the credit
/// window (plus at most one PTY chunk) of `terminal_data` bytes before the
/// server closes the connection, and the session/child survive.
#[test]
fn slow_client_receives_at_most_credit_window_bytes_before_overflow() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();
    let socket_text = socket.to_str().context("socket path is UTF-8")?.to_string();

    // The PTY-bearing session is created through the frozen CLI; the raw victim
    // only needs its block id (and the session id for the survivor check).
    let created = cli_json(
        &["new", "--json"],
        &[("RELAYABLE_SERVER", socket_text.as_str())],
    );
    let session_id: SessionId = created["session_id"]
        .as_str()
        .context("new.session_id")?
        .parse()
        .context("parse session id")?;
    let block_id: BlockId = created["block_id"]
        .as_str()
        .context("new.block_id")?
        .parse()
        .context("parse block id")?;
    let program_pid = u32::try_from(created["program_pid"].as_u64().context("new.program_pid")?)
        .context("program_pid fits u32")?;
    assert!(program_pid > 0, "program_pid must be a real pid");
    server.track_child(program_pid);

    let (mut victim, limits) = raw_handshake(&socket)?;
    let window = u64::from(limits.credit_window_bytes);
    assert_eq!(
        window,
        u64::from(DEFAULT_LIMITS.credit_window_bytes),
        "the foreground server must advertise the protocol default credit window"
    );
    assert_eq!(
        limits.max_frame_bytes, DEFAULT_LIMITS.max_frame_bytes,
        "the foreground server must advertise the protocol default frame bound"
    );

    let stream_id = raw_attach(&mut victim, &block_id)?;

    // 1.5 windows of newline-free output: the PTY's ONLCR would otherwise expand
    // every `\n` to `\r\n` (see the client-API test above). The raw victim never
    // acks, so the server can only deliver credit-window bytes before overflow.
    let payload = window + window / 2;
    let command = format!("head -c {payload} /dev/zero | tr '\\0' 'x'\n");
    raw_send_input(&mut victim, stream_id, command.as_bytes())?;

    // Read `terminal_data` frames, counting payload bytes, until the server
    // closes the connection (EOF or a non-timeout transport error).
    let mut delivered: u64 = 0;
    let mut frames: u64 = 0;
    let mut closed = false;
    let deadline = Instant::now() + WAIT;
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        victim.set_read_timeout(Some(remaining)).ok();
        match victim.recv() {
            Ok(Some(envelope)) => {
                if let Some(Body::TerminalData(output)) = envelope.body {
                    if output.stream_id == stream_id {
                        delivered += output.bytes.len() as u64;
                        frames += 1;
                    }
                }
            }
            Ok(None) => {
                closed = true;
                break;
            }
            Err(error) => {
                if raw_is_timeout(&error) {
                    continue;
                }
                // `ConnectionShared::close_now` shuts the socket down, which
                // may surface as a reset rather than a clean EOF.
                closed = true;
                break;
            }
        }
    }

    eprintln!(
        "[QA-FLOW-001] raw-victim-1: window={window}B frames={frames} delivered={delivered}B closed={closed} bound={}B",
        window + MAX_CHUNK
    );
    assert!(
        closed,
        "the overflowing raw connection must reach EOF/close within WAIT"
    );
    assert!(
        delivered > 0,
        "the raw victim must receive real output before the overflow (0 bytes means the bound would pass vacuously)"
    );
    // `Subscriber::deliver` consumes credit per chunk *before* enqueueing it and
    // `try_consume` is all-or-nothing (it fails when the remaining credit is
    // smaller than the chunk), so the delivered chunk lengths sum to at most the
    // window exactly; `+ MAX_CHUNK` is conservative headroom around that. A
    // server that buffered 10 MB and then detached would fail this assertion.
    assert!(
        delivered <= window + MAX_CHUNK,
        "slow client received {delivered}B of terminal_data, beyond the credit-window bound of {}B",
        window + MAX_CHUNK
    );

    // The raw victim's overflow must not kill the session, PTY, or child.
    let mut probe = Client::connect(&socket, ClientConfig::new("flow-raw-probe"))?;
    let after = probe
        .session_snapshot(&session_id)
        .context("fresh client session.view after raw overflow")?;
    assert_eq!(
        after
            .session
            .as_ref()
            .context("session record survives raw overflow")?
            .state,
        SessionState::Active as i32,
        "the session stays active after the raw client overflow"
    );
    assert!(
        proc_exists(program_pid),
        "the server-owned child must survive the raw client overflow"
    );
    eprintln!(
        "[QA-FLOW-001] raw-victim-2: session {session_id} state=active, program_pid={program_pid} alive after raw overflow"
    );

    Ok(())
}
