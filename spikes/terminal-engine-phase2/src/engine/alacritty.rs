//! `alacritty_terminal 0.26.0` adapter.
//!
//! API details were verified against the fetched crate source (see
//! `notes/engine-api.md`), in particular:
//! * the parser is `vte::ansi::Processor` + `vte::ansi::Handler`, **not**
//!   `vte::Perform`; `Term<T>` already implements `Handler`;
//! * this module implements the single-method `EventListener` trait, and title /
//!   bell / wakeup arrive as `Event` variants;
//! * OSC 8 hyperlinks are per-cell (`Cell::hyperlink()`), not events;
//! * `Term::size()` does not exist — geometry is read through the `Dimensions`
//!   trait.

use std::sync::{Arc, Mutex};

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line, Point, Side};
use alacritty_terminal::selection::{Selection, SelectionType};
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::test::TermSize;
use alacritty_terminal::term::{point_to_viewport, Config, Term, TermMode};
use alacritty_terminal::vte::ansi;
use alacritty_terminal::vte::ansi::{Color as VteColor, NamedColor};

use crate::model::{
    Cell, CellStyle, Color, Cursor, EngineAdapter, EngineEvent, MouseMode, ScrollbackStats,
    Snapshot,
};

/// Stable engine identifier used in every artifact.
pub const NAME: &str = "alacritty_terminal";

/// Engine version. Mirrors the exact `Cargo.toml` pin (`alacritty_terminal
/// =0.26.0`); the crate does not export its own version string. The resolved
/// version is additionally recorded from `Cargo.lock` in `run_all_meta.txt`.
pub const VERSION: &str = "0.26.0";

/// Events accumulated by the `EventListener` (which takes `&self`).
#[derive(Debug, Default)]
struct EventLog {
    events: Vec<EngineEvent>,
    bell_count: u64,
    title: Option<String>,
}

/// `EventListener` implementation bridging alacritty `Event`s into the neutral
/// model.
struct EventProxy(Arc<Mutex<EventLog>>);

impl EventListener for EventProxy {
    fn send_event(&self, event: Event) {
        let mut log = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        match event {
            Event::Title(title) => {
                log.title = Some(title.clone());
                log.events.push(EngineEvent::Title(title));
            }
            Event::ResetTitle => {
                log.title = None;
                log.events.push(EngineEvent::ResetTitle);
            }
            Event::Bell => {
                log.bell_count += 1;
                log.events.push(EngineEvent::Bell);
            }
            Event::Wakeup => log.events.push(EngineEvent::Wakeup),
            // Other variants (clipboard, colour requests, exit, ...) are not
            // part of the neutral model.
            _ => {}
        }
    }
}

/// Adapter backed by `alacritty_terminal::Term`.
pub struct AlacrittyAdapter {
    term: Term<EventProxy>,
    parser: ansi::Processor,
    log: Arc<Mutex<EventLog>>,
    configured_bound: usize,
}

impl AlacrittyAdapter {
    /// Build an adapter with the requested geometry and scrollback bound.
    #[must_use]
    pub fn new(cols: u16, rows: u16, scrollback: usize) -> Self {
        let log = Arc::new(Mutex::new(EventLog::default()));
        let proxy = EventProxy(Arc::clone(&log));
        let config = Config {
            scrolling_history: scrollback,
            ..Config::default()
        };
        let size = TermSize::new(cols as usize, rows as usize);
        let term = Term::new(config, &size, proxy);
        Self {
            term,
            parser: ansi::Processor::new(),
            log,
            configured_bound: scrollback,
        }
    }

    /// Number of retained scrollback lines (above the viewport).
    #[must_use]
    pub fn history_size(&self) -> usize {
        self.term.history_size()
    }

    /// Dump retained scrollback as `(age, text)` with `age == 1` the most
    /// recently scrolled-off line (`Line(-1)`).
    #[must_use]
    pub fn scrollback_lines(&self) -> Vec<(usize, String)> {
        let history = self.term.history_size();
        let mut out = Vec::with_capacity(history);
        for age in 1..=history {
            let line = Line(-(age as i32));
            out.push((age, self.row_text_at(line)));
        }
        out
    }

