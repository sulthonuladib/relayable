//! GPUI-free terminal engine: `alacritty_terminal` 0.26.0 (headless `Term`) driven
//! by the `vte` 0.15.0 parser.
//!
//! This module is the UI's screen model. It owns no renderer and no PTY: bytes
//! arrive from the server (already framed and credit-acked by [`crate::api`])
//! and are fed to a headless [`Term`] through a [`vte::ansi::Processor`]. The UI
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

use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line, Point};
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{Config, Term};
use vte::ansi::{Color as VteColor, CursorShape, NamedColor, Processor, Rgb};

/// Terminal geometry: character grid plus the pixel size reported to the PTY.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    /// Visible columns.
    pub cols: u16,
    /// Visible rows.
    pub rows: u16,
    /// Viewport width in pixels (may be 0 when unknown).
    pub pixel_width: u32,
    /// Viewport height in pixels (may be 0 when unknown).
    pub pixel_height: u32,
}

impl TerminalSize {
    /// A size with unknown pixel dimensions.
    #[must_use]
    pub const fn cells(cols: u16, rows: u16) -> Self {
        Self {
            cols,
            rows,
            pixel_width: 0,
            pixel_height: 0,
        }
    }
}

/// A resolved terminal colour.
///
/// [`TermColor::Default`] means "the terminal's default" and must be resolved
/// by the UI against the measured background ramp ([`crate::theme`]) rather
/// than a value invented here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TermColor {
    /// Use the terminal default fg/bg.
    Default,
    /// A 256-colour-palette index.
    Indexed(u8),
    /// A direct RGB colour.
    Rgb(u8, u8, u8),
}

/// Resolved text attributes for one cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellStyle {
    /// Foreground colour ([`TermColor::Default`] = terminal default).
    pub fg: TermColor,
    /// Background colour; `None` means "terminal default background", so the
    /// measured background ramp shows through.
    pub bg: Option<TermColor>,
    /// Bold attribute set.
    pub bold: bool,
    /// Italic attribute set.
    pub italic: bool,
    /// Any underline variant set (single/double/curl/dotted/dashed).
    pub underline: bool,
    /// Reverse-video attribute set.
    pub inverse: bool,
    /// Dim attribute set.
    pub dim: bool,
}

/// One rendered cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenCell {
    /// The glyph (a space for blank cells).
    pub ch: char,
    /// Resolved attributes.
    pub style: CellStyle,
    /// `true` when this cell is the trailing half of a wide glyph and should not
    /// be painted.
    pub wide_continuation: bool,
}

/// A viewport snapshot: `cols * rows` cells in row-major order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenSnapshot {
    /// Snapshot columns.
    pub cols: u16,
    /// Snapshot rows.
    pub rows: u16,
    /// Row-major cells (`cols * rows`, padded with blanks if necessary).
    pub cells: Vec<ScreenCell>,
    /// Cursor `(row, col)` when it is visible in this snapshot.
    pub cursor: Option<(u16, u16)>,
    /// Whether the terminal cursor is visible at all.
    pub cursor_visible: bool,
}

/// Grid dimensions handed to `Term::new` / `Term::resize`.
struct Dims {
    cols: usize,
    rows: usize,
}

impl Dimensions for Dims {
    fn total_lines(&self) -> usize {
        self.rows
    }
    fn screen_lines(&self) -> usize {
        self.rows
    }
    fn columns(&self) -> usize {
        self.cols
    }
}

/// The headless screen model.
pub struct TerminalModel {
    term: Term<VoidListener>,
    parser: Processor,
    size: TerminalSize,
}

impl TerminalModel {
    /// Create an empty terminal of `size`.
    #[must_use]
    pub fn new(size: TerminalSize) -> Self {
        let dims = Dims {
            cols: usize::from(size.cols.max(1)),
            rows: usize::from(size.rows.max(1)),
        };
        let term = Term::new(Config::default(), &dims, VoidListener);
        Self {
            term,
            parser: Processor::new(),
            size: TerminalSize {
                cols: dims.cols as u16,
                rows: dims.rows as u16,
                pixel_width: size.pixel_width,
                pixel_height: size.pixel_height,
            },
        }
    }

    /// Feed raw VT bytes. Invalid UTF-8 and truncated escape sequences are
    /// buffered by the parser and never panic.
    pub fn process(&mut self, bytes: &[u8]) {
        self.parser.advance(&mut self.term, bytes);
    }

    /// Resize the grid (and reflow) to `size`.
    pub fn resize(&mut self, size: TerminalSize) {
        let cols = size.cols.max(1);
        let rows = size.rows.max(1);
        let dims = Dims {
            cols: usize::from(cols),
            rows: usize::from(rows),
        };
        self.term.resize(dims);
        self.size = TerminalSize {
            cols,
            rows,
            pixel_width: size.pixel_width,
            pixel_height: size.pixel_height,
        };
    }

