//! Request dispatch for the CORE-001 method set.
//!
//! Every handler validates the target, performs the mutation or read against the
//! server registry, and answers with exactly one `Response`. Errors are always
//! structured contract errors; malformed input never panics the server.

use std::collections::HashMap;
use std::sync::atomic::Ordering;

use relayable_model::TerminalSize;
use relayable_protocol::ids;
use relayable_protocol::v1::{
    self, envelope::Body, request, response, AttachMode, BlockDetachResponse, Envelope, ErrorCode,
    FlowControl, Request, TerminalInputAccepted, TerminalResizeAccepted,
};

use crate::connection::{expected_method, ConnectionCtx, StreamEntry};
use crate::events::{writer_lease_event, DisconnectReason};
use crate::lock;
use crate::registry::CloseSessionOutcome;
use crate::session::{InputError, Subscriber, WriterAcquire};

/// Handle one inbound envelope.
pub(crate) fn handle_envelope(ctx: &ConnectionCtx, envelope: Envelope) {
    match envelope.body {
        Some(Body::Request(request)) => handle_request(ctx, envelope.request_id, request),
        Some(Body::FlowControl(flow_control)) => handle_flow_control(ctx, flow_control),
        // Heartbeats and event/terminal frames from a client are ignored: the
        // server only emits heartbeats, and clients never send terminal data.
        _ => {}
    }
}

fn handle_request(ctx: &ConnectionCtx, request_id: Option<u64>, request: Request) {
    let Some(request_id) = request_id else {
        ctx.send_error(
            0,
            ErrorCode::BadRequest,
            "a request envelope must carry request_id",
            HashMap::new(),
            false,
        );
        return;
    };
    if !lock(&ctx.dedup).record(request_id) {
        ctx.send_error(
            request_id,
            ErrorCode::DuplicateRequest,
            "this request_id was already received on this connection",
            HashMap::new(),
            false,
        );
        return;
    }
    let Some(operation) = request.operation else {
        ctx.send_error(
            request_id,
            ErrorCode::MethodNotFound,
            &format!("unknown method {:?}", request.method),
            HashMap::new(),
            false,
        );
        return;
    };
    if request.method != expected_method(&operation) {
        ctx.send_error(
            request_id,
            ErrorCode::MethodNotFound,
            &format!(
                "method {:?} does not match the request payload",
                request.method
            ),
            HashMap::new(),
            false,
        );
        return;
    }

    match operation {
        request::Operation::SessionCreate(create) => create_session(ctx, request_id, create),
        request::Operation::SessionList(list) => list_sessions(ctx, request_id, list),
        request::Operation::SessionView(view) => view_session(ctx, request_id, view),
        request::Operation::SessionClose(close) => close_session(ctx, request_id, close),
        request::Operation::BlockAttach(attach) => attach_block(ctx, request_id, attach),
        request::Operation::BlockDetach(detach) => detach_block(ctx, request_id, detach),
        request::Operation::TerminalInput(input) => terminal_input(ctx, request_id, input),
        request::Operation::TerminalResize(resize) => terminal_resize(ctx, request_id, resize),
        request::Operation::SessionResume(resume) => resume_session(ctx, request_id, resume),
    }
}

fn create_session(ctx: &ConnectionCtx, request_id: u64, request: v1::SessionCreateRequest) {
    match ctx.shared.create_session(request.label) {
        Ok(data) => ctx.send_result(
            request_id,
            response::Result::SessionCreate(v1::SessionCreateResponse {
                session: Some(data.session),
                window_id: data.window_id.to_string(),
                block_id: data.block_id.to_string(),
                pty_process_id: data.pty_process_id.to_string(),
                program_pid: data.program_pid.unwrap_or(0),
            }),
        ),
        Err(error) => ctx.send_error(
            request_id,
            ErrorCode::Internal,
            &format!("failed to create session: {error}"),
            HashMap::new(),
            true,
        ),
    }
}

