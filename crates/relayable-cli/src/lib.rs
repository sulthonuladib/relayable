//! Relayable command-line client library.
//!
//! The shipped binary is `relayable` (`full.md` §0.1). This module wires the
//! frozen command surface (see [`args`]) to the synchronous
//! [`relayable_client::Client`] and renders either human output or the stable
//! JSON shapes in [`json`]. Target resolution (full ID, case-insensitive label,
//! 1-based position, ID suffix) lives in [`resolve`], never in
//! `relayable-model` (contract §1).
//!
//! `relayable-cli` depends only on `{model, protocol, client}` (ADR-001 §7), so
//! there is deliberately no `server` subcommand: the foreground server is the
//! separate `relayable-server` binary.

#![deny(unsafe_code)]

pub mod args;
pub mod exit;
pub mod json;
pub mod resolve;

mod command;
mod error;
mod output;

pub use command::run;

#[cfg(test)]
use error::CliError;
#[cfg(test)]
use relayable_client::ClientError;
#[cfg(test)]
use resolve::ResolveError;

#[cfg(test)]
mod tests {
    use super::*;
    use relayable_protocol::v1::ErrorCode;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn help_exits_zero() {
        assert_eq!(run(&args(&["--help"])), exit::SUCCESS);
        assert_eq!(run(&args(&["-h"])), exit::SUCCESS);
    }

    #[test]
    fn unknown_command_exits_two() {
        assert_eq!(run(&args(&["bogus"])), exit::USAGE);
        assert_eq!(run(&[]), exit::USAGE);
        assert_eq!(run(&args(&["session", "inspect", "-s"])), exit::USAGE);
    }

    #[test]
    fn version_is_offline() {
        assert_eq!(run(&args(&["version"])), exit::SUCCESS);
        assert_eq!(run(&args(&["version", "--json"])), exit::SUCCESS);
        assert_eq!(run(&args(&["--version"])), exit::SUCCESS);
        assert_eq!(run(&args(&["-V"])), exit::SUCCESS);
    }

    #[test]
    fn cli_error_exit_codes() {
        let missing = CliError::Resolve(ResolveError::Missing);
        assert_eq!(missing.exit_code(), exit::USAGE);
        assert_eq!(missing.code(), "bad_request");

        let not_found = CliError::Resolve(ResolveError::NotFound("x".to_string()));
        assert_eq!(not_found.exit_code(), exit::TARGET);
        assert_eq!(not_found.code(), "unknown_entity");

        let ambiguous = CliError::Resolve(ResolveError::Ambiguous {
            target: Some("work".to_string()),
            matches: vec![resolve::SessionMatch {
                session_id: "session:01a10cf3-2655-7e48-9b9c-43dd6612bef5".to_string(),
                label: None,
            }],
        });
        assert_eq!(ambiguous.exit_code(), exit::TARGET);
        assert_eq!(ambiguous.code(), "ambiguous_target");
        assert!(ambiguous.message().contains("ambiguous"));
    }

    #[test]
    fn server_error_is_rendered_with_details() {
        let mut details = std::collections::HashMap::new();
        details.insert("hint".to_string(), "try again".to_string());
        let err = CliError::Client(ClientError::Server(relayable_protocol::v1::Error {
            code: ErrorCode::UnknownEntity as i32,
            message: "no such session".to_string(),
            details,
            retryable: false,
        }));
        assert_eq!(err.exit_code(), exit::TARGET);
        let rendered = err.render();
        assert_eq!(rendered["error"]["code"], "unknown_entity");
        assert_eq!(rendered["error"]["message"], "no such session");
        assert_eq!(rendered["error"]["details"]["hint"], "try again");
    }

    #[test]
    fn incomplete_target_without_server_is_not_reachable_through_parse() {
        // `session inspect` with `-s` missing a value never reaches a connect.
        assert_eq!(run(&args(&["session", "close", "--session"])), exit::USAGE);
    }
}
