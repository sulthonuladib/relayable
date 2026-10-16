//! CTX-001: ADR-005 Decision 2 and the `full.md` §8.1 context-resolution test.
//!
//! Proves two things end to end against the real binaries:
//!
//! 1. every managed terminal spawned by the server receives
//!    `RELAYABLE_SESSION`, `RELAYABLE_BLOCK`, and `RELAYABLE_SERVER` with the
//!    exact session id, block id, and bound socket path; and
//! 2. the CLI resolves a session target in the ADR-005 precedence order —
//!    explicit `-s/--session` flag, then `$RELAYABLE_SESSION`, then no default —
//!    failing with the documented exit code instead of guessing.
//!
//! The documented unique-session convenience default (a single session with no
//! target) is accepted behavior (`protocol/compatibility/core-001-cli-json.md`
//! §2), so the no-context cases use zero or two live sessions, never one.

use std::time::Duration;

use relayable_integration_tests::harness::{cli, cli_json, poll_until, step, TestServer};
use serde_json::Value;

/// The value of one `KEY=VALUE` entry in `/proc/<pid>/environ`.
///
/// The file holds the environment captured at `execve`, NUL-separated; a
/// missing file (child already reaped) yields `None`.
fn environ_value(pid: u32, key: &str) -> Option<String> {
    let bytes = std::fs::read(format!("/proc/{pid}/environ")).ok()?;
    bytes.split(|byte| *byte == 0).find_map(|entry| {
        let text = String::from_utf8_lossy(entry);
        let (name, value) = text.split_once('=')?;
        (name == key).then(|| value.to_string())
    })
}

/// The server's bound socket path as UTF-8 text (the value `RELAYABLE_SERVER`
/// must carry).
fn socket_text(server: &TestServer) -> String {
    server
        .socket_path()
        .to_str()
        .expect("socket path is UTF-8")
        .to_string()
}

/// Create one session and return `(session_id, block_id, program_pid)`.
fn create_session(server: &mut TestServer, socket: &str) -> (String, String, u32) {
    let created = cli_json(&["new", "--json"], &[("RELAYABLE_SERVER", socket)]);
    let session_id = created["session_id"]
        .as_str()
        .expect("new.session_id")
        .to_string();
    let block_id = created["block_id"]
        .as_str()
        .expect("new.block_id")
        .to_string();
    let program_pid = u32::try_from(created["program_pid"].as_u64().expect("new.program_pid"))
        .expect("program_pid fits u32");
    assert!(program_pid > 0, "program_pid must be a real pid");
    server.track_child(program_pid);
    (session_id, block_id, program_pid)
}

/// Close a session through the CLI and return its parsed JSON record.
fn close_session(socket: &str, session_id: &str) -> Value {
    cli_json(
        &["session", "close", "--json", "-s", session_id],
        &[("RELAYABLE_SERVER", socket)],
    )
}

/// (a) A managed terminal's process environment carries the three context
/// variables with the exact session id, block id, and bound socket path.
#[test]
fn managed_terminal_exposes_relayable_context() {
    let mut server = TestServer::spawn();
    let socket = socket_text(&server);
    let (session_id, block_id, program_pid) = create_session(&mut server, &socket);

    // The variables are part of the environment captured at exec; poll briefly
    // so a scheduling race between `session.create` returning and the child
    // reaching `execve` cannot flake the assertion.
    let observed = poll_until(Duration::from_secs(5), Duration::from_millis(20), || {
        environ_value(program_pid, "RELAYABLE_SESSION").is_some()
    });
    assert!(
        observed,
        "managed terminal {program_pid} never exposed RELAYABLE_SESSION"
    );

    assert_eq!(
        environ_value(program_pid, "RELAYABLE_SESSION").as_deref(),
        Some(session_id.as_str()),
        "RELAYABLE_SESSION must be the exact session id"
    );
    assert_eq!(
        environ_value(program_pid, "RELAYABLE_BLOCK").as_deref(),
        Some(block_id.as_str()),
        "RELAYABLE_BLOCK must be the exact block id"
    );
    assert_eq!(
        environ_value(program_pid, "RELAYABLE_SERVER").as_deref(),
        Some(socket.as_str()),
        "RELAYABLE_SERVER must be the server's bound socket path"
    );
    assert!(
        environ_value(program_pid, "REX_SESSION").is_none(),
        "ADR-005 Decision 1 forbids any REX_* alias in a managed terminal"
    );

    let closed = close_session(&socket, &session_id);
    assert_eq!(closed["session_id"], session_id);
    assert_eq!(closed["state"], "closed");
    step(
        "ctx-001-managed-env",
        &format!(
            "pid={program_pid} RELAYABLE_SESSION={session_id} RELAYABLE_BLOCK={block_id} RELAYABLE_SERVER={socket}"
        ),
    );
}

