use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use relayable_model::BlockId;
use relayable_protocol::limits::DEFAULT_LIMITS;
use relayable_protocol::v1::{
    envelope::Body, event, response::Result as ResponseResult, AttachMode, Limits,
    SessionListResponse, SessionState, SessionSummary,
};

use super::harness::*;

#[test]
fn wait_for_event_surfaces_a_live_event() {
    let (mut client, server) = spawn_scripted_server(|conn| {
        conn.send(&event_envelope(7)).unwrap();
        // Keep the connection open until the client is done.
        let _ = conn.recv();
    });

    let event = client
        .wait_for_event(Duration::from_secs(2))
        .unwrap()
        .expect("an event should arrive");
    assert_eq!(event.sequence, 7);
    assert!(matches!(
        event.event,
        Some(event::Event::ClientConnectionStateChanged(_))
    ));
    assert!(client.drain_events().is_empty());

    drop(client);
    let _ = server.join();
}

#[test]
fn events_are_retained_across_control_rpcs_and_reachable_by_drain() {
    let (mut client, server) = spawn_scripted_server(|conn| {
        let request = conn.recv().unwrap().unwrap();
        let request_id = request.request_id.unwrap();
        // The event arrives between the request and its response: it must be
        // buffered without corrupting the pending control RPC.
        conn.send(&event_envelope(9)).unwrap();
        conn.send(&response_for(
            request_id,
            ResponseResult::SessionList(SessionListResponse {
                sessions: vec![SessionSummary {
                    session_id: "session:01a10cf3-2655-7e48-9b9c-43dd6612bef5".to_string(),
                    label: Some("demo".to_string()),
                    state: SessionState::Active as i32,
                    revision: 1,
                }],
                sequence: 9,
            }),
        ))
        .unwrap();
    });

    let sessions = client.list_sessions().unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].label.as_deref(), Some("demo"));

    let events = client.drain_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].sequence, 9);
    assert!(client.drain_events().is_empty());

    drop(client);
    let _ = server.join();
}

#[test]
fn events_buffered_during_a_rpc_can_be_waited_for_later() {
    let (mut client, server) = spawn_scripted_server(|conn| {
        conn.send(&event_envelope(11)).unwrap();
        let request = conn.recv().unwrap().unwrap();
        let request_id = request.request_id.unwrap();
        conn.send(&response_for(
            request_id,
            ResponseResult::SessionList(SessionListResponse {
                sessions: Vec::new(),
                sequence: 11,
            }),
        ))
        .unwrap();
        // Keep the connection open until the client is done.
        let _ = conn.recv();
    });

    // The list RPC drains the event frame into the bounded pending list.
    assert!(client.list_sessions().unwrap().is_empty());
    let event = client
        .wait_for_event(Duration::from_millis(50))
        .unwrap()
        .expect("buffered event");
    assert_eq!(event.sequence, 11);
    assert!(client
        .wait_for_event(Duration::from_millis(10))
        .unwrap()
        .is_none());

    drop(client);
    let _ = server.join();
}

/// Receiving frames must never reset the outbound heartbeat deadline: a
/// client blocked in one long `collect_events` call while the server streams
/// events every few milliseconds (so the client's blocking read always
/// returns a frame and never reaches a read timeout) must still send its own
/// `Heartbeat` frames on its advertised cadence. With the previous
/// "send-on-read-timeout" logic this produced zero heartbeats.
#[test]
fn inbound_frames_do_not_starve_the_outbound_heartbeat() {
    const CADENCE_MS: u64 = 40;
    let limits = Limits {
        max_frame_bytes: 4096,
        heartbeat_interval_millis: CADENCE_MS as u32,
        ..DEFAULT_LIMITS
    };
    let (hb_tx, hb_rx) = mpsc::channel();
    let (mut client, server) = spawn_scripted_server_with_limits(limits, move |conn| {
        // Stream events faster than the heartbeat cadence, for longer than
        // the client's collect window, so inbound frames keep the client's
        // blocking read busy the whole time.
        let start = Instant::now();
        let mut sequence = 0u64;
        while start.elapsed() < Duration::from_millis(320) {
            sequence += 1;
            conn.send(&event_envelope(sequence)).unwrap();
            thread::sleep(Duration::from_millis(5));
        }
        // The client has returned from `collect_events` by now; drain the
        // heartbeats it sent while the window was open.
        conn.set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let mut heartbeats = 0usize;
        let drain_deadline = Instant::now() + Duration::from_millis(300);
        while Instant::now() < drain_deadline {
            match conn.recv() {
                Ok(Some(envelope)) => {
                    if matches!(envelope.body, Some(Body::Heartbeat(_))) {
                        heartbeats += 1;
                    }
                }
                Ok(None) | Err(_) => break,
            }
        }
        hb_tx.send(heartbeats).unwrap();
    });

    let events = client.collect_events(Duration::from_millis(300)).unwrap();
    assert!(!events.is_empty(), "the server streamed events");
    let heartbeats = hb_rx.recv().unwrap();
    assert!(
        heartbeats >= 1,
        "expected an outbound heartbeat while inbound frames kept arriving, got {heartbeats}"
    );

    drop(client);
    let _ = server.join();
}