fn list_sessions(ctx: &ConnectionCtx, request_id: u64, _request: v1::SessionListRequest) {
    ctx.send_result(
        request_id,
        response::Result::SessionList(v1::SessionListResponse {
            sessions: ctx.shared.list_active_sessions(),
            sequence: ctx.shared.sequence(),
        }),
    );
}

fn view_session(ctx: &ConnectionCtx, request_id: u64, request: v1::SessionSnapshotRequest) {
    let session_id = match ids::session_id_from_wire(&request.session_id) {
        Ok(id) => id,
        Err(error) => {
            return ctx.send_error(
                request_id,
                ErrorCode::BadRequest,
                &format!("invalid session id: {error}"),
                HashMap::new(),
                false,
            );
        }
    };
    match ctx.shared.snapshot(session_id) {
        Some(snapshot) => {
            ctx.send_result(request_id, response::Result::SessionView(snapshot));
            // A completed view synchronizes the connection; it does NOT
            // subscribe it to live events (frozen CORE-002 §C).
            ctx.shared.client_ready(ctx.client_id);
        }
        None => ctx.send_error(
            request_id,
            ErrorCode::UnknownEntity,
            "no such session",
            HashMap::new(),
            false,
        ),
    }
}

/// `session.resume`: replay retained events or fall back to a fresh snapshot.
///
/// The response enqueue and subscriber registration happen inside the single
/// [`crate::registry::ServerShared::resume_session`] bus-lock critical section,
/// so the response always precedes any event with sequence greater than the
/// captured head.
fn resume_session(ctx: &ConnectionCtx, request_id: u64, request: v1::SessionResumeRequest) {
    let session_id = match ids::session_id_from_wire(&request.session_id) {
        Ok(id) => id,
        Err(error) => {
            return ctx.send_error(
                request_id,
                ErrorCode::BadRequest,
                &format!("invalid session id: {error}"),
                HashMap::new(),
                false,
            );
        }
    };
    match ctx.shared.resume_session(
        request_id,
        session_id,
        request.last_sequence_applied,
        &ctx.conn,
    ) {
        Ok(true) => ctx.shared.client_ready(ctx.client_id),
        Ok(false) => ctx.conn.close_now(DisconnectReason::TransportError),
        Err(code) => ctx.send_error(request_id, code, "no such session", HashMap::new(), false),
    }
}

fn close_session(ctx: &ConnectionCtx, request_id: u64, request: v1::SessionCloseRequest) {
    let session_id = match ids::session_id_from_wire(&request.session_id) {
        Ok(id) => id,
        Err(error) => {
            return ctx.send_error(
                request_id,
                ErrorCode::BadRequest,
                &format!("invalid session id: {error}"),
                HashMap::new(),
                false,
            );
        }
    };
    match ctx.shared.close_session(session_id) {
        CloseSessionOutcome::Closed(session) => ctx.send_result(
            request_id,
            response::Result::SessionClose(v1::SessionCloseResponse {
                session: Some(session),
            }),
        ),
        CloseSessionOutcome::NotConfirmed { pid } => {
            let mut details = HashMap::new();
            if let Some(pid) = pid {
                details.insert("pid".to_string(), pid.to_string());
            }
            ctx.send_error(
                request_id,
                ErrorCode::Internal,
                "child did not exit within 500 ms",
                details,
                true,
            );
        }
        CloseSessionOutcome::Unknown => ctx.send_error(
            request_id,
            ErrorCode::UnknownEntity,
            "no such session",
            HashMap::new(),
            false,
        ),
    }
}

