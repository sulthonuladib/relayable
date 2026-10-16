/// One unmeasured category plus the document's wording about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unmeasured {
    /// Stable identifier (snake_case), for tests and cross-references.
    pub category: &'static str,
    /// The document's own wording (`docs/research/design-tokens.md` §4 or
    /// the relevant §3 row).
    pub document_wording: &'static str,
}

/// Font family and weight — cannot be inferred from pixels.
pub const FONT_FAMILY_AND_WEIGHT: Unmeasured = Unmeasured {
    category: "font_family_and_weight",
    document_wording: "Font family and weight — cannot be inferred from pixels; no frame resolves the terminal font.",
};

/// Exact type sizes.
pub const EXACT_TYPE_SIZES: Unmeasured = Unmeasured {
    category: "exact_type_sizes",
    document_wording: "Exact type sizes — only glyph pixel extents are measurable (Neovim digits ≈ 9 px tall at V1's 1170-px frame; V4 prompt glyph ≈ 12–13 px cap height in the 2× crop). There is no reliable px→pt conversion (see §2.3), and line-height alone does not determine type size.",
};

/// Animation/interaction timings.
pub const ANIMATION_INTERACTION_TIMINGS: Unmeasured = Unmeasured {
    category: "animation_interaction_timings",
    document_wording: "Animation/interaction timings — no motion analysis was done; hover-only appearances were never captured (catalogued as not observed in `ui-catalog.md`), so hover colours/timings are `not measurable`.",
};

/// Pane header height in V1.
pub const PANE_HEADER_HEIGHT_V1: Unmeasured = Unmeasured {
    category: "pane_header_height_v1",
    document_wording: "Pane header height in V1 — no colour boundary between header and terminal body (§3.1). The V4 measurement (40–42 px) is the evidence to use.",
};

/// V4 window vertical bounds.
pub const V4_WINDOW_VERTICAL_BOUNDS: Unmeasured = Unmeasured {
    category: "v4_window_vertical_bounds",
    document_wording: "V4 window vertical bounds — window bottom edge not measured this pass (presenter PiP region and rounded corners); only width and header-region height were measured.",
};

/// Pixel-exact corner radii.
pub const PIXEL_EXACT_CORNER_RADII: Unmeasured = Unmeasured {
    category: "pixel_exact_corner_radii",
    document_wording: "Pixel-exact corner radii for window, cards, pills, and popups — only visual estimates (section 3.1 lists the pill and card figures with low/medium confidence); sub-pixel antialiasing over compressed video does not support better.",
};

/// Terminal colour under translucency.
pub const TERMINAL_COLOUR_UNDER_TRANSLUCENCY: Unmeasured = Unmeasured {
    category: "terminal_colour_under_translucency",
    document_wording: "Exact terminal colour values under translucency — the measured ramp may be partly the desktop wallpaper showing through; whether the app's terminal is translucent cannot be decided from a single recording.",
};

/// Scrollbar width/colour, cursor style, selection colour, scrollback state.
pub const SCROLLBAR_CURSOR_SELECTION: Unmeasured = Unmeasured {
    category: "scrollbar_cursor_selection_scrollback",
    document_wording: "Scrollbar width/colour, cursor style, selection colour, scrollback loading state — not measured from the new stills (scrollbar exists in V1 2:03 but was not scanned here).",
};

/// Video 5 CLI text.
pub const VIDEO5_CLI_TEXT: Unmeasured = Unmeasured {
    category: "video5_cli_text",
    document_wording: "Video 5 CLI text — not re-captured in this pass; no new 1080p measurements for `cli-*` entries.",
};

/// Traffic-light-adjacent toolbar button.
pub const TRAFFIC_LIGHT_ADJACENT_TOOLBAR_BUTTON: Unmeasured = Unmeasured {
    category: "traffic_light_adjacent_toolbar_button",
    document_wording: "Traffic-light-adjacent toolbar button — a macOS chrome item (seen at V1 0:47 left of the tabs) has no confirmed role; not a Relayable token.",
};

/// Every unmeasured category, in document order.
pub const UNMEASURED: &[Unmeasured] = &[
    FONT_FAMILY_AND_WEIGHT,
    EXACT_TYPE_SIZES,
    ANIMATION_INTERACTION_TIMINGS,
    PANE_HEADER_HEIGHT_V1,
    V4_WINDOW_VERTICAL_BOUNDS,
    PIXEL_EXACT_CORNER_RADII,
    TERMINAL_COLOUR_UNDER_TRANSLUCENCY,
    SCROLLBAR_CURSOR_SELECTION,
    VIDEO5_CLI_TEXT,
    TRAFFIC_LIGHT_ADJACENT_TOOLBAR_BUTTON,
];
