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
