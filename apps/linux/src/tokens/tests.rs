use super::geometry::{v1, v4};
use super::*;

/// Every geometry `Approx`, with a label, for structural checks.
fn all_geometry() -> Vec<(&'static str, Approx)> {
    vec![
        ("v1::WINDOW_OUTER_X_PX", v1::WINDOW_OUTER_X_PX.value),
        ("v1::WINDOW_OUTER_Y_PX", v1::WINDOW_OUTER_Y_PX.value),
        ("v1::WINDOW_OUTER_ASPECT", v1::WINDOW_OUTER_ASPECT.value),
        ("v1::WINDOW_INNER_X_PX", v1::WINDOW_INNER_X_PX.value),
        ("v1::WINDOW_INNER_Y_PX", v1::WINDOW_INNER_Y_PX.value),
        ("v1::WINDOW_INNER_ASPECT", v1::WINDOW_INNER_ASPECT.value),
        ("v1::TOP_BAR_HEIGHT_PX", v1::TOP_BAR_HEIGHT_PX.value),
        ("v1::TOP_BAR_SEPARATOR_PX", v1::TOP_BAR_SEPARATOR_PX.value),
        (
            "v1::ACTIVE_TAB_PILL_WIDTH_PX",
            v1::ACTIVE_TAB_PILL_WIDTH_PX.value,
        ),
        (
            "v1::ACTIVE_TAB_PILL_HEIGHT_PX",
            v1::ACTIVE_TAB_PILL_HEIGHT_PX.value,
        ),
        (
            "v1::TAB_PILL_CORNER_RADIUS_PX",
            v1::TAB_PILL_CORNER_RADIUS_PX.value,
        ),
        (
            "v1::INACTIVE_TAB_DIVIDER_PX",
            v1::INACTIVE_TAB_DIVIDER_PX.value,
        ),
        (
            "v1::SESSION_PICKER_WIDTH_PX",
            v1::SESSION_PICKER_WIDTH_PX.value,
        ),
        (
            "v1::SESSION_PICKER_HEIGHT_PX",
            v1::SESSION_PICKER_HEIGHT_PX.value,
        ),
        (
            "v1::SESSION_PICKER_SEARCH_FIELD_PX",
            v1::SESSION_PICKER_SEARCH_FIELD_PX.value,
        ),
        (
            "v1::SESSION_PICKER_LIST_ROW_PX",
            v1::SESSION_PICKER_LIST_ROW_PX.value,
        ),
        (
            "v1::SESSION_PICKER_ROW_PITCH_PX",
            v1::SESSION_PICKER_ROW_PITCH_PX.value,
        ),
        ("v1::TERMINAL_ROW_PITCH_PX", v1::TERMINAL_ROW_PITCH_PX.value),
        ("v4::WINDOW_WIDTH_X_PX", v4::WINDOW_WIDTH_X_PX.value),
        ("v4::HEADER_REGION_PX", v4::HEADER_REGION_PX.value),
        ("v4::PANE_HEADER_HEIGHT_PX", v4::PANE_HEADER_HEIGHT_PX.value),
        (
            "v4::PANE_CARD_BORDER_FOCUSED_PX",
            v4::PANE_CARD_BORDER_FOCUSED_PX.value,
        ),
        (
            "v4::PANE_CARD_BORDER_UNFOCUSED_PX",
            v4::PANE_CARD_BORDER_UNFOCUSED_PX.value,
        ),
        ("v4::INTER_PANE_GAP_PX", v4::INTER_PANE_GAP_PX.value),
        ("v4::OUTER_MARGIN_LEFT_PX", v4::OUTER_MARGIN_LEFT_PX.value),
        ("v4::OUTER_MARGIN_RIGHT_PX", v4::OUTER_MARGIN_RIGHT_PX.value),
        ("v4::SPLIT_PANE_LEFT_PX", v4::SPLIT_PANE_LEFT_PX.value),
        ("v4::SPLIT_PANE_RIGHT_PX", v4::SPLIT_PANE_RIGHT_PX.value),
        (
            "v4::TOP_BAR_TO_PALETTE_OFFSET_PX",
            v4::TOP_BAR_TO_PALETTE_OFFSET_PX.value,
        ),
        (
            "v4::COMMAND_PALETTE_WIDTH_PX",
            v4::COMMAND_PALETTE_WIDTH_PX.value,
        ),
        (
            "v4::COMMAND_PALETTE_HEIGHT_PX",
            v4::COMMAND_PALETTE_HEIGHT_PX.value,
        ),
        (
            "v4::COMMAND_PALETTE_ROW_PITCH_PX",
            v4::COMMAND_PALETTE_ROW_PITCH_PX.value,
        ),
        (
            "v4::COMMAND_PALETTE_SELECTED_ROW_PX",
            v4::COMMAND_PALETTE_SELECTED_ROW_PX.value,
        ),
    ]
}