    /// Resolve a simple text selection over the visible grid back to a String
    /// using `Term::selection_to_string()`
    /// (`alacritty_terminal-0.26.0/src/term/mod.rs:529`). `start`/`end` are
    /// 0-based `(row, col)`; alacritty's simple-selection convention includes
    /// the end cell.
    #[must_use]
    pub fn select_range_to_string(&mut self, start: (u16, u16), end: (u16, u16)) -> Option<String> {
        let start_point = Point::new(Line(i32::from(start.0)), Column(usize::from(start.1)));
        let end_point = Point::new(Line(i32::from(end.0)), Column(usize::from(end.1)));
        let mut selection = Selection::new(SelectionType::Simple, start_point, Side::Left);
        selection.update(end_point, Side::Right);
        self.term.selection = Some(selection);
        self.term.selection_to_string()
    }

    fn row_text_at(&self, line: Line) -> String {
        let cols = self.term.columns();
        let mut s = String::new();
        for col in 0..cols {
            let cell = &self.term.grid()[line][Column(col)];
            append_cell_text(&mut s, cell);
        }
        s.trim_end_matches(' ').to_string()
    }
}

/// Append a cell's visible text (wide-glyph continuations contribute nothing).
fn append_cell_text(out: &mut String, cell: &alacritty_terminal::term::cell::Cell) {
    if cell
        .flags
        .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER)
    {
        return;
    }
    out.push(cell.c);
    if let Some(zero) = cell.zerowidth() {
        out.extend(zero.iter().copied());
    }
}

fn map_named(n: NamedColor) -> Color {
    match n {
        NamedColor::Foreground | NamedColor::Background => Color::Default,
        _ => {
            let idx = n as usize;
            if idx < 16 {
                Color::Indexed(idx as u8)
            } else {
                // Cursor / Dim* / BrightForeground have no exact neutral
                // representation without an engine palette; recorded as
                // UNSUPPORTED by the colour check rather than guessed.
                Color::Default
            }
        }
    }
}

fn map_color(color: VteColor) -> Color {
    match color {
        VteColor::Named(n) => map_named(n),
        VteColor::Spec(rgb) => Color::Rgb(rgb.r, rgb.g, rgb.b),
        VteColor::Indexed(i) => Color::Indexed(i),
    }
}

impl EngineAdapter for AlacrittyAdapter {
    fn name(&self) -> &'static str {
        NAME
    }

    fn feed(&mut self, bytes: &[u8]) {
        // `Term<T>` implements `vte::ansi::Handler`; the processor drives it.
        self.parser.advance(&mut self.term, bytes);
    }

    fn resize(&mut self, cols: u16, rows: u16) {
        self.term
            .resize(TermSize::new(cols as usize, rows as usize));
    }

    fn snapshot(&self) -> Snapshot {
        let cols = self.term.columns() as u16;
        let rows = self.term.screen_lines() as u16;
        let mut cells = Vec::with_capacity(cols as usize * rows as usize);
        for line in 0..rows as i32 {
            for col in 0..cols as usize {
                let cell = &self.term.grid()[Line(line)][Column(col)];
                let mut text = String::new();
                append_cell_text(&mut text, cell);
                let style = CellStyle {
                    fg: map_color(cell.fg),
                    bg: map_color(cell.bg),
                    bold: cell.flags.contains(Flags::BOLD),
                    italic: cell.flags.contains(Flags::ITALIC),
                    underline: cell.flags.intersects(Flags::ALL_UNDERLINES),
                    reverse: cell.flags.contains(Flags::INVERSE),
                };
                let hyperlink = cell.hyperlink().map(|h| h.uri().to_string());
                cells.push(Cell {
                    text,
                    style,
                    hyperlink,
                });
            }
        }

        let mode = self.term.mode();
        let visible_cursor = mode.contains(TermMode::SHOW_CURSOR);
        let grid_cursor = self.term.grid().cursor.point;
        let viewport = point_to_viewport(self.term.grid().display_offset(), grid_cursor)
            .filter(|p| p.line < rows as usize);
        let cursor = match viewport {
            Some(point) => Cursor {
                row: point.line as u16,
                col: point.column.0 as u16,
                visible: visible_cursor,
            },
            None => Cursor {
                row: 0,
                col: 0,
                visible: false,
            },
        };

        let mouse_mode = if mode.contains(TermMode::MOUSE_MOTION) {
            MouseMode::Any
        } else if mode.contains(TermMode::MOUSE_DRAG) {
            MouseMode::Drag
        } else if mode.contains(TermMode::MOUSE_REPORT_CLICK) {
            MouseMode::Click
        } else {
            MouseMode::Off
        };

        let log = match self.log.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        Snapshot {
            cols,
            rows,
            cells,
            cursor,
            alt_screen: mode.contains(TermMode::ALT_SCREEN),
            bracketed_paste: mode.contains(TermMode::BRACKETED_PASTE),
            app_cursor_keys: mode.contains(TermMode::APP_CURSOR),
            mouse_mode,
            mouse_sgr: mode.contains(TermMode::SGR_MOUSE),
            title: log.title.clone(),
            events: log.events.clone(),
            bell_count: log.bell_count,
            scrollback: ScrollbackStats {
                retained_lines: self.term.history_size(),
                configured_bound: self.configured_bound,
            },
        }
    }
}