    /// Snapshot the current viewport.
    #[must_use]
    pub fn snapshot(&self) -> ScreenSnapshot {
        let content = self.term.renderable_content();
        let cols = self.size.cols;
        let rows = self.size.rows;
        let cells = collect_cells(content.display_iter, u32::from(cols) * u32::from(rows));
        let (cursor, cursor_visible) =
            cursor_of(&content.cursor, content.display_offset, self.size);
        ScreenSnapshot {
            cols,
            rows,
            cells,
            cursor,
            cursor_visible,
        }
    }

    /// Number of scrollback history lines currently retained.
    #[must_use]
    pub fn history_len(&self) -> usize {
        self.term.grid().history_size()
    }

    /// Snapshot the viewport scrolled `history_line_offset` lines into history
    /// (`0` = live view). Offsets beyond the retained history clamp to the
    /// oldest line.
    #[must_use]
    pub fn snapshot_at(&self, history_line_offset: isize) -> ScreenSnapshot {
        let grid = self.term.grid();
        let cols = self.size.cols.max(1);
        let rows = self.size.rows.max(1);
        let offset = history_line_offset.max(0) as usize;
        let offset = offset.min(grid.history_size());

        let last_col = Column(usize::from(cols).saturating_sub(1));
        let start = Point::new(Line(-(offset as i32) - 1), last_col);
        let take = usize::from(cols) * usize::from(rows);
        let cells = collect_cells(grid.iter_from(start), take as u32);

        // The cursor is only meaningful at the live view.
        let (cursor, cursor_visible) = if offset == 0 {
            let content = self.term.renderable_content();
            cursor_of(&content.cursor, content.display_offset, self.size)
        } else {
            (None, false)
        };

        ScreenSnapshot {
            cols,
            rows,
            cells,
            cursor,
            cursor_visible,
        }
    }

    /// The current size.
    #[must_use]
    pub fn size(&self) -> TerminalSize {
        self.size
    }
}

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

/// Small helper so `snapshot` and `snapshot_at` share the columns count.
fn collect_cells<'a>(
    iter: alacritty_terminal::grid::GridIterator<'a, alacritty_terminal::term::cell::Cell>,
    count: u32,
) -> Vec<ScreenCell> {
    let take = count as usize;
    let mut cells: Vec<ScreenCell> = iter
        .take(take)
        .map(|indexed| convert_cell(indexed.cell))
        .collect();
    cells.resize(take, BLANK_CELL);
    cells
}

fn cursor_of(
    cursor: &alacritty_terminal::term::RenderableCursor,
    display_offset: usize,
    size: TerminalSize,
) -> (Option<(u16, u16)>, bool) {
    if cursor.shape == CursorShape::Hidden {
        return (None, false);
    }
    let row = cursor.point.line.0 + display_offset as i32;
    if row < 0 || row >= i32::from(size.rows) {
        return (None, true);
    }
    let col = cursor.point.column.0;
    if col >= usize::from(size.cols) {
        return (None, true);
    }
    (Some((row as u16, col as u16)), true)
}

/// Convert a grid cell into a [`ScreenCell`], mapping the terminal *default*
/// background to `bg: None` so the measured ramp is not overwritten.
fn convert_cell(cell: &alacritty_terminal::term::cell::Cell) -> ScreenCell {
    let flags = cell.flags;
    ScreenCell {
        ch: cell.c,
        style: CellStyle {
            fg: convert_color(cell.fg),
            bg: convert_bg(cell.bg),
            bold: flags.contains(Flags::BOLD),
            italic: flags.contains(Flags::ITALIC),
            underline: flags.intersects(Flags::ALL_UNDERLINES),
            inverse: flags.contains(Flags::INVERSE),
            dim: flags.contains(Flags::DIM),
        },
        wide_continuation: flags
            .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER),
    }
}

/// Foreground conversion. `Foreground`/`Background` names mean "terminal
/// default", everything else is a concrete index or RGB.
fn convert_color(color: VteColor) -> TermColor {
    match color {
        VteColor::Spec(Rgb { r, g, b }) => TermColor::Rgb(r, g, b),
        VteColor::Indexed(index) => TermColor::Indexed(index),
        VteColor::Named(named) => match named {
            NamedColor::Foreground
            | NamedColor::Background
            | NamedColor::Cursor
            | NamedColor::BrightForeground
            | NamedColor::DimForeground => TermColor::Default,
            other => match named_index(other) {
                Some(index) => TermColor::Indexed(index),
                None => TermColor::Default,
            },
        },
    }
}

/// Background conversion: the terminal default background is reported as
/// `None` (never a synthesised colour).
fn convert_bg(color: VteColor) -> Option<TermColor> {
    match color {
        VteColor::Named(NamedColor::Background) => None,
        other => Some(convert_color(other)),
    }
}

