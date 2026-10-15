//! CORE-001 assignment E — the required automated acceptance test.
//!
//! One test, `core_001_vertical_slice`, drives the real `relayable-server` and
//! `relayable` binaries and the real `relayable-client` library end-to-end and
//! proves the five required acceptance properties (plus the resize property):
//!
//! 1. the server owns a real PTY (child parented to the server, stdin on
//!    `/dev/pts/`, `stty size` round-trip);
//! 2. `relayable new` yields a stable, well-formed tagged UUIDv7 session id;
//! 3. terminal input/output round-trips through the server (a computed value
//!    that never appears in the echoed input);
//! 4. dropping the client leaves the server-owned child alive;
//! 5. reconnecting retrieves current state, including surviving the disconnect.
//!
//! Every wait goes through the bounded helpers in `relayable_integration_tests::harness`.

use std::collections::BTreeSet;
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use relayable_client::{Client, ClientConfig, ClientError};
use relayable_integration_tests::harness::{
    assert_tagged_uuid_v7, cli, cli_json, contains, nonce, poll_until, proc_exists, proc_fd0,
    proc_ppid, socket_mode, step, TestServer,
};
use relayable_model::{BlockId, SessionId};
use relayable_protocol::v1::{AttachMode, ErrorCode};
use serde_json::Value;

const OUTPUT_WAIT: Duration = Duration::from_secs(10);

/// Serializes the only test that mutates the process-global environment.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn cli_json_at(socket: &str, args: &[&str]) -> Value {
    cli_json(args, &[("RELAYABLE_SERVER", socket)])
}

fn inspect_at(socket: &str, session_id: &str) -> Value {
    cli_json_at(socket, &["session", "inspect", "-s", session_id, "--json"])
}

/// Attach a writer lease, retrying while a stale lease is being released.
fn attach_writer(client: &mut Client, block_id: &BlockId) -> Result<u64> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match client.attach_block(block_id, AttachMode::Writer) {
            Ok(outcome) => return Ok(outcome.stream_id),
            Err(ClientError::Server(ref error))
                if error.code == ErrorCode::InputNotOwned as i32 && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(error) => return Err(error.into()),
        }
    }
}

/// Find the first `<prefix><digits>` in `bytes`, skipping the echoed command
/// line that contains the literal `%s` placeholder.
fn parse_prefixed_pid(bytes: &[u8], prefix: &str) -> Option<u32> {
    let text = String::from_utf8_lossy(bytes);
    for (index, _) in text.match_indices(prefix) {
        let rest = &text[index + prefix.len()..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            return digits.parse().ok();
        }
    }
    None
}

/// Read PTY output until the shell prints `<label><pid>`. A first
/// [`Client::wait_for_output`] may match the echoed command (which contains the
/// literal `%s`), so keep reading until the expanded digits appear.
fn wait_for_pid(client: &mut Client, stream: u64, label: &str) -> Result<u32> {
    let deadline = Instant::now() + OUTPUT_WAIT;
    loop {
        let output = client.wait_for_output(stream, label.as_bytes(), OUTPUT_WAIT)?;
        if let Some(pid) = parse_prefixed_pid(&output, label) {
            return Ok(pid);
        }
        assert!(
            Instant::now() < deadline,
            "never saw {label}<digits>: {:?}",
            String::from_utf8_lossy(&output)
        );
    }
}

/// Result of the bounded bulk-input regression run on a worker thread.
struct BulkOutcome {
    accepted: usize,
    rate_limited: bool,
    max_frame: Duration,
    responsive: bool,
}

