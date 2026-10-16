//! Request-retry (`duplicate_request`) safety scenario.

use anyhow::Result;
use relayable_client::{Client, ClientConfig};
use relayable_integration_tests::harness::{step, TestServer};
use relayable_protocol::methods;
use relayable_protocol::v1::{self, envelope::Body, request, response, ErrorCode, Request};

use crate::frames::{raw_connect, raw_envelope, raw_handshake, raw_response, resume_envelope};

/// §D/D8 retry-safety: replaying the exact same `session.resume` request id is
/// answered with `duplicate_request` (not re-run), and the connection's stream
/// stays healthy afterwards.
#[test]
fn replayed_resume_request_returns_the_original_outcome() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut setup = Client::connect(&socket, ClientConfig::new("dup-setup"))?;
    let created = setup.create_session(Some("dup"))?;
    let session_id = created.session.session_id.clone();
    server.track_child(created.program_pid);

    let mut conn = raw_connect(&socket);
    raw_handshake(&mut conn);

    // First resume (request id 2) succeeds and subscribes the connection.
    conn.send(&resume_envelope(2, &session_id, 1))
        .expect("first resume");
    let first = raw_response(&mut conn, 2);
    assert!(
        matches!(first, response::Result::SessionResume(_)),
        "the first resume must succeed, got {first:?}"
    );

    // The exact same request id replayed: the dedup path must answer
    // duplicate_request instead of running resume again.
    conn.send(&resume_envelope(2, &session_id, 1))
        .expect("replayed resume");
    let second = raw_response(&mut conn, 2);
    match second {
        response::Result::Error(error) => assert_eq!(
            error.code,
            ErrorCode::DuplicateRequest as i32,
            "second frame must be duplicate_request, got {error:?}"
        ),
        other => panic!("a replayed request id must be duplicate_request, got {other:?}"),
    }

    // The server did not corrupt the connection: a fresh request id answers.
    conn.send(&raw_envelope(
        Some(3),
        Body::Request(Request {
            method: methods::SESSION_LIST.to_string(),
            operation: Some(request::Operation::SessionList(v1::SessionListRequest {})),
        }),
    ))
    .expect("follow-up session.list");
    match raw_response(&mut conn, 3) {
        response::Result::SessionList(_) => {}
        other => panic!("the stream was corrupted after a duplicate resume: {other:?}"),
    }
    step(
        "events-7",
        "same resume request id twice -> duplicate_request; follow-up session.list still answers",
    );
    Ok(())
}
