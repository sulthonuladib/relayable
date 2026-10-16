use std::sync::mpsc::Sender;

use relayable_protocol::v1::SessionState;

use super::types::ApiEvent;

pub(super) fn emit(events: &Sender<ApiEvent>, event: ApiEvent) {
    // Unbounded channel: the UI is expected to drain each frame.
    let _ = events.send(event);
}

pub(super) fn session_state_str(state: i32) -> String {
    match SessionState::try_from(state) {
        Ok(SessionState::Active) => "active".to_string(),
        Ok(SessionState::Closed) => "closed".to_string(),
        _ => "unspecified".to_string(),
    }
}

pub(super) fn translate_wire_event(event: relayable_protocol::v1::Event) -> ApiEvent {
    use relayable_protocol::v1::event::Event as Kind;
    let (kind, detail) = match event.event {
        Some(Kind::ClientConnectionStateChanged(e)) => (
            "client_connection_state_changed",
            format!("client_id={} state={}", e.client_id, e.connection_state),
        ),
        Some(Kind::SessionStateChanged(e)) => (
            "session_state_changed",
            format!(
                "session_id={} state={}",
                e.session_id,
                session_state_str(e.state)
            ),
        ),
        Some(Kind::ClientDisconnected(e)) => (
            "client_disconnected",
            format!("client_id={} reason={}", e.client_id, e.reason),
        ),
        Some(Kind::WriterLeaseChanged(e)) => (
            "writer_lease_changed",
            format!(
                "session_id={} block_id={} stream_id={} previous={} new={} held={}",
                e.session_id,
                e.block_id,
                e.stream_id,
                e.previous_client_id,
                e.new_client_id,
                e.lease_held
            ),
        ),
        Some(Kind::TerminalChildExited(e)) => (
            "terminal_child_exited",
            format!(
                "session_id={} block_id={} exit_code={} signal={} exited={}",
                e.session_id, e.block_id, e.exit_code, e.signal, e.exited
            ),
        ),
        None => ("unknown", format!("sequence={}", event.sequence)),
    };
    ApiEvent::ServerEvent {
        kind: kind.to_string(),
        detail,
    }
}