#[test]
fn core_001_vertical_slice() -> Result<()> {
    let started = Instant::now();
    let mut server = TestServer::spawn();
    let socket = server
        .socket_path()
        .to_str()
        .context("socket path is UTF-8")?
        .to_string();
    let server_pid = server.pid();

    // --- 1. start: the socket exists and is 0600 (ADR-002 D6).
    assert!(server.socket_path().exists(), "socket file must exist");
    assert_eq!(
        socket_mode(server.socket_path()),
        0o600,
        "socket mode must be 0600"
    );
    step(
        "property-0-start",
        &format!("server pid={server_pid} socket={socket} mode=0600"),
    );

    // --- 2. CLI creates a session with a stable, well-formed tagged UUIDv7 id.
    let created = cli_json_at(&socket, &["new", "--json"]);
    let session_id_text = created["session_id"]
        .as_str()
        .context("new.session_id")?
        .to_string();
    assert_tagged_uuid_v7(&session_id_text, "session");
    assert_tagged_uuid_v7(
        created["window_id"].as_str().context("new.window_id")?,
        "window",
    );
    assert_tagged_uuid_v7(
        created["block_id"].as_str().context("new.block_id")?,
        "block",
    );
    assert_tagged_uuid_v7(
        created["pty_process_id"]
            .as_str()
            .context("new.pty_process_id")?,
        "pty",
    );
    let program_pid = u32::try_from(created["program_pid"].as_u64().context("new.program_pid")?)
        .context("program_pid fits u32")?;
    assert!(program_pid > 0, "program_pid must be a real pid");
    server.track_child(program_pid);
    let block_id: BlockId = created["block_id"]
        .as_str()
        .context("new.block_id")?
        .parse()
        .context("parse block id")?;
    let pty_process_id = created["pty_process_id"]
        .as_str()
        .context("new.pty_process_id")?
        .to_string();
    let session_id: SessionId = session_id_text.parse().context("parse session id")?;

    // ls: the session is present with exactly the matrix §7.4 keys.
    let ls = cli_json_at(&socket, &["ls", "--json"]);
    let sessions = ls["sessions"].as_array().context("ls.sessions")?;
    let listed = sessions
        .iter()
        .find(|entry| entry["session_id"] == session_id_text)
        .context("new session must appear in ls")?;
    let keys: BTreeSet<&str> = listed
        .as_object()
        .context("ls entry object")?
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        BTreeSet::from(["session_id", "label"]),
        "ls session object must have exactly session_id and label"
    );
    assert_eq!(listed["label"], Value::Null, "this slice creates no label");

    // Inspecting twice yields the same id: the session id is stable.
    let first = inspect_at(&socket, &session_id_text);
    let second = inspect_at(&socket, &session_id_text);
    assert_eq!(first["session"]["session_id"], session_id_text);
    assert_eq!(second["session"]["session_id"], session_id_text);
    step(
        "property-2-session-id",
        &format!("session_id={session_id_text} window/block/pty tagged, program_pid={program_pid}, ls keys={{session_id,label}}, inspect stable"),
    );

    // --- 3. the server owns a real PTY.
    let mut client = Client::connect(server.socket_path(), ClientConfig::new("lifecycle"))?;
    let snapshot = client.session_snapshot(&session_id)?;
    let block = snapshot.blocks.first().context("snapshot has a block")?;
    assert_eq!(block.program_pid, program_pid, "snapshot pid matches new");
    assert_eq!(block.cols, 80, "initial cols");
    assert_eq!(block.rows, 24, "initial rows");
    assert!(proc_exists(program_pid), "PTY child must exist");
    assert_eq!(
        proc_ppid(program_pid),
        Some(server_pid),
        "the server is the PTY child's parent"
    );
    let fd0 = proc_fd0(program_pid).context("read /proc/<pid>/fd/0")?;
    assert!(
        fd0.to_string_lossy().starts_with("/dev/pts/"),
        "PTY child stdin must be a pts device, got {fd0:?}"
    );

    let stream = attach_writer(&mut client, &block_id)?;
    client.send_input(stream, b"tty\n")?;
    let output = client.wait_for_output(stream, b"/dev/pts/", OUTPUT_WAIT)?;
    assert!(contains(&output, b"/dev/pts/"), "tty reports a pts device");
    client.send_input(stream, b"stty size\n")?;
    let output = client.wait_for_output(stream, b"24 80", OUTPUT_WAIT)?;
    assert!(contains(&output, b"24 80"), "stty size reports 24 80");
    step(
        "property-1-server-owned-pty",
        &format!("pid={program_pid} ppid={server_pid} fd0={fd0:?} tty=/dev/pts/* stty='24 80'"),
    );

    // --- 4. input/output round-trips through the server.
    client.send_input(stream, b"printf 'ROUNDTRIP-%s\\n' \"$((2+3))\"\n")?;
    let output = client.wait_for_output(stream, b"ROUNDTRIP-5", OUTPUT_WAIT)?;
    assert!(
        contains(&output, b"ROUNDTRIP-5"),
        "computed value must round-trip (echo alone cannot produce it)"
    );
    let inspected = inspect_at(&socket, &session_id_text);
    assert_eq!(inspected["blocks"][0]["cols"], 80, "inspect cols");
    assert_eq!(inspected["blocks"][0]["rows"], 24, "inspect rows");
    assert_eq!(
        inspected["blocks"][0]["program_status"]["state"], "running",
        "program is running"
    );
    step(
        "property-3-io-round-trip",
        "printf ROUNDTRIP-5 round-tripped; inspect cols=80 rows=24 running",
    );

    // --- 4b. resize (extra property beyond the brief's list).
    client.resize(stream, 100, 40)?;
    client.send_input(stream, b"stty size\n")?;
    let output = client.wait_for_output(stream, b"40 100", OUTPUT_WAIT)?;
    assert!(contains(&output, b"40 100"), "stty size reports 40 100");
    let resized = inspect_at(&socket, &session_id_text);
    assert_eq!(resized["blocks"][0]["cols"], 100, "inspect reflects cols");
    assert_eq!(resized["blocks"][0]["rows"], 40, "inspect reflects rows");
    step(
        "property-1b-resize",
        "resize(100,40) -> stty '40 100' and inspect cols=100 rows=40",
    );

    // --- 5. disconnect leaves the server-owned child alive.
    let survival = nonce("survival");
    client.send_input(
        stream,
        format!("export RELAYABLE_SURVIVAL={survival}\n").as_bytes(),
    )?;
    client.send_input(stream, format!("echo SET-{survival}\n").as_bytes())?;
    let output =
        client.wait_for_output(stream, format!("SET-{survival}").as_bytes(), OUTPUT_WAIT)?;
    assert!(
        contains(&output, format!("SET-{survival}").as_bytes()),
        "the survival variable must be set in the live shell"
    );
    drop(client);

    // Bounded proof (>=300 ms) that the session stays ACTIVE with the same child.
    let observation = Instant::now();
    while observation.elapsed() < Duration::from_millis(400) {
        let view = inspect_at(&socket, &session_id_text);
        assert_eq!(view["session"]["state"], "active", "session stays active");
        assert_eq!(
            view["blocks"][0]["program_pid"].as_u64(),
            Some(u64::from(program_pid)),
            "same program pid after the disconnect"
        );
        assert!(
            proc_exists(program_pid),
            "child stays alive after disconnect"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        observation.elapsed() >= Duration::from_millis(300),
        "must observe survival for at least 300 ms"
    );
    step(
        "property-4-disconnect-survival",
        &format!(
            "client dropped; same pid={program_pid} observed active for {:?}",
            observation.elapsed()
        ),
    );

    // --- 6. reconnect retrieves current state and the surviving shell.
    let mut client2 = Client::connect(server.socket_path(), ClientConfig::new("lifecycle-r"))?;
    let reconnect_view = inspect_at(&socket, &session_id_text);
    assert_eq!(reconnect_view["session"]["session_id"], session_id_text);
    assert_eq!(reconnect_view["session"]["state"], "active");
    let reconnected_block = &reconnect_view["blocks"][0];
    assert_eq!(reconnected_block["block_id"], block_id.to_string());
    assert_eq!(reconnected_block["pty_process_id"], pty_process_id);
    assert_eq!(
        reconnected_block["program_pid"].as_u64(),
        Some(u64::from(program_pid))
    );
    assert_eq!(
        reconnected_block["cols"], 100,
        "post-resize cols are current"
    );
    assert_eq!(
        reconnected_block["rows"], 40,
        "post-resize rows are current"
    );

    let stream2 = attach_writer(&mut client2, &block_id)?;
    client2.send_input(stream2, b"echo $RELAYABLE_SURVIVAL\n")?;
    let output = client2.wait_for_output(stream2, survival.as_bytes(), OUTPUT_WAIT)?;
    assert!(
        contains(&output, survival.as_bytes()),
        "the SAME shell process survived the disconnect"
    );
    assert!(
        proc_exists(program_pid),
        "child still exists after reconnect"
    );
    step(
        "property-5-reconnect",
        &format!("same session/block/pty/pid, state=active, cols=100 rows=40, writer lease re-acquired, survival nonce {survival} recovered"),
    );

    // --- 7. explicit close and cleanup.
    let closed = cli_json_at(
        &socket,
        &["session", "close", "-s", &session_id_text, "--json"],
    );
    assert_eq!(closed["state"], "closed", "close reports the closed state");
    let ls_after = cli_json_at(&socket, &["ls", "--json"]);
    assert!(
        ls_after["sessions"]
            .as_array()
            .context("ls.sessions")?
            .iter()
            .all(|entry| entry["session_id"] != session_id_text),
        "closed session must not be listed"
    );
    assert!(
        poll_until(Duration::from_secs(2), Duration::from_millis(20), || {
            !proc_exists(program_pid)
        }),
        "the server must reap the PTY child after session.close"
    );
    drop(client2);
    step(
        "property-6-close",
        &format!("session closed, ls empty, pid={program_pid} reaped"),
    );

    // --- 8. graceful shutdown: exit 0 and the socket file is removed.
    let status = server.shutdown().context("server did not exit")?;
    assert_eq!(
        status.code(),
        Some(0),
        "graceful SIGTERM shutdown must exit 0, got {status:?}"
    );
    assert!(
        !server.socket_path().exists(),
        "the server must unlink its socket on graceful shutdown"
    );
    step(
        "property-7-shutdown",
        &format!("SIGTERM -> exit={:?}, socket removed", status.code()),
    );

    step(
        "runtime",
        &format!(
            "lifecycle acceptance test finished in {:?}",
            started.elapsed()
        ),
    );
    Ok(())
}

