//! CORE-001 assignment E — raw-framing protocol conformance.
//!
//! These tests use `relayable_protocol::transport::FrameConnection` and the
//! generated `v1` types (the client-side view of the wire) against a real
//! `relayable-server`. Error codes are asserted against the frozen lowercase
//! contract strings via `relayable_protocol::error_code::contract_str`.
//!
//! The length-prefix corruption cases inject raw bytes directly, because the
//! frame codec never emits an invalid prefix.

use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::{Duration, Instant};

use relayable_integration_tests::harness::{step, TestServer, RETRY, WAIT};
use relayable_model::SessionId;
use relayable_protocol::codec::CodecError;
use relayable_protocol::error_code::contract_str;
use relayable_protocol::limits::DEFAULT_LIMITS;
use relayable_protocol::methods;
use relayable_protocol::transport::{FrameConnection, TransportError};
use relayable_protocol::v1::{
    envelope::Body, request, response, Envelope, Error, ErrorCode, Handshake, Request,
    SessionListRequest, SessionSnapshotRequest, TerminalInput,
};
use relayable_protocol::PROTOCOL_VERSION;

fn envelope(request_id: Option<u64>, body: Body) -> Envelope {
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

/// Extract the underlying I/O error from either transport error variant.
fn io_error(error: &TransportError) -> Option<&std::io::Error> {
    match error {
        TransportError::Io(error) => Some(error),
        TransportError::Codec(CodecError::Io(error)) => Some(error),
        TransportError::Codec(_) => None,
    }
}

fn is_timeout_or_interrupted(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::WouldBlock
            | std::io::ErrorKind::TimedOut
            | std::io::ErrorKind::Interrupted
    )
}

/// True only for accepted close/EOF evidence on a framed connection.
///
/// A timeout is deliberately **not** evidence of a close: the peer may simply be
/// idle, and accepting it would let a server that failed to close the connection
/// pass. A codec error that is not an I/O timeout means the peer stopped
/// mid-frame (truncated) or sent garbage, so the connection is gone.
fn is_close_evidence(error: &TransportError) -> bool {
    match io_error(error) {
        Some(io) => !is_timeout_or_interrupted(io),
        None => true,
    }
}

fn connect(socket: &Path) -> FrameConnection {
    let deadline = Instant::now() + WAIT;
    loop {
        match FrameConnection::connect(socket) {
            Ok(connection) => return connection,
            Err(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            Err(error) => panic!("connect to {}: {error}", socket.display()),
        }
    }
}

fn recv_response(
    connection: &mut FrameConnection,
    request_id: u64,
    timeout: Duration,
) -> response::Result {
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(!remaining.is_zero(), "no response for request {request_id}");
        connection.set_read_timeout(Some(remaining)).ok();
        match connection.recv() {
            Ok(Some(frame)) => {
                if frame.request_id != Some(request_id) {
                    continue;
                }
                if let Some(Body::Response(response)) = frame.body {
                    return response.result.expect("response result");
                }
            }
            Ok(None) => panic!("connection closed before response {request_id}"),
            Err(error) => match io_error(&error) {
                Some(io) if is_timeout_or_interrupted(io) => {}
                _ => panic!("recv error for request {request_id}: {error}"),
            },
        }
    }
}

fn handshake_version(connection: &mut FrameConnection, version: &str) -> response::Result {
    connection
        .send(&envelope(
            Some(1),
            Body::Handshake(Handshake {
                client_kind: "protocol-test".to_string(),
                platform: "linux".to_string(),
                client_version: "0".to_string(),
                protocol_version: version.to_string(),
                capabilities: Vec::new(),
            }),
        ))
        .expect("send handshake");
    recv_response(connection, 1, WAIT)
}

fn handshake_ok(socket: &Path) -> FrameConnection {
    let mut connection = connect(socket);
    match handshake_version(&mut connection, PROTOCOL_VERSION) {
        response::Result::Hello(hello) => {
            assert_eq!(hello.protocol_version, PROTOCOL_VERSION);
            assert_eq!(hello.limits, Some(DEFAULT_LIMITS));
        }
        other => panic!("handshake failed: {other:?}"),
    }
    connection
}

fn expect_error(result: response::Result, code: ErrorCode) {
    match result {
        response::Result::Error(error) => {
            assert_eq!(error.code, code as i32, "expected {code:?}, got {error:?}");
        }
        other => panic!("expected {code:?}, got {other:?}"),
    }
}

