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

use relayable_client::{Client, ClientConfig, ClientError};
use relayable_protocol::ids;
use relayable_protocol::v1::SessionSnapshotResponse;
use serde_json::{Map, Value};

use args::Command;
use resolve::ResolveError;

/// A failure that maps to a documented exit code.
#[derive(Debug)]
enum CliError {
    Client(ClientError),
    Resolve(ResolveError),
}

impl CliError {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Client(err) => exit::for_client_error(err),
            Self::Resolve(ResolveError::NotFound(_) | ResolveError::Ambiguous { .. }) => {
                exit::TARGET
            }
            Self::Resolve(ResolveError::Missing) => exit::USAGE,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::Client(err) => exit::code_for_client_error(err),
            Self::Resolve(ResolveError::NotFound(_)) => "unknown_entity",
            Self::Resolve(ResolveError::Ambiguous { .. }) => "ambiguous_target",
            Self::Resolve(ResolveError::Missing) => "bad_request",
        }
    }

    fn message(&self) -> String {
        match self {
            Self::Client(ClientError::Server(err)) => err.message.clone(),
            Self::Client(err) => err.to_string(),
            Self::Resolve(ResolveError::Missing) => {
                "no session target given and no sessions exist".to_string()
            }
            Self::Resolve(ResolveError::NotFound(target)) => {
                format!("session target \"{target}\" not found")
            }
            Self::Resolve(ResolveError::Ambiguous {
                target: Some(target),
                ..
            }) => format!("session target \"{target}\" is ambiguous"),
            Self::Resolve(ResolveError::Ambiguous { target: None, .. }) => {
                "several sessions exist; specify one with -s/--session".to_string()
            }
        }
    }

    fn details(&self) -> Value {
        match self {
            Self::Client(ClientError::Server(err)) => {
                let details: Map<String, Value> = err
                    .details
                    .iter()
                    .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                    .collect();
                Value::Object(details)
            }
            Self::Resolve(ResolveError::Ambiguous { matches, .. }) => {
                json::ambiguous_details(matches)
            }
            _ => Value::Object(Map::new()),
        }
    }

    fn render(&self) -> Value {
        json::error_json(self.code(), &self.message(), self.details())
    }
}

/// Run the CLI for `args` (excluding the program name) and return the exit
/// code. All output is written to stdout/stderr; the caller exits with the
/// returned code.
#[must_use]
pub fn run(args: &[String]) -> i32 {
    let command = match args::parse(args) {
        Ok(command) => command,
        Err(_) => {
            eprintln!("{}", args::USAGE);
            return exit::USAGE;
        }
    };

    match command {
        Command::Help => {
            println!("{}", args::USAGE);
            exit::SUCCESS
        }
        Command::Version { json } => {
            print_version(json);
            exit::SUCCESS
        }
        Command::Ls { json } => dispatch(json, run_ls(json)),
        Command::New { json } => dispatch(json, run_new(json)),
        Command::SessionInspect { session, json } => {
            dispatch(json, run_inspect(session.as_deref(), json))
        }
        Command::SessionClose { session, json } => {
            dispatch(json, run_close(session.as_deref(), json))
        }
    }
}

fn dispatch(json: bool, result: Result<(), CliError>) -> i32 {
    match result {
        Ok(()) => exit::SUCCESS,
        Err(err) => {
            report(json, &err);
            err.exit_code()
        }
    }
}

fn print_version(json: bool) {
    if json {
        println!("{}", json::version_json());
    } else {
        println!(
            "relayable {} (protocol {})",
            env!("CARGO_PKG_VERSION"),
            relayable_protocol::PROTOCOL_VERSION
        );
    }
}

fn connect() -> Result<Client, CliError> {
    Client::from_env(ClientConfig::new("cli")).map_err(CliError::Client)
}

