//! Reference geometry tokens for the UI-002 workspace, as **pure data**
//! (GPUI-free, no rendering).
//!
//! The values in [`v1`] and [`v4`] are **re-declared** from the documented
//! ranges in `docs/research/design-tokens.md` §3.1 (they are not read out of
//! [`crate::tokens`], which carries the colour sheet); `source`, `confidence`
//! and `caveat` are carried in the same [`Measured`] wrapper the token sheet
//! uses, so provenance cannot be dropped in transit. Where the document reports
//! a *range*, the value is the range midpoint and the caveat states the range —
//! no measured number is invented.
//!
//! [`invented`] is deliberately separate: it holds Relayable-original layout
//! values that were never measured and are labelled `*_ORIGINAL`.
//!
//! # Warning — values are raw video pixels, not logical pixels
//!
//! `design-tokens.md` §2.3 records that **no px → logical-pixel/point
//! conversion was performed** and that V1 (1170×1080 frame) and V4
//! (1624×1080 frame) absolute pixel values are **not comparable across
//! sources**. The modules below keep the V1/V4 namespaces separate for exactly
//! that reason.
//!
//! [`BASELINE_CANVAS_OUTER`] / [`BASELINE_CANVAS_INNER`] are the reference V1
//! window dimensions. They are treated as *logical* pixels **by assumption
//! only** — the documents provide no conversion, so this is not a measurement.

use crate::tokens::{Confidence, Measured};

/// V1 geometry, measured at the 1170×1080 frame scale.
pub mod v1 {
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
}

/// V4 geometry, measured at the 1624×1080 frame scale.
pub mod v4 {
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
}

/// Relayable-original layout values (NON-REFERENCE / INVENTED).
///
/// These are app decisions, not measurements: `docs/research/design-tokens.md`
/// §4 records that pixel-exact corner radii, paddings and control sizes were
/// never measurable from the reference videos. They exist so `app.rs` does not
/// scatter bare literals; each is suffixed `_ORIGINAL` and this module is
/// deliberately kept out of [`v1`]/[`v4`] so provenance cannot be confused with
/// the measured tokens above. Moving a value here must not change what the app
/// paints.
pub mod invented {
    /// Gap between the tab pill glyph and its label, in logical px.
    pub const TAB_PILL_CONTENT_GAP_ORIGINAL: f32 = 8.0;
    /// Horizontal padding inside a tab pill, in logical px.
    pub const TAB_PILL_PADDING_X_ORIGINAL: f32 = 16.0;
    /// Top padding of the tab strip inside the 40 px top bar, in logical px.
    pub const TAB_STRIP_PADDING_TOP_ORIGINAL: f32 = 4.0;
    /// Side length of the top-bar `+` new-tab control, in logical px.
    pub const NEW_TAB_CONTROL_SIZE_ORIGINAL: f32 = 40.0;
    /// Gap between the status dot and the text in a banner, in logical px.
    pub const BANNER_CONTENT_GAP_ORIGINAL: f32 = 8.0;
    /// Horizontal padding inside a banner, in logical px.
    pub const BANNER_PADDING_X_ORIGINAL: f32 = 12.0;
    /// Diameter of the banner status dot, in logical px.
    pub const BANNER_DOT_SIZE_ORIGINAL: f32 = 8.0;
    /// Corner radius of the banner status dot, in logical px.
    pub const BANNER_DOT_RADIUS_ORIGINAL: f32 = 4.0;
    /// Vertical gap between the title and message in the empty state, in
    /// logical px.
    pub const EMPTY_STATE_GAP_ORIGINAL: f32 = 8.0;
    /// Gap between the pane-header glyph and label, in logical px.
    pub const PANE_HEADER_GAP_ORIGINAL: f32 = 8.0;
    /// Horizontal padding inside a pane header, in logical px.
    pub const PANE_HEADER_PADDING_X_ORIGINAL: f32 = 10.0;
    /// Side length of a pane-header icon button, in logical px.
    pub const PANE_ICON_BUTTON_SIZE_ORIGINAL: f32 = 22.0;
    /// Corner radius of a pane-header icon button, in logical px.
    pub const PANE_ICON_BUTTON_RADIUS_ORIGINAL: f32 = 4.0;
    /// Vertical gap between the picker search field and its rows, in logical px.
    pub const PICKER_COLUMN_GAP_ORIGINAL: f32 = 2.0;
    /// Padding inside the picker popup, in logical px.
    pub const PICKER_PADDING_ORIGINAL: f32 = 4.0;
    /// Offset of the picker popup below the top-bar separator, in logical px.
    pub const PICKER_OFFSET_BELOW_SEPARATOR_ORIGINAL: f32 = 4.0;
    /// Gap between the check mark and label in a picker row, in logical px.
    pub const PICKER_ROW_GAP_ORIGINAL: f32 = 6.0;
    /// Horizontal padding inside a picker row, in logical px.
    pub const PICKER_ROW_PADDING_X_ORIGINAL: f32 = 6.0;
    /// Corner radius of a picker row, in logical px.
    pub const PICKER_ROW_RADIUS_ORIGINAL: f32 = 4.0;
    /// Horizontal padding inside the picker search field, in logical px.
    pub const PICKER_SEARCH_PADDING_X_ORIGINAL: f32 = 6.0;
    /// Corner radius of the picker search field, in logical px.
    pub const PICKER_SEARCH_RADIUS_ORIGINAL: f32 = 4.0;
    /// Side length of the split-right/split-down icon glyph, in logical px.
    pub const ICON_GLYPH_SPLIT_SIZE_ORIGINAL: f32 = 12.0;
    /// Thickness of the divider inside the split icon glyph, in logical px.
    pub const ICON_GLYPH_SPLIT_LINE_ORIGINAL: f32 = 1.0;
    /// Side length of the zoom icon glyph, in logical px.
    pub const ICON_GLYPH_ZOOM_SIZE_ORIGINAL: f32 = 11.0;
    /// Font size of the `\u{d7}` close glyph, in logical px.
    pub const ICON_GLYPH_CLOSE_FONT_SIZE_ORIGINAL: f32 = 14.0;
}