/// Every `Measured` source/caveat pair, with a label.
fn all_provenance() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (
            "v1::WINDOW_OUTER_X_PX",
            v1::WINDOW_OUTER_X_PX.source,
            v1::WINDOW_OUTER_X_PX.caveat,
        ),
        (
            "v1::WINDOW_OUTER_Y_PX",
            v1::WINDOW_OUTER_Y_PX.source,
            v1::WINDOW_OUTER_Y_PX.caveat,
        ),
        (
            "v1::WINDOW_OUTER_ASPECT",
            v1::WINDOW_OUTER_ASPECT.source,
            v1::WINDOW_OUTER_ASPECT.caveat,
        ),
        (
            "v1::WINDOW_INNER_X_PX",
            v1::WINDOW_INNER_X_PX.source,
            v1::WINDOW_INNER_X_PX.caveat,
        ),
        (
            "v1::WINDOW_INNER_Y_PX",
            v1::WINDOW_INNER_Y_PX.source,
            v1::WINDOW_INNER_Y_PX.caveat,
        ),
        (
            "v1::WINDOW_INNER_ASPECT",
            v1::WINDOW_INNER_ASPECT.source,
            v1::WINDOW_INNER_ASPECT.caveat,
        ),
        (
            "v1::TOP_BAR_HEIGHT_PX",
            v1::TOP_BAR_HEIGHT_PX.source,
            v1::TOP_BAR_HEIGHT_PX.caveat,
        ),
        (
            "v1::TOP_BAR_SEPARATOR_PX",
            v1::TOP_BAR_SEPARATOR_PX.source,
            v1::TOP_BAR_SEPARATOR_PX.caveat,
        ),
        (
            "v1::ACTIVE_TAB_PILL_WIDTH_PX",
            v1::ACTIVE_TAB_PILL_WIDTH_PX.source,
            v1::ACTIVE_TAB_PILL_WIDTH_PX.caveat,
        ),
        (
            "v1::ACTIVE_TAB_PILL_HEIGHT_PX",
            v1::ACTIVE_TAB_PILL_HEIGHT_PX.source,
            v1::ACTIVE_TAB_PILL_HEIGHT_PX.caveat,
        ),
        (
            "v1::TAB_PILL_CORNER_RADIUS_PX",
            v1::TAB_PILL_CORNER_RADIUS_PX.source,
            v1::TAB_PILL_CORNER_RADIUS_PX.caveat,
        ),
        (
            "v1::INACTIVE_TAB_DIVIDER_PX",
            v1::INACTIVE_TAB_DIVIDER_PX.source,
            v1::INACTIVE_TAB_DIVIDER_PX.caveat,
        ),
        (
            "v1::SESSION_PICKER_WIDTH_PX",
            v1::SESSION_PICKER_WIDTH_PX.source,
            v1::SESSION_PICKER_WIDTH_PX.caveat,
        ),
        (
            "v1::SESSION_PICKER_HEIGHT_PX",
            v1::SESSION_PICKER_HEIGHT_PX.source,
            v1::SESSION_PICKER_HEIGHT_PX.caveat,
        ),
        (
            "v1::SESSION_PICKER_SEARCH_FIELD_PX",
            v1::SESSION_PICKER_SEARCH_FIELD_PX.source,
            v1::SESSION_PICKER_SEARCH_FIELD_PX.caveat,
        ),
        (
            "v1::SESSION_PICKER_LIST_ROW_PX",
            v1::SESSION_PICKER_LIST_ROW_PX.source,
            v1::SESSION_PICKER_LIST_ROW_PX.caveat,
        ),
        (
            "v1::SESSION_PICKER_ROW_PITCH_PX",
            v1::SESSION_PICKER_ROW_PITCH_PX.source,
            v1::SESSION_PICKER_ROW_PITCH_PX.caveat,
        ),
        (
            "v1::TERMINAL_ROW_PITCH_PX",
            v1::TERMINAL_ROW_PITCH_PX.source,
            v1::TERMINAL_ROW_PITCH_PX.caveat,
        ),
        (
            "v4::WINDOW_WIDTH_X_PX",
            v4::WINDOW_WIDTH_X_PX.source,
            v4::WINDOW_WIDTH_X_PX.caveat,
        ),
        (
            "v4::HEADER_REGION_PX",
            v4::HEADER_REGION_PX.source,
            v4::HEADER_REGION_PX.caveat,
        ),
        (
            "v4::PANE_HEADER_HEIGHT_PX",
            v4::PANE_HEADER_HEIGHT_PX.source,
            v4::PANE_HEADER_HEIGHT_PX.caveat,
        ),
        (
            "v4::PANE_CARD_BORDER_FOCUSED_PX",
            v4::PANE_CARD_BORDER_FOCUSED_PX.source,
            v4::PANE_CARD_BORDER_FOCUSED_PX.caveat,
        ),
        (
            "v4::PANE_CARD_BORDER_UNFOCUSED_PX",
            v4::PANE_CARD_BORDER_UNFOCUSED_PX.source,
            v4::PANE_CARD_BORDER_UNFOCUSED_PX.caveat,
        ),
        (
            "v4::INTER_PANE_GAP_PX",
            v4::INTER_PANE_GAP_PX.source,
            v4::INTER_PANE_GAP_PX.caveat,
        ),
        (
            "v4::OUTER_MARGIN_LEFT_PX",
            v4::OUTER_MARGIN_LEFT_PX.source,
            v4::OUTER_MARGIN_LEFT_PX.caveat,
        ),
        (
            "v4::OUTER_MARGIN_RIGHT_PX",
            v4::OUTER_MARGIN_RIGHT_PX.source,
            v4::OUTER_MARGIN_RIGHT_PX.caveat,
        ),
        (
            "v4::SPLIT_PANE_LEFT_PX",
            v4::SPLIT_PANE_LEFT_PX.source,
            v4::SPLIT_PANE_LEFT_PX.caveat,
        ),
        (
            "v4::SPLIT_PANE_RIGHT_PX",
            v4::SPLIT_PANE_RIGHT_PX.source,
            v4::SPLIT_PANE_RIGHT_PX.caveat,
        ),
        (
            "v4::TOP_BAR_TO_PALETTE_OFFSET_PX",
            v4::TOP_BAR_TO_PALETTE_OFFSET_PX.source,
            v4::TOP_BAR_TO_PALETTE_OFFSET_PX.caveat,
        ),
        (
            "v4::COMMAND_PALETTE_WIDTH_PX",
            v4::COMMAND_PALETTE_WIDTH_PX.source,
            v4::COMMAND_PALETTE_WIDTH_PX.caveat,
        ),
        (
            "v4::COMMAND_PALETTE_HEIGHT_PX",
            v4::COMMAND_PALETTE_HEIGHT_PX.source,
            v4::COMMAND_PALETTE_HEIGHT_PX.caveat,
        ),
        (
            "v4::COMMAND_PALETTE_ROW_PITCH_PX",
            v4::COMMAND_PALETTE_ROW_PITCH_PX.source,
            v4::COMMAND_PALETTE_ROW_PITCH_PX.caveat,
        ),
        (
            "v4::COMMAND_PALETTE_SELECTED_ROW_PX",
            v4::COMMAND_PALETTE_SELECTED_ROW_PX.source,
            v4::COMMAND_PALETTE_SELECTED_ROW_PX.caveat,
        ),
        (
            "colors::TOP_BAR_BACKGROUND",
            colors::TOP_BAR_BACKGROUND.source,
            colors::TOP_BAR_BACKGROUND.caveat,
        ),
        (
            "colors::ACTIVE_TAB_PILL_FILL",
            colors::ACTIVE_TAB_PILL_FILL.source,
            colors::ACTIVE_TAB_PILL_FILL.caveat,
        ),
        (
            "colors::ACTIVE_TAB_PILL_EDGE_TOP",
            colors::ACTIVE_TAB_PILL_EDGE_TOP.source,
            colors::ACTIVE_TAB_PILL_EDGE_TOP.caveat,
        ),
        (
            "colors::ACTIVE_TAB_PILL_EDGE_BOTTOM",
            colors::ACTIVE_TAB_PILL_EDGE_BOTTOM.source,
            colors::ACTIVE_TAB_PILL_EDGE_BOTTOM.caveat,
        ),
        (
            "colors::PANE_HEADER_SURFACE_V1",
            colors::PANE_HEADER_SURFACE_V1.source,
            colors::PANE_HEADER_SURFACE_V1.caveat,
        ),
        (
            "colors::PANE_CARD_BORDER_FOCUSED",
            colors::PANE_CARD_BORDER_FOCUSED.source,
            colors::PANE_CARD_BORDER_FOCUSED.caveat,
        ),
        (
            "colors::PANE_CARD_BORDER_UNFOCUSED",
            colors::PANE_CARD_BORDER_UNFOCUSED.source,
            colors::PANE_CARD_BORDER_UNFOCUSED.caveat,
        ),
        (
            "colors::PALETTE_PANEL_BACKGROUND",
            colors::PALETTE_PANEL_BACKGROUND.source,
            colors::PALETTE_PANEL_BACKGROUND.caveat,
        ),
        (
            "colors::PALETTE_SEARCH_FIELD",
            colors::PALETTE_SEARCH_FIELD.source,
            colors::PALETTE_SEARCH_FIELD.caveat,
        ),
        (
            "colors::PALETTE_BORDER",
            colors::PALETTE_BORDER.source,
            colors::PALETTE_BORDER.caveat,
        ),
        (
            "colors::SESSION_PICKER_BACKGROUND",
            colors::SESSION_PICKER_BACKGROUND.source,
            colors::SESSION_PICKER_BACKGROUND.caveat,
        ),
        (
            "colors::SESSION_PICKER_BORDER",
            colors::SESSION_PICKER_BORDER.source,
            colors::SESSION_PICKER_BORDER.caveat,
        ),
        (
            "colors::TOP_BAR_SEPARATOR",
            colors::TOP_BAR_SEPARATOR.source,
            colors::TOP_BAR_SEPARATOR.caveat,
        ),
        (
            "colors::TAB_DIVIDER_INACTIVE",
            colors::TAB_DIVIDER_INACTIVE.source,
            colors::TAB_DIVIDER_INACTIVE.caveat,
        ),
        (
            "colors::PRIMARY_TEXT",
            colors::PRIMARY_TEXT.source,
            colors::PRIMARY_TEXT.caveat,
        ),
        (
            "colors::SECONDARY_TEXT",
            colors::SECONDARY_TEXT.source,
            colors::SECONDARY_TEXT.caveat,
        ),
        (
            "colors::TERMINAL_PROMPT_V1_LOCAL",
            colors::TERMINAL_PROMPT_V1_LOCAL.source,
            colors::TERMINAL_PROMPT_V1_LOCAL.caveat,
        ),
        (
            "colors::REMOTE_PROMPT_V4",
            colors::REMOTE_PROMPT_V4.source,
            colors::REMOTE_PROMPT_V4.caveat,
        ),
        (
            "colors::SELECTION_TAB_FILL",
            colors::SELECTION_TAB_FILL.source,
            colors::SELECTION_TAB_FILL.caveat,
        ),
        (
            "colors::SELECTION_SESSION_PICKER_ROW",
            colors::SELECTION_SESSION_PICKER_ROW.source,
            colors::SELECTION_SESSION_PICKER_ROW.caveat,
        ),
        (
            "colors::SELECTION_COMMAND_PALETTE_ROW",
            colors::SELECTION_COMMAND_PALETTE_ROW.source,
            colors::SELECTION_COMMAND_PALETTE_ROW.caveat,
        ),
        (
            "colors::SELECTION_COMMAND_PALETTE_ROW_EDGE",
            colors::SELECTION_COMMAND_PALETTE_ROW_EDGE.source,
            colors::SELECTION_COMMAND_PALETTE_ROW_EDGE.caveat,
        ),
        (
            "terminal::MAIN_TERMINAL_BACKGROUND_RAMP_V1",
            terminal::MAIN_TERMINAL_BACKGROUND_RAMP_V1.source,
            terminal::MAIN_TERMINAL_BACKGROUND_RAMP_V1.caveat,
        ),
        (
            "terminal::FOCUSED_PANE_SURFACE_V4",
            terminal::FOCUSED_PANE_SURFACE_V4.source,
            terminal::FOCUSED_PANE_SURFACE_V4.caveat,
        ),
        (
            "terminal::UNFOCUSED_PANE_SURFACE_V4",
            terminal::UNFOCUSED_PANE_SURFACE_V4.source,
            terminal::UNFOCUSED_PANE_SURFACE_V4.caveat,
        ),
        (
            "not_app_tokens::MACOS_TRAFFIC_LIGHTS",
            not_app_tokens::MACOS_TRAFFIC_LIGHTS.source,
            not_app_tokens::MACOS_TRAFFIC_LIGHTS.caveat,
        ),
        (
            "not_app_tokens::ACCENT_TERMINAL_GLYPH_RANGE",
            not_app_tokens::ACCENT_TERMINAL_GLYPH_RANGE.source,
            not_app_tokens::ACCENT_TERMINAL_GLYPH_RANGE.caveat,
        ),
        (
            "not_app_tokens::TRAFFIC_LIGHT_ADJACENT_TOOLBAR_BUTTON",
            not_app_tokens::TRAFFIC_LIGHT_ADJACENT_TOOLBAR_BUTTON.source,
            not_app_tokens::TRAFFIC_LIGHT_ADJACENT_TOOLBAR_BUTTON.caveat,
        ),
    ]
}

