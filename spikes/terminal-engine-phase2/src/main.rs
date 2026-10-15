//! Provenance banner entry point for the phase-2 terminal-engine spike.
//!
//! Intentionally minimal: this binary exists so the standalone crate has a real
//! `[[bin]]` target and to print the version/provenance the spike records.
//! Engine adapters and the PTY harness are owned by later phase-B work.

fn main() {
    println!(
        "{}",
        relayable_terminal_engine_phase2_spike::version_banner()
    );
    println!(
        "pins: portable-pty=0.9.0 alacritty_terminal=0.26.0 vte=0.15.0 \
         (comparison: vt100=0.16.2 termwiz=0.23.3 avt=0.18.0)"
    );
    println!("api notes: spikes/terminal-engine-phase2/notes/engine-api.md");
}