/// Map the 16 ANSI names (and their dim variants) to palette indices.
fn named_index(named: NamedColor) -> Option<u8> {
    Some(match named {
        NamedColor::Black | NamedColor::DimBlack => 0,
        NamedColor::Red | NamedColor::DimRed => 1,
        NamedColor::Green | NamedColor::DimGreen => 2,
        NamedColor::Yellow | NamedColor::DimYellow => 3,
        NamedColor::Blue | NamedColor::DimBlue => 4,
        NamedColor::Magenta | NamedColor::DimMagenta => 5,
        NamedColor::Cyan | NamedColor::DimCyan => 6,
        NamedColor::White | NamedColor::DimWhite => 7,
        NamedColor::BrightBlack => 8,
        NamedColor::BrightRed => 9,
        NamedColor::BrightGreen => 10,
        NamedColor::BrightYellow => 11,
        NamedColor::BrightBlue => 12,
        NamedColor::BrightMagenta => 13,
        NamedColor::BrightCyan => 14,
        NamedColor::BrightWhite => 15,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(cols: u16, rows: u16) -> TerminalModel {
        TerminalModel::new(TerminalSize::cells(cols, rows))
    }

    fn row_text(snapshot: &ScreenSnapshot, row: u16) -> String {
        let cols = usize::from(snapshot.cols);
        let start = usize::from(row) * cols;
        snapshot.cells[start..start + cols]
            .iter()
            .map(|cell| cell.ch)
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    #[test]
    fn sgr_red_sets_indexed_foreground_and_reset_clears_it() {
        let mut term = model(20, 2);
        term.process(b"\x1b[31mred\x1b[0m");

        let snapshot = term.snapshot();
        assert_eq!(row_text(&snapshot, 0), "red");

        let r = snapshot.cells[0].style;
        assert_eq!(r.fg, TermColor::Indexed(1));
        assert_eq!(r.bg, None);

        let after_reset = snapshot.cells[3].style;
        assert_eq!(after_reset.fg, TermColor::Default);
        assert_eq!(after_reset.bg, None);
    }

    #[test]
    fn default_background_cells_report_bg_none() {
        let mut term = model(8, 2);
        term.process(b"hello");

        let snapshot = term.snapshot();
        for cell in &snapshot.cells {
            assert_eq!(cell.style.bg, None, "no invented default background");
        }
    }

    #[test]
    fn explicit_background_is_reported() {
        let mut term = model(8, 2);
        term.process(b"\x1b[41mX\x1b[0m");

        let snapshot = term.snapshot();
        assert_eq!(snapshot.cells[0].style.bg, Some(TermColor::Indexed(1)));
    }

    #[test]
    fn resize_changes_dimensions() {
        let mut term = model(80, 24);
        assert_eq!(term.snapshot().cols, 80);
        assert_eq!(term.snapshot().rows, 24);

        term.resize(TerminalSize::cells(120, 40));
        let snapshot = term.snapshot();
        assert_eq!(snapshot.cols, 120);
        assert_eq!(snapshot.rows, 40);
        assert_eq!(snapshot.cells.len(), 120 * 40);
    }

    #[test]
    fn process_handles_partial_escape_and_invalid_utf8_without_panicking() {
        let mut term = model(40, 4);
        // A truncated CSI ...
        term.process(b"\x1b[38;5;");
        // ... completed later.
        term.process(b"196mZ");
        // Invalid UTF-8 continuation bytes.
        term.process(&[0xFF, 0xFE, 0x80, b'?']);
        // Truncated OSC that never terminates.
        term.process(b"\x1b]0;partial-title");
        // Must still snapshot.
        let snapshot = term.snapshot();
        assert_eq!(snapshot.cells.len(), 40 * 4);
    }

    #[test]
    fn truecolor_foreground_is_rgb() {
        let mut term = model(8, 2);
        term.process(b"\x1b[38;2;10;20;30mQ");
        let snapshot = term.snapshot();
        assert_eq!(snapshot.cells[0].style.fg, TermColor::Rgb(10, 20, 30));
    }

    #[test]
    fn scrollback_grows_and_snapshot_at_reads_history() {
        let mut term = model(10, 2);
        term.process(b"one\r\ntwo\r\nthree\r\nfour");
        assert!(term.history_len() >= 2, "history should have grown");

        let live = term.snapshot();
        assert_eq!(row_text(&live, 1), "four");
        let older = term.snapshot_at(2);
        assert_eq!(older.cells.len(), 10 * 2);
    }

    #[test]
    fn snapshot_at_clamps_beyond_history() {
        let mut term = model(10, 2);
        term.process(b"only");
        // A large offset must clamp, not panic.
        let snapshot = term.snapshot_at(100_000);
        assert_eq!(snapshot.cells.len(), 10 * 2);
    }
}
