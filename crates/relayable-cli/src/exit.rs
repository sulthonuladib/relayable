//! Exit codes and error-code mapping.
//!
//! The exit table is adopted verbatim from the reference behavior matrix §7.5
//! (documented `rex` status semantics; the shipped binary is `relayable`):
//!
//! | Code | Meaning |
//! | --- | --- |
//! | 0 | success |
//! | 1 | command failed |
//! | 2 | invalid command line |
//! | 3 | target not found or ambiguous |
//! | 4 | server could not be reached |
//! | 130 | interrupted |
//!
//! The server-error mapping is the one frozen by
//! `protocol/compatibility/core-001-method-set.md` §10: contract
//! `unknown_entity` / `ambiguous_target` → 3; connect/handshake failure or a
//! transport error → 4; local usage error → 2; any other server-reported
//! error code → 1.

use relayable_client::ClientError;
use relayable_protocol::error_code::contract_str;
use relayable_protocol::v1::ErrorCode;

/// The command succeeded.
pub const SUCCESS: i32 = 0;
/// The command failed.
pub const FAILURE: i32 = 1;
/// The command line was invalid.
pub const USAGE: i32 = 2;
/// A target was not found or was ambiguous.
pub const TARGET: i32 = 3;
/// The server could not be reached.
pub const UNREACHABLE: i32 = 4;
/// The command was interrupted (see the note below).
pub const INTERRUPTED: i32 = 130;

/// The exit code for a server-reported error code.
#[must_use]
pub fn for_server_code(code: ErrorCode) -> i32 {
    match code {
        ErrorCode::UnknownEntity | ErrorCode::AmbiguousTarget => TARGET,
        ErrorCode::IncompatibleVersion => UNREACHABLE,
        _ => FAILURE,
    }
}

/// The exit code for a client failure.
#[must_use]
pub fn for_client_error(err: &ClientError) -> i32 {
    match err {
        ClientError::Connect(_)
        | ClientError::Transport(_)
        | ClientError::Disconnected
        | ClientError::NotHandshaken => UNREACHABLE,
        ClientError::Protocol(_) | ClientError::Timeout => FAILURE,
        ClientError::Server(err) => {
            let code = ErrorCode::try_from(err.code).unwrap_or(ErrorCode::Unspecified);
            for_server_code(code)
        }
    }
}

/// The contract lowercase code reported for a client failure in `--json` mode.
///
/// Server errors keep their own contract code. Local failures that have no
/// server code map to the closest contract code: a read timeout is `timeout`
/// and every other local failure is `internal` (documented in
/// `protocol/compatibility/core-001-cli-json.md`).
#[must_use]
pub fn code_for_client_error(err: &ClientError) -> &'static str {
    match err {
        ClientError::Server(err) => {
            let code = ErrorCode::try_from(err.code).unwrap_or(ErrorCode::Unspecified);
            contract_str(code)
        }
        ClientError::Timeout => "timeout",
        _ => "internal",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use relayable_protocol::transport::TransportError;
    use relayable_protocol::v1::Error;
    use std::collections::HashMap;
    use std::io;

    fn server_error(code: ErrorCode) -> ClientError {
        ClientError::Server(Error {
            code: code as i32,
            message: "test".to_string(),
            details: HashMap::new(),
            retryable: false,
        })
    }

    #[test]
    fn every_server_error_code_maps_to_a_documented_exit_code() {
        let cases = [
            (ErrorCode::Unspecified, FAILURE),
            (ErrorCode::BadRequest, FAILURE),
            (ErrorCode::MethodNotFound, FAILURE),
            (ErrorCode::IncompatibleVersion, UNREACHABLE),
            (ErrorCode::Unauthenticated, FAILURE),
            (ErrorCode::Unauthorized, FAILURE),
            (ErrorCode::UnknownEntity, TARGET),
            (ErrorCode::AmbiguousTarget, TARGET),
            (ErrorCode::StaleSequence, FAILURE),
            (ErrorCode::SnapshotRequired, FAILURE),
            (ErrorCode::DuplicateRequest, FAILURE),
            (ErrorCode::InputNotOwned, FAILURE),
            (ErrorCode::Timeout, FAILURE),
            (ErrorCode::Cancelled, FAILURE),
            (ErrorCode::RateLimited, FAILURE),
            (ErrorCode::Internal, FAILURE),
        ];
        for (code, expected) in cases {
            assert_eq!(for_server_code(code), expected, "{code:?}");
            assert_eq!(for_client_error(&server_error(code)), expected, "{code:?}");
        }
    }

    #[test]
    fn local_client_errors_map_to_exit_codes() {
        assert_eq!(SUCCESS, 0);
        assert_eq!(USAGE, 2);
        assert_eq!(INTERRUPTED, 130);

        let connect = ClientError::Connect(io::Error::new(io::ErrorKind::NotFound, "missing"));
        assert_eq!(for_client_error(&connect), UNREACHABLE);
        assert_eq!(code_for_client_error(&connect), "internal");

        let transport = ClientError::Transport(TransportError::Io(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "boom",
        )));
        assert_eq!(for_client_error(&transport), UNREACHABLE);

        assert_eq!(for_client_error(&ClientError::Disconnected), UNREACHABLE);
        assert_eq!(for_client_error(&ClientError::NotHandshaken), UNREACHABLE);
        assert_eq!(
            for_client_error(&ClientError::Protocol("x".into())),
            FAILURE
        );
        assert_eq!(for_client_error(&ClientError::Timeout), FAILURE);
        assert_eq!(code_for_client_error(&ClientError::Timeout), "timeout");
        assert_eq!(
            code_for_client_error(&ClientError::Protocol("x".into())),
            "internal"
        );
    }

    #[test]
    fn server_codes_are_reported_verbatim() {
        assert_eq!(
            code_for_client_error(&server_error(ErrorCode::UnknownEntity)),
            "unknown_entity"
        );
        assert_eq!(
            code_for_client_error(&server_error(ErrorCode::IncompatibleVersion)),
            "incompatible_version"
        );
    }
}
