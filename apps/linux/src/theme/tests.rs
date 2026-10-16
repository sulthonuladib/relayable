use super::*;

/// `(label, packed, channels, token)` for one theme colour.
type ColourPairing = (&'static str, u32, (u8, u8, u8), Rgb8);

/// Every theme colour paired with the token it must equal.
fn pairs() -> Vec<ColourPairing> {
    vec![
        (
            "TOP_BAR_BG",
            TOP_BAR_BG,
            TOP_BAR_BG_RGB,
            colors::TOP_BAR_BACKGROUND.value,
        ),
        (
            "TOP_BAR_SEPARATOR",
            TOP_BAR_SEPARATOR,
            TOP_BAR_SEPARATOR_RGB,
            colors::TOP_BAR_SEPARATOR.value,
        ),
        (
            "ACTIVE_TAB_PILL_FILL",
            ACTIVE_TAB_PILL_FILL,
            ACTIVE_TAB_PILL_FILL_RGB,
            colors::ACTIVE_TAB_PILL_FILL.value,
        ),
        (
            "ACTIVE_TAB_PILL_EDGE_TOP",
            ACTIVE_TAB_PILL_EDGE_TOP,
            ACTIVE_TAB_PILL_EDGE_TOP_RGB,
            colors::ACTIVE_TAB_PILL_EDGE_TOP.value,
        ),
        (
            "ACTIVE_TAB_PILL_EDGE_BOTTOM",
            ACTIVE_TAB_PILL_EDGE_BOTTOM,
            ACTIVE_TAB_PILL_EDGE_BOTTOM_RGB,
            colors::ACTIVE_TAB_PILL_EDGE_BOTTOM.value,
        ),
        (
            "INACTIVE_TAB_DIVIDER",
            INACTIVE_TAB_DIVIDER,
            INACTIVE_TAB_DIVIDER_RGB,
            colors::TAB_DIVIDER_INACTIVE.value,
        ),
        (
            "PANE_HEADER_SURFACE_V1",
            PANE_HEADER_SURFACE_V1,
            PANE_HEADER_SURFACE_V1_RGB,
            colors::PANE_HEADER_SURFACE_V1.value,
        ),
        (
            "TERMINAL_BG_TOP_V1",
            TERMINAL_BG_TOP_V1,
            TERMINAL_BG_TOP_V1_RGB,
            terminal::MAIN_TERMINAL_BACKGROUND_RAMP_V1.value.top,
        ),
        (
            "TERMINAL_BG_BOTTOM_V1",
            TERMINAL_BG_BOTTOM_V1,
            TERMINAL_BG_BOTTOM_V1_RGB,
            terminal::MAIN_TERMINAL_BACKGROUND_RAMP_V1.value.bottom,
        ),
        (
            "FOCUSED_PANE_HEADER_V4",
            FOCUSED_PANE_HEADER_V4,
            FOCUSED_PANE_HEADER_V4_RGB,
            terminal::FOCUSED_PANE_SURFACE_V4.value.header,
        ),
        (
            "FOCUSED_PANE_BODY_TOP_V4",
            FOCUSED_PANE_BODY_TOP_V4,
            FOCUSED_PANE_BODY_TOP_V4_RGB,
            terminal::FOCUSED_PANE_SURFACE_V4.value.body.top,
        ),
        (
            "FOCUSED_PANE_BODY_BOTTOM_V4",
            FOCUSED_PANE_BODY_BOTTOM_V4,
            FOCUSED_PANE_BODY_BOTTOM_V4_RGB,
            terminal::FOCUSED_PANE_SURFACE_V4.value.body.bottom,
        ),
        (
            "UNFOCUSED_PANE_HEADER_V4",
            UNFOCUSED_PANE_HEADER_V4,
            UNFOCUSED_PANE_HEADER_V4_RGB,
            terminal::UNFOCUSED_PANE_SURFACE_V4.value.header,
        ),
        (
            "UNFOCUSED_PANE_BODY_TOP_V4",
            UNFOCUSED_PANE_BODY_TOP_V4,
            UNFOCUSED_PANE_BODY_TOP_V4_RGB,
            terminal::UNFOCUSED_PANE_SURFACE_V4.value.body.top,
        ),
        (
            "UNFOCUSED_PANE_BODY_BOTTOM_V4",
            UNFOCUSED_PANE_BODY_BOTTOM_V4,
            UNFOCUSED_PANE_BODY_BOTTOM_V4_RGB,
            terminal::UNFOCUSED_PANE_SURFACE_V4.value.body.bottom,
        ),
        (
            "PANE_BORDER_FOCUSED_V4",
            PANE_BORDER_FOCUSED_V4,
            PANE_BORDER_FOCUSED_V4_RGB,
            colors::PANE_CARD_BORDER_FOCUSED.value,
        ),
        (
            "PANE_BORDER_UNFOCUSED_V4",
            PANE_BORDER_UNFOCUSED_V4,
            PANE_BORDER_UNFOCUSED_V4_RGB,
            colors::PANE_CARD_BORDER_UNFOCUSED.value,
        ),
        (
            "PRIMARY_TEXT",
            PRIMARY_TEXT,
            PRIMARY_TEXT_RGB,
            colors::PRIMARY_TEXT.value,
        ),
        (
            "SECONDARY_TEXT",
            SECONDARY_TEXT,
            SECONDARY_TEXT_RGB,
            colors::SECONDARY_TEXT.value,
        ),
        (
            "SESSION_PICKER_BG",
            SESSION_PICKER_BG,
            SESSION_PICKER_BG_RGB,
            colors::SESSION_PICKER_BACKGROUND.value,
        ),
        (
            "SESSION_PICKER_BORDER",
            SESSION_PICKER_BORDER,
            SESSION_PICKER_BORDER_RGB,
            colors::SESSION_PICKER_BORDER.value,
        ),
        (
            "SESSION_PICKER_SELECTED_ROW",
            SESSION_PICKER_SELECTED_ROW,
            SESSION_PICKER_SELECTED_ROW_RGB,
            colors::SELECTION_SESSION_PICKER_ROW.value,
        ),
        (
            "PALETTE_BG",
            PALETTE_BG,
            PALETTE_BG_RGB,
            colors::PALETTE_PANEL_BACKGROUND.value,
        ),
        (
            "PALETTE_SEARCH",
            PALETTE_SEARCH,
            PALETTE_SEARCH_RGB,
            colors::PALETTE_SEARCH_FIELD.value,
        ),
        (
            "PALETTE_BORDER",
            PALETTE_BORDER,
            PALETTE_BORDER_RGB,
            colors::PALETTE_BORDER.value,
        ),
        (
            "PALETTE_SELECTED_ROW",
            PALETTE_SELECTED_ROW,
            PALETTE_SELECTED_ROW_RGB,
            colors::SELECTION_COMMAND_PALETTE_ROW.value,
        ),
    ]
}

#[test]
fn every_theme_colour_equals_its_token() {
    for (name, packed, channels, token) in pairs() {
        assert_eq!(
            channels,
            (token.r, token.g, token.b),
            "{name} channels must equal the tokens.rs value"
        );
        assert_eq!(
            packed,
            rgb_u32(token.r, token.g, token.b),
            "{name} packed word must equal rgb_u32 of the tokens.rs value"
        );
        assert_eq!(
            packed,
            rgb_u32(channels.0, channels.1, channels.2),
            "{name} packed word must equal rgb_u32 of its channels"
        );
    }
}

#[test]
fn rgb_u32_packs_hex() {
    assert_eq!(rgb_u32(0x04, 0x1B, 0x2E), 0x041B2E);
    assert_eq!(TOP_BAR_BG, 0x041B2E);
}

#[test]
fn session_picker_selected_row_is_blue_and_palette_row_is_mint() {
    // The document's V1-blue vs V4-mint distinction (design-tokens §3.1).
    assert_eq!(SESSION_PICKER_SELECTED_ROW_RGB.2, 255);
    assert!(PALETTE_SELECTED_ROW_RGB.1 > PALETTE_SELECTED_ROW_RGB.2);
}

#[test]
fn font_preference_is_a_non_empty_ordered_list() {
    assert_eq!(FONT_FAMILY_PREFERENCE.len(), 4);
    assert_eq!(FONT_FAMILY_PREFERENCE[0], "JetBrains Mono");
    assert!(FONT_FAMILY_PREFERENCE.contains(&"JetBrainsMono Nerd Font"));
}

#[test]
fn relayable_original_chrome_is_clearly_separate_from_measured_tokens() {
    // The cursor colour reuses the measured low-confidence prompt value, but
    // is labelled original; the banners must not collide with measured fills.
    assert_eq!(CURSOR_FILL_ORIGINAL, rgb_u32(125, 128, 207));
    for banner in [
        CONNECTING_BG_ORIGINAL,
        RECONNECTING_BG_ORIGINAL,
        DISCONNECTED_BG_ORIGINAL,
        ERROR_BG_ORIGINAL,
        MOCK_BANNER_BG_ORIGINAL,
    ] {
        assert_ne!(banner, TOP_BAR_BG);
        assert_ne!(banner, ACTIVE_TAB_PILL_FILL);
    }
}

#[test]
fn relayable_original_rendering_tuning_stays_in_valid_ranges() {
    const { assert!(TERMINAL_RAMP_BANDS_ORIGINAL >= 2) };
    assert!((0.0..=1.0).contains(&SCROLLBAR_TRACK_ALPHA_ORIGINAL));
    assert!((0.0..=1.0).contains(&SCROLLBAR_THUMB_ALPHA_ORIGINAL));
    const { assert!(SCROLLBAR_THUMB_ALPHA_ORIGINAL > SCROLLBAR_TRACK_ALPHA_ORIGINAL) };
    const { assert!(SCROLLBAR_THUMB_MIN_LENGTH_ORIGINAL > 0.0) };
    const { assert!(UNDERLINE_THICKNESS_ORIGINAL > 0.0) };
    assert!((0.0..=1.0).contains(&DIM_OPACITY_ORIGINAL));
}
