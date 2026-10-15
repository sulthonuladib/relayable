//! Contract §2 snapshot projection.
//!
//! A snapshot is a consistent projection of the domain at one server
//! `sequence`. `layout`, `status`, and `automation_scripts` are deferred and
//! have no fields in the CORE-001 schema. `clients` lists every client currently
//! connected to the server, across all sessions (documented interpretation);
//! `principal` is unset because authentication is deferred.

use relayable_model::{Client, ConnectionState};
use relayable_protocol::v1::{
    ClientRecord, ServerRecord, SessionRecord, SessionSnapshotResponse, SessionState, WindowRecord,
};
use relayable_protocol::PROTOCOL_VERSION;

use crate::registry::ServerShared;
use crate::session::SessionRuntime;

pub(crate) fn server_record(shared: &ServerShared) -> ServerRecord {
    ServerRecord {
        server_id: shared.server_id.to_string(),
        protocol_version: PROTOCOL_VERSION.to_string(),
        sequence: shared.sequence(),
        started_at_unix_millis: shared.started_at_millis,
    }
}

pub(crate) fn session_record(session: &SessionRuntime) -> SessionRecord {
    SessionRecord {
        session_id: session.id.to_string(),
        label: session.label.clone(),
        revision: session.revision(),
        state: if session.is_closed() {
            SessionState::Closed as i32
        } else {
            SessionState::Active as i32
        },
        created_at_unix_millis: session.created_at_millis,
    }
}

pub(crate) fn window_record(session: &SessionRuntime) -> WindowRecord {
    WindowRecord {
        window_id: session.window_id.to_string(),
        session_id: session.id.to_string(),
        label: None,
        revision: session.window_revision(),
        active: true,
    }
}

pub(crate) fn client_record(client: &Client) -> ClientRecord {
    // Contract §4 spells the terminal failure only as `Failed`; the reason is
    // diagnostics, not wire data, so log it and never transmit it.
    if let ConnectionState::Failed(reason) = &client.connection_state {
        eprintln!(
            "relayable-server: client {} connection failed: {reason}",
            client.id
        );
    }
    ClientRecord {
        client_id: client.id.to_string(),
        kind: client.kind.clone(),
        platform: client.platform.clone(),
        version: client.version.clone(),
        connection_state: client.connection_state.as_str().to_string(),
        principal: client.principal.clone(),
        connected_at_unix_millis: client.connected_at.get(),
    }
}

pub(crate) fn build(shared: &ServerShared, session: &SessionRuntime) -> SessionSnapshotResponse {
    SessionSnapshotResponse {
        sequence: shared.sequence(),
        server: Some(server_record(shared)),
        session: Some(session_record(session)),
        windows: vec![window_record(session)],
        blocks: vec![session.block.record()],
        clients: shared.client_records(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use relayable_model::{ClientId, ConnectionState, Timestamp};

    #[test]
    fn failed_connection_state_is_transmitted_as_failed() {
        let client = Client {
            id: ClientId::new(),
            kind: "test".to_string(),
            platform: "linux".to_string(),
            version: "0".to_string(),
            connection_state: ConnectionState::Failed("boom".to_string()),
            principal: None,
            connected_at: Timestamp::new(1),
        };
        // The reason is logged, never put on the wire; the wire value is the
        // contract §4 spelling.
        assert_eq!(client_record(&client).connection_state, "Failed");
    }
}
