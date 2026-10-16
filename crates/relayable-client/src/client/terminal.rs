use std::collections::VecDeque;
use std::time::{Duration, Instant};

use relayable_model::{BlockId, SessionId};
use relayable_protocol::ids;
use relayable_protocol::methods;
use relayable_protocol::transport::TransportError;
use relayable_protocol::v1::{
    envelope::Body, request::Operation, response::Result as ResponseResult, AttachMode,
    BlockAttachRequest, BlockAttachResponse, BlockDetachRequest, Envelope, TerminalInput,
    TerminalResize, TerminalResizeAccepted,
};

use crate::client::Client;
use crate::error::ClientError;
use crate::outcome::AttachOutcome;
use crate::rpc::{contains, missing, unexpected};

impl Client {
    /// Attach to a terminal block (`block.attach`).
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn attach_block(
        &mut self,
        block_id: &BlockId,
        mode: AttachMode,
    ) -> Result<AttachOutcome, ClientError> {
        let result = self.request_operation(
            methods::BLOCK_ATTACH,
            Operation::BlockAttach(BlockAttachRequest {
                block_id: ids::block_id_to_wire(block_id),
                mode: mode as i32,
                takeover: false,
            }),
        )?;
        match result {
            ResponseResult::BlockAttach(response) => Ok(AttachOutcome {
                stream_id: response.stream_id,
                mode: AttachMode::try_from(response.mode).unwrap_or(AttachMode::Unspecified),
                writer_lease: response.writer_lease,
                snapshot: response
                    .snapshot
                    .ok_or_else(|| missing("block.attach snapshot"))?,
            }),
            _ => Err(unexpected("block.attach", &result)),
        }
    }

    /// Attach to a terminal block (`block.attach`) without taking over an
    /// existing writer lease (`takeover = false`).
    ///
    /// `session` is accepted for call-site parity with
    /// [`Client::attach_with_takeover`]; `block.attach` itself is addressed by
    /// `block` plus the acting client identity on the local socket
    /// (`protocol/compatibility/core-002-method-set.md` §E).
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn attach(
        &mut self,
        session: &SessionId,
        block: &BlockId,
        mode: AttachMode,
    ) -> Result<BlockAttachResponse, ClientError> {
        self.attach_with_takeover(session, block, mode, false)
    }

    /// Attach to a terminal block (`block.attach`), optionally taking over the
    /// single writer lease from the current holder (CORE-002, contract §5).
    ///
    /// `takeover = false` is exactly [`Client::attach`] / [`Client::attach_block`]
    /// (a second writer is rejected with `input_not_owned`). `takeover = true`
    /// asks the server to transfer the lease and emit `writer_lease_changed`.
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn attach_with_takeover(
        &mut self,
        _session: &SessionId,
        block: &BlockId,
        mode: AttachMode,
        takeover: bool,
    ) -> Result<BlockAttachResponse, ClientError> {
        let result = self.request_operation(
            methods::BLOCK_ATTACH,
            Operation::BlockAttach(BlockAttachRequest {
                block_id: ids::block_id_to_wire(block),
                mode: mode as i32,
                takeover,
            }),
        )?;
        match result {
            ResponseResult::BlockAttach(response) => Ok(response),
            _ => Err(unexpected("block.attach", &result)),
        }
    }

    /// Detach from a terminal block (`block.detach`).
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn detach_block(&mut self, stream_id: u64) -> Result<(), ClientError> {
        let result = self.request_operation(
            methods::BLOCK_DETACH,
            Operation::BlockDetach(BlockDetachRequest { stream_id }),
        )?;
        match result {
            ResponseResult::BlockDetach(_) => Ok(()),
            _ => Err(unexpected("block.detach", &result)),
        }
    }

    /// Send terminal input (`terminal.input`); returns the accepted byte count.
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn send_input(&mut self, stream_id: u64, bytes: &[u8]) -> Result<u64, ClientError> {
        let result = self.request_operation(
            methods::TERMINAL_INPUT,
            Operation::TerminalInput(TerminalInput {
                stream_id,
                bytes: bytes.to_vec(),
            }),
        )?;
        match result {
            ResponseResult::TerminalInput(accepted) => Ok(accepted.accepted_bytes),
            _ => Err(unexpected("terminal.input", &result)),
        }
    }