#[test]
fn documented_colour_equals_exact_triple() {
    // design-tokens.md §3.2: "Top bar background | rgb(4,27,46)".
    assert_eq!(
        colors::TOP_BAR_BACKGROUND.value,
        Rgb8::new(4, 27, 46),
        "top bar background must be exactly the documented triple"
    );
    // §3.2: "Active tab pill fill | rgb(19,41,62)".
    assert_eq!(colors::ACTIVE_TAB_PILL_FILL.value, Rgb8::new(19, 41, 62));
    // §3.2: "Palette panel background | rgb(10,29,41)".
    assert_eq!(
        colors::PALETTE_PANEL_BACKGROUND.value,
        Rgb8::new(10, 29, 41)
    );
    // §3.2: "Active/selected — session picker row | blue fill p50 rgb(68,156,255)".
    assert_eq!(
        colors::SELECTION_SESSION_PICKER_ROW.value,
        Rgb8::new(68, 156, 255)
    );
    // design-tokens.md §3.1 (Geometry) line 127: "Pane card border / focused
    // | 2–3 px, rgb ≈ (31,82,115)"; restated in §5 (Reconciliation) line 201.
    assert_eq!(
        colors::PANE_CARD_BORDER_FOCUSED.value,
        Rgb8::new(31, 82, 115)
    );
    // design-tokens.md §3.1 (Geometry) line 128: "Pane card border / unfocused
    // | 2–3 px, rgb ≈ (17,54,71)"; restated in §5 (Reconciliation) line 201.
    assert_eq!(
        colors::PANE_CARD_BORDER_UNFOCUSED.value,
        Rgb8::new(17, 54, 71)
    );
}