/// Reference V1 window **outer** size in logical px, treated as the baseline
/// canvas (1099 × 706).
///
/// [`Measured::caveat`] states the assumption: `design-tokens.md` §2.3 records
/// that **no px → logical conversion exists**, so these are logical px by
/// assumption only.
pub const BASELINE_CANVAS_OUTER: (f32, f32) = (1099.0, 706.0);

/// Reference V1 window **inner** size in logical px (1081 × 697).
pub const BASELINE_CANVAS_INNER: (f32, f32) = (1081.0, 697.0);

/// [`BASELINE_CANVAS_OUTER`] with its provenance.
pub const BASELINE_CANVAS_OUTER_MEASURED: Measured<(f32, f32)> = Measured::new(
    BASELINE_CANVAS_OUTER,
    "design-tokens.md \u{a7}3.1 v1-t0024 (V1 0:24)",
    Confidence::High,
    "raw video px; \u{a7}2.3 states there is NO px\u{2192}logical conversion, so these are treated as logical px by assumption only",
);

/// [`BASELINE_CANVAS_INNER`] with its provenance.
pub const BASELINE_CANVAS_INNER_MEASURED: Measured<(f32, f32)> = Measured::new(
    BASELINE_CANVAS_INNER,
    "design-tokens.md \u{a7}3.1 v1-t0024 (V1 0:24)",
    Confidence::Medium,
    "1–2 px border ambiguity; raw video px, treated as logical px by assumption only (\u{a7}2.3)",
);

/// Decompose the V1 vertical workspace chrome.
///
/// Returns `(top_bar_height, separator_thickness, body_height)` for an inner
/// content height. The body is what remains after the top bar and its
/// separator, clamped at zero so a tiny window cannot underflow.
///
/// The brief names this `workspace_columns`; the returned values are the V1
/// row heights of the top bar / separator / terminal body.
#[must_use]
pub fn workspace_columns(inner_height: f32) -> (f32, f32, f32) {
    let top_bar = v1::TOP_BAR_HEIGHT.value;
    let separator = v1::TOP_BAR_SEPARATOR.value;
    let body = (inner_height - top_bar - separator).max(0.0);
    (top_bar, separator, body)
}