/// Regression A: `session.close` must terminate a descendant that holds the
/// PTY slave, and must return within a bounded window. This is the integration
/// counterpart of the server's own descendant test.
#[test]
fn session_close_terminates_descendant_holding_the_slave() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server
        .socket_path()
        .to_str()
        .context("socket is UTF-8")?
        .to_string();

    let created = cli_json_at(&socket, &["new", "--json"]);
    let session_id_text = created["session_id"]
        .as_str()
        .context("session id")?
        .to_string();
    let block_id: BlockId = created["block_id"].as_str().context("block id")?.parse()?;
    let child_pid = u32::try_from(created["program_pid"].as_u64().context("program pid")?)?;
    server.track_child(child_pid);

    let mut client = Client::connect(server.socket_path(), ClientConfig::new("descendant"))?;
    let stream = attach_writer(&mut client, &block_id)?;
    // Background a long-lived descendant that keeps the slave open and prints
    // its pid; the echoed command contains the literal `DESC=%s`.
    client.send_input(stream, b"sleep 300 & printf 'DESC=%s\\n' \"$!\"\n")?;
    let descendant_pid = wait_for_pid(&mut client, stream, "DESC=")?;
    server.track_child(descendant_pid);
    assert!(proc_exists(child_pid), "direct child {child_pid} is alive");
    assert!(
        proc_exists(descendant_pid),
        "descendant {descendant_pid} is alive"
    );

    let started = Instant::now();
    let closed = cli_json_at(
        &socket,
        &["session", "close", "-s", &session_id_text, "--json"],
    );
    let elapsed = started.elapsed();
    assert_eq!(closed["state"], "closed", "close reports closed");
    assert!(
        elapsed < Duration::from_secs(5),
        "session.close blocked for {elapsed:?} (must be < 5 s)"
    );
    assert!(
        poll_until(Duration::from_secs(3), Duration::from_millis(20), || {
            !proc_exists(child_pid) && !proc_exists(descendant_pid)
        }),
        "session.close must reap the direct child {child_pid} and descendant {descendant_pid}"
    );
    drop(client);
    step(
        "regression-A-descendant-close",
        &format!(
            "close returned in {elapsed:?}; child {child_pid} and descendant {descendant_pid} reaped"
        ),
    );
    Ok(())
}

