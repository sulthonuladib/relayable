//! Human-readable rendering: the version banner, the `session inspect`
//! snapshot, and error reporting.

use relayable_protocol::v1::SessionSnapshotResponse;

use crate::error::CliError;
use crate::json;
use crate::resolve::{self, ResolveError};

pub(crate) fn print_version(json: bool) {
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

pub(crate) fn print_snapshot(snapshot: &SessionSnapshotResponse) {
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

pub(crate) fn report(json: bool, err: &CliError) {
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
