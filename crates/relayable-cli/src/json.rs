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
mod tests {
    use super::*;
    use relayable_model::{BlockId, PtyProcessId, WindowId};
    use relayable_protocol::v1::{ServerRecord, SessionSnapshotResponse};
    use std::collections::HashSet;

    fn keys(value: &Value) -> HashSet<String> {
        value.as_object().expect("object").keys().cloned().collect()
    }

    fn set(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| (*s).to_string()).collect()
    }

    fn summary(id: &str, label: Option<&str>) -> SessionSummary {
        SessionSummary {
            session_id: id.to_string(),
            label: label.map(str::to_owned),
            state: SessionState::Active as i32,
            revision: 1,
        }
    }

    #[test]
    fn version_shape() {
        let value = version_json();
        assert_eq!(keys(&value), set(&["name", "version", "protocol_version"]));
        assert_eq!(value["name"], "relayable");
        assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn ls_shape_matches_the_documented_sample() {
        let value = ls_json(&[
            summary("session:x", Some("Research")),
            summary("session:y", None),
        ]);
        assert_eq!(keys(&value), set(&["sessions"]));
        let sessions = value["sessions"].as_array().unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(keys(&sessions[0]), set(&["session_id", "label"]));
        assert_eq!(sessions[0]["session_id"], "session:x");
        assert_eq!(sessions[0]["label"], "Research");
        assert!(sessions[1]["label"].is_null());
    }

    #[test]
    fn new_shape() {
        let created = CreatedSession {
            session: SessionRecord {
                session_id: "session:01a10cf3-2655-7e48-9b9c-43dd6612bef5".to_string(),
                label: None,
                revision: 1,
                state: SessionState::Active as i32,
                created_at_unix_millis: 123,
            },
            window_id: WindowId::new(),
            block_id: BlockId::new(),
            pty_process_id: PtyProcessId::new(),
            program_pid: 4242,
        };
        let value = new_json(&created);
        assert_eq!(
            keys(&value),
            set(&[
                "session_id",
                "window_id",
                "block_id",
                "pty_process_id",
                "program_pid",
                "label",
            ])
        );
        assert_eq!(value["program_pid"], 4242);
        assert!(value["label"].is_null());
        assert!(value["window_id"].as_str().unwrap().starts_with("window:"));
    }

    fn snapshot() -> SessionSnapshotResponse {
        SessionSnapshotResponse {
            sequence: 7,
            server: Some(ServerRecord {
                server_id: "server:01a10cf3-2655-7e48-9b9c-43dd6612bef5".to_string(),
                protocol_version: "1.0".to_string(),
                sequence: 7,
                started_at_unix_millis: 1,
            }),
            session: Some(SessionRecord {
                session_id: "session:01a10cf3-2655-7e48-9b9c-43dd6612bef5".to_string(),
                label: Some("Research".to_string()),
                revision: 2,
                state: SessionState::Active as i32,
                created_at_unix_millis: 1,
            }),
            windows: vec![WindowRecord {
                window_id: "window:01a10cf3-2655-7e48-9b9c-43dd6612be01".to_string(),
                session_id: "session:01a10cf3-2655-7e48-9b9c-43dd6612bef5".to_string(),
                label: None,
                revision: 1,
                active: true,
            }],
            blocks: vec![BlockRecord {
                block_id: "block:01a10cf3-2655-7e48-9b9c-43dd6612be02".to_string(),
                window_id: "window:01a10cf3-2655-7e48-9b9c-43dd6612be01".to_string(),
                session_id: "session:01a10cf3-2655-7e48-9b9c-43dd6612bef5".to_string(),
                kind: BlockKind::Terminal as i32,
                label: None,
                revision: 1,
                cols: 80,
                rows: 24,
                program_status: Some(ProgramStatus {
                    state: ProgramState::Running as i32,
                    exit_code: None,
                    signal: None,
                    pwd: Some("/home".to_string()),
                    title: Some("sh".to_string()),
                }),
                pty_process_id: "pty:01a10cf3-2655-7e48-9b9c-43dd6612be03".to_string(),
                program_pid: 99,
                writer_attached: false,
            }],
            clients: vec![ClientRecord {
                client_id: "client:01a10cf3-2655-7e48-9b9c-43dd6612be04".to_string(),
                kind: "cli".to_string(),
                platform: "linux".to_string(),
                version: "0.0.0".to_string(),
                connection_state: "Ready".to_string(),
                principal: None,
                connected_at_unix_millis: 1,
            }],
        }
    }

    #[test]
    fn inspect_shape_and_nested_keys() {
        let value = inspect_json(&snapshot());
        assert_eq!(
            keys(&value),
            set(&["sequence", "server", "session", "windows", "blocks", "clients"])
        );
        assert_eq!(
            keys(&value["server"]),
            set(&["server_id", "protocol_version", "sequence"])
        );
        assert_eq!(
            keys(&value["session"]),
            set(&["session_id", "label", "revision", "state"])
        );
        assert_eq!(
            keys(&value["windows"][0]),
            set(&["window_id", "session_id", "label", "revision", "active"])
        );
        assert_eq!(
            keys(&value["blocks"][0]),
            set(&[
                "block_id",
                "window_id",
                "session_id",
                "kind",
                "label",
                "revision",
                "cols",
                "rows",
                "program_status",
                "pty_process_id",
                "program_pid",
                "writer_attached",
            ])
        );
        assert_eq!(
            keys(&value["blocks"][0]["program_status"]),
            set(&["state", "exit_code", "signal"])
        );
        assert_eq!(
            keys(&value["clients"][0]),
            set(&[
                "client_id",
                "kind",
                "platform",
                "version",
                "connection_state",
                "principal",
            ])
        );
        assert_eq!(value["session"]["state"], "active");
        assert_eq!(value["blocks"][0]["kind"], "terminal");
        assert_eq!(value["blocks"][0]["program_status"]["state"], "running");
        assert!(value["blocks"][0]["program_status"]["exit_code"].is_null());
        // No timestamps anywhere.
        assert!(!value.to_string().contains("unix_millis"));
    }

    #[test]
    fn close_shape() {
        let record = SessionRecord {
            session_id: "session:01a10cf3-2655-7e48-9b9c-43dd6612bef5".to_string(),
            label: None,
            revision: 2,
            state: SessionState::Closed as i32,
            created_at_unix_millis: 1,
        };
        let value = close_json(&record);
        assert_eq!(keys(&value), set(&["session_id", "state"]));
        assert_eq!(value["state"], "closed");
    }

    #[test]
    fn error_shape() {
        let value = error_json("unknown_entity", "nope", json!({}));
        assert_eq!(keys(&value), set(&["error"]));
        assert_eq!(keys(&value["error"]), set(&["code", "message", "details"]));
        assert_eq!(value["error"]["code"], "unknown_entity");
    }
}
