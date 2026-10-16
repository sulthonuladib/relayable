//! Command execution: parse the frozen CLI surface, dispatch, and run the
//! `ls` / `new` / `session inspect` / `session close` handlers.
//!
//! [`run`] is the crate entry point and is re-exported at the crate root so
//! `relayable_cli::run` keeps resolving.

use relayable_client::{Client, ClientConfig, ClientError};
use relayable_protocol::ids;

use crate::args::{self, Command};
use crate::error::CliError;
use crate::exit;
use crate::json;
use crate::output::{print_snapshot, print_version, report};
use crate::resolve;

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
