use std::sync::mpsc::{Receiver, Sender};

use super::types::{ApiCommand, ApiEvent, ConnectionState, SessionSummaryView};
use super::wire::emit;

// --- mock ------------------------------------------------------------------

pub(super) fn mock_control_thread(commands: Receiver<ApiCommand>, events: Sender<ApiEvent>) {
    emit(
        &events,
        ApiEvent::ConnectionStateChanged(ConnectionState::Connected),
    );
    emit(
        &events,
        ApiEvent::SessionsListed(vec![SessionSummaryView {
            session_id: "session:mock".to_string(),
            label: Some("mock session".to_string()),
            state: "active".to_string(),
        }]),
    );
    // Blocking receive: process every command for as long as the handle lives.
    while let Ok(command) = commands.recv() {
        match command {
            ApiCommand::Shutdown => return,
            ApiCommand::RefreshSessions => emit(
                &events,
                ApiEvent::SessionsListed(vec![SessionSummaryView {
                    session_id: "session:mock".to_string(),
                    label: Some("mock session".to_string()),
                    state: "active".to_string(),
                }]),
            ),
            ApiCommand::CreateSession { label: _ } => emit(
                &events,
                ApiEvent::SessionCreated {
                    session_id: "session:mock".to_string(),
                    window_id: "window:mock".to_string(),
                    block_id: "block:mock".to_string(),
                    program_pid: 0,
                    pty_process_id: "pty:mock".to_string(),
                },
            ),
            _ => {}
        }
    }
}

pub(super) fn mock_data_thread(commands: Receiver<ApiCommand>, events: Sender<ApiEvent>) {
    // Blocking receive: process every command for as long as the handle lives.
    while let Ok(command) = commands.recv() {
        match command {
            ApiCommand::Shutdown => return,
            ApiCommand::AttachBlock { .. } => {
                emit(
                    &events,
                    ApiEvent::Attached {
                        stream_id: 1,
                        cols: 80,
                        rows: 24,
                        screen_bytes: b"mock$ ".to_vec(),
                    },
                );
                emit(
                    &events,
                    ApiEvent::Output {
                        stream_id: 1,
                        bytes: b"mock output\r\n".to_vec(),
                    },
                );
            }
            ApiCommand::Input { stream_id, bytes } => {
                emit(&events, ApiEvent::Output { stream_id, bytes })
            }
            ApiCommand::Resize {
                stream_id,
                cols,
                rows,
                ..
            } => emit(
                &events,
                ApiEvent::Resized {
                    stream_id,
                    cols,
                    rows,
                },
            ),
            _ => {}
        }
    }
}
