use std::os::unix::net::UnixListener;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use relayable_model::{BlockId, SessionId};
use relayable_protocol::ids;
use relayable_protocol::limits::DEFAULT_LIMITS;
use relayable_protocol::transport::FrameConnection;
use relayable_protocol::v1::{
    envelope::Body, response::Result as ResponseResult, AttachMode, Error, ErrorCode, ServerHello,
};
use relayable_protocol::PROTOCOL_VERSION;

use crate::client::Client;
use crate::config::ClientConfig;
use crate::error::ClientError;
use crate::rpc::contains;

use super::harness::*;

#[test]
fn handshake_captures_hello_and_limits() {
    let (client, server) = client_connected();
    assert_eq!(client.server_hello().capabilities, vec!["test".to_string()]);
    assert_eq!(client.limits().max_frame_bytes, 4096);
    assert_eq!(client.socket_path(), Path::new("/tmp/relayable-test.sock"));
    drop(client);
    let _ = server.join();
}

#[test]
fn list_sessions_returns_the_server_list() {
    let (mut client, server) = client_connected();
    let sessions = client.list_sessions().unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].label.as_deref(), Some("demo"));
    drop(client);
    let _ = server.join();
}

/// A real listening Unix socket whose peer completes the handshake and then
/// never answers a control RPC must not hang the caller: `list_sessions`
/// must return [`ClientError::Timeout`] inside a bounded window. The
/// production deadline is shortened in-test so the assertion is fast.
#[test]
fn control_rpc_times_out_when_the_server_never_replies() {
    let dir = std::env::temp_dir().join(format!(
        "relayable-client-rpc-timeout-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("relayable.sock");
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).unwrap();

    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut conn = FrameConnection::from_stream(stream, DEFAULT_LIMITS.max_frame_bytes);

        // Complete the handshake so the client gets past `connect`.
        let handshake = conn.recv().unwrap().unwrap();
        let request_id = handshake.request_id.unwrap();
        conn.send(&response_for(
            request_id,
            ResponseResult::Hello(ServerHello {
                protocol_version: PROTOCOL_VERSION.to_string(),
                capabilities: Vec::new(),
                limits: None,
            }),
        ))
        .unwrap();

        // Consume the next request but deliberately never answer it, and
        // outlive the client's shortened deadline before closing.
        let request = conn.recv().unwrap().unwrap();
        assert!(matches!(request.body, Some(Body::Request(_))));
        thread::sleep(Duration::from_secs(2));
        drop(conn);
    });

    let mut client = Client::connect(&path, ClientConfig::new("test")).unwrap();
    client.control_rpc_timeout = Duration::from_millis(250);

    let started = Instant::now();
    let err = client.list_sessions().unwrap_err();
    let elapsed = started.elapsed();

    assert!(matches!(err, ClientError::Timeout), "got {err:?}");
    assert!(
        elapsed >= Duration::from_millis(200),
        "returned before the deadline: {elapsed:?}"
    );
    assert!(
        elapsed < Duration::from_secs(2),
        "did not respect the deadline: {elapsed:?}"
    );

    drop(client);
    let _ = server.join();
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn server_error_code_is_exposed() {
    let err = ClientError::Server(Error {
        code: ErrorCode::UnknownEntity as i32,
        message: "nope".to_string(),
        details: std::collections::HashMap::new(),
        retryable: false,
    });
    assert_eq!(err.server_error_code(), Some(ErrorCode::UnknownEntity));
    assert!(err.to_string().contains("unknown_entity"));
    assert!(ClientError::Timeout.server_error_code().is_none());
}

#[test]
fn contains_matches_across_chunk_boundaries() {
    assert!(contains(b"hello world", b"lo wo"));
    assert!(!contains(b"hello", b"world"));
    assert!(!contains(b"", b"x"));
}

#[test]
fn attach_sets_takeover_false_and_attach_with_takeover_sets_true() {
    let session = SessionId::new();
    let block = BlockId::new();

    let (mut client, server, rx) = attach_server();
    let response = client.attach(&session, &block, AttachMode::Writer).unwrap();
    assert_eq!(response.stream_id, 7);
    assert_eq!(response.mode, AttachMode::Writer as i32);
    assert!(response.writer_lease);
    let request = block_attach_of(rx.recv().unwrap());
    assert!(!request.takeover, "plain attach must not take over");
    assert_eq!(request.block_id, ids::block_id_to_wire(&block));
    drop(client);
    let _ = server.join();

    let (mut client, server, rx) = attach_server();
    let response = client
        .attach_with_takeover(&session, &block, AttachMode::Writer, true)
        .unwrap();
    assert_eq!(response.stream_id, 7);
    let request = block_attach_of(rx.recv().unwrap());
    assert!(request.takeover, "attach_with_takeover must set takeover");
    assert_eq!(request.block_id, ids::block_id_to_wire(&block));
    drop(client);
    let _ = server.join();
}

#[test]
fn resume_session_encodes_the_last_applied_sequence() {
    let session = SessionId::new();

    let (mut client, server, rx) = resume_server();
    client.resume_session(&session, None).unwrap();
    let request = session_resume_of(rx.recv().unwrap());
    assert_eq!(
        request.last_sequence_applied, 0,
        "None is the no-cursor sentinel"
    );
    assert_eq!(request.session_id, ids::session_id_to_wire(&session));
    drop(client);
    let _ = server.join();

    let (mut client, server, rx) = resume_server();
    client.resume_session(&session, Some(41)).unwrap();
    let request = session_resume_of(rx.recv().unwrap());
    assert_eq!(request.last_sequence_applied, 41);
    drop(client);
    let _ = server.join();
}
