//! Relayable terminal-engine phase-2 spike (TERM-002 / ADR-003).
//!
//! This crate is the phase-B harness: it drives a **real PTY** through
//! `portable-pty 0.9.0`, feeds the captured byte stream into
//! `alacritty_terminal 0.26.0` + `vte 0.15.0`, and asserts observed screen/grid
//! state through an engine-neutral model.
//!
//! Layout:
//! * [`model`] — engine-neutral snapshot/interface (the lead's design decision);
//! * [`engine`] — engine adapters: `alacritty_terminal` (the gating engine),
//!   `vt100` and `avt` (wired comparison engines), and `termwiz` (a deliberate
//!   `NOT_IMPLEMENTED` stub with a source-cited reason);
//! * [`pty`] — portable-pty driver with reader thread, timed input, resize, and
//!   guaranteed child kill/reap;
//! * [`fixtures`] — deterministic VT/ANSI byte fixtures;
//! * [`checks`] — expected-vs-actual check definitions;
//! * [`report`] — artifact writers (per-check files, summary, provenance);
//! * [`sha256`], [`palette`] — provenance and colour helpers.
//!
//! The authoritative, source-verified engine API notes live in
//! `notes/engine-api.md` next to this crate. Nothing here is fabricated: every
//! result file comes from a real command run on this host.

pub mod checks;
pub mod engine;
pub mod fixtures;
pub mod model;
pub mod palette;
pub mod pty;
pub mod report;
pub mod sha256;

/// Crate version as resolved by Cargo.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name as resolved by Cargo.
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

/// One-line provenance banner shared by the binaries and the harness.
#[must_use]
pub fn version_banner() -> String {
    format!(
        "{CRATE_NAME} {CRATE_VERSION} (rustc {}, edition 2021) \
         engines: alacritty_terminal {} + vte {}, portable-pty {}",
        env!("CARGO_PKG_RUST_VERSION"),
        engine::alacritty::VERSION,
        "0.15.0",
        "0.9.0"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_mentions_crate_and_version() {
        let banner = version_banner();
        assert!(banner.contains(CRATE_NAME), "banner: {banner}");
        assert!(banner.contains(CRATE_VERSION), "banner: {banner}");
        assert!(banner.contains("alacritty_terminal"), "banner: {banner}");
    }
}
