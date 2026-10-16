/// Terminal dimensions in character cells and (optionally) pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerminalSize {
    /// Terminal columns.
    pub cols: u16,
    /// Terminal rows.
    pub rows: u16,
    /// Pixel width, when known.
    pub pixel_width: u16,
    /// Pixel height, when known.
    pub pixel_height: u16,
}

impl TerminalSize {
    /// The conventional default terminal size (80×24, zero pixels).
    pub const DEFAULT_SIZE: TerminalSize = TerminalSize {
        cols: 80,
        rows: 24,
        pixel_width: 0,
        pixel_height: 0,
    };
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self::DEFAULT_SIZE
    }
}
