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
pub mod v1;

/// V4 geometry, measured at the 1624×1080 frame scale.
pub mod v4;

/// Relayable-original layout values (NON-REFERENCE / INVENTED).
///
/// These are app decisions, not measurements: `docs/research/design-tokens.md`
/// §4 records that pixel-exact corner radii, paddings and control sizes were
/// never measurable from the reference videos. They exist so `app.rs` does not
/// scatter bare literals; each is suffixed `_ORIGINAL` and this module is
/// deliberately kept out of [`v1`]/[`v4`] so provenance cannot be confused with
/// the measured tokens above. Moving a value here must not change what the app
/// paints.
pub mod invented;

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
