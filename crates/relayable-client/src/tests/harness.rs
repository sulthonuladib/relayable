use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use relayable_protocol::codec::CodecError;
use relayable_protocol::limits::DEFAULT_LIMITS;
use relayable_protocol::transport::{FrameConnection, TransportError};
use relayable_protocol::v1::{
    envelope::Body, event, request::Operation, response::Result as ResponseResult, AttachMode,
    BlockAttachRequest, BlockAttachResponse, ClientConnectionStateChanged, Envelope, Event, Limits,
    Request, Response, ServerHello, SessionListResponse, SessionResumeRequest,
    SessionResumeResponse, SessionState, SessionSummary, TerminalOutput, TerminalSnapshot,
};
use relayable_protocol::{CONTROL_STREAM_ID, PROTOCOL_VERSION};

use crate::client::Client;
use crate::config::ClientConfig;
use crate::error::is_timeout;

pub(crate) fn response_for(request_id: u64, result: ResponseResult) -> Envelope {
    Envelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        request_id: Some(request_id),
        stream_id: CONTROL_STREAM_ID,
        body: Some(Body::Response(Response {
            request_id,
            result: Some(result),
        })),
        ..Envelope::default()
    }
}

/// Receive one envelope for a stub server, treating a closed connection or a
/// torn-down socket as "the stub is finished" instead of panicking.
///
/// The tests assert on the *client* side, so a stub that loses a teardown
/// race with a dropping client should just stop. `Ok(None)` is EOF; a
/// timeout/reset `Err` is the same teardown condition observed from the read
/// side. Returns `None` in either case.
pub(crate) fn recv_or_finished(conn: &mut FrameConnection) -> Option<Envelope> {
    match conn.recv() {
        Ok(Some(envelope)) => Some(envelope),
        Ok(None) | Err(_) => None,
    }
}

pub(crate) fn fake_server(mut conn: FrameConnection) {
    conn.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let Some(handshake) = recv_or_finished(&mut conn) else {
        return;
    };
    assert!(matches!(handshake.body, Some(Body::Handshake(_))));
    assert_eq!(handshake.request_id, Some(1));
    conn.send(&response_for(
        1,
        ResponseResult::Hello(ServerHello {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: vec!["test".to_string()],
            limits: Some(Limits {
                max_frame_bytes: 4096,
                ..DEFAULT_LIMITS
            }),
        }),
    ))
    .unwrap();

    // A client that only exercises the handshake (e.g.
    // `handshake_captures_hello_and_limits`) drops without ever sending a
    // `session.list`; its EOF must end the stub cleanly, not panic.
    let Some(list) = recv_or_finished(&mut conn) else {
        return;
    };
    let request_id = list.request_id.unwrap();
    assert!(matches!(list.body, Some(Body::Request(_))));
    conn.send(&response_for(
        request_id,
        ResponseResult::SessionList(SessionListResponse {
            sessions: vec![SessionSummary {
                session_id: "session:01a10cf3-2655-7e48-9b9c-43dd6612bef5".to_string(),
                label: Some("demo".to_string()),
                state: SessionState::Active as i32,
                revision: 1,
            }],
            sequence: 3,
        }),
    ))
    .unwrap();
}

pub(crate) fn client_connected() -> (Client, thread::JoinHandle<()>) {
    let (client_stream, server_stream) = UnixStream::pair().unwrap();
    let server = thread::spawn(move || {
        fake_server(FrameConnection::from_stream(
            server_stream,
            DEFAULT_LIMITS.max_frame_bytes,
        ));
    });
    let client = Client::from_connection(
        FrameConnection::from_stream(client_stream, DEFAULT_LIMITS.max_frame_bytes),
        PathBuf::from("/tmp/relayable-test.sock"),
        ClientConfig::new("test"),
    )
    .unwrap();
    (client, server)
}

/// Complete the client handshake on a server-side connection, advertising
/// `limits`. Returns `false` (without panicking) if the client has already
/// gone away, so the caller skips its script.
pub(crate) fn complete_handshake(conn: &mut FrameConnection, limits: Limits) -> bool {
    conn.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let Some(handshake) = recv_or_finished(conn) else {
        return false;
    };
    let request_id = handshake.request_id.unwrap();
    assert!(matches!(handshake.body, Some(Body::Handshake(_))));
    conn.send(&response_for(
        request_id,
        ResponseResult::Hello(ServerHello {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: vec!["test".to_string()],
            limits: Some(limits),
        }),
    ))
    .unwrap();
    true
}

/// Spawn a paired server thread that completes the handshake and then runs
/// `script` against the connection.
pub(crate) fn spawn_scripted_server<F>(script: F) -> (Client, thread::JoinHandle<()>)
where
    F: FnOnce(&mut FrameConnection) + Send + 'static,
{
    let limits = Limits {
        max_frame_bytes: 4096,
        ..DEFAULT_LIMITS
    };
    spawn_scripted_server_with_limits(limits, script)
}

/// Like [`spawn_scripted_server`], but advertises `limits` (so a test can
/// pick a short heartbeat cadence).
pub(crate) fn spawn_scripted_server_with_limits<F>(
    limits: Limits,
    script: F,
) -> (Client, thread::JoinHandle<()>)
where
    F: FnOnce(&mut FrameConnection) + Send + 'static,
{
    let (client_stream, server_stream) = UnixStream::pair().unwrap();
    let server = thread::spawn(move || {
        let mut conn = FrameConnection::from_stream(server_stream, DEFAULT_LIMITS.max_frame_bytes);
        if complete_handshake(&mut conn, limits) {
            script(&mut conn);
        }
    });
    let client = Client::from_connection(
        FrameConnection::from_stream(client_stream, DEFAULT_LIMITS.max_frame_bytes),
        PathBuf::from("/tmp/relayable-test.sock"),
        ClientConfig::new("test"),
    )
    .unwrap();
    (client, server)
}

