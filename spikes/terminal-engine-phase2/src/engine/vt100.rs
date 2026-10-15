//! `vt100 0.16.2` comparison-engine adapter.
//!
//! Wired behind the frozen [`EngineAdapter`] trait and honest about the two
//! features it cannot express without extra machinery:
//!
//! * OSC 8 hyperlinks are not implemented by `vt100` at all, so the
//!   `osc8_hyperlinks` capability is declared absent;
//! * resize does not reflow logical lines (`Grid::set_size` clears row wrap),
//!   so `reflow_on_resize` is declared absent.
//!
//! Window title and bell *are* implemented, via the crate's callback API
//! (`Parser::new_with_callbacks` + `impl vt100::Callbacks`), which is the only
//! way `vt100 0.16.2` exposes them after they were removed from `Screen` in
//! 0.16.0.
//!
//! Every API used below was read from the fetched crate source
//! (`vt100-0.16.2/src/{lib,parser,screen,cell,attrs,callbacks}.rs`):
//! `Parser::new_with_callbacks(rows, cols, scrollback_len, callbacks)`,
//! `Parser::process(&[u8])`, `Screen::size() -> (rows, cols)`,
//! `Screen::cell(row, col) -> Option<&Cell>`, `Cell::contents() -> &str`,
//! `Cell::is_wide_continuation()`, `Screen::cursor_position() -> (row, col)`,
//! `Screen::hide_cursor()`, `Screen::alternate_screen()`,
//! `Screen::bracketed_paste()`, `Screen::application_cursor()`,
//! `Screen::mouse_protocol_mode()`, `Screen::mouse_protocol_encoding()`,
//! `Screen::set_size(rows, cols)`, `Screen::set_scrollback(usize)`.

use crate::model::{
    Capabilities, Cell, CellStyle, Color, Cursor, EngineAdapter, EngineEvent, MouseMode,
    ScrollbackStats, Snapshot,
};

/// Stable engine identifier.
pub const NAME: &str = "vt100";
/// Exact `Cargo.toml` pin.
pub const VERSION: &str = "0.16.2";

/// Features `vt100 0.16.2` exposes through this adapter.
const CAPABILITIES: Capabilities = Capabilities {
    title: true,
    bell: true,
    osc8_hyperlinks: false,
    bracketed_paste_mode: true,
    mouse_modes: true,
    mouse_sgr_flag: true,
    app_cursor_keys: true,
    alt_screen: true,
    scrollback_bound: true,
    // `Grid::set_size` clears row wrapping instead of reflowing.
    reflow_on_resize: false,
    wide_glyphs: true,
    combining_chars: true,
    truecolor: true,
    indexed_256: true,
    per_cell_style: true,
    // `Screen::contents_between` (`vt100-0.16.2/src/screen.rs:167`) resolves a
    // cell range back to a String.
    selection_to_string: true,
};

/// Callback state captured out of the parser (title and bell are not stored on
/// `Screen` in `vt100 0.16.2`).
#[derive(Debug, Default)]
struct Vt100Callbacks {
    title: Option<String>,
    bell_count: u64,
    events: Vec<EngineEvent>,
}

impl vt100::Callbacks for Vt100Callbacks {
    fn audible_bell(&mut self, _screen: &mut vt100::Screen) {
        self.bell_count += 1;
        self.events.push(EngineEvent::Bell);
    }

    fn set_window_title(&mut self, _screen: &mut vt100::Screen, title: &[u8]) {
        let title = String::from_utf8_lossy(title).into_owned();
        self.title = Some(title.clone());
        self.events.push(EngineEvent::Title(title));
    }
}

/// Adapter backed by `vt100::Parser`.
pub struct Vt100Adapter {
    parser: vt100::Parser<Vt100Callbacks>,
    cols: u16,
    rows: u16,
    scrollback: usize,
}

impl Vt100Adapter {
    /// Build an adapter with the requested geometry and scrollback bound.
    #[must_use]
    pub fn new(cols: u16, rows: u16, scrollback: usize) -> Self {
        // `Parser::new_with_callbacks` takes (rows, cols, scrollback_len).
        let parser =
            vt100::Parser::new_with_callbacks(rows, cols, scrollback, Vt100Callbacks::default());
        Self {
            parser,
            cols,
            rows,
            scrollback,
        }
    }

    /// Text between two cells via `Screen::contents_between`
    /// (`vt100-0.16.2/src/screen.rs:167`), the crate's clipboard-selection
    /// helper. `start_col` is inclusive and `end_col` is exclusive (vt100's
    /// documented convention: "up until end_col").
    #[must_use]
    pub fn contents_between(
        &self,
        start_row: u16,
        start_col: u16,
        end_row: u16,
        end_col: u16,
    ) -> String {
        self.parser
            .screen()
            .contents_between(start_row, start_col, end_row, end_col)
    }
}