    /// Resize a terminal stream (`terminal.resize`), reporting the pane's pixel
    /// dimensions alongside its column/row count, and return the geometry the
    /// server accepted.
    ///
    /// `pixel_width`/`pixel_height` are the pane's pixel dimensions (for
    /// example the 1920x1080 a GUI pane measured). Both are optional on the wire
    /// (`protocol/schema/terminal.proto`: `optional uint32 pixel_width = 4;
    /// optional uint32 pixel_height = 5;`); `None` omits the field, and the
    /// server then substitutes `0`
    /// (`crates/relayable-server/src/dispatch.rs`:
    /// `request.pixel_width.unwrap_or(0)`). The server applies the whole
    /// geometry (columns, rows, and both pixel dimensions) to the block's PTY
    /// window size, so the returned [`TerminalResizeAccepted`] carries the
    /// dimensions the **server** accepted, not the values sent.
    ///
    /// [`Client::resize`] is this same call with no pixel dimensions.
    ///
    /// # Errors
    ///
    /// [`ClientError::Server`] when the server rejects the resize (for example
    /// when this client does not hold the writer lease), and any other
    /// [`ClientError`] from the transport.
    pub fn resize_with_pixels(
        &mut self,
        stream_id: u64,
        cols: u16,
        rows: u16,
        pixel_width: Option<u16>,
        pixel_height: Option<u16>,
    ) -> Result<TerminalResizeAccepted, ClientError> {
        let result = self.request_operation(
            methods::TERMINAL_RESIZE,
            Operation::TerminalResize(TerminalResize {
                stream_id,
                cols: u32::from(cols),
                rows: u32::from(rows),
                pixel_width: pixel_width.map(u32::from),
                pixel_height: pixel_height.map(u32::from),
            }),
        )?;
        match result {
            ResponseResult::TerminalResize(accepted) => Ok(accepted),
            _ => Err(unexpected("terminal.resize", &result)),
        }
    }

    /// Resize a terminal stream (`terminal.resize`) with columns and rows only.
    ///
    /// This is [`Client::resize_with_pixels`] with no pixel dimensions, and it
    /// discards the server's accepted geometry.
    ///
    /// # Errors
    ///
    /// See [`ClientError`].
    pub fn resize(&mut self, stream_id: u64, cols: u16, rows: u16) -> Result<(), ClientError> {
        self.resize_with_pixels(stream_id, cols, rows, None, None)?;
        Ok(())
    }

    /// Read `TerminalOutput` frames for `stream_id` until `needle` appears in
    /// the accumulated bytes, or `timeout` elapses ([`ClientError::Timeout`]).
    ///
    /// While reading, consumed bytes are acknowledged with a `FlowControl`
    /// frame, a `Heartbeat` is sent whenever an idle read deadline expires, and
    /// the socket read timeout is held at or below the heartbeat interval.
    /// Frames that do not belong to the target stream (events, responses for
    /// other request ids, other streams) are retained and never lost.
    ///
    /// # Errors
    ///
    /// [`ClientError::Timeout`] when `needle` does not appear before the
    /// deadline, and any other [`ClientError`] from the transport.
    pub fn wait_for_output(
        &mut self,
        stream_id: u64,
        needle: &[u8],
        timeout: Duration,
    ) -> Result<Vec<u8>, ClientError> {
        let mut deferred: VecDeque<Envelope> = VecDeque::new();
        let result = self.wait_for_output_inner(stream_id, needle, timeout, &mut deferred);
        // Restore everything we set aside, preserving arrival order.
        for env in deferred.into_iter().rev() {
            self.pending.push_front(env);
        }
        result
    }

