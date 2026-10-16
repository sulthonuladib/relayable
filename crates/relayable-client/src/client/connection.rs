use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use relayable_protocol::codec::CodecError;
use relayable_protocol::limits::DEFAULT_LIMITS;
use relayable_protocol::transport::{default_socket_path, FrameConnection, TransportError};
use relayable_protocol::v1::{
    envelope::Body, request::Operation, response::Result as ResponseResult, Envelope, FlowControl,
    Handshake, Heartbeat, Limits, Request, Response, ServerHello,
};
use relayable_protocol::{CONTROL_STREAM_ID, PROTOCOL_VERSION};

use crate::client::Client;
use crate::config::{ClientConfig, CONTROL_RPC_TIMEOUT};
use crate::error::{is_timeout, ClientError};

impl Client {
    /// Connect to `socket_path` and perform the handshake.
    ///
    /// Callers resolve `RELAYABLE_SERVER` themselves, or use [`Client::from_env`].
    ///
    /// # Errors
    ///
    /// [`ClientError::Connect`] if the socket cannot be opened,
    /// [`ClientError::Transport`] if the handshake frame cannot be exchanged,
    /// and [`ClientError::Server`] if the server rejects the handshake.
    pub fn connect(socket_path: &Path, config: ClientConfig) -> Result<Self, ClientError> {
        let conn = FrameConnection::connect(socket_path).map_err(ClientError::Connect)?;
        Self::from_connection(conn, socket_path.to_path_buf(), config)
    }

    /// Connect using `$RELAYABLE_SERVER` if set and non-empty, else the
    /// protocol crate's default socket path.
    ///
    /// # Errors
    ///
    /// See [`Client::connect`].
    pub fn from_env(config: ClientConfig) -> Result<Self, ClientError> {
        let path = match std::env::var_os("RELAYABLE_SERVER") {
            Some(value) if !value.is_empty() => PathBuf::from(value),
            _ => default_socket_path(),
        };
        Self::connect(&path, config)
    }

    /// The path this client connected to.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// The server's handshake reply.
    #[must_use]
    pub fn server_hello(&self) -> &ServerHello {
        &self.hello
    }

    /// The effective limits: the server-advertised limits, or the protocol
    /// defaults when the server advertised none.
    #[must_use]
    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    // --- internals -------------------------------------------------------

    pub(crate) fn from_connection(
        conn: FrameConnection,
        socket_path: PathBuf,
        config: ClientConfig,
    ) -> Result<Self, ClientError> {
        let mut client = Self {
            conn,
            socket_path,
            hello: ServerHello::default(),
            limits: DEFAULT_LIMITS,
            config,
            next_request_id: 1,
            pending: VecDeque::new(),
            pending_events: VecDeque::new(),
            last_heartbeat_sent: Instant::now(),
            control_rpc_timeout: CONTROL_RPC_TIMEOUT,
        };
        let hello = client.perform_handshake()?;
        client.limits = hello.limits.unwrap_or(DEFAULT_LIMITS);
        client.hello = hello;
        Ok(client)
    }