/// Regression B: a large `terminal.input` to a program that is not reading must
/// not wedge the connection. Every call is bounded and the connection stays
/// responsive; a legitimate overflow is `rate_limited` and retryable.
#[test]
fn large_input_to_a_non_reading_program_stays_bounded() -> Result<()> {
    let mut server = TestServer::spawn();
    let socket = server
        .socket_path()
        .to_str()
        .context("socket is UTF-8")?
        .to_string();

    let created = cli_json_at(&socket, &["new", "--json"]);
    let session_id_text = created["session_id"]
        .as_str()
        .context("session id")?
        .to_string();
    let session_id: SessionId = session_id_text.parse().context("parse session id")?;
    let block_id: BlockId = created["block_id"].as_str().context("block id")?.parse()?;
    let child_pid = u32::try_from(created["program_pid"].as_u64().context("program pid")?)?;
    server.track_child(child_pid);

    let mut client = Client::connect(server.socket_path(), ClientConfig::new("bulk"))?;
    let stream = attach_writer(&mut client, &block_id)?;
    // Disable echo, then block the shell in `wait` on a background sleep so it
    // is not consuming stdin; the sleep keeps the slave open and prints its pid.
    client.send_input(
        stream,
        b"stty -echo; sleep 300 & printf 'BULKSLEEP=%s\\n' \"$!\"; wait\n",
    )?;
    let sleep_pid = wait_for_pid(&mut client, stream, "BULKSLEEP=")?;
    server.track_child(sleep_pid);

    // ~1 MiB across frames that each fit the 1 MiB frame limit. Run the whole
    // loop on a worker thread so a regressed (blocking) server fails the test
    // instead of hanging it; the client has no per-call timeout of its own.
    let frame = vec![b'x'; 128 * 1024 - 1024];
    let frame_len = frame.len();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let outcome: std::result::Result<BulkOutcome, ClientError> = (|| {
            let mut accepted = 0usize;
            let mut rate_limited = false;
            let mut max_frame = Duration::ZERO;
            for _ in 0..8 {
                let started = Instant::now();
                match client.send_input(stream, &frame) {
                    Ok(bytes) => accepted += bytes as usize,
                    Err(ClientError::Server(ref error))
                        if error.code == ErrorCode::RateLimited as i32 =>
                    {
                        assert!(error.retryable, "rate_limited must be marked retryable");
                        rate_limited = true;
                    }
                    Err(error) => return Err(error),
                }
                max_frame = max_frame.max(started.elapsed());
            }
            let responsive = client.session_snapshot(&session_id).is_ok();
            Ok(BulkOutcome {
                accepted,
                rate_limited,
                max_frame,
                responsive,
            })
        })();
        let _ = tx.send(outcome);
    });
    let outcome = rx
        .recv_timeout(Duration::from_secs(20))
        .context("bulk terminal.input loop did not complete within 20 s (server wedged)")??;
    assert!(
        outcome.max_frame < Duration::from_secs(2),
        "a terminal.input call took {:?} (must be < 2 s)",
        outcome.max_frame
    );
    assert!(
        outcome.accepted > 0,
        "at least one input frame must be accepted"
    );
    assert!(
        outcome.responsive,
        "the connection must stay responsive after bulk input"
    );

    // A separate process can still list the session.
    let ls = cli_json_at(&socket, &["ls", "--json"]);
    assert!(
        ls["sessions"]
            .as_array()
            .context("ls.sessions")?
            .iter()
            .any(|entry| entry["session_id"] == session_id_text),
        "the session must still be listed after bulk input"
    );

    let started = Instant::now();
    let closed = cli_json_at(
        &socket,
        &["session", "close", "-s", &session_id_text, "--json"],
    );
    let elapsed = started.elapsed();
    assert_eq!(closed["state"], "closed");
    assert!(
        elapsed < Duration::from_secs(5),
        "close after bulk input took {elapsed:?}"
    );
    assert!(
        poll_until(Duration::from_secs(3), Duration::from_millis(20), || {
            !proc_exists(child_pid) && !proc_exists(sleep_pid)
        }),
        "close must reap child {child_pid} and sleep {sleep_pid}"
    );
    step(
        "regression-B-bounded-input",
        &format!(
            "8x{frame_len}B frames; max call {:?}; accepted {}; rate_limited={}; close {elapsed:?}",
            outcome.max_frame, outcome.accepted, outcome.rate_limited
        ),
    );
    Ok(())
}

