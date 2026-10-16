//! Relayable client library.
//!
//! This crate owns the connection lifecycle, the synchronous request/response
//! RPC surface, and terminal-stream reads over the local Unix-domain socket
//! transport. It depends on `relayable-model` and `relayable-protocol` only and
//! never on server internals (ADR-001 §7).
//!
//! # Concurrency
//!
//! The client is **synchronous and blocking**: it uses `std::io` over
//! `std::os::unix::net`, with no async runtime (`docs/adr/ADR-002-protocol-schema-and-transport.md`;
//! `full.md` §5.2). A dropped socket is surfaced as [`ClientError::Disconnected`]
//! on the next call; no I/O or protocol path panics.
//!
//! # Handshake (frozen by the CORE-001 slice note)
//!
//! The first frame on a connection is
//! `Envelope{ protocol_version: "1.0", request_id: Some(1), stream_id: 0,
//! body: Handshake{..} }`. The server echoes `request_id = 1` with
//! `Response.result = Hello`. Ordinary request ids then start at `2` and
//! increment monotonically per connection. Every control request uses
//! `stream_id = 0` (`relayable_protocol::CONTROL_STREAM_ID`).
//!
//! # Control RPC deadlines
//!
//! Every control RPC (and the initial handshake) waits at most
//! [`CONTROL_RPC_TIMEOUT`] for its response and then fails with
//! [`ClientError::Timeout`] instead of blocking forever. [`Client::wait_for_output`]
//! instead uses its caller-supplied timeout.
//!
//! # Re-exports
//!
//! Protocol types that appear in this crate's public API are re-exported so a
//! consumer such as the future `relayable-automation` (ADR-001
//! `automation -> {model, client}`) does not need a direct
//! `relayable-protocol` dependency:
//!
//! ```
//! use relayable_client::{
//!     AttachMode, Error, ErrorCode, Limits, ServerHello, SessionRecord,
//!     SessionSnapshotResponse, SessionSummary, TerminalSnapshot, TransportError,
//! };
//! let _ = AttachMode::Writer;
//! let _ = ErrorCode::Internal;
//! let _ = ServerHello::default();
//! let _ = SessionRecord::default();
//! let _ = SessionSnapshotResponse::default();
//! let _ = SessionSummary::default();
//! let _ = TerminalSnapshot::default();
//! let _ = Limits::default();
//! let _: Option<Error> = None;
//! fn _accepts_transport(_: TransportError) {}
//! ```

#![deny(unsafe_code)]

mod client;
mod config;
mod error;
mod outcome;
mod rpc;

pub use client::Client;
pub use config::{ClientConfig, CONTROL_RPC_TIMEOUT, MAX_PENDING_EVENTS};
pub use error::ClientError;
pub use outcome::{AttachOutcome, CreatedSession};

/// Protocol types that appear in this crate's public API, re-exported so
/// consumers (`relayable-automation`: ADR-001 `automation -> {model, client}`)
/// do not need a direct `relayable-protocol` dependency.
pub use relayable_protocol::transport::TransportError;
pub use relayable_protocol::v1::{
    AttachMode, BlockAttachResponse, Error, ErrorCode, Event, Limits, ServerHello, SessionRecord,
    SessionResumeResponse, SessionSnapshotResponse, SessionSummary, TerminalSnapshot,
};

