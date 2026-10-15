//! Relayable server foreground binary.
//!
//! Usage: `relayable-server run` (foreground, no systemd unit, no forking). The
//! only inputs are the environment read by [`relayable_server::ServerConfig`].

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
    let config = relayable_server::ServerConfig::from_env();
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
