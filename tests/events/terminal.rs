//! Terminal child-exit event scenario.

use std::time::Duration;

use anyhow::{Context, Result};
use relayable_client::{Client, ClientConfig};
use relayable_integration_tests::harness::{collect_until_quiet, step, TestServer};
use relayable_model::SessionId;
use relayable_protocol::v1::event::Event as EventKind;
use relayable_protocol::v1::{self, AttachMode};

/// §D: the reaper emits `terminal_child_exited` for a real observed child exit,
/// with the server's actual exit code/signal, at most once.
#[test]
fn terminal_child_exited_is_emitted_when_the_program_exits() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server.socket_path().to_path_buf();

    let mut a = Client::connect(&socket, ClientConfig::new("exit-subscriber"))?;
    let created = a.create_session(Some("exit"))?;
    let session_id = created.session.session_id.clone();
    let parsed: SessionId = session_id.parse().context("session id")?;
    server.track_child(created.program_pid);
    let _ = a.resume_session(&parsed, None)?;
    let attached = a.attach_block(&created.block_id, AttachMode::Writer)?;
    assert!(
        attached.writer_lease,
        "the subscriber holds the writer lease"
    );

    // Make the interactive shell exit with a deterministic status.
    a.send_input(attached.stream_id, b"exit 7\n")?;

    let events = collect_until_quiet(&mut a, Duration::from_millis(700), Duration::from_secs(5))?;
    let exits: Vec<v1::TerminalChildExited> = events
        .iter()
        .filter_map(|event| match &event.event {
            Some(EventKind::TerminalChildExited(child))
                if child.block_id == created.block_id.to_string() =>
            {
                Some(child.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        exits.len(),
        1,
        "exactly one TerminalChildExited for the block, saw {exits:?}"
    );
    let child = &exits[0];
    assert_eq!(
        child.session_id, session_id,
        "the exit event names the session"
    );
    assert_eq!(child.block_id, created.block_id.to_string());
    assert!(child.exited, "the reaper observed an exited program");
    assert_eq!(child.exit_code, 7, "the real observed exit code");
    assert_eq!(
        child.signal, "",
        "a normal `exit 7` reports no terminating signal"
    );

    // At most once: a further bounded window must not yield a second event.
    let later = collect_until_quiet(&mut a, Duration::from_millis(700), Duration::from_secs(2))?;
    assert!(
        later.iter().all(|event| !matches!(
            &event.event,
            Some(EventKind::TerminalChildExited(child))
                if child.block_id == created.block_id.to_string()
        )),
        "terminal_child_exited must be emitted at most once"
    );
    step(
        "events-8-exit",
        &format!(
            "TerminalChildExited session={} block={} exit_code={} signal={:?} exited={}, exactly once",
            child.session_id, child.block_id, child.exit_code, child.signal, child.exited
        ),
    );
    Ok(())
}
