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
