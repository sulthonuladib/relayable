//! Engine-neutral snapshot model (the lead's interface decision for TERM-002
//! phase B).
//!
//! This module is deliberately free of any terminal-engine dependency so the
//! same types can be produced by `alacritty_terminal`, the comparison engines,
//! and (later) consumed by the Android / UI-001 rendering path.
//!
//! Honesty rules (full.md §0.3.6–7):
//! * a field an engine cannot express is empty / `None` / `Off`, and the
//!   corresponding check is recorded `UNSUPPORTED`, never guessed;
//! * `cells` is the *visible* screen only; scrollback beyond the viewport is
//!   summarised by [`ScrollbackStats::retained_lines`];
//! * a wide glyph occupies two grid columns: the first cell carries the glyph,
//!   the continuation cell is represented with an empty [`Cell::text`].

/// A resolved colour value from the perspective of the neutral model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// The engine's default foreground/background.
    Default,
    /// A palette index (`0..=255`).
    Indexed(u8),
    /// A 24-bit direct colour.
    Rgb(u8, u8, u8),
}

/// Attribute state of one cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellStyle {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
}

impl CellStyle {
    /// The engine-neutral "no attributes" style.
    #[must_use]
    pub fn plain() -> Self {
        Self {
            fg: Color::Default,
            bg: Color::Default,
            bold: false,
            italic: false,
            underline: false,
            reverse: false,
        }
    }
}

/// One grid cell on the visible screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// Text content. Empty for a wide-glyph continuation cell.
    pub text: String,
    pub style: CellStyle,
    /// OSC 8 target URI, `None` when the cell carries no hyperlink.
    pub hyperlink: Option<String>,
}

impl Cell {
    /// A blank default cell.
    #[must_use]
    pub fn blank() -> Self {
        Self {
            text: " ".to_string(),
            style: CellStyle::plain(),
            hyperlink: None,
        }
    }
}

/// Visible cursor position and visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub row: u16,
    pub col: u16,
    pub visible: bool,
}

/// Scrollback retention summary (not a text dump).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollbackStats {
    /// Lines currently held above the viewport.
    pub retained_lines: usize,
    /// Configured bound (`scrolling_history`).
    pub configured_bound: usize,
}

/// Mouse reporting mode observed from the engine mode flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseMode {
    Off,
    Click,
    Drag,
    Any,
}

/// Discrete engine events (title, bell, redraw).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineEvent {
    Title(String),
    ResetTitle,
    Bell,
    Wakeup,
}

/// A full snapshot of the visible screen plus mode/event state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub cols: u16,
    pub rows: u16,
    /// Row-major, `len == rows * cols`, row 0 = top of the visible screen.
    pub cells: Vec<Cell>,
    pub cursor: Cursor,
    pub alt_screen: bool,
    pub bracketed_paste: bool,
    pub app_cursor_keys: bool,
    pub mouse_mode: MouseMode,
    pub mouse_sgr: bool,
    pub title: Option<String>,
    pub events: Vec<EngineEvent>,
    pub bell_count: u64,
    pub scrollback: ScrollbackStats,
}

impl Snapshot {
    /// Cell at `(row, col)`. Panics when out of bounds (a programming error,
    /// not engine data).
    #[must_use]
    pub fn cell(&self, row: u16, col: u16) -> &Cell {
        assert!(
            row < self.rows && col < self.cols,
            "cell({row},{col}) out of bounds for {}x{}",
            self.rows,
            self.cols
        );
        &self.cells[row as usize * self.cols as usize + col as usize]
    }

    /// Text of one row with trailing spaces trimmed.
    #[must_use]
    pub fn row_text(&self, row: u16) -> String {
        assert!(row < self.rows, "row {row} out of bounds");
        let start = row as usize * self.cols as usize;
        let mut s = String::new();
        for cell in &self.cells[start..start + self.cols as usize] {
            s.push_str(&cell.text);
        }
        s.trim_end_matches(' ').to_string()
    }

    /// Full-screen plain-text dump; rows joined by `'\n'`.
    #[must_use]
    pub fn to_plain(&self) -> String {
        (0..self.rows)
            .map(|r| self.row_text(r))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Number of visible cells carrying an OSC 8 hyperlink.
    #[must_use]
    pub fn count_hyperlink_cells(&self) -> usize {
        self.cells.iter().filter(|c| c.hyperlink.is_some()).count()
    }
}

/// Implementation status reported by an engine adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplementationStatus {
    /// The adapter is a fully wired terminal engine.
    Implemented,
    /// The adapter is a compiling stub. Every check against it is recorded
    /// `UNSUPPORTED` (never PASS/FAIL) and `reason` is surfaced in the
    /// artifacts, so the report can show "not yet wired" vs "failed".
    NotImplemented {
        engine: &'static str,
        reason: &'static str,
    },
}

impl ImplementationStatus {
    /// True for [`ImplementationStatus::Implemented`].
    #[must_use]
    pub fn is_implemented(self) -> bool {
        matches!(self, Self::Implemented)
    }

    /// The `NOT_IMPLEMENTED` reason, if this is a stub.
    #[must_use]
    pub fn reason(self) -> Option<&'static str> {
        match self {
            Self::Implemented => None,
            Self::NotImplemented { reason, .. } => Some(reason),
        }
    }
}