/// The outbound baseline must survive **across calls**: a pump loop that
/// waits in short repeated slices spanning the cadence still emits
/// heartbeats — with a per-call baseline it emits none and is idle-dropped —
/// yet never emits faster than the cadence.
#[test]
fn short_repeated_waits_accumulate_toward_the_heartbeat_cadence() {
    const CADENCE_MS: u64 = 40;
    let limits = Limits {
        max_frame_bytes: 4096,
        heartbeat_interval_millis: CADENCE_MS as u32,
        ..DEFAULT_LIMITS
    };
    let (hb_tx, hb_rx) = mpsc::channel();
    let (mut client, server) = spawn_scripted_server_with_limits(limits, move |conn| {
        let start = Instant::now();
        let mut sequence = 0u64;
        while start.elapsed() < Duration::from_millis(320) {
            sequence += 1;
            conn.send(&event_envelope(sequence)).unwrap();
            thread::sleep(Duration::from_millis(5));
        }
        conn.set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let mut heartbeats = 0usize;
        let drain_deadline = Instant::now() + Duration::from_millis(300);
        while Instant::now() < drain_deadline {
            match conn.recv() {
                Ok(Some(envelope)) => {
                    if matches!(envelope.body, Some(Body::Heartbeat(_))) {
                        heartbeats += 1;
                    }
                }
                Ok(None) | Err(_) => break,
            }
        }
        hb_tx.send(heartbeats).unwrap();
    });

    // Pump in 10 ms slices for ~300 ms, the way a CLI/output pump would.
    let pump_deadline = Instant::now() + Duration::from_millis(300);
    let mut events_seen = 0usize;
    while Instant::now() < pump_deadline {
        events_seen += client
            .collect_events(Duration::from_millis(10))
            .unwrap()
            .len();
    }
    assert!(events_seen > 0, "the server streamed events");

    let heartbeats = hb_rx.recv().unwrap();
    assert!(
        heartbeats >= 1,
        "short repeated waits must accumulate toward the cadence, got {heartbeats}"
    );
    // ~300 ms / 40 ms cadence is at most ~8 sends; allow slack for scheduling
    // but reject per-call flooding (a 10 ms slice would send ~30 times).
    assert!(
        heartbeats <= 12,
        "heartbeats must not be sent faster than the cadence, got {heartbeats}"
    );

    drop(client);
    let _ = server.join();
}