fn map_color(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Default,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

fn map_mouse_mode(mode: vt100::MouseProtocolMode) -> MouseMode {
    use vt100::MouseProtocolMode as M;
    match mode {
        // `vt100` distinguishes X10 (`Press`, DECSET 9) from VT200
        // (`PressRelease`, DECSET 1000); the neutral model has a single
        // `Click` for button reporting, so both collapse to `Click`. This
        // approximation is recorded in the `mouse_modes` check notes.
        M::None => MouseMode::Off,
        M::Press | M::PressRelease => MouseMode::Click,
        M::ButtonMotion => MouseMode::Drag,
        M::AnyMotion => MouseMode::Any,
    }
}

impl EngineAdapter for Vt100Adapter {
    fn name(&self) -> &'static str {
        NAME
    }

    fn capabilities(&self) -> Capabilities {
        CAPABILITIES
    }

    fn feed(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    fn resize(&mut self, cols: u16, rows: u16) {
        // In 0.16.x the size lives on `Screen`; there is no `Parser::set_size`.
        self.parser.screen_mut().set_size(rows, cols);
        self.cols = cols;
        self.rows = rows;
    }

    fn snapshot(&self) -> Snapshot {
        let screen = self.parser.screen();
        // `Screen::size()` is (rows, cols).
        let (rows, cols) = screen.size();
        let mut cells = Vec::with_capacity(usize::from(cols) * usize::from(rows));
        for row in 0..rows {
            for col in 0..cols {
                let cell = match screen.cell(row, col) {
                    Some(cell) => {
                        let text = if cell.is_wide_continuation() {
                            String::new()
                        } else if cell.has_contents() {
                            cell.contents().to_string()
                        } else {
                            // The neutral model represents a blank cell as a
                            // single space (matching the alacritty adapter).
                            " ".to_string()
                        };
                        Cell {
                            text,
                            style: CellStyle {
                                fg: map_color(cell.fgcolor()),
                                bg: map_color(cell.bgcolor()),
                                bold: cell.bold(),
                                italic: cell.italic(),
                                underline: cell.underline(),
                                reverse: cell.inverse(),
                            },
                            hyperlink: None,
                        }
                    }
                    None => Cell::blank(),
                };
                cells.push(cell);
            }
        }

        let (cursor_row, cursor_col) = screen.cursor_position();
        let cursor = Cursor {
            row: cursor_row,
            col: cursor_col,
            visible: !screen.hide_cursor(),
        };

        // The retained scrollback count is not exported directly. Cloning the
        // screen and scrolling the clone to the top clamps the offset to the
        // real scrollback length (`Grid::set_scrollback` uses
        // `rows.min(self.scrollback.len())`), which is the count we want.
        let mut probe = screen.clone();
        probe.set_scrollback(usize::MAX);
        let retained_lines = probe.scrollback();

        let callbacks = self.parser.callbacks();
        Snapshot {
            cols,
            rows,
            cells,
            cursor,
            alt_screen: screen.alternate_screen(),
            bracketed_paste: screen.bracketed_paste(),
            app_cursor_keys: screen.application_cursor(),
            mouse_mode: map_mouse_mode(screen.mouse_protocol_mode()),
            mouse_sgr: screen.mouse_protocol_encoding() == vt100::MouseProtocolEncoding::Sgr,
            title: callbacks.title.clone(),
            events: callbacks.events.clone(),
            bell_count: callbacks.bell_count,
            scrollback: ScrollbackStats {
                retained_lines,
                configured_bound: self.scrollback,
            },
        }
    }
}

/// Factory matching [`crate::model::EngineFactory`].
#[must_use]
pub fn factory(cols: u16, rows: u16, scrollback: usize) -> Box<dyn EngineAdapter> {
    Box::new(Vt100Adapter::new(cols, rows, scrollback))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_and_styles() {
        let mut a = Vt100Adapter::new(20, 3, 100);
        a.feed(b"hello");
        let s = a.snapshot();
        assert_eq!(s.row_text(0), "hello");
        assert_eq!(s.cursor.col, 5);
        assert!(s.cursor.visible);
        assert!(!s.alt_screen);
    }

    #[test]
    fn wide_and_combining_cells() {
        let mut a = Vt100Adapter::new(20, 3, 100);
        a.feed("寬字元".as_bytes());
        let s = a.snapshot();
        assert_eq!(s.cell(0, 0).text, "寬");
        assert_eq!(s.cell(0, 1).text, "");
        assert_eq!(s.row_text(0), "寬字元");

        let mut a = Vt100Adapter::new(20, 3, 100);
        a.feed("e\u{301}x".as_bytes());
        let s = a.snapshot();
        assert_eq!(s.cell(0, 0).text, "e\u{301}");
        assert_eq!(s.cell(0, 1).text, "x");
    }

    #[test]
    fn title_and_bell_via_callbacks() {
        let mut a = Vt100Adapter::new(20, 3, 100);
        a.feed(b"\x1b]2;Title\x07\x07");
        let s = a.snapshot();
        assert_eq!(s.title.as_deref(), Some("Title"));
        assert_eq!(s.bell_count, 1);
        assert!(s.events.contains(&EngineEvent::Bell));
    }

    #[test]
    fn scrollback_bound_is_reported() {
        let mut a = Vt100Adapter::new(20, 3, 5);
        for i in 0..20u32 {
            a.feed(format!("line{i}\r\n").as_bytes());
        }
        let s = a.snapshot();
        assert_eq!(s.scrollback.configured_bound, 5);
        assert_eq!(s.scrollback.retained_lines, 5);
    }

    #[test]
    fn capabilities_declare_reflow_and_osc8_absent() {
        let a = Vt100Adapter::new(20, 3, 100);
        let caps = a.capabilities();
        assert!(!caps.osc8_hyperlinks);
        assert!(!caps.reflow_on_resize);
        assert!(caps.title);
        assert!(caps.mouse_modes);
    }
}
