//! `avt 0.18.0` comparison-engine adapter.
//!
//! `avt` is char-based (`Vt::feed_str(&str)` / `feed(char)`), so this adapter
//! owns an incremental UTF-8 decoder: bytes are buffered until a complete
//! sequence is available, which makes the feed invariant under arbitrary chunk
//! boundaries (exercised by the `chunk_boundary_invariance` check).
//!
//! # API correction
//!
//! The brief suggested `avt::Vt`, but `Vt` does **not** expose
//! `active_buffer_type()` in 0.18.0 (it is re-exported nowhere on `Vt`; see
//! `avt-0.18.0/src/vt.rs`, which has no `active_buffer_type` method), so the
//! alternate-screen state would be unobservable. This adapter therefore drives
//! the public `avt::parser::Parser` + `avt::terminal::Terminal` pair directly
//! (the same pair `Vt::feed_str` wraps internally), which does expose
//! `Terminal::active_buffer_type()`, `cursor()`, `view()`, `lines()`,
//! `resize()`, and `cursor_keys_app_mode()`.
//!
//! Declared-absent capabilities, verified against the crate source:
//! * no OSC 0/2 title (`parser::osc_put` is a no-op; `terminal.rs` has no OSC
//!   handling) and no OSC 8 hyperlinks;
//! * no mouse mode/encoding tracking (no `1000`/`1002`/`1003`/`1006` handling);
//! * no bracketed-paste mode (`DecMode` only lists 1/6/7/25/1047/1048/1049);
//! * no bell (there is no BEL action in `parser::Function`);
//! * one `char` per cell, so combining marks overwrite rather than attach.

use crate::model::{
    Capabilities, Cell, CellStyle, Color, Cursor, EngineAdapter, MouseMode, ScrollbackStats,
    Snapshot,
};

/// Stable engine identifier.
pub const NAME: &str = "avt";
/// Exact `Cargo.toml` pin.
pub const VERSION: &str = "0.18.0";

/// Features `avt 0.18.0` exposes through this adapter.
const CAPABILITIES: Capabilities = Capabilities {
    title: false,
    bell: false,
    osc8_hyperlinks: false,
    bracketed_paste_mode: false,
    mouse_modes: false,
    mouse_sgr_flag: false,
    app_cursor_keys: true,
    alt_screen: true,
    scrollback_bound: true,
    reflow_on_resize: true,
    wide_glyphs: true,
    combining_chars: false,
    truecolor: true,
    indexed_256: true,
    per_cell_style: true,
    // No selection-range API in avt 0.18.0 (`grep -rni selection avt-0.18.0/src`
    // is empty); declared absent so the check is UNSUPPORTED, never FAIL.
    selection_to_string: false,
};

/// Adapter backed by `avt::terminal::Terminal` + `avt::parser::Parser`.
pub struct AvtAdapter {
    parser: avt::parser::Parser,
    terminal: avt::terminal::Terminal,
    /// Bytes of a possibly-incomplete UTF-8 sequence carried across `feed`.
    pending: Vec<u8>,
    cols: usize,
    rows: usize,
    bound: usize,
}

impl AvtAdapter {
    /// Build an adapter with the requested geometry and scrollback bound.
    #[must_use]
    pub fn new(cols: u16, rows: u16, scrollback: usize) -> Self {
        let cols = usize::from(cols);
        let rows = usize::from(rows);
        Self {
            parser: avt::parser::Parser::new(),
            terminal: avt::terminal::Terminal::new((cols, rows), Some(scrollback)),
            pending: Vec::new(),
            cols,
            rows,
            bound: scrollback,
        }
    }

    fn feed_char(&mut self, ch: char) {
        if let Some(op) = self.parser.feed(ch) {
            self.terminal.execute(op);
        }
    }

    /// Feed a fully-decoded `&str`, then let the buffer trim its scrollback.
    fn feed_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.feed_char(ch);
        }
        // `gc()` performs the bounded-scrollback trim eagerly even if the
        // returned iterator is dropped (see `Buffer::gc`).
        let _ = self.terminal.gc();
    }
}

fn map_color(color: Option<avt::Color>) -> Color {
    match color {
        None => Color::Default,
        Some(avt::Color::Indexed(i)) => Color::Indexed(i),
        Some(avt::Color::RGB(rgb)) => Color::Rgb(rgb.r, rgb.g, rgb.b),
    }
}

