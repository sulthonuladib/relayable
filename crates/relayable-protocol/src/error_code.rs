//! Structured error codes (contract §6).
//!
//! [`contract_str`] maps a wire [`ErrorCode`](crate::v1::ErrorCode) to the exact
//! lowercase name used by `docs/architecture/interface-contract.md` §6. Only
//! `bad_request` and `method_not_found` are reused deliberately from the
//! reference docs; the rest are Relayable-defined (no REX evidence).

use crate::v1::ErrorCode;

/// The exact lowercase contract name for an error code.
///
/// `Unspecified` is never sent on the wire; it maps to `"unspecified"`.
#[must_use]
pub fn contract_str(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::Unspecified => "unspecified",
        ErrorCode::BadRequest => "bad_request",
        ErrorCode::MethodNotFound => "method_not_found",
        ErrorCode::IncompatibleVersion => "incompatible_version",
        ErrorCode::Unauthenticated => "unauthenticated",
        ErrorCode::Unauthorized => "unauthorized",
        ErrorCode::UnknownEntity => "unknown_entity",
        ErrorCode::AmbiguousTarget => "ambiguous_target",
        ErrorCode::StaleSequence => "stale_sequence",
        ErrorCode::SnapshotRequired => "snapshot_required",
        ErrorCode::DuplicateRequest => "duplicate_request",
        ErrorCode::InputNotOwned => "input_not_owned",
        ErrorCode::Timeout => "timeout",
        ErrorCode::Cancelled => "cancelled",
        ErrorCode::RateLimited => "rate_limited",
        ErrorCode::Internal => "internal",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_contract_name_is_exact() {
        let pairs = [
            (ErrorCode::BadRequest, "bad_request"),
            (ErrorCode::MethodNotFound, "method_not_found"),
            (ErrorCode::IncompatibleVersion, "incompatible_version"),
            (ErrorCode::Unauthenticated, "unauthenticated"),
            (ErrorCode::Unauthorized, "unauthorized"),
            (ErrorCode::UnknownEntity, "unknown_entity"),
            (ErrorCode::AmbiguousTarget, "ambiguous_target"),
            (ErrorCode::StaleSequence, "stale_sequence"),
            (ErrorCode::SnapshotRequired, "snapshot_required"),
            (ErrorCode::DuplicateRequest, "duplicate_request"),
            (ErrorCode::InputNotOwned, "input_not_owned"),
            (ErrorCode::Timeout, "timeout"),
            (ErrorCode::Cancelled, "cancelled"),
            (ErrorCode::RateLimited, "rate_limited"),
            (ErrorCode::Internal, "internal"),
        ];
        for (code, expected) in pairs {
            assert_eq!(contract_str(code), expected);
        }
        assert_eq!(contract_str(ErrorCode::Unspecified), "unspecified");
    }
}