    /// Read the next `TerminalOutput` frame for `stream_id` and return its
    /// bytes, or `Ok(None)` when `timeout` elapses with no such frame.
    ///
    /// Each returned chunk is acknowledged exactly once through the same
    /// `FlowControl` credit path as [`Client::wait_for_output`], so a caller
    /// that keeps consuming keeps the server's credit window refilled. Unlike
    /// [`Client::wait_for_output`], an expired deadline is **not** an error:
    /// bytes are acknowledged only when they are returned in the same call, so
    /// a timeout never acknowledges and then discards output. A zero `timeout`
    /// never waits on the deadline and performs no socket read: it serves any
    /// already-buffered target bytes and otherwise returns `Ok(None)` promptly.
    /// The only I/O it may still perform is the pending ack or heartbeat write.
    /// Frames for other streams, events, and other control frames are
    /// retained in arrival order and never dropped; events stay reachable
    /// through [`Client::drain_events`], [`Client::wait_for_event`], and
    /// [`Client::collect_events`].
    ///
    /// # Errors
    ///
    /// [`ClientError::Disconnected`] if the peer closes the connection, and any
    /// other [`ClientError`] from the transport or from acknowledging output.
    pub fn next_output(
        &mut self,
        stream_id: u64,
        timeout: Duration,
    ) -> Result<Option<Vec<u8>>, ClientError> {
        let heartbeat = self.heartbeat_interval();
        let start = Instant::now();
        let deadline = start.checked_add(timeout).unwrap_or(start);

        // Serve anything already buffered before touching the socket, at any
        // timeout. The scan is bounded by the queue length and leaves every
        // frame it does not consume in its original arrival order (a bare
        // rotation would move frames popped before a match behind frames that
        // followed it), so repeated calls stay lossless and ordered for every
        // stream.
        let buffered = self.pending.len();
        let mut deferred: VecDeque<Envelope> = VecDeque::new();
        for _ in 0..buffered {
            let Some(envelope) = self.pending.pop_front() else {
                break;
            };
            match envelope.body {
                Some(Body::TerminalData(output)) if output.stream_id == stream_id => {
                    for env in deferred.into_iter().rev() {
                        self.pending.push_front(env);
                    }
                    let len = output.bytes.len() as u64;
                    self.send_flow_control(stream_id, len)?;
                    return Ok(Some(output.bytes));
                }
                Some(Body::Event(event)) => self.enqueue_event(event),
                _ => deferred.push_back(envelope),
            }
        }
        for env in deferred.into_iter().rev() {
            self.pending.push_front(env);
        }

        loop {
            self.heartbeat_if_due(heartbeat)?;
            if Instant::now() >= deadline {
                return Ok(None);
            }
            let read_timeout = self.next_read_timeout(deadline, heartbeat);
            if read_timeout.is_zero() {
                continue;
            }
            self.conn
                .set_read_timeout(Some(read_timeout))
                .map_err(|err| ClientError::Transport(TransportError::Io(err)))?;
            let envelope = match self.recv_network() {
                Ok(envelope) => envelope,
                Err(ClientError::Timeout) => continue,
                Err(err) => return Err(err),
            };
            match envelope.body {
                Some(Body::TerminalData(output)) if output.stream_id == stream_id => {
                    let len = output.bytes.len() as u64;
                    self.send_flow_control(stream_id, len)?;
                    return Ok(Some(output.bytes));
                }
                Some(Body::Event(event)) => self.enqueue_event(event),
                _ => self.pending.push_back(envelope),
            }
        }
    }

    fn wait_for_output_inner(
        &mut self,
        stream_id: u64,
        needle: &[u8],
        timeout: Duration,
        deferred: &mut VecDeque<Envelope>,
    ) -> Result<Vec<u8>, ClientError> {
        let heartbeat = self.heartbeat_interval();
        let start = Instant::now();
        let deadline = start.checked_add(timeout).unwrap_or(start);
        let mut accumulated: Vec<u8> = Vec::new();

        loop {
            if needle.is_empty() || contains(&accumulated, needle) {
                return Ok(accumulated);
            }
            if Instant::now() >= deadline {
                return Err(ClientError::Timeout);
            }

            self.heartbeat_if_due(heartbeat)?;
            if Instant::now() >= deadline {
                return Err(ClientError::Timeout);
            }
            let read_timeout = self.next_read_timeout(deadline, heartbeat);
            if read_timeout.is_zero() {
                continue;
            }
            self.conn
                .set_read_timeout(Some(read_timeout))
                .map_err(|err| ClientError::Transport(TransportError::Io(err)))?;

            let frame = match self.pending.pop_front() {
                Some(env) => Some(env),
                None => match self.recv_network() {
                    Ok(env) => Some(env),
                    Err(ClientError::Timeout) => None,
                    Err(err) => return Err(err),
                },
            };

            match frame {
                // A read-deadline expiry is not an error here: the loop top
                // sends a heartbeat if the send cadence elapsed, or returns
                // `Timeout` once the caller's deadline passes.
                None => {}
                Some(envelope) => match envelope.body {
                    Some(Body::TerminalData(output)) if output.stream_id == stream_id => {
                        let len = output.bytes.len() as u64;
                        accumulated.extend_from_slice(&output.bytes);
                        if len > 0 {
                            self.send_flow_control(stream_id, len)?;
                        }
                    }
                    Some(Body::Event(event)) => self.enqueue_event(event),
                    _ => deferred.push_back(envelope),
                },
            }
        }
    }
}