/// (b) `$RELAYABLE_SESSION` resolves the target when no `-s` flag is given.
#[test]
fn relayable_session_resolves_without_flag() {
    let mut server = TestServer::spawn();
    let socket = socket_text(&server);
    let (session_id, _block_id, _program_pid) = create_session(&mut server, &socket);

    let inspected = cli_json(
        &["session", "inspect", "--json"],
        &[
            ("RELAYABLE_SERVER", socket.as_str()),
            ("RELAYABLE_SESSION", session_id.as_str()),
        ],
    );
    assert_eq!(
        inspected["session"]["session_id"], session_id,
        "with no -s, RELAYABLE_SESSION must select the session"
    );

    close_session(&socket, &session_id);
    step(
        "ctx-001-env-resolution",
        &format!("no -s; RELAYABLE_SESSION={session_id} inspected successfully"),
    );
}

/// (c) An explicit `-s/--session` flag wins over `$RELAYABLE_SESSION`.
#[test]
fn explicit_flag_wins_over_relayable_session() {
    let mut server = TestServer::spawn();
    let socket = socket_text(&server);
    let (session_a, _block_a, _pid_a) = create_session(&mut server, &socket);
    let (session_b, _block_b, _pid_b) = create_session(&mut server, &socket);
    assert_ne!(session_a, session_b, "two distinct sessions are required");

    let inspected = cli_json(
        &["session", "inspect", "--json", "-s", session_b.as_str()],
        &[
            ("RELAYABLE_SERVER", socket.as_str()),
            ("RELAYABLE_SESSION", session_a.as_str()),
        ],
    );
    assert_eq!(
        inspected["session"]["session_id"], session_b,
        "the explicit -s target must win over RELAYABLE_SESSION"
    );

    close_session(&socket, &session_a);
    close_session(&socket, &session_b);
    step(
        "ctx-001-flag-precedence",
        &format!("-s {session_b} won over RELAYABLE_SESSION={session_a}"),
    );
}

/// (d) Without context and without a unique session, the command fails with the
/// documented exit code instead of guessing.
#[test]
fn no_context_does_not_guess() {
    // Two live sessions and no RELAYABLE_SESSION/-s -> exit 3, ambiguous_target.
    let mut populated = TestServer::spawn();
    let populated_socket = socket_text(&populated);
    let (session_a, _block_a, _pid_a) = create_session(&mut populated, &populated_socket);
    let (session_b, _block_b, _pid_b) = create_session(&mut populated, &populated_socket);

    let ambiguous = cli(
        &["session", "inspect", "--json"],
        &[("RELAYABLE_SERVER", populated_socket.as_str())],
    );
    assert_eq!(
        ambiguous.status, 3,
        "two sessions without context must exit 3; stdout={:?} stderr={:?}",
        ambiguous.stdout, ambiguous.stderr
    );
    let error: Value =
        serde_json::from_str(&ambiguous.stdout).expect("ambiguous stdout is error JSON");
    assert_eq!(error["error"]["code"], "ambiguous_target");

    close_session(&populated_socket, &session_a);
    close_session(&populated_socket, &session_b);

    // A fresh server with no sessions and no context -> exit 2, bad_request.
    let empty = TestServer::spawn();
    let empty_socket = socket_text(&empty);
    let missing = cli(
        &["session", "inspect", "--json"],
        &[("RELAYABLE_SERVER", empty_socket.as_str())],
    );
    assert_eq!(
        missing.status, 2,
        "no sessions without context must exit 2; stdout={:?} stderr={:?}",
        missing.stdout, missing.stderr
    );
    let error: Value = serde_json::from_str(&missing.stdout).expect("missing stdout is error JSON");
    assert_eq!(error["error"]["code"], "bad_request");

    step(
        "ctx-001-no-context",
        "two sessions -> exit 3 ambiguous_target; empty server -> exit 2 bad_request",
    );
}