/// Wait until the peer closes the connection (clean EOF), draining any frames.
///
/// Only genuine close evidence is accepted; a timeout or interrupt fails the
/// test rather than passing as a close (see [`is_close_evidence`]).
fn expect_closed(connection: &mut FrameConnection, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(!remaining.is_zero(), "connection stayed open");
        connection.set_read_timeout(Some(remaining)).ok();
        match connection.recv() {
            Ok(Some(_)) => {}
            Ok(None) => return,
            Err(error) => {
                assert!(
                    is_close_evidence(&error),
                    "a timeout/interrupt is not evidence of a close: {error}"
                );
                return;
            }
        }
    }
}

fn server_is_alive(socket: &Path) {
    let mut connection = handshake_ok(socket);
    let result = {
        connection
            .send(&envelope(
                Some(2),
                Body::Request(Request {
                    method: methods::SESSION_LIST.to_string(),
                    operation: Some(request::Operation::SessionList(SessionListRequest {})),
                }),
            ))
            .expect("send session.list");
        recv_response(&mut connection, 2, WAIT)
    };
    assert!(
        matches!(result, response::Result::SessionList(_)),
        "server must still serve after malformed frames, got {result:?}"
    );
}

#[test]
fn incompatible_version_is_rejected_then_closed() {
    let server = TestServer::spawn();
    let mut connection = connect(server.socket_path());
    let result = handshake_version(&mut connection, "2.0");
    match result {
        response::Result::Error(Error { code, .. }) => {
            assert_eq!(code, ErrorCode::IncompatibleVersion as i32);
            assert_eq!(
                contract_str(ErrorCode::IncompatibleVersion),
                "incompatible_version"
            );
        }
        other => panic!("expected incompatible_version, got {other:?}"),
    }
    expect_closed(&mut connection, RETRY);
    step(
        "protocol-1",
        "protocol_version 2.0 -> incompatible_version, then close",
    );
}

#[test]
fn request_before_handshake_is_bad_request() {
    let server = TestServer::spawn();
    let mut connection = connect(server.socket_path());
    connection
        .send(&envelope(
            Some(1),
            Body::Request(Request {
                method: methods::SESSION_LIST.to_string(),
                operation: Some(request::Operation::SessionList(SessionListRequest {})),
            }),
        ))
        .expect("send pre-handshake request");
    expect_error(
        recv_response(&mut connection, 1, WAIT),
        ErrorCode::BadRequest,
    );
    assert_eq!(contract_str(ErrorCode::BadRequest), "bad_request");
    expect_closed(&mut connection, RETRY);
    step(
        "protocol-2",
        "request before handshake -> bad_request, then close",
    );
}

#[test]
fn unknown_method_is_method_not_found() {
    let server = TestServer::spawn();
    let mut connection = handshake_ok(server.socket_path());
    connection
        .send(&envelope(
            Some(2),
            Body::Request(Request {
                method: "bogus.method".to_string(),
                operation: Some(request::Operation::SessionList(SessionListRequest {})),
            }),
        ))
        .expect("send unknown method");
    expect_error(
        recv_response(&mut connection, 2, WAIT),
        ErrorCode::MethodNotFound,
    );
    assert_eq!(contract_str(ErrorCode::MethodNotFound), "method_not_found");
    step("protocol-3", "unknown method -> method_not_found");
}

#[test]
fn bad_and_unknown_session_ids_are_distinguished() {
    let server = TestServer::spawn();
    let mut connection = handshake_ok(server.socket_path());

    // Unparsable id -> bad_request.
    connection
        .send(&envelope(
            Some(2),
            Body::Request(Request {
                method: methods::SESSION_VIEW.to_string(),
                operation: Some(request::Operation::SessionView(SessionSnapshotRequest {
                    session_id: "not-an-id".to_string(),
                })),
            }),
        ))
        .expect("send garbage id");
    expect_error(
        recv_response(&mut connection, 2, WAIT),
        ErrorCode::BadRequest,
    );

    // Well-formed but absent id -> unknown_entity.
    connection
        .send(&envelope(
            Some(3),
            Body::Request(Request {
                method: methods::SESSION_VIEW.to_string(),
                operation: Some(request::Operation::SessionView(SessionSnapshotRequest {
                    session_id: SessionId::new().to_string(),
                })),
            }),
        ))
        .expect("send absent id");
    expect_error(
        recv_response(&mut connection, 3, WAIT),
        ErrorCode::UnknownEntity,
    );
    assert_eq!(contract_str(ErrorCode::UnknownEntity), "unknown_entity");
    step(
        "protocol-4",
        "garbage session id -> bad_request; valid absent id -> unknown_entity",
    );
}