#[cfg(test)]
mod tests {
    use super::*;
    use relayable_model::{BlockId, SessionId};
    use relayable_protocol::codec::CodecError;
    use relayable_protocol::ids;
    use relayable_protocol::limits::DEFAULT_LIMITS;
    use relayable_protocol::transport::FrameConnection;
    use relayable_protocol::v1::{
        envelope::Body, event, request::Operation, response::Result as ResponseResult,
        BlockAttachRequest, ClientConnectionStateChanged, Envelope, Request, Response,
        SessionListResponse, SessionResumeRequest, SessionState, TerminalOutput,
    };
    use relayable_protocol::{CONTROL_STREAM_ID, PROTOCOL_VERSION};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::{Path, PathBuf};
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    use crate::error::is_timeout;
    use crate::rpc::contains;
    fn response_for(request_id: u64, result: ResponseResult) -> Envelope {
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
    fn recv_or_finished(conn: &mut FrameConnection) -> Option<Envelope> {
        match conn.recv() {
            Ok(Some(envelope)) => Some(envelope),
            Ok(None) | Err(_) => None,
        }
    }

    fn fake_server(mut conn: FrameConnection) {
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

    fn client_connected() -> (Client, thread::JoinHandle<()>) {
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
    fn complete_handshake(conn: &mut FrameConnection, limits: Limits) -> bool {
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
    fn spawn_scripted_server<F>(script: F) -> (Client, thread::JoinHandle<()>)
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
    fn spawn_scripted_server_with_limits<F>(
        limits: Limits,
        script: F,
    ) -> (Client, thread::JoinHandle<()>)
    where
        F: FnOnce(&mut FrameConnection) + Send + 'static,
    {
        let (client_stream, server_stream) = UnixStream::pair().unwrap();
        let server = thread::spawn(move || {
            let mut conn =
                FrameConnection::from_stream(server_stream, DEFAULT_LIMITS.max_frame_bytes);
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

    fn block_attach_response() -> BlockAttachResponse {
        BlockAttachResponse {
            stream_id: 7,
            mode: AttachMode::Writer as i32,
            writer_lease: true,
            snapshot: Some(TerminalSnapshot::default()),
        }
    }

    fn block_attach_of(envelope: Envelope) -> BlockAttachRequest {
        match envelope.body {
            Some(Body::Request(Request {
                operation: Some(Operation::BlockAttach(request)),
                ..
            })) => request,
            other => panic!("expected a block.attach request, got {other:?}"),
        }
    }

    fn session_resume_of(envelope: Envelope) -> SessionResumeRequest {
        match envelope.body {
            Some(Body::Request(Request {
                operation: Some(Operation::SessionResume(request)),
                ..
            })) => request,
            other => panic!("expected a session.resume request, got {other:?}"),
        }
    }

    fn event_envelope(sequence: u64) -> Envelope {
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
    fn attach_server() -> (Client, thread::JoinHandle<()>, mpsc::Receiver<Envelope>) {
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
    fn resume_server() -> (Client, thread::JoinHandle<()>, mpsc::Receiver<Envelope>) {
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
    fn terminal_data(stream_id: u64, stream_seq: u64, bytes: &[u8]) -> Envelope {
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
    fn is_transport_timeout(err: &TransportError) -> bool {
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
    fn spawn_output_server(
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

    /// CLIENT-API-001 rule 1: repeated `next_output` calls return the target
    /// stream's frames exactly once each, in arrival order, with no byte loss.
    #[test]
    fn next_output_returns_successive_target_frames_without_loss() {
        let target = 7u64;
        let (mut client, server, rx) = spawn_output_server(
            vec![
                terminal_data(target, 1, b"first-"),
                terminal_data(target, 2, b"second"),
            ],
            Duration::from_millis(500),
        );
        client
            .attach_block(&BlockId::new(), AttachMode::Writer)
            .unwrap();

        let first = client
            .next_output(target, Duration::from_millis(500))
            .unwrap()
            .expect("first target frame");
        let second = client
            .next_output(target, Duration::from_millis(500))
            .unwrap()
            .expect("second target frame");
        assert_eq!(first, b"first-");
        assert_eq!(second, b"second");
        let mut joined = first;
        joined.extend_from_slice(&second);
        assert_eq!(joined, b"first-second", "no byte may be lost between calls");

        drop(client);
        let _ = server.join();
        drop(rx);
    }

    /// CLIENT-API-001 rule 3: a normal timeout is `Ok(None)`, never
    /// [`ClientError::Timeout`].
    #[test]
    fn next_output_returns_none_on_timeout() {
        let target = 7u64;
        let (mut client, server, rx) = spawn_output_server(Vec::new(), Duration::from_millis(300));
        client
            .attach_block(&BlockId::new(), AttachMode::Writer)
            .unwrap();

        let result = client
            .next_output(target, Duration::from_millis(50))
            .expect("a timeout is not an error");
        assert!(result.is_none(), "expected Ok(None), got {result:?}");

        drop(client);
        let _ = server.join();
        drop(rx);
    }

    /// CLIENT-API-001 rule 5: `Duration::ZERO` serves frames already buffered in
    /// `pending` (deferred by an earlier call) and returns `Ok(None)` promptly
    /// with an empty queue, never reading the socket.
    #[test]
    fn zero_timeout_serves_buffered_frames_without_blocking() {
        let target = 7u64;
        let (mut client, server, rx) = spawn_output_server(
            vec![
                terminal_data(target, 1, b"buffered-bytes"),
                event_envelope(21),
            ],
            Duration::from_millis(500),
        );
        client
            .attach_block(&BlockId::new(), AttachMode::Writer)
            .unwrap();

        // `wait_for_event` reads the target data frame first (deferring it into
        // `pending`) and then the event, so the bytes are only reachable through
        // public API.
        let event = client
            .wait_for_event(Duration::from_millis(500))
            .unwrap()
            .expect("buffered event");
        assert_eq!(event.sequence, 21);

        let started = Instant::now();
        let bytes = client
            .next_output(target, Duration::ZERO)
            .unwrap()
            .expect("buffered target frame");
        assert_eq!(bytes, b"buffered-bytes");
        assert!(
            started.elapsed() < Duration::from_millis(250),
            "a zero-timeout drain must not block: {:?}",
            started.elapsed()
        );

        // With an empty queue the same call returns promptly without a read.
        let started = Instant::now();
        let nothing = client.next_output(target, Duration::ZERO).unwrap();
        assert!(nothing.is_none());
        assert!(
            started.elapsed() < Duration::from_millis(250),
            "an empty zero-timeout call must not block: {:?}",
            started.elapsed()
        );

        drop(client);
        let _ = server.join();
        drop(rx);
    }

    /// CLIENT-API-001 rule 2: every returned chunk is acknowledged exactly once
    /// with `ack_bytes == chunk.len()`, and the totals agree.
    #[test]
    fn one_flow_control_ack_per_returned_chunk() {
        let target = 7u64;
        let chunks: &[&[u8]] = &[b"aaaa", b"bb", b"cccccc"];
        let script = chunks
            .iter()
            .enumerate()
            .map(|(index, bytes)| terminal_data(target, index as u64 + 1, bytes))
            .collect();
        let (mut client, server, rx) = spawn_output_server(script, Duration::from_millis(500));
        client
            .attach_block(&BlockId::new(), AttachMode::Writer)
            .unwrap();

        let mut returned: Vec<Vec<u8>> = Vec::new();
        for _ in 0..chunks.len() {
            returned.push(
                client
                    .next_output(target, Duration::from_millis(500))
                    .unwrap()
                    .expect("scripted frame"),
            );
        }
        drop(client);

        let mut acks = Vec::new();
        while let Ok(envelope) = rx.recv_timeout(Duration::from_millis(500)) {
            if let Some(Body::FlowControl(flow)) = envelope.body {
                acks.push(flow);
            }
        }
        let _ = server.join();

        assert_eq!(
            acks.len(),
            chunks.len(),
            "exactly one ack per returned chunk"
        );
        for (chunk, ack) in returned.iter().zip(&acks) {
            assert_eq!(ack.stream_id, target);
            assert_eq!(ack.ack_bytes, chunk.len() as u64);
        }
        let returned_total: u64 = returned.iter().map(|chunk| chunk.len() as u64).sum();
        let acked_total: u64 = acks.iter().map(|ack| ack.ack_bytes).sum();
        assert_eq!(acked_total, returned_total);
    }

    /// CLIENT-API-001 rule 2 for a zero-length frame: the protocol layer does
    /// not reject `ack_bytes == 0` (nothing validates a `FlowControl` body; the
    /// server's `add_credit` is a no-op for zero), so the ack is still sent
    /// exactly once and the "one ack per returned chunk" invariant holds.
    #[test]
    fn zero_length_chunk_is_acked_exactly_once() {
        let target = 7u64;
        let (mut client, server, rx) = spawn_output_server(
            vec![terminal_data(target, 1, b"")],
            Duration::from_millis(500),
        );
        client
            .attach_block(&BlockId::new(), AttachMode::Writer)
            .unwrap();

        let chunk = client
            .next_output(target, Duration::from_millis(500))
            .unwrap()
            .expect("zero-length frame");
        assert!(chunk.is_empty());
        drop(client);

        let mut acks = Vec::new();
        while let Ok(envelope) = rx.recv_timeout(Duration::from_millis(500)) {
            if let Some(Body::FlowControl(flow)) = envelope.body {
                acks.push(flow);
            }
        }
        let _ = server.join();
        assert_eq!(acks.len(), 1, "exactly one ack for the empty chunk");
        assert_eq!(acks[0].stream_id, target);
        assert_eq!(acks[0].ack_bytes, 0);
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
}