    fn perform_handshake(&mut self) -> Result<ServerHello, ClientError> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: Some(request_id),
            stream_id: CONTROL_STREAM_ID,
            body: Some(Body::Handshake(Handshake {
                client_kind: self.config.client_kind.clone(),
                platform: self.config.platform.clone(),
                client_version: self.config.client_version.clone(),
                protocol_version: PROTOCOL_VERSION.to_string(),
                capabilities: Vec::new(),
            })),
            ..Envelope::default()
        };
        self.set_control_read_timeout()?;
        self.send(&envelope)?;
        let response = self.await_response(request_id)?;
        match response.result {
            Some(ResponseResult::Hello(hello)) => Ok(hello),
            Some(ResponseResult::Error(err)) => Err(ClientError::Server(err)),
            _ => Err(ClientError::Protocol(
                "expected a server hello from the handshake".to_string(),
            )),
        }
    }

    pub(crate) fn request_operation(
        &mut self,
        method: &str,
        operation: Operation,
    ) -> Result<ResponseResult, ClientError> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        self.set_control_read_timeout()?;
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: Some(request_id),
            stream_id: CONTROL_STREAM_ID,
            body: Some(Body::Request(Request {
                method: method.to_string(),
                operation: Some(operation),
            })),
            ..Envelope::default()
        };
        self.send(&envelope)?;
        let response = self.await_response(request_id)?;
        match response.result {
            Some(ResponseResult::Error(err)) => Err(ClientError::Server(err)),
            Some(result) => Ok(result),
            None => Err(ClientError::Protocol(format!(
                "response to {method} carried no result"
            ))),
        }
    }

    /// Read until the response for `request_id` arrives. Unrelated frames are
    /// moved to `deferred` (and restored by the caller) so nothing is lost and
    /// no queued frame is re-examined.
    fn await_response(&mut self, request_id: u64) -> Result<Response, ClientError> {
        let mut deferred: VecDeque<Envelope> = VecDeque::new();
        let result = self.await_response_inner(request_id, &mut deferred);
        for env in deferred.into_iter().rev() {
            self.pending.push_front(env);
        }
        result
    }

    fn await_response_inner(
        &mut self,
        request_id: u64,
        deferred: &mut VecDeque<Envelope>,
    ) -> Result<Response, ClientError> {
        loop {
            let envelope = match self.pending.pop_front() {
                Some(env) => env,
                None => self.recv_network()?,
            };
            match envelope.body {
                Some(Body::Response(response)) => {
                    if response.request_id == request_id {
                        return Ok(response);
                    }
                    return Err(ClientError::Protocol(format!(
                        "response for unexpected request id {}",
                        response.request_id
                    )));
                }
                Some(Body::Event(event)) => self.enqueue_event(event),
                Some(Body::TerminalData(_) | Body::FlowControl(_) | Body::Heartbeat(_)) => {
                    deferred.push_back(envelope);
                }
                _ => {
                    return Err(ClientError::Protocol(
                        "unexpected frame while awaiting a response".to_string(),
                    ));
                }
            }
        }
    }

    pub(crate) fn recv_network(&mut self) -> Result<Envelope, ClientError> {
        match self.conn.recv() {
            Ok(Some(envelope)) => Ok(envelope),
            Ok(None) => Err(ClientError::Disconnected),
            // A read timeout can surface either directly (`TransportError::Io`)
            // or wrapped by the frame codec (`TransportError::Codec` around
            // `CodecError::Io`); both mean the deadline expired.
            Err(TransportError::Io(err)) if is_timeout(&err) => Err(ClientError::Timeout),
            Err(TransportError::Codec(CodecError::Io(err))) if is_timeout(&err) => {
                Err(ClientError::Timeout)
            }
            Err(err) => Err(ClientError::Transport(err)),
        }
    }

    fn send(&mut self, envelope: &Envelope) -> Result<(), ClientError> {
        self.conn.send(envelope).map_err(ClientError::Transport)
    }

    pub(crate) fn heartbeat_interval(&self) -> Duration {
        Duration::from_millis(u64::from(self.limits.heartbeat_interval_millis.max(1)))
    }

    /// Send a heartbeat if the outbound cadence has elapsed since this client
    /// last sent one, advancing the shared baseline only when a heartbeat is
    /// actually sent.
    ///
    /// The baseline lives on the `Client` so it survives across calls: repeated
    /// short waits accumulate toward the cadence instead of resetting it, and a
    /// heartbeat is never sent more often than once per cadence. Inbound frames
    /// never feed this timer.
    pub(crate) fn heartbeat_if_due(&mut self, cadence: Duration) -> Result<(), ClientError> {
        if Instant::now().duration_since(self.last_heartbeat_sent) >= cadence {
            self.send_heartbeat()?;
            self.last_heartbeat_sent = Instant::now();
        }
        Ok(())
    }

    /// The blocking-read cap that wakes the caller when either its own deadline
    /// or the next outbound heartbeat falls first.
    pub(crate) fn next_read_timeout(&self, deadline: Instant, cadence: Duration) -> Duration {
        let now = Instant::now();
        let remaining = deadline.saturating_duration_since(now);
        let until_heartbeat = cadence.saturating_sub(now.duration_since(self.last_heartbeat_sent));
        remaining.min(until_heartbeat)
    }

    /// Read one frame for the event-reading methods. The caller owns the
    /// outbound heartbeat cadence; an expired read deadline is reported as
    /// `Ok(None)` so the caller can re-check its timers rather than mistaking it
    /// for a transport error.
    pub(crate) fn read_event_frame(
        &mut self,
        read_timeout: Duration,
    ) -> Result<Option<Envelope>, ClientError> {
        self.conn
            .set_read_timeout(Some(read_timeout))
            .map_err(|err| ClientError::Transport(TransportError::Io(err)))?;
        match self.recv_network() {
            Ok(envelope) => Ok(Some(envelope)),
            Err(ClientError::Timeout) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn send_heartbeat(&mut self) -> Result<(), ClientError> {
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            stream_id: CONTROL_STREAM_ID,
            body: Some(Body::Heartbeat(Heartbeat { sequence: None })),
            ..Envelope::default()
        };
        self.send(&envelope)
    }

    pub(crate) fn send_flow_control(
        &mut self,
        stream_id: u64,
        ack_bytes: u64,
    ) -> Result<(), ClientError> {
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION.to_string(),
            stream_id,
            body: Some(Body::FlowControl(FlowControl {
                stream_id,
                ack_bytes,
            })),
            ..Envelope::default()
        };
        self.send(&envelope)
    }

    /// Arm the socket read timeout used while waiting for a control response.
    ///
    /// On expiry the read returns a would-block error, which
    /// [`Client::recv_network`] maps to [`ClientError::Timeout`]. A well-behaved
    /// peer that has sent nothing leaves the connection usable; a partially
    /// consumed frame would force a fresh connect.
    fn set_control_read_timeout(&mut self) -> Result<(), ClientError> {
        self.conn
            .set_read_timeout(Some(self.control_rpc_timeout))
            .map_err(|err| ClientError::Transport(TransportError::Io(err)))
    }
}