/// The explicit `-s/--session` value, else `$RELAYABLE_SESSION` when set and
/// non-empty. Socket discovery (`RELAYABLE_SERVER`) is handled by
/// [`Client::from_env`].
fn effective_target(explicit: Option<&str>) -> Option<String> {
    if let Some(value) = explicit {
        return Some(value.to_string());
    }
    std::env::var("RELAYABLE_SESSION")
        .ok()
        .filter(|value| !value.is_empty())
}

fn run_ls(json: bool) -> Result<(), CliError> {
    let mut client = connect()?;
    let sessions = client.list_sessions().map_err(CliError::Client)?;
    if json {
        println!("{}", json::ls_json(&sessions));
    } else {
        for session in &sessions {
            println!(
                "{}  {}",
                session.session_id,
                session.label.as_deref().unwrap_or("-")
            );
        }
    }
    Ok(())
}

fn run_new(json: bool) -> Result<(), CliError> {
    let mut client = connect()?;
    let created = client.create_session(None).map_err(CliError::Client)?;
    if json {
        println!("{}", json::new_json(&created));
    } else {
        println!("{}", created.session.session_id);
        println!("{}", created.window_id);
        println!("{}", created.block_id);
        println!("{}", created.pty_process_id);
        println!("pid:{}", created.program_pid);
    }
    Ok(())
}

fn run_inspect(target: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut client = connect()?;
    let sessions = client.list_sessions().map_err(CliError::Client)?;
    let target = effective_target(target);
    let session =
        resolve::resolve_session(target.as_deref(), &sessions).map_err(CliError::Resolve)?;
    let session_id = ids::session_id_from_wire(&session.session_id)
        .map_err(|err| CliError::Client(ClientError::Protocol(format!("bad session id: {err}"))))?;
    let snapshot = client
        .session_snapshot(&session_id)
        .map_err(CliError::Client)?;
    if json {
        println!("{}", json::inspect_json(&snapshot));
    } else {
        print_snapshot(&snapshot);
    }
    Ok(())
}

fn run_close(target: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut client = connect()?;
    let sessions = client.list_sessions().map_err(CliError::Client)?;
    let target = effective_target(target);
    let session =
        resolve::resolve_session(target.as_deref(), &sessions).map_err(CliError::Resolve)?;
    let session_id = ids::session_id_from_wire(&session.session_id)
        .map_err(|err| CliError::Client(ClientError::Protocol(format!("bad session id: {err}"))))?;
    let record = client
        .close_session(&session_id)
        .map_err(CliError::Client)?;
    if json {
        println!("{}", json::close_json(&record));
    } else {
        println!(
            "{}  {}",
            record.session_id,
            json::session_state_str(record.state)
        );
    }
    Ok(())
}

fn print_snapshot(snapshot: &SessionSnapshotResponse) {
    if let Some(session) = &snapshot.session {
        println!(
            "{}  {}",
            session.session_id,
            session.label.as_deref().unwrap_or("-")
        );
    }
    for window in &snapshot.windows {
        println!(
            "{}  active={}  {}",
            window.window_id,
            window.active,
            window.label.as_deref().unwrap_or("-")
        );
    }
    for block in &snapshot.blocks {
        let program = block
            .program_status
            .as_ref()
            .map_or("unknown", |status| json::program_state_str(status.state));
        println!(
            "{}  {}x{}  {}  pid={}  {}",
            block.block_id,
            block.cols,
            block.rows,
            program,
            block.program_pid,
            block.pty_process_id
        );
    }
}

fn report(json: bool, err: &CliError) {
    if json {
        println!("{}", err.render());
        return;
    }
    eprintln!("Error: {}", err.message());
    if let CliError::Resolve(ResolveError::Ambiguous { matches: found, .. }) = err {
        for matched in found {
            eprintln!(
                "  {}  {}",
                resolve::short_id(&matched.session_id),
                matched.label.as_deref().unwrap_or("-")
            );
        }
    }
}

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