/// m8: the harness clears the CLI environment, so a stray `RELAYABLE_SESSION`
/// in the parent process cannot change the target; an explicit `-s` wins.
#[test]
fn harness_environment_is_hermetic() -> Result<()> {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut server = TestServer::spawn();
    let socket = server
        .socket_path()
        .to_str()
        .context("socket is UTF-8")?
        .to_string();

    let first = cli_json_at(&socket, &["new", "--json"]);
    let second = cli_json_at(&socket, &["new", "--json"]);
    let first_id = first["session_id"]
        .as_str()
        .context("first id")?
        .to_string();
    let second_id = second["session_id"]
        .as_str()
        .context("second id")?
        .to_string();
    assert_ne!(first_id, second_id, "two distinct sessions are required");
    server.track_child(u32::try_from(
        first["program_pid"].as_u64().context("first pid")?,
    )?);
    server.track_child(u32::try_from(
        second["program_pid"].as_u64().context("second pid")?,
    )?);

    // Setting a process-global env var is safe for the other tests in this
    // binary precisely because `cli()`/`TestServer` clear the child environment;
    // the lock serializes this test's own set/restore.
    let previous = std::env::var_os("RELAYABLE_SESSION");
    std::env::set_var("RELAYABLE_SESSION", &first_id);

    // No `-s` with two live sessions: a leaked value would select `first`; a
    // cleared environment sees neither and reports ambiguity (exit 3).
    let no_target = cli(
        &["session", "inspect", "--json"],
        &[("RELAYABLE_SERVER", &socket)],
    );
    assert_eq!(
        no_target.status, 3,
        "expected exit 3 (ambiguous target), stdout={:?} stderr={:?}",
        no_target.stdout, no_target.stderr
    );
    let error: Value = serde_json::from_str(&no_target.stdout).context("error json")?;
    assert_eq!(error["error"]["code"], "ambiguous_target");

    // An explicit target wins over any stray value.
    let explicit = cli_json_at(&socket, &["session", "inspect", "-s", &second_id, "--json"]);
    assert_eq!(explicit["session"]["session_id"], second_id);

    match previous {
        Some(value) => std::env::set_var("RELAYABLE_SESSION", value),
        None => std::env::remove_var("RELAYABLE_SESSION"),
    }

    cli_json_at(&socket, &["session", "close", "-s", &first_id, "--json"]);
    cli_json_at(&socket, &["session", "close", "-s", &second_id, "--json"]);
    step(
        "m8-hermetic-env",
        "parent RELAYABLE_SESSION did not leak (no -s -> exit 3 ambiguous); explicit -s wins",
    );
    Ok(())
}
