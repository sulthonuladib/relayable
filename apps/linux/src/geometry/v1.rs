use super::{Confidence, Measured};

/// Top bar height: 40 px (y 32→72).
pub const TOP_BAR_HEIGHT: Measured<f32> = Measured::new(
    40.0,
    "design-tokens.md \u{a7}3.1 v1-t0024/v1-t0047 (V1 0:24/0:47)",
    Confidence::High,
    "measured at the 1170-px frame scale; not converted to logical px (\u{a7}2.3)",
);

/// Top-bar separator thickness: **1.5 px**, the midpoint of the measured
/// 1–2 px range (y 72–73).
pub const TOP_BAR_SEPARATOR: Measured<f32> = Measured::new(
    1.5,
    "design-tokens.md \u{a7}3.1 v1-t0024/v1-t0047 (V1 0:24/0:47)",
    Confidence::High,
    "documented range 1.0–2.0 px; 1.5 is the range midpoint",
);

/// Active tab pill width: 205 px (x ≈573–778).
pub const ACTIVE_TAB_PILL_WIDTH: Measured<f32> = Measured::new(
    205.0,
    "design-tokens.md \u{a7}3.1 v1-t0047 (V1 0:47)",
    Confidence::High,
    "one frame only; pill width depends on tab count/label",
);

/// Active tab pill height: 30 px (y ≈36–66).
pub const ACTIVE_TAB_PILL_HEIGHT: Measured<f32> = Measured::new(
    30.0,
    "design-tokens.md \u{a7}3.1 v1-t0047 (V1 0:47)",
    Confidence::High,
    "one frame only",
);

/// Terminal row pitch (cell height): 17.25 px, the document's mean over 49
/// sampled rows (measured range 17.0–17.5).
pub const TERMINAL_ROW_PITCH: Measured<f32> = Measured::new(
    17.25,
    "design-tokens.md \u{a7}3.1 v1-t0140 (V1 1:40)",
    Confidence::High,
    "documented range 17.0–17.5 px; the document gives the mean 17.25",
);

/// Window outer size: ≈1099 × 706 px.
pub const WINDOW_OUTER: Measured<(f32, f32)> = Measured::new(
    (1099.0, 706.0),
    "design-tokens.md \u{a7}3.1 v1-t0024 (V1 0:24)",
    Confidence::High,
    "macOS rounded corners + shadow blur ±1–2 px at the corners",
);

/// Window inner content size: ≈1081 × 697 px.
pub const WINDOW_INNER: Measured<(f32, f32)> = Measured::new(
    (1081.0, 697.0),
    "design-tokens.md \u{a7}3.1 v1-t0024 (V1 0:24)",
    Confidence::Medium,
    "1–2 px border ambiguity",
);

/// V1 pane-header height.
///
/// **Explicitly unmeasurable**: `design-tokens.md` §4 records "Pane header
/// height in V1 — no colour boundary between header and terminal body".
/// This stays `None`; the V4 measurement is the evidence to use.
pub const PANE_HEADER_V1_UNMEASURED: Option<f32> = None;

/// Session picker popup width: ≈183 px (x 117–300).
///
/// (The brief groups this with the V4 tokens, but the measurement source is
/// V1 1:40 — recorded here in the V1 namespace to keep sources honest.)
pub const SESSION_PICKER_WIDTH: Measured<f32> = Measured::new(
    183.0,
    "design-tokens.md \u{a7}3.1 v1-t0140 (V1 1:40)",
    Confidence::Medium,
    "bottom edge soft (shadow)",
);

/// Session picker popup height: **139.5 px**, the midpoint of the measured
/// 137–142 px range (y ≈68–210).
pub const SESSION_PICKER_HEIGHT: Measured<f32> = Measured::new(
    139.5,
    "design-tokens.md \u{a7}3.1 v1-t0140 (V1 1:40)",
    Confidence::Medium,
    "documented range 137.0–142.0 px; 139.5 is the range midpoint; bottom edge soft (shadow)",
);

/// Tab-pill corner radius: **9.0 px**, the midpoint of the low-confidence
/// 8–10 px visual estimate.
pub const TAB_PILL_CORNER_RADIUS: Measured<f32> = Measured::new(
    9.0,
    "design-tokens.md \u{a7}3.1 v1-t0047 (V1 0:47), crop v1-0047-tabstrip.png",
    Confidence::Low,
    "documented visual estimate 8.0–10.0 px; 9.0 is the range midpoint; compression softens corners",
);

/// Tab-strip leading inset: **128 px**.
///
/// **Derived, not directly measured.** The measured active pill occupies
/// window-relative x 538–743; with three adjacent 205 px tabs that is
/// `128 + 205 + 205`, so the strip begins at x = 128. In the reference that
/// region held the macOS traffic lights, which are explicitly **not app
/// tokens**; Relayable leaves it empty (\u{a7}2.4 — a new product name does
/// not justify changing established geometry).
pub const TAB_LEADING_INSET: Measured<f32> = Measured::new(
    128.0,
    "derived from design-tokens.md \u{a7}3.1 v1-t0047 (V1 0:47): 538 = 128 + 205 + 205",
    Confidence::MediumHigh,
    "derived arithmetic, not a direct measurement; the region held the macOS traffic lights, which are not app tokens",
);

/// Inactive-tab divider thickness as rendered: **1 px** (lower bound of the
/// measured 1–2 px range; the local re-measure of `v1-t0047` found exactly
/// 1 px).
pub const TAB_DIVIDER_RENDER_PX: Measured<f32> = Measured::new(
    1.0,
    "design-tokens.md \u{a7}3.1 v1-t0047 (V1 0:47), local re-measure",
    Confidence::MediumHigh,
    "rendered integral value; the measured token range is 1\u{2013}2 px with a 1.5 px midpoint",
);

/// Session-picker search-field height: **28.5 px**, the midpoint of the
/// measured 27–30 px range.
pub const SESSION_PICKER_SEARCH_FIELD: Measured<f32> = Measured::new(
    28.5,
    "design-tokens.md \u{a7}3.1 v1-t0140 (V1 1:40)",
    Confidence::Medium,
    "documented range 27.0–30.0 px; 28.5 is the range midpoint",
);

/// Session-picker list-row height: **35 px** (documented point estimate).
pub const SESSION_PICKER_LIST_ROW: Measured<f32> = Measured::new(
    35.0,
    "design-tokens.md \u{a7}3.1 v1-t0140 (V1 1:40)",
    Confidence::Medium,
    "documented point estimate; row pitch is 35–40 px",
);

/// Top-bar separator thickness as rendered: **1 px** (the local re-measure
/// of `v1-t0047`/`v1-t0024` found exactly 1 px; the token's range is 1–2 px
/// with a 1.5 px midpoint).
pub const TOP_BAR_SEPARATOR_RENDER_PX: Measured<f32> = Measured::new(
    1.0,
    "design-tokens.md \u{a7}3.1 v1-t0024/v1-t0047 (V1 0:24/0:47), local re-measure",
    Confidence::High,
    "rendered integral value; the measured token range is 1\u{2013}2 px with a 1.5 px midpoint",
);
