use portable_pty::PtySize as RawPtySize;

/// Terminal geometry for a pseudo-terminal.
///
/// Mirrors the wire `TerminalSize` shape but is deliberately a local type so no
/// transport or domain crate is pulled into the PTY boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PtySize {
    pub cols: u16,
    pub rows: u16,
    pub pixel_width: u16,
    pub pixel_height: u16,
}

impl PtySize {
    /// The default geometry: 80 columns by 24 rows, no pixel metrics.
    pub const DEFAULT: PtySize = PtySize {
        cols: 80,
        rows: 24,
        pixel_width: 0,
        pixel_height: 0,
    };

    /// Creates a geometry from a column/row pair with no pixel metrics.
    pub const fn new(cols: u16, rows: u16) -> Self {
        PtySize {
            cols,
            rows,
            pixel_width: 0,
            pixel_height: 0,
        }
    }

    pub(crate) fn to_raw(self) -> RawPtySize {
        RawPtySize {
            rows: self.rows,
            cols: self.cols,
            pixel_width: self.pixel_width,
            pixel_height: self.pixel_height,
        }
    }

    pub(crate) fn from_raw(raw: RawPtySize) -> Self {
        PtySize {
            cols: raw.cols,
            rows: raw.rows,
            pixel_width: raw.pixel_width,
            pixel_height: raw.pixel_height,
        }
    }
}

impl Default for PtySize {
    fn default() -> Self {
        Self::DEFAULT
    }
}