fn attach_block(ctx: &ConnectionCtx, request_id: u64, request: v1::BlockAttachRequest) {
    let block_id = match ids::block_id_from_wire(&request.block_id) {
        Ok(id) => id,
        Err(error) => {
            return ctx.send_error(
                request_id,
                ErrorCode::BadRequest,
                &format!("invalid block id: {error}"),
                HashMap::new(),
                false,
            );
        }
    };
    let Some(block) = ctx.shared.find_block(block_id) else {
        return ctx.send_error(
            request_id,
            ErrorCode::UnknownEntity,
            "no such block",
            HashMap::new(),
            false,
        );
    };
    let writer = match AttachMode::try_from(request.mode) {
        Ok(AttachMode::Writer) => true,
        Ok(AttachMode::Observer) => false,
        Ok(AttachMode::Unspecified) | Err(_) => {
            return ctx.send_error(
                request_id,
                ErrorCode::BadRequest,
                "block.attach requires mode WRITER or OBSERVER",
                HashMap::new(),
                false,
            );
        }
    };

    let limit =
        usize::try_from(ctx.shared.config.limits.max_concurrent_streams).unwrap_or(usize::MAX);
    if lock(&ctx.streams).len() >= limit {
        let mut details = HashMap::new();
        details.insert("limit".to_string(), limit.to_string());
        return ctx.send_error(
            request_id,
            ErrorCode::BadRequest,
            "max_concurrent_streams exceeded for this connection",
            details,
            false,
        );
    }

    let stream_id = ctx.next_stream.fetch_add(1, Ordering::SeqCst) + 1;
    if writer {
        match block.acquire_writer(ctx.client_id, stream_id, request.takeover) {
            WriterAcquire::Busy => {
                return ctx.send_error(
                    request_id,
                    ErrorCode::InputNotOwned,
                    "another client already holds the writer lease for this block",
                    HashMap::new(),
                    false,
                );
            }
            WriterAcquire::Acquired => {
                ctx.shared.events.append(writer_lease_event(
                    &block.session_id.to_string(),
                    &block.id.to_string(),
                    stream_id,
                    block.revision(),
                    "",
                    &ctx.client_id.to_string(),
                    true,
                ));
            }
            WriterAcquire::Taken { previous_client_id } => {
                ctx.shared.events.append(writer_lease_event(
                    &block.session_id.to_string(),
                    &block.id.to_string(),
                    stream_id,
                    block.revision(),
                    &previous_client_id.to_string(),
                    &ctx.client_id.to_string(),
                    true,
                ));
            }
        }
    }

    let subscriber = std::sync::Arc::new(Subscriber::new(
        stream_id,
        ctx.client_id,
        std::sync::Arc::clone(&ctx.conn),
        ctx.shared.config.limits.credit_window_bytes,
    ));
    // The tail snapshot and subscriber registration happen under the same lock,
    // so output produced at the attach boundary is neither lost nor duplicated.
    let snapshot = block.attach_stream(stream_id, subscriber);
    ctx.send_result(
        request_id,
        response::Result::BlockAttach(v1::BlockAttachResponse {
            stream_id,
            mode: request.mode,
            writer_lease: writer,
            snapshot: Some(snapshot),
        }),
    );
    lock(&ctx.streams).insert(stream_id, StreamEntry { block, writer });
}

fn detach_block(ctx: &ConnectionCtx, request_id: u64, request: v1::BlockDetachRequest) {
    let stream_id = request.stream_id;
    let entry = lock(&ctx.streams).remove(&stream_id);
    match entry {
        Some(entry) if !entry.block.is_dead() => {
            entry.block.remove_subscriber(ctx.client_id, stream_id);
            if entry.writer && entry.block.release_writer(ctx.client_id, stream_id) {
                ctx.shared.events.append(writer_lease_event(
                    &entry.block.session_id.to_string(),
                    &entry.block.id.to_string(),
                    stream_id,
                    entry.block.revision(),
                    &ctx.client_id.to_string(),
                    "",
                    false,
                ));
            }
            ctx.send_result(
                request_id,
                response::Result::BlockDetach(BlockDetachResponse { stream_id }),
            );
        }
        // A stream whose block has been closed is treated as gone.
        Some(_) | None => ctx.send_error(
            request_id,
            ErrorCode::UnknownEntity,
            "unknown stream",
            HashMap::new(),
            false,
        ),
    }
}

