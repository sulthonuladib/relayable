//! `termwiz` comparison-engine adapter — **NOT IMPLEMENTED**.
//!
//! Owned by a different phase-B workstream. See [`crate::engine::vt100`] for the
//! rationale; behaviour is identical: constructible, blank snapshot, marked
//! `NOT_IMPLEMENTED`, never registered as a gating result.
//!
//! # Why this one is deliberately left unimplemented
//!
//! `termwiz 0.23.3` has no byte-stream → screen-state emulator, so it cannot be
//! driven headlessly from captured PTY bytes the way `alacritty_terminal` can:
//!
//! * `termwiz::escape::parser::Parser::parse` only decodes bytes into
//!   `escape::Action`
//!   (`termwiz-0.23.3/src/escape/parser/mod.rs:86`,
//!   `termwiz-0.23.3/src/escape/mod.rs:28`).
//! * Nothing maps `Action` → `surface::Surface`: `grep -rn Action src/surface/`
//!   returns nothing, and `surface::Surface` is at
//!   `termwiz-0.23.3/src/surface/mod.rs:105`.
//! * `termwiz::terminal::new_terminal` uses the platform terminal, and
//!   `UnixTerminal::new` opens `/dev/tty`
//!   (`termwiz-0.23.3/src/terminal/unix.rs:266`) then captures the tty termios
//!   via `Termios::from_fd`
//!   (`termwiz-0.23.3/src/terminal/unix.rs:170`), i.e. it needs a real tty
//!   rather than accepting a captured byte stream.
//!
//! These paths were verified on this host before being quoted. The exact pin
//! for the comparison leg is `termwiz =0.23.3` (see `../../Cargo.toml`).

use crate::engine::empty_snapshot;
use crate::model::{EngineAdapter, ImplementationStatus, Snapshot};

/// Stable engine identifier.
pub const NAME: &str = "termwiz";
/// Exact `Cargo.toml` pin.
pub const VERSION: &str = "0.23.3";
/// Marker that this adapter is a stub.
pub const NOT_IMPLEMENTED: &str = "NOT_IMPLEMENTED";
/// Source-cited reason surfaced in every check recorded against this stub.
pub const NOT_IMPLEMENTED_REASON: &str = "termwiz 0.23.3 is deliberately NOT implemented for the comparison leg: escape::parser::Parser::parse (src/escape/parser/mod.rs:86) only decodes into escape::Action (src/escape/mod.rs:28) and nothing maps Action->surface::Surface (src/surface/mod.rs:105 has no Action reference), while terminal::UnixTerminal::new opens /dev/tty (src/terminal/unix.rs:266) and captures tty termios via Termios::from_fd (src/terminal/unix.rs:171), so it cannot be driven headlessly from captured bytes.";

/// Placeholder `termwiz` adapter.
#[derive(Debug, Default)]
pub struct TermwizAdapter {
    cols: u16,
    rows: u16,
    scrollback: usize,
}

impl TermwizAdapter {
    #[must_use]
    pub fn new(cols: u16, rows: u16, scrollback: usize) -> Self {
        Self {
            cols,
            rows,
            scrollback,
        }
    }

    /// Always [`NOT_IMPLEMENTED`].
    #[must_use]
    pub fn status(&self) -> &'static str {
        NOT_IMPLEMENTED
    }
}

impl EngineAdapter for TermwizAdapter {
    fn name(&self) -> &'static str {
        NAME
    }

    fn implementation_status(&self) -> ImplementationStatus {
        ImplementationStatus::NotImplemented {
            engine: NAME,
            reason: NOT_IMPLEMENTED_REASON,
        }
    }

    fn feed(&mut self, _bytes: &[u8]) {}

    fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
    }

    fn snapshot(&self) -> Snapshot {
        empty_snapshot(self.cols, self.rows, self.scrollback)
    }
}

/// Factory matching [`crate::model::EngineFactory`].
#[must_use]
pub fn factory(cols: u16, rows: u16, scrollback: usize) -> Box<dyn EngineAdapter> {
    Box::new(TermwizAdapter::new(cols, rows, scrollback))
}
