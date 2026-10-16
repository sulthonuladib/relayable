use super::{Confidence, Measured};

/// Pane header height: **41.0 px**, the midpoint of the measured 40–42 px
/// range.
pub const PANE_HEADER_HEIGHT: Measured<f32> = Measured::new(
    41.0,
    "design-tokens.md \u{a7}3.1 v4-t0058 (V4 0:58)",
    Confidence::Medium,
    "documented range 40.0–42.0 px; 41.0 is the range midpoint; no colour step at the boundary",
);

/// Inter-pane gap: **9.5 px**, the midpoint of the measured 8–11 px range.
pub const INTER_PANE_GAP: Measured<f32> = Measured::new(
    9.5,
    "design-tokens.md \u{a7}3.1 v4-t0058 (V4 0:58)",
    Confidence::High,
    "documented range 8.0–11.0 px; 9.5 is the range midpoint",
);

/// Horizontal split ratio: 0.5 (documented ≈50/50, left ≈757 px / right
/// ≈767 px).
pub const SPLIT_RATIO: Measured<f32> = Measured::new(
    0.5,
    "design-tokens.md \u{a7}3.1 v4-t0058 (V4 0:58)",
    Confidence::MediumHigh,
    "exact ratio depends on the padding definition",
);

/// Focused pane border width: **2.5 px**, the midpoint of the measured 2–3
/// px range.
pub const PANE_BORDER_FOCUSED: Measured<f32> = Measured::new(
    2.5,
    "design-tokens.md \u{a7}3.1 v4-t0058 (V4 0:58)",
    Confidence::MediumHigh,
    "documented range 2.0–3.0 px; 2.5 is the range midpoint; includes antialias",
);

/// Unfocused pane border width: **2.5 px**, the midpoint of the measured
/// 2–3 px range.
pub const PANE_BORDER_UNFOCUSED: Measured<f32> = Measured::new(
    2.5,
    "design-tokens.md \u{a7}3.1 v4-t0058 (V4 0:58)",
    Confidence::Medium,
    "documented range 2.0–3.0 px; 2.5 is the range midpoint",
);

/// Command palette size: ≈442 × 667 px (outer bounds incl. border).
pub const COMMAND_PALETTE: Measured<(f32, f32)> = Measured::new(
    (442.0, 667.0),
    "design-tokens.md \u{a7}3.1 v4-t0030 (V4 0:30)",
    Confidence::High,
    "outer bounds incl. border",
);

/// Command palette row pitch: 37.5 px (documented range 36.0–38.0).
pub const COMMAND_PALETTE_ROW_PITCH: Measured<f32> = Measured::new(
    37.5,
    "design-tokens.md \u{a7}3.1 v4-t0030 (V4 0:30)",
    Confidence::High,
    "group header `Pane` breaks the rhythm (43/34 px gaps); the document states \u{2248}37.5 px",
);

/// Command palette selected row height: ≈35 px.
pub const COMMAND_PALETTE_SELECTED_ROW: Measured<f32> = Measured::new(
    35.0,
    "design-tokens.md \u{a7}3.1 v4-t0030 (V4 0:30)",
    Confidence::High,
    "no per-row caveat recorded in design-tokens.md \u{a7}3.1",
);

/// Top-bar → palette offset: ≈99 px.
pub const TOP_BAR_TO_PALETTE_OFFSET: Measured<f32> = Measured::new(
    99.0,
    "design-tokens.md \u{a7}3.1 v4-t0030 (V4 0:30)",
    Confidence::Medium,
    "panel float offset; moveable?",
);