fn terminal_input(ctx: &ConnectionCtx, request_id: u64, request: v1::TerminalInput) {
    let Some(entry) = lock(&ctx.streams).get(&request.stream_id).cloned() else {
        return ctx.send_error(
            request_id,
            ErrorCode::UnknownEntity,
            "unknown stream",
            HashMap::new(),
            false,
        );
    };
    if entry.block.is_dead() {
        return ctx.send_error(
            request_id,
            ErrorCode::UnknownEntity,
            "unknown stream",
            HashMap::new(),
            false,
        );
    }
    if !entry.writer || !entry.block.holds_writer(ctx.client_id, request.stream_id) {
        return ctx.send_error(
            request_id,
            ErrorCode::InputNotOwned,
            "this client does not hold the writer lease for the block",
            HashMap::new(),
            false,
        );
    }
    // `accepted_bytes` means "accepted into the bounded input queue", not
    // "written to the PTY"; the dedicated writer thread performs the write.
    match entry.block.enqueue_input(request.bytes) {
        Ok(accepted) => ctx.send_result(
            request_id,
            response::Result::TerminalInput(TerminalInputAccepted {
                stream_id: request.stream_id,
                accepted_bytes: accepted as u64,
            }),
        ),
        Err(InputError::Full) => ctx.send_error(
            request_id,
            ErrorCode::RateLimited,
            "terminal input queue is full",
            HashMap::new(),
            true,
        ),
        Err(InputError::Closed) => ctx.send_error(
            request_id,
            ErrorCode::Internal,
            "the terminal is closing",
            HashMap::new(),
            true,
        ),
    }
}

fn terminal_resize(ctx: &ConnectionCtx, request_id: u64, request: v1::TerminalResize) {
    let Some(entry) = lock(&ctx.streams).get(&request.stream_id).cloned() else {
        return ctx.send_error(
            request_id,
            ErrorCode::UnknownEntity,
            "unknown stream",
            HashMap::new(),
            false,
        );
    };
    if entry.block.is_dead() {
        return ctx.send_error(
            request_id,
            ErrorCode::UnknownEntity,
            "unknown stream",
            HashMap::new(),
            false,
        );
    }
    if !entry.writer || !entry.block.holds_writer(ctx.client_id, request.stream_id) {
        return ctx.send_error(
            request_id,
            ErrorCode::InputNotOwned,
            "this client does not hold the writer lease for the block",
            HashMap::new(),
            false,
        );
    }
    let Some(cols) = u16::try_from(request.cols).ok().filter(|value| *value > 0) else {
        return invalid_resize(ctx, request_id);
    };
    let Some(rows) = u16::try_from(request.rows).ok().filter(|value| *value > 0) else {
        return invalid_resize(ctx, request_id);
    };
    let pixel_width = request.pixel_width.unwrap_or(0);
    let pixel_height = request.pixel_height.unwrap_or(0);
    let (Ok(pixel_width), Ok(pixel_height)) =
        (u16::try_from(pixel_width), u16::try_from(pixel_height))
    else {
        return invalid_resize(ctx, request_id);
    };

    match entry.block.apply_resize(TerminalSize {
        cols,
        rows,
        pixel_width,
        pixel_height,
    }) {
        Ok(()) => ctx.send_result(
            request_id,
            response::Result::TerminalResize(TerminalResizeAccepted {
                stream_id: request.stream_id,
                cols: u32::from(cols),
                rows: u32::from(rows),
            }),
        ),
        Err(error) => ctx.send_error(
            request_id,
            ErrorCode::Internal,
            &format!("failed to resize the terminal: {error}"),
            HashMap::new(),
            false,
        ),
    }
}

fn invalid_resize(ctx: &ConnectionCtx, request_id: u64) {
    ctx.send_error(
        request_id,
        ErrorCode::BadRequest,
        "resize dimensions must be non-zero and fit in 16 bits",
        HashMap::new(),
        false,
    );
}

fn handle_flow_control(ctx: &ConnectionCtx, flow_control: FlowControl) {
    let entry = lock(&ctx.streams).get(&flow_control.stream_id).cloned();
    if let Some(entry) = entry {
        entry.block.add_credit(
            ctx.client_id,
            flow_control.stream_id,
            flow_control.ack_bytes,
        );
    }
}