/// Split a horizontal extent into two panes around a `gap`.
///
/// `ratio` is the left pane's share of the usable width (after the gap) and is
/// clamped to `0.0..=1.0`. Returns `(left_width, right_width)`; the two widths
/// plus `gap` reconstruct `total_width` whenever `total_width >= gap`, and tiny
/// widths clamp to zero instead of panicking.
#[must_use]
pub fn split_panes(total_width: f32, gap: f32, ratio: f32) -> (f32, f32) {
    let usable = (total_width - gap.max(0.0)).max(0.0);
    let ratio = ratio.clamp(0.0, 1.0);
    let left = (usable * ratio).max(0.0);
    let right = (usable - left).max(0.0);
    (left, right)
}

/// The horizontal (or vertical) extent of pane `index` when `count` panes are
/// laid out in a line with `gap` between adjacent panes and a total extent of
/// `total`.
///
/// The UI lays panes out with `flex: 1`, so from three panes up every pane
/// receives an equal share of the usable extent. With exactly two panes the
/// measured creation ratio ([`v4::SPLIT_RATIO`]) is honoured instead; because
/// that ratio is 0.5 the result is the same equal share, but routing it through
/// [`split_panes`] keeps the measured token visible in the code path.
///
/// Tiny totals clamp to zero instead of producing negatives, and `count == 0`
/// returns zero.
#[must_use]
pub fn pane_extent(total: f32, gap: f32, count: usize, ratio: f32, index: usize) -> f32 {
    if count == 0 || index >= count {
        return 0.0;
    }
    if count == 1 {
        return total.max(0.0);
    }
    if count == 2 {
        let (left, right) = split_panes(total, gap, ratio);
        return if index == 0 { left } else { right };
    }
    let gaps = gap.max(0.0) * (count as f32 - 1.0);
    ((total - gaps).max(0.0) / count as f32).max(0.0)
}

/// Ensure every value used by [`workspace_columns`] is a real measurement.
#[must_use]
pub fn top_bar_provenance() -> (Measured<f32>, Measured<f32>) {
    (v1::TOP_BAR_HEIGHT, v1::TOP_BAR_SEPARATOR)
}

