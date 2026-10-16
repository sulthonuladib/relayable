//! The main end-to-end scenario: handshake, lifecycle, PTY I/O, resize,
//! disconnect survival, close, and protocol conformance.

use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use relayable_protocol::limits::DEFAULT_LIMITS;
use relayable_protocol::methods;
use relayable_protocol::v1::{
    envelope::Body, request, response, AttachMode, BlockAttachRequest, Envelope, ErrorCode,
    Request, SessionCloseRequest, SessionSnapshotRequest, TerminalInput, TerminalResize,
};
use relayable_protocol::PROTOCOL_VERSION;

use crate::harness::*;

#[test]
fn server_end_to_end() {
    let mut server = ServerGuard::spawn();

    // (a) handshake
    let mut client = Client::new(server.connect());
    let hello = client.handshake();
    assert_eq!(hello.protocol_version, "1.0");
    assert_eq!(hello.limits, Some(DEFAULT_LIMITS));

    // (b) session.create
    let created = create(&mut client, "e2e");
    let session_id = created
        .session
        .as_ref()
        .expect("session record")
        .session_id
        .clone();
    let window_id = created.window_id.clone();
    let block_id = created.block_id.clone();
    let pty_process_id = created.pty_process_id.clone();
    let program_pid = created.program_pid;
    assert_tagged_uuid_v7(&session_id, "session");
    assert_tagged_uuid_v7(&window_id, "window");
    assert_tagged_uuid_v7(&block_id, "block");
    assert_tagged_uuid_v7(&pty_process_id, "pty");
    assert!(program_pid > 0, "program_pid must be a real pid");
    server.track(program_pid);

    // (c) session.list
    match client.request(
        methods::SESSION_LIST,
        request::Operation::SessionList(relayable_protocol::v1::SessionListRequest {}),
    ) {
        response::Result::SessionList(list) => {
            assert!(
                list.sessions
                    .iter()
                    .any(|summary| summary.session_id == session_id),
                "session.list must include the new session"
            );
        }
        other => panic!("session.list failed: {other:?}"),
    }

    // (d) session.view
    let snapshot = view(&mut client, &session_id);
    assert!(snapshot.sequence >= 1);
    let session = snapshot.session.as_ref().expect("session");
    assert_eq!(session.session_id, session_id);
    assert_eq!(
        session.state,
        relayable_protocol::v1::SessionState::Active as i32
    );
    assert_eq!(session.revision, 1);
    assert_eq!(snapshot.windows.len(), 1);
    assert_eq!(snapshot.windows[0].window_id, window_id);
    assert_eq!(snapshot.blocks.len(), 1);
    let block = &snapshot.blocks[0];
    assert_eq!(block.block_id, block_id);
    assert_eq!(block.cols, 80);
    assert_eq!(block.rows, 24);
    assert_eq!(block.pty_process_id, pty_process_id);
    assert_eq!(block.program_pid, program_pid);
    assert_eq!(
        block.program_status.as_ref().expect("status").state,
        relayable_protocol::v1::ProgramState::Running as i32
    );
    assert!(
        !snapshot.clients.is_empty(),
        "the connected client is listed"
    );

    // (e) block.attach (writer)
    let attached = attach(&mut client, &block_id, AttachMode::Writer);
    let stream_id = attached.stream_id;
    assert!(stream_id >= 1, "stream ids start at 1");
    assert_eq!(attached.mode, AttachMode::Writer as i32);
    assert!(attached.writer_lease);
    let attached_snapshot = attached.snapshot.expect("terminal snapshot");
    assert_eq!(attached_snapshot.cols, 80);
    assert_eq!(attached_snapshot.rows, 24);

    // (f) terminal I/O round-trip proves a real PTY
    assert!(input(&mut client, stream_id, "tty\n") > 0);
    client.read_until("/dev/pts/", Instant::now() + WAIT);

    // (g) stty sees the initial size
    input(&mut client, stream_id, "stty size\n");
    client.read_until("24 80", Instant::now() + WAIT);

    // (h) resize, then stty and the snapshot both report 100x40
    match client.request(
        methods::TERMINAL_RESIZE,
        request::Operation::TerminalResize(TerminalResize {
            stream_id,
            cols: 100,
            rows: 40,
            pixel_width: None,
            pixel_height: None,
        }),
    ) {
        response::Result::TerminalResize(resized) => {
            assert_eq!(resized.cols, 100);
            assert_eq!(resized.rows, 40);
        }
        other => panic!("terminal.resize failed: {other:?}"),
    }
    input(&mut client, stream_id, "stty size\n");
    client.read_until("40 100", Instant::now() + WAIT);
    let resized_snapshot = view(&mut client, &session_id);
    assert_eq!(resized_snapshot.blocks[0].cols, 100);
    assert_eq!(resized_snapshot.blocks[0].rows, 40);

    // (i) disconnect survival: set a shell variable, drop the connection, then
    // reconnect and prove the SAME shell process (and variable) survived.
    let nonce = format!("survival-{}", UNIQUE.fetch_add(1, Ordering::SeqCst));
    input(
        &mut client,
        stream_id,
        &format!("RELAYABLE_SURVIVAL={nonce}\n"),
    );
    // Non-echoable confirmation that the shell stored the variable: the echoed
    // input line contains `SURV-%s`, not the expanded value.
    input(
        &mut client,
        stream_id,
        "printf 'SURV-%s\\n' \"$RELAYABLE_SURVIVAL\"\n",
    );
    client.read_until(&format!("SURV-{nonce}"), Instant::now() + WAIT);
    drop(client); // socket close

    // (i2) the child is still alive after the client disconnect
    assert!(
        proc_exists(program_pid),
        "the PTY child must survive the client disconnect"
    );

    let mut client2 = Client::new(server.connect());
    client2.handshake();
    let snapshot2 = view(&mut client2, &session_id);
    assert_eq!(
        snapshot2.session.as_ref().expect("session").session_id,
        session_id
    );
    assert_eq!(
        snapshot2.session.as_ref().expect("session").state,
        relayable_protocol::v1::SessionState::Active as i32
    );
    let block2 = &snapshot2.blocks[0];
    assert_eq!(block2.block_id, block_id);
    assert_eq!(block2.pty_process_id, pty_process_id);
    assert_eq!(block2.program_pid, program_pid);
    assert_eq!(
        block2.program_status.as_ref().expect("status").state,
        relayable_protocol::v1::ProgramState::Running as i32
    );

    // The lease is freed asynchronously once the server notices EOF; retry.
    let attach_deadline = Instant::now() + RETRY;
    let stream2 = loop {
        match client2.request(
            methods::BLOCK_ATTACH,
            request::Operation::BlockAttach(BlockAttachRequest {
                block_id: block_id.clone(),
                mode: AttachMode::Writer as i32,
                takeover: false,
            }),
        ) {
            response::Result::BlockAttach(attached) => break attached.stream_id,
            response::Result::Error(error)
                if error.code == ErrorCode::InputNotOwned as i32
                    && Instant::now() < attach_deadline =>
            {
                thread::sleep(Duration::from_millis(50));
            }
            other => panic!("post-reconnect attach failed: {other:?}"),
        }
    };
    assert!(stream2 >= 1);
    assert!(proc_exists(program_pid));
    input(&mut client2, stream2, "echo $RELAYABLE_SURVIVAL\n");
    client2.read_until(&nonce, Instant::now() + WAIT);

    // (j) session.close stops the child and removes the session from list
    match client2.request(
        methods::SESSION_CLOSE,
        request::Operation::SessionClose(SessionCloseRequest {
            session_id: session_id.clone(),
        }),
    ) {
        response::Result::SessionClose(closed) => {
            assert_eq!(
                closed.session.expect("closed session").state,
                relayable_protocol::v1::SessionState::Closed as i32
            );
        }
        other => panic!("session.close failed: {other:?}"),
    }
    match client2.request(
        methods::SESSION_LIST,
        request::Operation::SessionList(relayable_protocol::v1::SessionListRequest {}),
    ) {
        response::Result::SessionList(list) => {
            assert!(
                !list
                    .sessions
                    .iter()
                    .any(|summary| summary.session_id == session_id),
                "closed sessions must not appear in session.list"
            );
        }
        other => panic!("session.list failed: {other:?}"),
    }
    assert!(
        wait_gone(program_pid, RETRY),
        "the PTY child must be gone after session.close"
    );

    // (k) protocol conformance.

    // Oversized length prefix: rejected before any body and the connection is
    // closed without a server panic.
    let mut raw = UnixStream::connect(&server.socket).expect("raw connect");
    raw.write_all(&(DEFAULT_LIMITS.max_frame_bytes + 1).to_le_bytes())
        .expect("write oversized prefix");
    raw.flush().ok();
    raw.set_read_timeout(Some(RETRY)).ok();
    let mut byte = [0u8; 1];
    match raw.read(&mut byte) {
        Ok(0) | Err(_) => {}
        Ok(_) => panic!("server answered an oversized frame"),
    }

    // Zero-length frame: closed.
    let mut raw = UnixStream::connect(&server.socket).expect("raw connect");
    raw.write_all(&0u32.to_le_bytes())
        .expect("write zero prefix");
    raw.flush().ok();
    raw.set_read_timeout(Some(RETRY)).ok();
    match raw.read(&mut byte) {
        Ok(0) | Err(_) => {}
        Ok(_) => panic!("server answered a zero-length frame"),
    }

    // Truncated body: closed.
    let mut raw = UnixStream::connect(&server.socket).expect("raw connect");
    raw.write_all(&10u32.to_le_bytes()).expect("write prefix");
    raw.write_all(&[1, 2, 3]).expect("write partial body");
    raw.shutdown(Shutdown::Write).expect("half-close");
    raw.set_read_timeout(Some(RETRY)).ok();
    match raw.read(&mut byte) {
        Ok(0) | Err(_) => {}
        Ok(_) => panic!("server answered a truncated frame"),
    }

    // The server is still alive and serving after the malformed frames.
    let mut probe = Client::new(server.connect());
    probe.handshake();
    match probe.request(
        methods::SESSION_LIST,
        request::Operation::SessionList(relayable_protocol::v1::SessionListRequest {}),
    ) {
        response::Result::SessionList(_) => {}
        other => panic!("server unhealthy after malformed frames: {other:?}"),
    }

    // Unknown method string -> METHOD_NOT_FOUND.
    let mut unknown_method = Client::new(server.connect());
    unknown_method.handshake();
    let unknown_id = unknown_method.next_id;
    unknown_method.next_id += 1;
    unknown_method.send(&Envelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        request_id: Some(unknown_id),
        stream_id: 0,
        sequence: None,
        trace_id: None,
        timestamp_unix_millis: None,
        body: Some(Body::Request(Request {
            method: "bogus.method".to_string(),
            operation: None,
        })),
    });
    let result = loop {
        let envelope = unknown_method
            .recv_frame(Instant::now() + WAIT)
            .expect("response");
        if envelope.request_id == Some(unknown_id) {
            if let Some(Body::Response(response)) = envelope.body {
                break response.result.expect("result");
            }
        }
    };
    expect_error(result, ErrorCode::MethodNotFound);

    // Unknown session id -> UNKNOWN_ENTITY.
    let missing_session = relayable_model::SessionId::new().to_string();
    let mut unknown_session = Client::new(server.connect());
    unknown_session.handshake();
    expect_error(
        unknown_session.request(
            methods::SESSION_VIEW,
            request::Operation::SessionView(SessionSnapshotRequest {
                session_id: missing_session,
            }),
        ),
        ErrorCode::UnknownEntity,
    );

    // Writer-lease conflict and observer input rejection.
    let mut holder = Client::new(server.connect());
    holder.handshake();
    let held = create(&mut holder, "lease");
    let held_session = held.session.as_ref().expect("session").session_id.clone();
    let held_block = held.block_id.clone();
    let held_pid = held.program_pid;
    server.track(held_pid);
    let held_stream = attach(&mut holder, &held_block, AttachMode::Writer).stream_id;

    let mut other = Client::new(server.connect());
    other.handshake();
    expect_error(
        other.request(
            methods::BLOCK_ATTACH,
            request::Operation::BlockAttach(BlockAttachRequest {
                block_id: held_block.clone(),
                mode: AttachMode::Writer as i32,
                takeover: false,
            }),
        ),
        ErrorCode::InputNotOwned,
    );
    let observer = attach(&mut other, &held_block, AttachMode::Observer);
    assert_eq!(observer.mode, AttachMode::Observer as i32);
    assert!(!observer.writer_lease);
    expect_error(
        other.request(
            methods::TERMINAL_INPUT,
            request::Operation::TerminalInput(TerminalInput {
                stream_id: observer.stream_id,
                bytes: b"whoami\n".to_vec(),
            }),
        ),
        ErrorCode::InputNotOwned,
    );

    // Close the lease session so the cleanup guard has nothing live.
    match holder.request(
        methods::SESSION_CLOSE,
        request::Operation::SessionClose(SessionCloseRequest {
            session_id: held_session,
        }),
    ) {
        response::Result::SessionClose(_) => {}
        other => panic!("close lease session failed: {other:?}"),
    }
    assert!(wait_gone(held_pid, RETRY));
    let _ = held_stream;
}
