//! Session/server lifecycle: graceful shutdown, descendant termination, and the
//! removal of a disconnected client record.

use std::thread;
use std::time::{Duration, Instant};

use relayable_protocol::methods;
use relayable_protocol::v1::{envelope::Body, request, response, AttachMode, SessionCloseRequest};

use crate::harness::*;

#[test]
fn graceful_shutdown_kills_live_children() {
    let mut server = ServerGuard::spawn();
    let mut client = Client::new(server.connect());
    client.handshake();
    let created = create(&mut client, "shutdown");
    let pid = created.program_pid;
    server.track(pid);
    assert!(proc_exists(pid), "child must be alive before shutdown");

    // SIGTERM the *foreground* server: its handler must run the graceful
    // shutdown path, which kills every live child before exiting.
    let server_pid = server.child.id();
    // SAFETY: signalling a process this test started.
    unsafe {
        libc::kill(server_pid as libc::pid_t, libc::SIGTERM);
    }
    let deadline = Instant::now() + WAIT;
    loop {
        match server.child.try_wait().expect("try_wait") {
            Some(_) => break,
            None if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
            None => panic!("server did not exit after SIGTERM"),
        }
    }
    assert!(
        wait_gone(pid, RETRY),
        "graceful shutdown must kill the live PTY child"
    );
}

/// M1: a descendant that inherited the PTY slave must not keep `session.close`
/// (or server shutdown) blocked. `/bin/sh` here enables job control, so the
/// background `sleep` is in its own process group but the child's session; the
/// server must terminate the whole session tree.
#[test]
fn session_close_terminates_descendants_holding_the_slave() {
    let mut server = ServerGuard::spawn();
    let mut client = Client::new(server.connect());
    client.handshake();
    let created = create(&mut client, "m1");
    let session_id = created
        .session
        .as_ref()
        .expect("session")
        .session_id
        .clone();
    let block_id = created.block_id.clone();
    let child_pid = created.program_pid;
    server.track(child_pid);
    let stream_id = attach(&mut client, &block_id, AttachMode::Writer).stream_id;

    // Start a background descendant that keeps the PTY slave open, and print its
    // pid so we can assert it dies too. The echoed command contains `DESC=%s`,
    // so we wait for the *expanded* `DESC=<digits>` output.
    input(
        &mut client,
        stream_id,
        "sleep 300 & printf 'DESC=%s\\n' \"$!\"\n",
    );
    let deadline = Instant::now() + WAIT;
    let descendant_pid = loop {
        if let Some(pid) = parse_descendant_pid(&client.output) {
            break pid;
        }
        assert!(
            Instant::now() < deadline,
            "no descendant pid in output: {:?}",
            String::from_utf8_lossy(&client.output)
        );
        if let Some(envelope) = client.recv_frame(deadline) {
            if let Some(Body::TerminalData(output)) = envelope.body {
                client.output.extend_from_slice(&output.bytes);
            }
        }
    };
    server.track(descendant_pid);
    assert!(proc_exists(descendant_pid));
    assert!(proc_exists(child_pid));

    let started = Instant::now();
    match client.request(
        methods::SESSION_CLOSE,
        request::Operation::SessionClose(SessionCloseRequest {
            session_id: session_id.clone(),
        }),
    ) {
        response::Result::SessionClose(closed) => assert_eq!(
            closed.session.expect("closed session").state,
            relayable_protocol::v1::SessionState::Closed as i32
        ),
        other => panic!("session.close failed: {other:?}"),
    }
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(5),
        "session.close took {elapsed:?}"
    );
    assert!(wait_gone(child_pid, RETRY), "direct child must be gone");
    assert!(
        wait_gone(descendant_pid, RETRY),
        "descendant holding the slave must be gone"
    );
}

/// CORE-002: disconnect emits the transition event pair and then removes the
/// client record, so a snapshot lists live clients only (CORE-001 semantics),
/// while the session and child survive.
#[test]
fn disconnected_client_record_is_removed() {
    let mut server = ServerGuard::spawn();
    let mut x = Client::new(server.connect());
    x.handshake();
    let created = create(&mut x, "removed");
    let session_id = created
        .session
        .as_ref()
        .expect("session")
        .session_id
        .clone();
    let program_pid = created.program_pid;
    server.track(program_pid);

    let before = view(&mut x, &session_id);
    let x_id = before
        .clients
        .iter()
        .find(|client| client.kind == "e2e")
        .map(|client| client.client_id.clone())
        .expect("the connected client is listed in the snapshot");
    drop(x);

    let mut y = Client::new(server.connect());
    y.handshake();
    let deadline = Instant::now() + RETRY;
    let mut removed = false;
    loop {
        let snapshot = view(&mut y, &session_id);
        if snapshot
            .clients
            .iter()
            .all(|client| client.client_id != x_id)
        {
            removed = true;
            break;
        }
        if Instant::now() >= deadline {
            break;
        }
        thread::sleep(Duration::from_millis(30));
    }
    assert!(
        removed,
        "the disconnected client record must be removed from the snapshot"
    );
    assert!(
        proc_exists(program_pid),
        "the session and child must survive the client disconnect"
    );
}
