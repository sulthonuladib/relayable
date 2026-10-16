use std::fmt;
use std::io;

use relayable_protocol::error_code::contract_str;
use relayable_protocol::transport::TransportError;
use relayable_protocol::v1::response::Result as ResponseResult;
use relayable_protocol::v1::{Error, ErrorCode};

#[cfg(doc)]
use crate::client::Client;
#[cfg(doc)]
use crate::config::CONTROL_RPC_TIMEOUT;

/// Failure modes of a client operation.
#[derive(Debug)]
pub enum ClientError {
    /// The socket could not be opened (missing or refused).
    Connect(io::Error),
    /// A framed transport operation failed.
    Transport(TransportError),
    /// The peer sent a malformed or unexpected frame.
    Protocol(String),
    /// The server answered with a structured error; the code, message, and
    /// details are preserved.
    Server(Error),
    /// A read deadline expired: a control RPC (see [`CONTROL_RPC_TIMEOUT`]) or
    /// a [`Client::wait_for_output`] call.
    Timeout,
    /// The peer closed the connection.
    Disconnected,
    /// The handshake never completed.
    NotHandshaken,
}

impl ClientError {
    /// The server-reported code, when this error came from the server.
    #[must_use]
    pub fn server_error_code(&self) -> Option<ErrorCode> {
        match self {
            Self::Server(err) => ErrorCode::try_from(err.code).ok(),
            _ => None,
        }
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(err) => write!(f, "could not connect to the server: {err}"),
            Self::Transport(err) => write!(f, "transport error: {err}"),
            Self::Protocol(msg) => write!(f, "protocol error: {msg}"),
            Self::Server(err) => {
                let code = ErrorCode::try_from(err.code).unwrap_or(ErrorCode::Unspecified);
                write!(f, "server error {}: {}", contract_str(code), err.message)
            }
            Self::Timeout => f.write_str("timed out waiting for the server"),
            Self::Disconnected => f.write_str("the server disconnected"),
            Self::NotHandshaken => f.write_str("the client is not handshaken"),
        }
    }
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Connect(err) => Some(err),
            Self::Transport(err) => Some(err),
            _ => None,
        }
    }
}

pub(crate) fn variant_name(result: &ResponseResult) -> &'static str {
    match result {
        ResponseResult::Error(_) => "error",
        ResponseResult::Hello(_) => "hello",
        ResponseResult::SessionCreate(_) => "session_create",
        ResponseResult::SessionList(_) => "session_list",
        ResponseResult::SessionView(_) => "session_view",
        ResponseResult::SessionClose(_) => "session_close",
        ResponseResult::BlockAttach(_) => "block_attach",
        ResponseResult::BlockDetach(_) => "block_detach",
        ResponseResult::TerminalInput(_) => "terminal_input",
        ResponseResult::TerminalResize(_) => "terminal_resize",
        ResponseResult::SessionResume(_) => "session_resume",
    }
}

pub(crate) fn is_timeout(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    )
}
