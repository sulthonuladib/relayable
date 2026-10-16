use std::path::PathBuf;
use std::time::Duration;

use vte::Perform;

use super::data::{strip_file_uri, tail, OscPerformer, TitleTracker};
use super::*;

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn api_handle_is_send_and_sync() {
    assert_send_sync::<ApiHandle>();
}

#[test]
fn mock_handle_is_labelled_and_non_blocking() {
    let handle = ApiHandle::connect_mock();
    assert!(handle.is_mock());
    assert_eq!(handle.socket_path(), PathBuf::from("<mock>"));
    // `send` must not block or panic; `try_recv` must not block.
    handle.send(ApiCommand::RefreshSessions);
    let _ = handle.try_recv();
    let _ = handle.drain();
    handle.send(ApiCommand::Shutdown);
}

#[test]
fn mock_replies_to_create_and_attach() {
    let handle = ApiHandle::connect_mock();
    handle.send(ApiCommand::CreateSession {
        label: Some("demo".to_string()),
    });
    handle.send(ApiCommand::AttachBlock {
        session_id: "session:mock".to_string(),
        block_id: "block:mock".to_string(),
        takeover: false,
    });

    let mut seen_created = false;
    let mut seen_attached = false;
    for _ in 0..200 {
        for event in handle.drain() {
            match event {
                ApiEvent::SessionCreated { block_id, .. } => {
                    assert_eq!(block_id, "block:mock");
                    seen_created = true;
                }
                ApiEvent::Attached { cols, rows, .. } => {
                    assert_eq!((cols, rows), (80, 24));
                    seen_attached = true;
                }
                _ => {}
            }
        }
        if seen_created && seen_attached {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(seen_created, "mock must answer session.create");
    assert!(seen_attached, "mock must answer block.attach");
    handle.send(ApiCommand::Shutdown);
}

#[test]
fn strip_file_uri_handles_host_and_bare_paths() {
    assert_eq!(strip_file_uri("file://host/tmp/x"), "/tmp/x");
    assert_eq!(strip_file_uri("file:///tmp/x"), "/tmp/x");
    assert_eq!(strip_file_uri("/tmp/x"), "/tmp/x");
    assert_eq!(strip_file_uri("relative"), "relative");
}

#[test]
fn tail_bounds_a_long_snapshot() {
    let bytes: Vec<u8> = (0..100u8).collect();
    let tailed = tail(&bytes, 10);
    assert_eq!(tailed.len(), 10);
    assert_eq!(tailed, (90..100u8).collect::<Vec<u8>>());
}

#[test]
fn osc_tracker_records_title_and_cwd() {
    let mut tracker = TitleTracker::default();
    let mut performer = OscPerformer {
        tracker: &mut tracker,
        changed: false,
    };
    performer.osc_dispatch(&[b"2", b"my-title"], false);
    performer.osc_dispatch(&[b"7", b"file://host/tmp/work"], false);
    assert!(performer.changed);
    assert_eq!(tracker.title.as_deref(), Some("my-title"));
    assert_eq!(tracker.cwd.as_deref(), Some("/tmp/work"));
}

#[test]
fn zero_timeout_drain_is_bounded_but_never_busy() {
    // The data plane's acknowledgement path is a zero-timeout `next_output` loop
    // (see `data.rs::drain_attached`): it must be short enough to keep a
    // 256 KiB per-stream credit window refilled while a pane floods
    // (`relayable-protocol/src/limits.rs`), and long enough that an idle data
    // thread is not a busy loop.
    let window = super::types::DATA_READ_WINDOW;
    assert!(
        window >= Duration::from_micros(100),
        "an idle plane must sleep"
    );
    assert!(
        window <= Duration::from_millis(10),
        "a busy plane must not fall behind"
    );
}