#[test]
fn rgb_packs_to_0x_rrggbb() {
    assert_eq!(Rgb8::new(4, 27, 46).to_packed_rgb_u32(), 0x04_1B_2E);
    assert_eq!(Rgb8::new(255, 255, 255).to_packed_rgb_u32(), 0xFF_FF_FF);
    assert_eq!(Rgb8::new(0, 0, 0).to_packed_rgb_u32(), 0x00_00_00);
    assert_eq!(Rgb8::new(191, 219, 228).to_packed_rgb_u32(), 0xBF_DB_E4);
}

#[test]
fn every_geometry_approx_is_ordered() {
    for (label, approx) in all_geometry() {
        assert!(
            approx.is_ordered(),
            "{label}: Approx must satisfy min <= max (got {approx:?})"
        );
        assert!(approx.min.is_finite() && approx.max.is_finite());
    }
}

#[test]
fn documented_ranges_are_not_collapsed() {
    // The document explicitly gives these as ranges; they must stay ranges.
    assert_eq!(v4::INTER_PANE_GAP_PX.value, Approx::range(8.0, 11.0));
    assert_eq!(v1::TERMINAL_ROW_PITCH_PX.value, Approx::range(17.0, 17.5));
    assert_eq!(
        v1::TAB_PILL_CORNER_RADIUS_PX.value,
        Approx::range(8.0, 10.0)
    );
    assert_eq!(
        v4::COMMAND_PALETTE_ROW_PITCH_PX.value,
        Approx::range(36.0, 38.0)
    );
    assert!(v4::INTER_PANE_GAP_PX.value.span() > 0.0);
}