pub(crate) fn block_attach_response() -> BlockAttachResponse {
    BlockAttachResponse {
        stream_id: 7,
        mode: AttachMode::Writer as i32,
        writer_lease: true,
        snapshot: Some(TerminalSnapshot::default()),
    }
}

pub(crate) fn block_attach_of(envelope: Envelope) -> BlockAttachRequest {
    match envelope.body {
        Some(Body::Request(Request {
            operation: Some(Operation::BlockAttach(request)),
            ..
        })) => request,
        other => panic!("expected a block.attach request, got {other:?}"),
    }
}

pub(crate) fn session_resume_of(envelope: Envelope) -> SessionResumeRequest {
    match envelope.body {
        Some(Body::Request(Request {
            operation: Some(Operation::SessionResume(request)),
            ..
        })) => request,
        other => panic!("expected a session.resume request, got {other:?}"),
    }
}

pub(crate) fn event_envelope(sequence: u64) -> Envelope {
    Envelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        request_id: None,
        stream_id: CONTROL_STREAM_ID,
        sequence: Some(sequence),
        body: Some(Body::Event(Event {
            sequence,
            revision: 0,
            session_id: String::new(),
            event: Some(event::Event::ClientConnectionStateChanged(
                ClientConnectionStateChanged {
                    client_id: "client:test".to_string(),
                    connection_state: "ready".to_string(),
                },
            )),
        })),
        ..Envelope::default()
    }
}

/// Spawn a server that captures exactly one `block.attach` request and
/// answers it with [`block_attach_response`].
pub(crate) fn attach_server() -> (Client, thread::JoinHandle<()>, mpsc::Receiver<Envelope>) {
    let (tx, rx) = mpsc::channel();
    let (client, server) = spawn_scripted_server(move |conn| {
        let Some(request) = recv_or_finished(conn) else {
            return;
        };
        let request_id = request.request_id.unwrap();
        tx.send(request).unwrap();
        conn.send(&response_for(
            request_id,
            ResponseResult::BlockAttach(block_attach_response()),
        ))
        .unwrap();
    });
    (client, server, rx)
}

/// Spawn a server that captures exactly one `session.resume` request and
/// answers it with a snapshot-required response.
pub(crate) fn resume_server() -> (Client, thread::JoinHandle<()>, mpsc::Receiver<Envelope>) {
    let (tx, rx) = mpsc::channel();
    let (client, server) = spawn_scripted_server(move |conn| {
        let Some(request) = recv_or_finished(conn) else {
            return;
        };
        let request_id = request.request_id.unwrap();
        tx.send(request).unwrap();
        conn.send(&response_for(
            request_id,
            ResponseResult::SessionResume(SessionResumeResponse {
                snapshot_required: true,
                snapshot_required_reason: "no_cursor".to_string(),
                ..SessionResumeResponse::default()
            }),
        ))
        .unwrap();
    });
    (client, server, rx)
}

/// A server-to-client `TerminalOutput` frame exactly as the server builds it
/// (`crates/relayable-server/src/connection.rs::terminal_output_envelope`).
pub(crate) fn terminal_data(stream_id: u64, stream_seq: u64, bytes: &[u8]) -> Envelope {
    Envelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        request_id: None,
        stream_id,
        body: Some(Body::TerminalData(TerminalOutput {
            stream_id,
            stream_seq,
            bytes: bytes.to_vec(),
        })),
        ..Envelope::default()
    }
}

/// A framed transport read timeout, as opposed to a real close/teardown.
pub(crate) fn is_transport_timeout(err: &TransportError) -> bool {
    match err {
        TransportError::Io(io) => is_timeout(io),
        TransportError::Codec(CodecError::Io(io)) => is_timeout(io),
        TransportError::Codec(_) => false,
    }
}

/// Spawn a server that completes the handshake, answers exactly one
/// `block.attach` with [`block_attach_response`], sends `script` frames, and
/// then forwards every inbound envelope into an `mpsc` channel until the
/// client closes the connection or `drain` elapses (so a misbehaving test
/// cannot hang the suite).
pub(crate) fn spawn_output_server(
    script: Vec<Envelope>,
    drain: Duration,
) -> (Client, thread::JoinHandle<()>, mpsc::Receiver<Envelope>) {
    let (tx, rx) = mpsc::channel();
    let (client, server) = spawn_scripted_server(move |conn| {
        let Some(request) = recv_or_finished(conn) else {
            return;
        };
        let request_id = request.request_id.unwrap();
        conn.send(&response_for(
            request_id,
            ResponseResult::BlockAttach(block_attach_response()),
        ))
        .unwrap();
        for envelope in &script {
            if conn.send(envelope).is_err() {
                return;
            }
        }
        let deadline = Instant::now() + drain;
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
                    let _ = tx.send(envelope);
                }
                Ok(None) => break,
                Err(err) if is_transport_timeout(&err) => {}
                Err(_) => break,
            }
        }
    });
    (client, server, rx)
}
