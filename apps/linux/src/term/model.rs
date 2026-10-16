use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line, Point};
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{Config, Term};
use vte::ansi::{Color as VteColor, CursorShape, NamedColor, Processor, Rgb};

use super::snapshot::{CellStyle, ScreenCell, ScreenSnapshot, TermColor, TerminalSize};
use super::BLANK_CELL;

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