/// Decompose the V1 vertical workspace chrome using the **rendered** 1 px
/// separator.
///
/// Same as [`workspace_columns`], but uses [`v1::TOP_BAR_SEPARATOR_RENDER_PX`]
/// (the measured integral 1 px) instead of the 1.5 px range midpoint, so the
/// painted chrome and the body rows cannot drift apart.
#[must_use]
pub fn workspace_columns_render(inner_height: f32) -> (f32, f32, f32) {
    let top_bar = v1::TOP_BAR_HEIGHT.value;
    let separator = v1::TOP_BAR_SEPARATOR_RENDER_PX.value;
    let body = (inner_height - top_bar - separator).max(0.0);
    (top_bar, separator, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_chrome_columns_sum_to_the_inner_height() {
        let (top, separator, body) = workspace_columns(697.0);
        assert_eq!(top, 40.0);
        assert_eq!(separator, 1.5);
        assert!((top + separator + body - 697.0).abs() < f32::EPSILON);
    }

    #[test]
    fn workspace_columns_clamps_tiny_heights_without_panicking() {
        for height in [-100.0, 0.0, 1.0, 10.0, 41.5] {
            let (top, separator, body) = workspace_columns(height);
            assert_eq!(top, 40.0);
            assert_eq!(separator, 1.5);
            assert!(body >= 0.0, "body must never be negative for {height}");
        }
    }

    #[test]
    fn split_panes_reconstructs_the_total_width() {
        let (left, right) = split_panes(1081.0, 9.5, 0.5);
        assert!((left + right + 9.5 - 1081.0).abs() < 0.01);
        assert!((left - right).abs() < 1.0);
    }

    #[test]
    fn split_panes_clamps_tiny_widths_and_ratios() {
        let (left, right) = split_panes(5.0, 9.5, 0.5);
        assert_eq!((left, right), (0.0, 0.0));

        let (left, right) = split_panes(-10.0, 0.0, 5.0);
        assert_eq!((left, right), (0.0, 0.0));

        let (left, right) = split_panes(100.0, 0.0, 2.0);
        assert_eq!((left, right), (100.0, 0.0));
    }

    #[test]
    fn pane_extent_divides_equally_for_three_or_more_and_honours_the_two_pane_ratio() {
        // Three panes over 1000 px with two 10 px gaps: 980 / 3 each.
        let first = pane_extent(1000.0, 10.0, 3, 0.5, 0);
        let second = pane_extent(1000.0, 10.0, 3, 0.5, 1);
        let third = pane_extent(1000.0, 10.0, 3, 0.5, 2);
        assert!((first - second).abs() < f32::EPSILON);
        assert!((second - third).abs() < f32::EPSILON);
        assert!((first + second + third + 20.0 - 1000.0).abs() < 0.01);

        // Two panes follow the measured split ratio (0.5 -> equal share).
        let left = pane_extent(1081.0, 9.5, 2, 0.5, 0);
        let right = pane_extent(1081.0, 9.5, 2, 0.5, 1);
        assert!((left + right + 9.5 - 1081.0).abs() < 0.01);
        assert!((left - right).abs() < 1.0);

        assert_eq!(pane_extent(500.0, 9.5, 1, 0.5, 0), 500.0);
        assert_eq!(pane_extent(500.0, 9.5, 0, 0.5, 0), 0.0);
        assert_eq!(pane_extent(500.0, 9.5, 3, 0.5, 3), 0.0);
    }

    #[test]
    fn pane_header_v1_stays_explicitly_unmeasured() {
        assert_eq!(v1::PANE_HEADER_V1_UNMEASURED, None);
    }

    #[test]
    fn rendered_chrome_uses_the_measured_one_pixel_separator() {
        let (top, separator, body) = workspace_columns_render(706.0);
        assert_eq!(top, 40.0);
        assert_eq!(separator, 1.0);
        assert!((top + separator + body - 706.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tab_leading_inset_matches_the_measured_pill_arithmetic() {
        assert_eq!(v1::TAB_LEADING_INSET.value, 128.0);
        assert_eq!(
            v1::TAB_LEADING_INSET.value + 2.0 * v1::ACTIVE_TAB_PILL_WIDTH.value,
            538.0,
            "the measured third-pill left edge is 538"
        );
    }

    #[test]
    fn tab_pill_radius_is_the_documented_range_midpoint() {
        assert!((8.0..=10.0).contains(&v1::TAB_PILL_CORNER_RADIUS.value));
    }

    #[test]
    fn baseline_canvas_matches_the_v1_window_tokens() {
        assert_eq!(BASELINE_CANVAS_OUTER, v1::WINDOW_OUTER.value);
        assert_eq!(BASELINE_CANVAS_INNER, v1::WINDOW_INNER.value);
        assert_eq!(BASELINE_CANVAS_OUTER_MEASURED.value, (1099.0, 706.0));
        assert_eq!(BASELINE_CANVAS_INNER_MEASURED.value, (1081.0, 697.0));
    }

    #[test]
    fn midpoints_are_inside_the_documented_ranges() {
        const { assert!(v1::TOP_BAR_SEPARATOR.value >= 1.0 && v1::TOP_BAR_SEPARATOR.value <= 2.0) };
        const { assert!(v4::PANE_HEADER_HEIGHT.value >= 40.0 && v4::PANE_HEADER_HEIGHT.value <= 42.0) };
        const { assert!(v4::INTER_PANE_GAP.value >= 8.0 && v4::INTER_PANE_GAP.value <= 11.0) };
        const { assert!(v4::PANE_BORDER_FOCUSED.value >= 2.0 && v4::PANE_BORDER_FOCUSED.value <= 3.0) };
        const {
            assert!(
                v1::SESSION_PICKER_HEIGHT.value >= 137.0
                    && v1::SESSION_PICKER_HEIGHT.value <= 142.0
            )
        };
    }

    #[test]
    fn baseline_canvas_carries_the_no_conversion_caveat() {
        assert!(BASELINE_CANVAS_OUTER_MEASURED.caveat.contains("NO px"));
        assert_eq!(
            BASELINE_CANVAS_OUTER_MEASURED.source,
            "design-tokens.md \u{a7}3.1 v1-t0024 (V1 0:24)"
        );
    }
}