#[test]
fn duplicate_request_id_is_rejected() {
    let server = TestServer::spawn();
    let mut connection = handshake_ok(server.socket_path());
    let body = || {
        Body::Request(Request {
            method: methods::SESSION_LIST.to_string(),
            operation: Some(request::Operation::SessionList(SessionListRequest {})),
        })
    };
    connection
        .send(&envelope(Some(2), body()))
        .expect("first request");
    assert!(
        matches!(
            recv_response(&mut connection, 2, WAIT),
            response::Result::SessionList(_)
        ),
        "first session.list succeeds"
    );
    connection
        .send(&envelope(Some(2), body()))
        .expect("duplicate request");
    expect_error(
        recv_response(&mut connection, 2, WAIT),
        ErrorCode::DuplicateRequest,
    );
    assert_eq!(
        contract_str(ErrorCode::DuplicateRequest),
        "duplicate_request"
    );
    step("protocol-5", "repeated request_id -> duplicate_request");
}

fn assert_raw_closed(socket: &Path, bytes: &[u8], half_close: bool) {
    let mut stream = UnixStream::connect(socket).expect("raw connect");
    stream.write_all(bytes).expect("write raw bytes");
    let _ = stream.flush();
    if half_close {
        let _ = stream.shutdown(Shutdown::Write);
    }
    stream.set_read_timeout(Some(RETRY)).ok();
    let mut buffer = [0u8; 1];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => return,
            Ok(_) => {}
            Err(error) => match error.kind() {
                // A timeout is not a close: the server may still be open.
                std::io::ErrorKind::WouldBlock
                | std::io::ErrorKind::TimedOut
                | std::io::ErrorKind::Interrupted => {
                    panic!("a timeout/interrupt is not evidence of a close: {error}");
                }
                std::io::ErrorKind::UnexpectedEof
                | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::BrokenPipe => return,
                _ => panic!("unexpected read error (not close evidence): {error}"),
            },
        }
    }
}

#[test]
fn oversized_prefix_closes_but_server_survives() {
    let server = TestServer::spawn();
    assert_raw_closed(server.socket_path(), &0x7fff_ffffu32.to_le_bytes(), false);
    server_is_alive(server.socket_path());
    step(
        "protocol-6",
        "0x7fffffff length prefix: connection closed, independent handshake+session.list still works",
    );
}

#[test]
fn zero_length_and_truncated_frames_close_but_server_survives() {
    let server = TestServer::spawn();
    assert_raw_closed(server.socket_path(), &0u32.to_le_bytes(), false);
    let mut truncated = 10u32.to_le_bytes().to_vec();
    truncated.extend_from_slice(&[1, 2, 3]);
    assert_raw_closed(server.socket_path(), &truncated, true);
    server_is_alive(server.socket_path());
    step(
        "protocol-7",
        "zero-length and truncated frames: closed, server alive",
    );
}

#[test]
fn stream_id_zero_is_handled_without_crashing() {
    let server = TestServer::spawn();
    let mut connection = handshake_ok(server.socket_path());
    connection
        .send(&envelope(
            Some(2),
            Body::Request(Request {
                method: methods::TERMINAL_INPUT.to_string(),
                operation: Some(request::Operation::TerminalInput(TerminalInput {
                    stream_id: 0,
                    bytes: b"x".to_vec(),
                })),
            }),
        ))
        .expect("send stream_id=0 input");
    // Documented gap: the server treats stream_id 0 as an unknown stream
    // (`unknown_entity`) rather than a reserved-control `bad_request`. Record
    // the observed contract error instead of inventing a spec.
    let result = recv_response(&mut connection, 2, WAIT);
    match result {
        response::Result::Error(Error { code, .. }) => {
            assert_eq!(
                code,
                ErrorCode::UnknownEntity as i32,
                "observed: stream_id=0 terminal.input -> unknown_entity (documented gap)"
            );
        }
        other => panic!("expected unknown_entity for stream_id=0, got {other:?}"),
    }
    // The server must not have crashed: a fresh request still answers.
    connection
        .send(&envelope(
            Some(3),
            Body::Request(Request {
                method: methods::SESSION_LIST.to_string(),
                operation: Some(request::Operation::SessionList(SessionListRequest {})),
            }),
        ))
        .expect("send follow-up");
    assert!(matches!(
        recv_response(&mut connection, 3, WAIT),
        response::Result::SessionList(_)
    ));
    step(
        "protocol-8",
        "terminal.input stream_id=0 -> unknown_entity, connection and server stay healthy",
    );
}
