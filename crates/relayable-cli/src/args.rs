//! Command-line parsing for the `relayable` binary.
//!
//! The surface is frozen by the CORE-001 CLI assignment: no extra flags and no
//! extra subcommands. The parser is hand-written because the approved
//! dependency set excludes `clap` (sync/blocking, minimal-dependency slice).

use std::fmt;

/// A parsed command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// `--help` / `-h`.
    Help,
    /// `--version` / `-V` / `version [--json]`.
    Version { json: bool },
    /// `ls [--json]`.
    Ls { json: bool },
    /// `new [--json]`.
    New { json: bool },
    /// `session inspect [-s|--session <target>] [--json]`.
    SessionInspect { session: Option<String>, json: bool },
    /// `session close [-s|--session <target>] [--json]`.
    SessionClose { session: Option<String>, json: bool },
}

/// The command line could not be parsed (exit code 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsageError;

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid command line")
    }
}

impl std::error::Error for UsageError {}

/// Usage text. The product and binary are `relayable`; the `rex` spellings in
/// `docs/compatibility/reference-behavior-matrix.md` are reference evidence
/// only (`full.md` §0.1).
pub const USAGE: &str = "\
relayable — Relayable command-line client

USAGE:
    relayable --help | -h
    relayable --version | -V
    relayable version [--json]
    relayable ls [--json]
    relayable new [--json]
    relayable session inspect [-s|--session <target>] [--json]
    relayable session close   [-s|--session <target>] [--json]

ENVIRONMENT:
    RELAYABLE_SERVER     Unix socket path of the server
    RELAYABLE_SESSION    Default session target for -s/--session";

/// Parse `args` (excluding the program name).
///
/// # Errors
///
/// Returns [`UsageError`] for an unknown command, an unknown flag, or a flag
/// missing its value.
pub fn parse(args: &[String]) -> Result<Command, UsageError> {
    match args.first().map(String::as_str) {
        Some("--help" | "-h") if args.len() == 1 => Ok(Command::Help),
        Some("--version" | "-V") if args.len() == 1 => Ok(Command::Version { json: false }),
        Some("version") => parse_json_only(&args[1..]).map(|json| Command::Version { json }),
        Some("ls") => parse_json_only(&args[1..]).map(|json| Command::Ls { json }),
        Some("new") => parse_json_only(&args[1..]).map(|json| Command::New { json }),
        Some("session") => parse_session(&args[1..]),
        _ => Err(UsageError),
    }
}

/// Parse `[--json]` only.
fn parse_json_only(rest: &[String]) -> Result<bool, UsageError> {
    let mut json = false;
    for arg in rest {
        match arg.as_str() {
            "--json" => json = true,
            _ => return Err(UsageError),
        }
    }
    Ok(json)
}

/// Parse `session <inspect|close> [-s|--session <target>] [--json]`.
fn parse_session(rest: &[String]) -> Result<Command, UsageError> {
    let subcommand = rest.first().ok_or(UsageError)?;
    let mut session: Option<String> = None;
    let mut json = false;
    let mut index = 1;
    while index < rest.len() {
        match rest[index].as_str() {
            "-s" | "--session" => {
                index += 1;
                let value = rest.get(index).ok_or(UsageError)?;
                if value.is_empty() {
                    return Err(UsageError);
                }
                session = Some(value.clone());
            }
            "--json" => json = true,
            _ => return Err(UsageError),
        }
        index += 1;
    }
    match subcommand.as_str() {
        "inspect" => Ok(Command::SessionInspect { session, json }),
        "close" => Ok(Command::SessionClose { session, json }),
        _ => Err(UsageError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn help_and_version() {
        assert_eq!(parse(&args(&["--help"])), Ok(Command::Help));
        assert_eq!(parse(&args(&["-h"])), Ok(Command::Help));
        assert_eq!(
            parse(&args(&["--version"])),
            Ok(Command::Version { json: false })
        );
        assert_eq!(parse(&args(&["-V"])), Ok(Command::Version { json: false }));
        assert_eq!(
            parse(&args(&["version", "--json"])),
            Ok(Command::Version { json: true })
        );
    }

    #[test]
    fn simple_commands() {
        assert_eq!(parse(&args(&["ls"])), Ok(Command::Ls { json: false }));
        assert_eq!(
            parse(&args(&["ls", "--json"])),
            Ok(Command::Ls { json: true })
        );
        assert_eq!(parse(&args(&["new"])), Ok(Command::New { json: false }));
        assert_eq!(
            parse(&args(&["new", "--json"])),
            Ok(Command::New { json: true })
        );
    }

    #[test]
    fn session_commands() {
        assert_eq!(
            parse(&args(&["session", "inspect"])),
            Ok(Command::SessionInspect {
                session: None,
                json: false
            })
        );
        assert_eq!(
            parse(&args(&["session", "inspect", "-s", "work", "--json"])),
            Ok(Command::SessionInspect {
                session: Some("work".to_string()),
                json: true
            })
        );
        assert_eq!(
            parse(&args(&["session", "close", "--session", "work"])),
            Ok(Command::SessionClose {
                session: Some("work".to_string()),
                json: false
            })
        );
    }

    #[test]
    fn unknown_command_is_a_usage_error() {
        assert_eq!(parse(&args(&["bogus"])), Err(UsageError));
        assert_eq!(parse(&[]), Err(UsageError));
        assert_eq!(parse(&args(&["ls", "--bogus"])), Err(UsageError));
        assert_eq!(parse(&args(&["session", "bogus"])), Err(UsageError));
        assert_eq!(parse(&args(&["help"])), Err(UsageError));
    }

    #[test]
    fn missing_session_value_is_a_usage_error() {
        assert_eq!(parse(&args(&["session", "inspect", "-s"])), Err(UsageError));
        assert_eq!(
            parse(&args(&["session", "close", "--session"])),
            Err(UsageError)
        );
        assert_eq!(
            parse(&args(&["session", "inspect", "-s", ""])),
            Err(UsageError)
        );
    }

    #[test]
    fn extra_arguments_are_rejected() {
        assert_eq!(parse(&args(&["--help", "extra"])), Err(UsageError));
        assert_eq!(parse(&args(&["ls", "extra"])), Err(UsageError));
    }
}
