use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Duration;

use vte::Perform;

use super::data::{should_ack, strip_file_uri, tail, OscPerformer, TitleTracker};
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
fn every_attached_stream_is_credit_acked_not_just_one() {
    let mut attached = BTreeSet::new();
    attached.insert(3u64);
    attached.insert(7u64);
    assert!(should_ack(&attached, 3));
    assert!(should_ack(&attached, 7));
    assert!(!should_ack(&attached, 4));
    attached.remove(&7);
    assert!(!should_ack(&attached, 7));
    assert!(should_ack(&attached, 3));
}
