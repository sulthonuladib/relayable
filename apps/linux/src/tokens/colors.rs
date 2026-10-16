use super::{Confidence, Measured, Rgb8};

/// Top bar background: rgb(4,27,46) (V1 t0047 p50 (3,27,46)).
pub const TOP_BAR_BACKGROUND: Measured<Rgb8> = Measured::new(
    Rgb8::new(4, 27, 46),
    "V1 0:24 `v1-t0024.png`; V1 0:47 `v1-t0047.png`",
    Confidence::High,
    "two frames agree ±1",
);

/// Active tab pill fill: rgb(19,41,62).
pub const ACTIVE_TAB_PILL_FILL: Measured<Rgb8> = Measured::new(
    Rgb8::new(19, 41, 62),
    "V1 0:47 `v1-t0047.png`",
    Confidence::High,
    "flat region, low variance",
);

/// Active tab pill edge/highlight, top: rgb ≈(50,75,98).
pub const ACTIVE_TAB_PILL_EDGE_TOP: Measured<Rgb8> = Measured::new(
    Rgb8::new(50, 75, 98),
    "V1 0:47 `v1-t0047.png`",
    Confidence::Medium,
    "1–2 px line; could be border or inner highlight",
);

/// Active tab pill edge/highlight, bottom: rgb ≈(54,76,99).
pub const ACTIVE_TAB_PILL_EDGE_BOTTOM: Measured<Rgb8> = Measured::new(
    Rgb8::new(54, 76, 99),
    "V1 0:47 `v1-t0047.png`",
    Confidence::Medium,
    "1–2 px line; could be border or inner highlight",
);

/// Pane header/surface background (V1): rgb(9,30,46).
pub const PANE_HEADER_SURFACE_V1: Measured<Rgb8> = Measured::new(
    Rgb8::new(9, 30, 46),
    "V1 0:24 `v1-t0024.png`",
    Confidence::High,
    "no visible separation between header and terminal",
);

/// Pane card border / focused (V4): rgb ≈(31,82,115).
///
/// `docs/research/design-tokens.md:127` (restated at `:201`).
pub const PANE_CARD_BORDER_FOCUSED: Measured<Rgb8> = Measured::new(
    Rgb8::new(31, 82, 115),
    "V4 0:58 `v4-t0058.png`",
    Confidence::MediumHigh,
    "includes antialias",
);

/// Pane card border / unfocused (V4): rgb ≈(17,54,71).
///
/// `docs/research/design-tokens.md:128` (restated at `:201`).
pub const PANE_CARD_BORDER_UNFOCUSED: Measured<Rgb8> = Measured::new(
    Rgb8::new(17, 54, 71),
    "V4 0:58 `v4-t0058.png`",
    Confidence::Medium,
    "one frame",
);

/// Palette panel background: rgb(10,29,41).
pub const PALETTE_PANEL_BACKGROUND: Measured<Rgb8> = Measured::new(
    Rgb8::new(10, 29, 41),
    "V4 0:30 `v4-t0030.png`",
    Confidence::High,
    "flat",
);

/// Palette search field: rgb(31,54,70).
pub const PALETTE_SEARCH_FIELD: Measured<Rgb8> = Measured::new(
    Rgb8::new(31, 54, 70),
    "V4 0:30 `v4-t0030.png`",
    Confidence::Medium,
    "includes placeholder antialias in p95 only",
);

/// Palette border: bottom edge (85,109,113); the document reports the range
/// as ≈(71–102,102–116,113–125) and says to treat it as one light neutral
/// border token.
pub const PALETTE_BORDER: Measured<Rgb8> = Measured::new(
    Rgb8::new(85, 109, 113),
    "V4 0:30 `v4-t0030.png`",
    Confidence::Medium,
    "edge antialias; treat as one light neutral border token (document range ≈(71–102,102–116,113–125))",
);

/// Session picker background: rgb ≈(14,28,39).
pub const SESSION_PICKER_BACKGROUND: Measured<Rgb8> = Measured::new(
    Rgb8::new(14, 28, 39),
    "V1 1:40 `v1-t0140.png`",
    Confidence::Medium,
    "darker than command palette (different frames/scales)",
);

