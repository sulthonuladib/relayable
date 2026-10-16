//! Tests for the `terminal.resize` client surface: pixel-dimension
//! transmission and the accepted-geometry return value.

use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

use relayable_protocol::v1::{
    envelope::Body, request::Operation, response::Result as ResponseResult, Envelope, Error,
    ErrorCode, Request, TerminalResize, TerminalResizeAccepted,
};

use crate::client::Client;
use crate::error::ClientError;

use super::harness::*;

/// Spawn a server that completes the handshake, captures exactly one
/// `terminal.resize` request, answers it with `accepted`, and forwards the
/// captured request to the test.
fn resize_server(
    accepted: ResponseResult,
) -> (Client, thread::JoinHandle<()>, mpsc::Receiver<Envelope>) {
    let (tx, rx) = mpsc::channel();
    let (client, server) = spawn_scripted_server(move |conn| {
        let Some(request) = recv_or_finished(conn) else {
            return;
        };
        let request_id = request.request_id.unwrap();
        tx.send(request).unwrap();
        conn.send(&response_for(request_id, accepted)).unwrap();
    });
    (client, server, rx)
}

fn terminal_resize_of(envelope: Envelope) -> TerminalResize {
    match envelope.body {
        Some(Body::Request(Request {
            operation: Some(Operation::TerminalResize(request)),
            ..
        })) => request,
        other => panic!("expected a terminal.resize request, got {other:?}"),
    }
}

#[test]
fn resize_with_pixels_sends_pixels_and_returns_the_accepted_geometry() {
    let (mut client, server, rx) =
        resize_server(ResponseResult::TerminalResize(TerminalResizeAccepted {
            stream_id: 9,
            cols: 120,
            rows: 40,
        }));

    let accepted = client
        .resize_with_pixels(9, 120, 40, Some(1920), Some(1080))
        .unwrap();
    assert_eq!(accepted.stream_id, 9);
    assert_eq!(accepted.cols, 120, "returns the server-accepted columns");
    assert_eq!(accepted.rows, 40, "returns the server-accepted rows");

    let request = terminal_resize_of(rx.recv().unwrap());
    assert_eq!(request.stream_id, 9);
    assert_eq!(request.cols, 120);
    assert_eq!(request.rows, 40);
    assert_eq!(request.pixel_width, Some(1920));
    assert_eq!(request.pixel_height, Some(1080));
    drop(client);
    let _ = server.join();
}

#[test]
fn resize_with_pixels_omits_absent_pixel_dimensions() {
    let (mut client, server, rx) =
        resize_server(ResponseResult::TerminalResize(TerminalResizeAccepted {
            stream_id: 3,
            cols: 80,
            rows: 24,
        }));

    client.resize_with_pixels(3, 80, 24, None, None).unwrap();

    let request = terminal_resize_of(rx.recv().unwrap());
    assert_eq!(request.pixel_width, None, "absent pixel width stays unset");
    assert_eq!(
        request.pixel_height, None,
        "absent pixel height stays unset"
    );
    drop(client);
    let _ = server.join();
}

#[test]
fn plain_resize_sends_no_pixel_dimensions_and_discards_the_geometry() {
    let (mut client, server, rx) =
        resize_server(ResponseResult::TerminalResize(TerminalResizeAccepted {
            stream_id: 5,
            cols: 100,
            rows: 40,
        }));

    // The documented signature returns `()`; the accepted geometry is dropped.
    client.resize(5, 100, 40).unwrap();

    let request = terminal_resize_of(rx.recv().unwrap());
    assert_eq!(request.stream_id, 5);
    assert_eq!(request.cols, 100);
    assert_eq!(request.rows, 40);
    assert_eq!(request.pixel_width, None);
    assert_eq!(request.pixel_height, None);
    drop(client);
    let _ = server.join();
}

#[test]
fn resize_with_pixels_returns_the_server_error() {
    let (mut client, server, _rx) = resize_server(ResponseResult::Error(Error {
        code: ErrorCode::InputNotOwned as i32,
        message: "this client does not hold the writer lease for the block".to_string(),
        details: HashMap::new(),
        retryable: false,
    }));

    let err = client
        .resize_with_pixels(1, 80, 24, Some(640), Some(480))
        .unwrap_err();
    assert!(
        matches!(err, ClientError::Server(_)),
        "expected a server error, got {err:?}"
    );
    assert_eq!(err.server_error_code(), Some(ErrorCode::InputNotOwned));
    drop(client);
    let _ = server.join();
}