impl EngineAdapter for AvtAdapter {
    fn name(&self) -> &'static str {
        NAME
    }

    fn capabilities(&self) -> Capabilities {
        CAPABILITIES
    }

    fn feed(&mut self, bytes: &[u8]) {
        // Move the carry buffer out so `self` can be borrowed mutably while the
        // bytes are decoded (the decoded `&str` borrows the local `buf`).
        let mut buf = std::mem::take(&mut self.pending);
        buf.extend_from_slice(bytes);
        let mut start = 0usize;
        while start < buf.len() {
            match std::str::from_utf8(&buf[start..]) {
                Ok(s) => {
                    self.feed_str(s);
                    start = buf.len();
                }
                Err(err) => {
                    let valid = err.valid_up_to();
                    if valid > 0 {
                        let s = std::str::from_utf8(&buf[start..start + valid])
                            .expect("valid_up_to() guarantees a valid prefix");
                        self.feed_str(s);
                        start += valid;
                    }
                    match err.error_len() {
                        // Invalid sequence: emit U+FFFD, skip the bad bytes.
                        Some(len) => {
                            self.feed_char('\u{FFFD}');
                            start += len;
                        }
                        // Incomplete trailing sequence: wait for the next chunk.
                        None => break,
                    }
                }
            }
        }
        self.pending.clear();
        self.pending.extend_from_slice(&buf[start..]);
    }

    fn resize(&mut self, cols: u16, rows: u16) {
        let cols = usize::from(cols);
        let rows = usize::from(rows);
        self.terminal.resize(cols, rows);
        let _ = self.terminal.gc();
        self.cols = cols;
        self.rows = rows;
    }

    fn snapshot(&self) -> Snapshot {
        let (cols, rows) = self.terminal.size();
        let visible: Vec<&avt::Line> = self.terminal.view().collect();
        let mut cells = Vec::with_capacity(cols * rows);
        for row in 0..rows {
            let line = visible.get(row);
            for col in 0..cols {
                let cell = match line.and_then(|l| l.cells().get(col)) {
                    Some(cell) => {
                        // `width() == 0` is a wide-glyph continuation cell.
                        let text = if cell.width() == 0 {
                            String::new()
                        } else {
                            cell.char().to_string()
                        };
                        let pen = cell.pen();
                        Cell {
                            text,
                            style: CellStyle {
                                fg: map_color(pen.foreground()),
                                bg: map_color(pen.background()),
                                bold: pen.is_bold(),
                                italic: pen.is_italic(),
                                underline: pen.is_underline(),
                                reverse: pen.is_inverse(),
                            },
                            hyperlink: None,
                        }
                    }
                    None => Cell::blank(),
                };
                cells.push(cell);
            }
        }

        let cursor = self.terminal.cursor();
        let retained_lines = self.terminal.lines().count().saturating_sub(rows);
        Snapshot {
            cols: cols as u16,
            rows: rows as u16,
            cells,
            cursor: Cursor {
                row: cursor.row as u16,
                col: cursor.col as u16,
                visible: cursor.visible,
            },
            alt_screen: self.terminal.active_buffer_type() == avt::terminal::BufferType::Alternate,
            // Not expressible by avt 0.18.0; declared absent in CAPABILITIES.
            bracketed_paste: false,
            app_cursor_keys: self.terminal.cursor_keys_app_mode(),
            mouse_mode: MouseMode::Off,
            mouse_sgr: false,
            title: None,
            events: Vec::new(),
            bell_count: 0,
            scrollback: ScrollbackStats {
                retained_lines,
                configured_bound: self.bound,
            },
        }
    }
}

/// Factory matching [`crate::model::EngineFactory`].
#[must_use]
pub fn factory(cols: u16, rows: u16, scrollback: usize) -> Box<dyn EngineAdapter> {
    Box::new(AvtAdapter::new(cols, rows, scrollback))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_and_alt_screen() {
        let mut a = AvtAdapter::new(20, 3, 100);
        a.feed(b"top\x1b[?1049halt");
        let s = a.snapshot();
        assert!(s.alt_screen);
        assert_eq!(s.row_text(0), "   alt");
        assert!(!s.bracketed_paste);
        a.feed(b"\x1b[?1049lback");
        let s = a.snapshot();
        assert!(!s.alt_screen);
        assert_eq!(s.row_text(0), "topback");
    }

    #[test]
    fn utf8_is_chunk_boundary_invariant() {
        let text = "寬字元 e\u{301} 😀".as_bytes();
        let mut whole = AvtAdapter::new(30, 3, 100);
        whole.feed(text);
        let expected = whole.snapshot();

        let mut bytewise = AvtAdapter::new(30, 3, 100);
        for b in text {
            bytewise.feed(std::slice::from_ref(b));
        }
        let got = bytewise.snapshot();
        assert_eq!(got.to_plain(), expected.to_plain());
        assert_eq!(got.cursor, expected.cursor);
    }

    #[test]
    fn scrollback_bound_is_reported() {
        let mut a = AvtAdapter::new(20, 3, 5);
        for i in 0..20u32 {
            a.feed(format!("line{i}\r\n").as_bytes());
        }
        let s = a.snapshot();
        assert_eq!(s.scrollback.configured_bound, 5);
        assert_eq!(s.scrollback.retained_lines, 5);
    }

    #[test]
    fn capabilities_declare_combining_and_mouse_absent() {
        let a = AvtAdapter::new(20, 3, 100);
        let caps = a.capabilities();
        assert!(!caps.combining_chars);
        assert!(!caps.mouse_modes);
        assert!(!caps.bracketed_paste_mode);
        assert!(caps.alt_screen);
        assert!(caps.reflow_on_resize);
    }
}
