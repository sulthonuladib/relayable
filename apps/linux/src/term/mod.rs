//! GPUI-free terminal engine: `alacritty_terminal` 0.26.0 (headless `Term`) driven
//! by the `vte` 0.15.0 parser.
//!
//! This module is the UI's screen model. It owns no renderer and no PTY: bytes
//! arrive from the server (already framed and credit-acked by [`crate::api`])
//! and are fed to a headless [`alacritty_terminal::term::Term`] through a [`vte::ansi::Processor`]. The UI
//! reads a [`ScreenSnapshot`] and paints it; nothing here depends on GPUI.
//!
//! # Verified upstream API (file:line in the local registry source)
//!
//! * `alacritty_terminal-0.26.0/src/term/mod.rs:410` — `Term::new(config, &dims, proxy)`
//! * `alacritty_terminal-0.26.0/src/term/mod.rs:655` — `Term::resize`
//! * `alacritty_terminal-0.26.0/src/term/mod.rs:637` — `Term::renderable_content`
//! * `alacritty_terminal-0.26.0/src/term/mod.rs:645` — `Term::grid`
//! * `alacritty_terminal-0.26.0/src/grid/mod.rs:486` — `Dimensions` (`columns`,
//!   `screen_lines`, `total_lines`, `history_size`)
//! * `alacritty_terminal-0.26.0/src/grid/mod.rs:412` — `Grid::iter_from`
//! * `alacritty_terminal-0.26.0/src/grid/mod.rs:422` — `Grid::display_iter`
//! * `alacritty_terminal-0.26.0/src/term/cell.rs:134` — `Cell { c, fg, bg, flags }`
//! * `alacritty_terminal-0.26.0/src/term/cell.rs:15` — `Flags` (BOLD, ITALIC,
//!   DIM, UNDERLINE, WIDE_CHAR_SPACER, …)
//! * `alacritty_terminal-0.26.0/src/event.rs:103` — `EventListener` (we use the
//!   crate's `VoidListener`; OSC term metadata is tracked by [`crate::api`])
//! * `vte-0.15.0/src/ansi.rs:287` — `Processor::new`
//! * `vte-0.15.0/src/ansi.rs:298` — `Processor::advance`
//! * `vte-0.15.0/src/ansi.rs:1128` — `Color`, `:1013` `NamedColor`, `:60` `Rgb`
//! * `vte-0.15.0/src/ansi.rs:828` — `CursorShape` (Hidden ⇒ no cursor)
//!
//! # Deliberate non-decisions
//!
//! `Config::scrolling_history` keeps the engine default (10 000 lines). ADR-003
//! Consequences and `full.md` §6.4/§12 require Relayable to choose a *measured*
//! bound before hard targets are set; that measurement is out of scope for this
//! slice and no number is invented here.

mod model;
mod snapshot;

pub use model::TerminalModel;
pub use snapshot::{CellStyle, ScreenCell, ScreenSnapshot, TermColor, TerminalSize};

/// A blank default cell, used to pad a short iteration (never expected).
pub(crate) const BLANK_CELL: ScreenCell = ScreenCell {
    ch: ' ',
    style: CellStyle {
        fg: TermColor::Default,
        bg: None,
        bold: false,
        italic: false,
        underline: false,
        inverse: false,
        dim: false,
    },
    wide_continuation: false,
};

#[cfg(test)]
mod tests;