/// CLIENT-API-001 rules 1 and 7: a non-target `TerminalData` frame is never
/// acked during a target read and is later returned intact; an `Event` frame
/// stays retrievable through the event API.
#[test]
fn non_target_frames_are_retained_and_not_acked() {
    let target = 7u64;
    let other = 9u64;
    let (mut client, server, rx) = spawn_output_server(
        vec![
            terminal_data(target, 1, b"alpha"),
            terminal_data(other, 1, b"beta"),
            event_envelope(33),
            terminal_data(target, 2, b"gamma"),
        ],
        // Keep the stub alive for the whole test: the last ack writes happen
        // well after the first read, so a loaded machine must not be able to
        // race the stub's drain window and turn an ack into EPIPE.
        Duration::from_secs(2),
    );
    client
        .attach_block(&BlockId::new(), AttachMode::Writer)
        .unwrap();

    let first = client
        .next_output(target, Duration::from_millis(500))
        .unwrap()
        .expect("target frame");
    assert_eq!(first, b"alpha");

    // Only the target stream may be acked so far.
    let mut first_wave = Vec::new();
    while let Ok(envelope) = rx.recv_timeout(Duration::from_millis(300)) {
        if let Some(Body::FlowControl(flow)) = envelope.body {
            first_wave.push(flow);
        }
    }
    assert_eq!(first_wave.len(), 1, "exactly the target frame is acked");
    assert_eq!(first_wave[0].stream_id, target);
    assert_eq!(first_wave[0].ack_bytes, b"alpha".len() as u64);
    assert!(
        first_wave.iter().all(|ack| ack.stream_id != other),
        "the other stream must not be acked during a target read"
    );

    let second = client
        .next_output(target, Duration::from_millis(500))
        .unwrap()
        .expect("second target frame");
    assert_eq!(second, b"gamma");
    assert_eq!(
        client
            .drain_events()
            .iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>(),
        vec![33],
        "the event frame stays retrievable"
    );

    let other_bytes = client
        .next_output(other, Duration::from_millis(500))
        .unwrap_or_else(|err| {
            panic!(
                "next_output for the other stream failed (the stub may have exited early): {err:?}"
            )
        })
        .expect("other stream frame");
    assert_eq!(
        other_bytes, b"beta",
        "the other stream's bytes are complete"
    );

    drop(client);
    let _ = server.join();
}

/// CLIENT-API-001 rule 8: while `next_output` waits for a delayed frame it
/// keeps the outbound heartbeat cadence.
#[test]
fn next_output_heartbeats_while_waiting_for_a_delayed_frame() {
    const CADENCE_MS: u64 = 50;
    let target = 7u64;
    let limits = Limits {
        max_frame_bytes: 4096,
        heartbeat_interval_millis: CADENCE_MS as u32,
        ..DEFAULT_LIMITS
    };
    let (hb_tx, hb_rx) = mpsc::channel();
    let (mut client, server) = spawn_scripted_server_with_limits(limits, move |conn| {
        let Some(request) = recv_or_finished(conn) else {
            return;
        };
        let request_id = request.request_id.unwrap();
        conn.send(&response_for(
            request_id,
            ResponseResult::BlockAttach(block_attach_response()),
        ))
        .unwrap();

        // Count the heartbeats the client sends while it waits for a frame
        // that is deliberately withheld, then release it.
        let deadline = Instant::now() + Duration::from_millis(250);
        let mut heartbeats = 0usize;
        while Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if conn
                .set_read_timeout(Some(remaining.min(Duration::from_millis(50))))
                .is_err()
            {
                break;
            }
            match conn.recv() {
                Ok(Some(envelope)) => {
                    if matches!(envelope.body, Some(Body::Heartbeat(_))) {
                        heartbeats += 1;
                    }
                }
                Ok(None) => break,
                Err(err) if is_transport_timeout(&err) => {}
                Err(_) => break,
            }
        }
        hb_tx.send(heartbeats).unwrap();
        let _ = conn.send(&terminal_data(target, 1, b"delayed"));
        // Keep the connection open until the client is done, so the client's
        // next heartbeat read/write does not race the server drop.
        conn.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let _ = conn.recv();
    });

    client
        .attach_block(&BlockId::new(), AttachMode::Writer)
        .unwrap();
    let bytes = client
        .next_output(target, Duration::from_millis(600))
        .unwrap()
        .expect("delayed frame");
    assert_eq!(bytes, b"delayed");

    let heartbeats = hb_rx.recv().unwrap();
    assert!(
        heartbeats >= 1,
        "waiting for a delayed frame must keep the heartbeat cadence, got {heartbeats}"
    );
    // ~250 ms at a 50 ms cadence is at most ~5 sends; mirror the
    // pre-existing cadence test and reject a runaway heartbeat loop without
    // being timing-fragile.
    assert!(
        heartbeats <= 12,
        "heartbeats must not be sent faster than the cadence, got {heartbeats}"
    );

    drop(client);
    let _ = server.join();
}