/// Session picker border: rgb ≈(80,97,111).
pub const SESSION_PICKER_BORDER: Measured<Rgb8> = Measured::new(
    Rgb8::new(80, 97, 111),
    "V1 1:40 `v1-t0140.png`",
    Confidence::Medium,
    "1–2 px, glow-like",
);

/// Top-bar separator (V1): rgb(30,49,66).
pub const TOP_BAR_SEPARATOR: Measured<Rgb8> = Measured::new(
    Rgb8::new(30, 49, 66),
    "V1 0:24/0:47 `v1-t0024.png`, `v1-t0047.png`",
    Confidence::High,
    "no per-row caveat recorded in design-tokens.md §3.1; §2.2 warns 1–2 px lines are less reliable than flat fills",
);

/// Inactive tab divider: rgb(57,73,95).
pub const TAB_DIVIDER_INACTIVE: Measured<Rgb8> = Measured::new(
    Rgb8::new(57, 73, 95),
    "V1 0:47 `v1-t0047.png`",
    Confidence::Medium,
    "visible between two inactive tabs only",
);

/// Primary text: bulk p95 ≈ rgb(191,219,228); bright-glyph samples reach
/// rgb(230,246,255).
pub const PRIMARY_TEXT: Measured<Rgb8> = Measured::new(
    Rgb8::new(191, 219, 228),
    "V1 0:47 `v1-t0047.png` (active tab `~`); V4 0:58 `v4-t0058.png` (table header)",
    Confidence::Medium,
    "thin-stroke p95 underestimates the true value; exact font colour not isolatable from compression (bright samples up to rgb(230,246,255))",
);

/// Secondary text: clock/date p95 rgb(108,111,159).
pub const SECONDARY_TEXT: Measured<Rgb8> = Measured::new(
    Rgb8::new(108, 111, 159),
    "V1 0:24 `v1-t0024.png`",
    Confidence::LowMedium,
    "antialiased thin text; compression lowers p95",
);

/// Terminal prompt (V1 local): p95 rgb(125,128,207).
pub const TERMINAL_PROMPT_V1_LOCAL: Measured<Rgb8> = Measured::new(
    Rgb8::new(125, 128, 207),
    "V1 0:24 `v1-t0024.png`",
    Confidence::LowMedium,
    "glyph pixels only; small sample",
);

/// Remote prompt (V4): p95 rgb(128,203,167).
pub const REMOTE_PROMPT_V4: Measured<Rgb8> = Measured::new(
    Rgb8::new(128, 203, 167),
    "V4 0:58 `v4-t0058.png`",
    Confidence::Medium,
    "matches the visible green prompt",
);

/// Active/selected horizontal tab: fill rgb(19,41,62) vs bar rgb(4,27,46).
pub const SELECTION_TAB_FILL: Measured<Rgb8> = Measured::new(
    Rgb8::new(19, 41, 62),
    "V1 0:47 `v1-t0047.png`",
    Confidence::High,
    "selection token for tabs",
);

/// Active/selected session-picker row: **blue** fill p50 rgb(68,156,255).
pub const SELECTION_SESSION_PICKER_ROW: Measured<Rgb8> = Measured::new(
    Rgb8::new(68, 156, 255),
    "V1 1:40 `v1-t0140.png`",
    Confidence::High,
    "large flat fill; the `✓ Demo` row",
);

/// Active/selected command-palette row: **mint/green** fill p50
/// rgb(33,150,137).
pub const SELECTION_COMMAND_PALETTE_ROW: Measured<Rgb8> = Measured::new(
    Rgb8::new(33, 150, 137),
    "V4 0:30 `v4-t0030.png`",
    Confidence::High,
    "confirms V1-blue vs V4-mint difference at 1080p",
);

/// Command-palette selected row brighter edge sample: rgb(54,171,158).
pub const SELECTION_COMMAND_PALETTE_ROW_EDGE: Measured<Rgb8> = Measured::new(
    Rgb8::new(54, 171, 158),
    "V4 0:30 `v4-t0030.png`",
    Confidence::High,
    "confirms V1-blue vs V4-mint difference at 1080p",
);
