//! Relayable server foreground binary.
//!
//! Usage: `relayable-server run` (foreground, no systemd unit, no forking). The
//! only inputs are the environment: the variables read by
//! [`relayable_server::ServerConfig`] (`RELAYABLE_SERVER`,
//! `RELAYABLE_EVENT_RETENTION_EVENTS`, `RELAYABLE_IDLE_TIMEOUT_MILLIS`,
//! `RELAYABLE_HEARTBEAT_INTERVAL_MILLIS`, `SHELL`), the state-directory inputs
//! `RELAYABLE_STATE_DIR` / `XDG_STATE_HOME` / `HOME`, and the durable store
//! retention overrides `RELAYABLE_STORE_{AUDIT_MAX_ROWS,AUDIT_MAX_AGE_MILLIS,`
//! `TOMBSTONES_MAX_ROWS,TOMBSTONES_MAX_AGE_MILLIS,SESSIONS_MAX_ROWS,`
//! `SESSIONS_MAX_AGE_MILLIS}` (read once at startup by
//! `relayable_store::PruneLimits::from_env`).

use std::process::ExitCode;

const USAGE: &str = "\
Usage: relayable-server <command>

Commands:
  run             Run the server in the foreground.

Options:
  -h, --help      Print this help.
      --version   Print the version and the protocol version.
";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("run") => {
            if args.next().is_some() {
                eprint!("{USAGE}");
                ExitCode::from(2)
            } else {
                run()
            }
        }
        Some("-h" | "--help") => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some("--version") => {
            println!(
                "relayable-server {} (protocol 1.0)",
                env!("CARGO_PKG_VERSION")
            );
            ExitCode::SUCCESS
        }
        _ => {
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn run() -> ExitCode {
    let config = match relayable_server::ServerConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("relayable-server: invalid configuration: {error}");
            return ExitCode::from(1);
        }
    };
    let socket = config.socket_path.display().to_string();
    match relayable_server::Server::bind(config) {
        Ok(server) => {
            eprintln!("relayable-server: listening on {socket} (protocol 1.0)");
            match server.run() {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("relayable-server: fatal: {error}");
                    ExitCode::from(1)
                }
            }
        }
        Err(error) => {
            eprintln!("relayable-server: failed to bind {socket}: {error}");
            ExitCode::from(1)
        }
    }
}