/// The set of check-expressible features an engine adapter provides.
///
/// One flag per feature the check suite can exercise. An engine that does not
/// provide a feature must declare it `false`; the corresponding check is then
/// recorded `UNSUPPORTED` (a *capability gap*) rather than `FAIL` (a defect).
///
/// This is deliberately a plain struct of `bool`s rather than a bitflag type:
/// it keeps the crate dependency-free and makes every declaration readable in
/// the adapter source, which is what the comparison evidence has to cite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    /// OSC 0/2 window title (and reset) observable from the engine.
    pub title: bool,
    /// BEL bell count / [`EngineEvent::Bell`].
    pub bell: bool,
    /// OSC 8 per-cell hyperlinks.
    pub osc8_hyperlinks: bool,
    /// DECSET/DECRST 2004 bracketed-paste *mode flag* (not payload handling).
    pub bracketed_paste_mode: bool,
    /// Mouse reporting mode flags (1000/1002/1003-ish).
    pub mouse_modes: bool,
    /// SGR (1006) mouse encoding flag.
    pub mouse_sgr_flag: bool,
    /// DECSET/DECRST 1 application-cursor-keys mode.
    pub app_cursor_keys: bool,
    /// Alternate screen (DECSET/DECRST 1049 / 47).
    pub alt_screen: bool,
    /// A bounded scrollback with a reportable retained-line count.
    pub scrollback_bound: bool,
    /// Logical-line reflow on resize.
    pub reflow_on_resize: bool,
    /// Wide (2-column) glyphs with a continuation cell.
    pub wide_glyphs: bool,
    /// Combining / zero-width characters attached to the base cell.
    pub combining_chars: bool,
    /// 24-bit direct colour (SGR 38;2 / 48;2).
    pub truecolor: bool,
    /// 256-colour index (SGR 38;5 / 48;5).
    pub indexed_256: bool,
    /// Per-cell style attributes (bold/italic/underline/reverse).
    pub per_cell_style: bool,
    /// A text-range selection can be resolved back to an exact `String`
    /// (`alacritty_terminal` `Term::selection_to_string`, `vt100`
    /// `Screen::contents_between`). `avt 0.18.0` has no such selection/clipboard
    /// helper API.
    pub selection_to_string: bool,
}

impl Capabilities {
    /// No feature is available.
    pub const NONE: Self = Self {
        title: false,
        bell: false,
        osc8_hyperlinks: false,
        bracketed_paste_mode: false,
        mouse_modes: false,
        mouse_sgr_flag: false,
        app_cursor_keys: false,
        alt_screen: false,
        scrollback_bound: false,
        reflow_on_resize: false,
        wide_glyphs: false,
        combining_chars: false,
        truecolor: false,
        indexed_256: false,
        per_cell_style: false,
        selection_to_string: false,
    };

    /// Every feature is available (the default, so the gating engine is
    /// unchanged unless it explicitly narrows its declaration).
    pub const ALL: Self = Self {
        title: true,
        bell: true,
        osc8_hyperlinks: true,
        bracketed_paste_mode: true,
        mouse_modes: true,
        mouse_sgr_flag: true,
        app_cursor_keys: true,
        alt_screen: true,
        scrollback_bound: true,
        reflow_on_resize: true,
        wide_glyphs: true,
        combining_chars: true,
        truecolor: true,
        indexed_256: true,
        per_cell_style: true,
        selection_to_string: true,
    };
}

/// A terminal-emulation engine adapted to the neutral model.
pub trait EngineAdapter {
    /// Stable engine identifier, e.g. `"alacritty_terminal"`.
    fn name(&self) -> &'static str;
    /// Feed raw program output bytes through the parser.
    fn feed(&mut self, bytes: &[u8]);
    /// Resize the emulated screen.
    fn resize(&mut self, cols: u16, rows: u16);
    /// Snapshot the current visible state.
    fn snapshot(&self) -> Snapshot;
    /// Whether this adapter is wired up. Default: implemented.
    fn implementation_status(&self) -> ImplementationStatus {
        ImplementationStatus::Implemented
    }
    /// Features this adapter provides. Default: [`Capabilities::ALL`], so an
    /// engine that does not override this keeps every check running.
    fn capabilities(&self) -> Capabilities {
        Capabilities::ALL
    }
}

/// Factory that builds an adapter with the requested geometry/scrollback.
pub type EngineFactory = fn(cols: u16, rows: u16, scrollback: usize) -> Box<dyn EngineAdapter>;

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(cols: u16, rows: u16) -> Snapshot {
        Snapshot {
            cols,
            rows,
            cells: vec![Cell::blank(); cols as usize * rows as usize],
            cursor: Cursor {
                row: 0,
                col: 0,
                visible: true,
            },
            alt_screen: false,
            bracketed_paste: false,
            app_cursor_keys: false,
            mouse_mode: MouseMode::Off,
            mouse_sgr: false,
            title: None,
            events: Vec::new(),
            bell_count: 0,
            scrollback: ScrollbackStats {
                retained_lines: 0,
                configured_bound: 0,
            },
        }
    }

    #[test]
    fn row_text_trims_trailing_spaces_only() {
        let mut s = snap(5, 2);
        s.cells[0].text = "ab".into();
        s.cells[1].text = " ".into();
        s.cells[2].text = "c".into();
        assert_eq!(s.row_text(0), "ab c");
        assert_eq!(s.row_text(1), "");
        assert_eq!(s.to_plain(), "ab c\n");
    }

    #[test]
    fn cell_is_row_major() {
        let mut s = snap(3, 2);
        s.cells[4].text = "X".into();
        assert_eq!(s.cell(1, 1).text, "X");
    }

    #[test]
    fn hyperlink_cells_are_counted() {
        let mut s = snap(4, 1);
        s.cells[1].hyperlink = Some("https://example.com".into());
        s.cells[3].hyperlink = Some("https://example.com".into());
        assert_eq!(s.count_hyperlink_cells(), 2);
    }
}
