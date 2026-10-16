use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use super::data::DataPlane;
use super::types::{
    ApiCommand, ApiEvent, POLL_INTERVAL, RECONNECT_BACKOFF, RECONNECT_MAX_ATTEMPTS,
};
use super::wire::emit;

/// Connect the data plane and, on success, tell the UI it is usable again.
///
/// The signal is a [`ApiEvent::ServerEvent`] (not a new [`ApiEvent`] variant,
/// which is frozen) so the UI can re-attach panes whose streams were invalidated
/// by a data-plane-only disconnect.
fn connect_data_plane(
    path: &std::path::Path,
    events: &Sender<ApiEvent>,
) -> Result<DataPlane, String> {
    let plane = DataPlane::connect(path, events.clone())?;
    emit(
        events,
        ApiEvent::ServerEvent {
            kind: "data_plane_connected".to_string(),
            detail: path.display().to_string(),
        },
    );
    Ok(plane)
}

pub(super) fn data_thread(path: PathBuf, commands: Receiver<ApiCommand>, events: Sender<ApiEvent>) {
    let mut plane = match connect_data_plane(&path, &events) {
        Ok(plane) => Some(plane),
        Err(error) => {
            emit(&events, ApiEvent::Error { message: error });
            None
        }
    };
    let mut attempts = 0u32;

    loop {
        if plane.is_none() {
            if attempts >= RECONNECT_MAX_ATTEMPTS {
                std::thread::sleep(POLL_INTERVAL);
                // Still drain commands so `Shutdown` is honoured.
                while let Ok(command) = commands.try_recv() {
                    if matches!(command, ApiCommand::Shutdown) {
                        return;
                    }
                }
                continue;
            }
            attempts += 1;
            match connect_data_plane(&path, &events) {
                Ok(connected) => {
                    attempts = 0;
                    plane = Some(connected);
                }
                Err(error) => {
                    emit(
                        &events,
                        ApiEvent::Error {
                            message: format!(
                                "data-plane connect attempt {attempts} failed: {error}"
                            ),
                        },
                    );
                    std::thread::sleep(RECONNECT_BACKOFF);
                    continue;
                }
            }
        }

        let mut shutdown = false;
        while let Ok(command) = commands.try_recv() {
            match command {
                ApiCommand::Shutdown => {
                    shutdown = true;
                    break;
                }
                ApiCommand::AttachBlock { block_id, takeover } => {
                    if let Some(plane) = plane.as_mut() {
                        if let Err(error) = plane.attach(&block_id, takeover) {
                            emit(&events, ApiEvent::Error { message: error });
                        }
                    }
                }
                ApiCommand::Detach { stream_id } => {
                    if let Some(plane) = plane.as_mut() {
                        if let Err(error) = plane.detach(stream_id) {
                            emit(&events, ApiEvent::Error { message: error });
                        }
                    }
                }
                ApiCommand::Input { stream_id, bytes } => {
                    if let Some(plane) = plane.as_mut() {
                        if let Err(error) = plane.send_input(stream_id, bytes) {
                            emit(&events, ApiEvent::Error { message: error });
                        }
                    }
                }
                ApiCommand::Resize {
                    stream_id,
                    cols,
                    rows,
                    pixel_width,
                    pixel_height,
                } => {
                    if let Some(plane) = plane.as_mut() {
                        if let Err(error) =
                            plane.send_resize(stream_id, cols, rows, pixel_width, pixel_height)
                        {
                            emit(&events, ApiEvent::Error { message: error });
                        }
                    }
                }
                _ => {}
            }
        }
        if shutdown {
            return;
        }

        if let Some(active_plane) = plane.as_mut() {
            if let Err(error) = active_plane.pump() {
                emit(
                    &events,
                    ApiEvent::Disconnected {
                        reason: format!("data connection lost: {error}"),
                    },
                );
                plane = None;
            }
        }
    }
}