/// Factory matching [`crate::model::EngineFactory`].
#[must_use]
pub fn factory(cols: u16, rows: u16, scrollback: usize) -> Box<dyn EngineAdapter> {
    Box::new(AlacrittyAdapter::new(cols, rows, scrollback))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_lands_on_the_grid() {
        let mut a = AlacrittyAdapter::new(20, 3, 100);
        a.feed(b"hello");
        let s = a.snapshot();
        assert_eq!(s.row_text(0), "hello");
        assert_eq!(s.cursor.row, 0);
        assert_eq!(s.cursor.col, 5);
        assert!(s.cursor.visible);
        assert!(!s.alt_screen);
    }

    #[test]
    fn osc_title_is_reported_and_reset() {
        let mut a = AlacrittyAdapter::new(20, 3, 100);
        a.feed(b"\x1b]0;T1\x07");
        assert_eq!(a.snapshot().title.as_deref(), Some("T1"));
        a.feed(b"\x1b]0;\x07");
        assert_eq!(a.snapshot().title.as_deref(), Some(""));
    }

    #[test]
    fn bell_and_alt_screen() {
        let mut a = AlacrittyAdapter::new(20, 3, 100);
        a.feed(b"\x07\x1b[?1049h");
        let s = a.snapshot();
        assert_eq!(s.bell_count, 1);
        assert!(s.events.contains(&EngineEvent::Bell));
        assert!(s.alt_screen);
        a.feed(b"\x1b[?1049l");
        assert!(!a.snapshot().alt_screen);
    }

    #[test]
    fn wide_char_uses_two_columns() {
        let mut a = AlacrittyAdapter::new(20, 3, 100);
        a.feed("寬字元".as_bytes());
        let s = a.snapshot();
        assert_eq!(s.cell(0, 0).text, "寬");
        assert_eq!(s.cell(0, 1).text, "");
        assert_eq!(s.cell(0, 2).text, "字");
        assert_eq!(s.cell(0, 3).text, "");
        assert_eq!(s.cell(0, 4).text, "元");
        assert_eq!(s.cursor.col, 6);
        assert_eq!(s.row_text(0), "寬字元");
    }

    #[test]
    fn colors_parse() {
        let mut a = AlacrittyAdapter::new(20, 3, 100);
        a.feed(b"\x1b[31mr\x1b[0m");
        assert_eq!(a.snapshot().cell(0, 0).style.fg, Color::Indexed(1));
        a.feed(b"\x1b[38;2;12;34;56mT");
        assert_eq!(a.snapshot().cell(0, 1).style.fg, Color::Rgb(12, 34, 56));
    }
}