#[test]
fn every_measured_token_has_non_empty_source_and_caveat() {
    let all = all_provenance();
    assert!(
        all.len() >= 55,
        "expected the full token set, got {}",
        all.len()
    );
    for (label, source, caveat) in all {
        assert!(
            !source.trim().is_empty(),
            "{label}: source must be non-empty"
        );
        assert!(
            !caveat.trim().is_empty(),
            "{label}: caveat must be non-empty"
        );
    }
}

#[test]
fn confidence_labels_match_document_vocabulary() {
    assert_eq!(Confidence::High.label(), "high");
    assert_eq!(Confidence::MediumHigh.label(), "medium-high");
    assert_eq!(Confidence::Medium.label(), "medium");
    assert_eq!(Confidence::LowMedium.label(), "low-medium");
    assert_eq!(Confidence::Low.label(), "low");
    // Spot-check tokens against the document's own labels.
    assert_eq!(v4::INTER_PANE_GAP_PX.confidence, Confidence::High);
    assert_eq!(v1::TAB_PILL_CORNER_RADIUS_PX.confidence, Confidence::Low);
    assert_eq!(
        terminal::FOCUSED_PANE_SURFACE_V4.confidence,
        Confidence::MediumHigh
    );
    assert_eq!(colors::SECONDARY_TEXT.confidence, Confidence::LowMedium);
}

