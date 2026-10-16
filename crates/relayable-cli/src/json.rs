//! Stable JSON builders for `relayable --json`.
//!
//! Every shape is frozen and documented in
//! `protocol/compatibility/core-001-cli-json.md`. The `ls` shape is the only
//! one backed by reference documentation (matrix §7.4); the others are
//! Relayable-defined projections of the contract §2 snapshot.
//!
//! Rules: snake_case keys, no floats, and no timestamps.

use relayable_client::CreatedSession;
use relayable_protocol::v1::{
    BlockKind, BlockRecord, ClientRecord, ProgramState, ProgramStatus, SessionRecord,
    SessionSnapshotResponse, SessionState, SessionSummary, WindowRecord,
};
use serde_json::{json, Map, Value};

/// `relayable version --json`.
#[must_use]
pub fn version_json() -> Value {
    json!({
        "name": "relayable",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol_version": relayable_protocol::PROTOCOL_VERSION,
    })
}

/// `relayable ls --json`. Matches the matrix §7.4 sample exactly: each session
/// has only `session_id` and `label`.
#[must_use]
pub fn ls_json(sessions: &[SessionSummary]) -> Value {
    let list: Vec<Value> = sessions
        .iter()
        .map(|session| {
            json!({
                "session_id": session.session_id.as_str(),
                "label": session.label.as_deref(),
            })
        })
        .collect();
    json!({ "sessions": list })
}

/// `relayable new --json`.
#[must_use]
pub fn new_json(created: &CreatedSession) -> Value {
    json!({
        "session_id": created.session.session_id.as_str(),
        "window_id": created.window_id.to_string(),
        "block_id": created.block_id.to_string(),
        "pty_process_id": created.pty_process_id.to_string(),
        "program_pid": created.program_pid,
        "label": created.session.label.as_deref(),
    })
}

/// `relayable session inspect --json`.
#[must_use]
pub fn inspect_json(snapshot: &SessionSnapshotResponse) -> Value {
    let server = snapshot.server.as_ref().map_or(Value::Null, server_json);
    let session = snapshot.session.as_ref().map_or(Value::Null, session_json);
    let windows: Vec<Value> = snapshot.windows.iter().map(window_json).collect();
    let blocks: Vec<Value> = snapshot.blocks.iter().map(block_json).collect();
    let clients: Vec<Value> = snapshot.clients.iter().map(client_json).collect();
    json!({
        "sequence": snapshot.sequence,
        "server": server,
        "session": session,
        "windows": windows,
        "blocks": blocks,
        "clients": clients,
    })
}

/// `relayable session close --json`.
#[must_use]
pub fn close_json(session: &SessionRecord) -> Value {
    json!({
        "session_id": session.session_id.as_str(),
        "state": session_state_str(session.state),
    })
}

/// The `{"error": {"code", "message", "details"}}` envelope used by `--json`.
#[must_use]
pub fn error_json(code: &str, message: &str, details: Value) -> Value {
    json!({
        "error": {
            "code": code,
            "message": message,
            "details": details,
        },
    })
}

/// The uppercase `"STRING"` names for a `SessionSummary` map to lowercase enum
/// values via [`session_state_str`].
#[must_use]
pub fn session_state_str(raw: i32) -> &'static str {
    match SessionState::try_from(raw).unwrap_or(SessionState::Unspecified) {
        SessionState::Unspecified => "unspecified",
        SessionState::Active => "active",
        SessionState::Closed => "closed",
    }
}

/// Lowercase program-state spelling.
#[must_use]
pub fn program_state_str(raw: i32) -> &'static str {
    match ProgramState::try_from(raw).unwrap_or(ProgramState::Unspecified) {
        ProgramState::Unspecified => "unspecified",
        ProgramState::Running => "running",
        ProgramState::Exited => "exited",
    }
}

/// Lowercase block-kind spelling.
#[must_use]
pub fn block_kind_str(raw: i32) -> &'static str {
    match BlockKind::try_from(raw).unwrap_or(BlockKind::Unspecified) {
        BlockKind::Unspecified => "unspecified",
        BlockKind::Terminal => "terminal",
    }
}

fn server_json(server: &relayable_protocol::v1::ServerRecord) -> Value {
    json!({
        "server_id": server.server_id.as_str(),
        "protocol_version": server.protocol_version.as_str(),
        "sequence": server.sequence,
    })
}

fn session_json(session: &SessionRecord) -> Value {
    json!({
        "session_id": session.session_id.as_str(),
        "label": session.label.as_deref(),
        "revision": session.revision,
        "state": session_state_str(session.state),
    })
}

fn window_json(window: &WindowRecord) -> Value {
    json!({
        "window_id": window.window_id.as_str(),
        "session_id": window.session_id.as_str(),
        "label": window.label.as_deref(),
        "revision": window.revision,
        "active": window.active,
    })
}

fn block_json(block: &BlockRecord) -> Value {
    json!({
        "block_id": block.block_id.as_str(),
        "window_id": block.window_id.as_str(),
        "session_id": block.session_id.as_str(),
        "kind": block_kind_str(block.kind),
        "label": block.label.as_deref(),
        "revision": block.revision,
        "cols": block.cols,
        "rows": block.rows,
        "program_status": block.program_status.as_ref().map_or(Value::Null, program_status_json),
        "pty_process_id": block.pty_process_id.as_str(),
        "program_pid": block.program_pid,
        "writer_attached": block.writer_attached,
    })
}

fn program_status_json(status: &ProgramStatus) -> Value {
    json!({
        "state": program_state_str(status.state),
        "exit_code": status.exit_code,
        "signal": status.signal.as_deref(),
    })
}

fn client_json(client: &ClientRecord) -> Value {
    json!({
        "client_id": client.client_id.as_str(),
        "kind": client.kind.as_str(),
        "platform": client.platform.as_str(),
        "version": client.version.as_str(),
        "connection_state": client.connection_state.as_str(),
        "principal": client.principal.as_deref(),
    })
}

/// The `{"matches": [...]}` details object for an ambiguous target.
#[must_use]
pub fn ambiguous_details(matches: &[crate::resolve::SessionMatch]) -> Value {
    let list: Vec<Value> = matches
        .iter()
        .map(|found| {
            json!({
                "session_id": found.session_id.as_str(),
                "label": found.label.as_deref(),
            })
        })
        .collect();
    Value::Object(Map::from_iter([(
        "matches".to_string(),
        Value::Array(list),
    )]))
}

#[cfg(test)]
mod tests;