#[test]
fn unmeasured_categories_match_the_document() {
    let categories: Vec<&str> = unmeasured::UNMEASURED.iter().map(|u| u.category).collect();
    assert_eq!(
        categories.len(),
        10,
        "document §4 lists ten not-measurable items"
    );
    for expected in [
        "font_family_and_weight",
        "exact_type_sizes",
        "animation_interaction_timings",
        "pane_header_height_v1",
        "v4_window_vertical_bounds",
        "pixel_exact_corner_radii",
        "terminal_colour_under_translucency",
        "scrollbar_cursor_selection_scrollback",
        "video5_cli_text",
        "traffic_light_adjacent_toolbar_button",
    ] {
        assert!(
            categories.contains(&expected),
            "missing unmeasured category: {expected}"
        );
    }
    for item in unmeasured::UNMEASURED {
        assert!(
            !item.document_wording.trim().is_empty(),
            "{}: document wording must be non-empty",
            item.category
        );
    }
}

#[test]
fn not_app_tokens_are_present_and_reasoned() {
    // macOS traffic lights must carry the "not app tokens" reason verbatim.
    assert_eq!(
        not_app_tokens::MACOS_TRAFFIC_LIGHTS.confidence,
        Confidence::High
    );
    assert!(not_app_tokens::MACOS_TRAFFIC_LIGHTS
        .caveat
        .contains("must not copy"));
    assert!(not_app_tokens::ACCENT_TERMINAL_GLYPH_RANGE
        .caveat
        .contains("not a safe token"));
}

#[test]
fn terminal_background_is_a_ramp_not_one_value() {
    let ramp = terminal::MAIN_TERMINAL_BACKGROUND_RAMP_V1.value;
    assert_eq!(ramp.top, Rgb8::new(10, 29, 43));
    assert_eq!(ramp.bottom, Rgb8::new(17, 27, 35));
    assert_ne!(
        ramp.top, ramp.bottom,
        "the document gives two endpoints; do not collapse"
    );
}
