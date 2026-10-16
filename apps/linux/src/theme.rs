//! Theme palette for the UI-002 workspace: a **thin, GPUI-free mapping** from
//! the measured tokens in [`crate::tokens`] to the exact values the UI paints.
//!
//! The point of this module is that no UI file re-derives a colour, and no
//! colour is invented here: every `u32` below is the packed form of a
//! `Measured<Rgb8>` constant in `crate::tokens`, which in turn cites
//! `docs/research/design-tokens.md` §3.1. Colours are exposed as
//! `0xRRGGBB` `u32` consts because gpui 0.2.2 takes `rgb(hex: u32) -> Rgba`
//! (`gpui-0.2.2/src/color.rs:14`); there is no `From<(u8, u8, u8)>`.
//!
//! The `*_RGB` companions expose the same value as `(u8, u8, u8)` for callers
//! that want channels rather than a packed word.
//!
//! # Typography is explicitly unmeasured
//!
//! `docs/research/design-tokens.md` §4 reports font family/weight and exact
//! type sizes as **not measurable**, so the three typography constants at the
//! bottom are marked **Relayable-original / non-reference** and are proposals,
//! not measurements.

use crate::tokens::{colors, terminal, Rgb8};

/// Pack three channels into `0xRRGGBB`.
#[must_use]
pub const fn rgb_u32(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Pack a token `Rgb8`.
const fn pack(color: Rgb8) -> u32 {
    rgb_u32(color.r, color.g, color.b)
}

/// A token `Rgb8` as channels.
const fn triple(color: Rgb8) -> (u8, u8, u8) {
    (color.r, color.g, color.b)
}

// --- V1 top bar -----------------------------------------------------------

/// Top bar background (`tokens::colors::TOP_BAR_BACKGROUND`).
pub const TOP_BAR_BG: u32 = pack(colors::TOP_BAR_BACKGROUND.value);
/// Top bar background channels.
pub const TOP_BAR_BG_RGB: (u8, u8, u8) = triple(colors::TOP_BAR_BACKGROUND.value);

/// Top-bar separator (`tokens::colors::TOP_BAR_SEPARATOR`).
pub const TOP_BAR_SEPARATOR: u32 = pack(colors::TOP_BAR_SEPARATOR.value);
/// Top-bar separator channels.
pub const TOP_BAR_SEPARATOR_RGB: (u8, u8, u8) = triple(colors::TOP_BAR_SEPARATOR.value);

/// Active tab pill fill (`tokens::colors::ACTIVE_TAB_PILL_FILL`).
pub const ACTIVE_TAB_PILL_FILL: u32 = pack(colors::ACTIVE_TAB_PILL_FILL.value);
/// Active tab pill fill channels.
pub const ACTIVE_TAB_PILL_FILL_RGB: (u8, u8, u8) = triple(colors::ACTIVE_TAB_PILL_FILL.value);

/// Active tab pill top edge/highlight (`tokens::colors::ACTIVE_TAB_PILL_EDGE_TOP`).
pub const ACTIVE_TAB_PILL_EDGE_TOP: u32 = pack(colors::ACTIVE_TAB_PILL_EDGE_TOP.value);
/// Active tab pill top edge/highlight channels.
pub const ACTIVE_TAB_PILL_EDGE_TOP_RGB: (u8, u8, u8) =
    triple(colors::ACTIVE_TAB_PILL_EDGE_TOP.value);

/// Active tab pill bottom edge/highlight (`tokens::colors::ACTIVE_TAB_PILL_EDGE_BOTTOM`).
pub const ACTIVE_TAB_PILL_EDGE_BOTTOM: u32 = pack(colors::ACTIVE_TAB_PILL_EDGE_BOTTOM.value);
/// Active tab pill bottom edge/highlight channels.
pub const ACTIVE_TAB_PILL_EDGE_BOTTOM_RGB: (u8, u8, u8) =
    triple(colors::ACTIVE_TAB_PILL_EDGE_BOTTOM.value);

/// Inactive tab divider (`tokens::colors::TAB_DIVIDER_INACTIVE`).
pub const INACTIVE_TAB_DIVIDER: u32 = pack(colors::TAB_DIVIDER_INACTIVE.value);
/// Inactive tab divider channels.
pub const INACTIVE_TAB_DIVIDER_RGB: (u8, u8, u8) = triple(colors::TAB_DIVIDER_INACTIVE.value);

/// V1 pane header/surface background (`tokens::colors::PANE_HEADER_SURFACE_V1`).
pub const PANE_HEADER_SURFACE_V1: u32 = pack(colors::PANE_HEADER_SURFACE_V1.value);
/// V1 pane header/surface background channels.
pub const PANE_HEADER_SURFACE_V1_RGB: (u8, u8, u8) = triple(colors::PANE_HEADER_SURFACE_V1.value);

// --- V1 terminal background ramp ------------------------------------------

/// V1 terminal background ramp, top endpoint.
pub const TERMINAL_BG_TOP_V1: u32 = pack(terminal::MAIN_TERMINAL_BACKGROUND_RAMP_V1.value.top);
/// V1 terminal background ramp, top endpoint channels.
pub const TERMINAL_BG_TOP_V1_RGB: (u8, u8, u8) =
    triple(terminal::MAIN_TERMINAL_BACKGROUND_RAMP_V1.value.top);
/// V1 terminal background ramp, lower endpoint.
pub const TERMINAL_BG_BOTTOM_V1: u32 =
    pack(terminal::MAIN_TERMINAL_BACKGROUND_RAMP_V1.value.bottom);
/// V1 terminal background ramp, lower endpoint channels.
pub const TERMINAL_BG_BOTTOM_V1_RGB: (u8, u8, u8) =
    triple(terminal::MAIN_TERMINAL_BACKGROUND_RAMP_V1.value.bottom);

// --- V4 pane surfaces + borders -------------------------------------------

/// V4 focused pane header surface.
pub const FOCUSED_PANE_HEADER_V4: u32 = pack(terminal::FOCUSED_PANE_SURFACE_V4.value.header);
/// V4 focused pane header surface channels.
pub const FOCUSED_PANE_HEADER_V4_RGB: (u8, u8, u8) =
    triple(terminal::FOCUSED_PANE_SURFACE_V4.value.header);
/// V4 focused pane body ramp, top endpoint.
pub const FOCUSED_PANE_BODY_TOP_V4: u32 = pack(terminal::FOCUSED_PANE_SURFACE_V4.value.body.top);
/// V4 focused pane body ramp, top endpoint channels.
pub const FOCUSED_PANE_BODY_TOP_V4_RGB: (u8, u8, u8) =
    triple(terminal::FOCUSED_PANE_SURFACE_V4.value.body.top);
/// V4 focused pane body ramp, lower endpoint.
pub const FOCUSED_PANE_BODY_BOTTOM_V4: u32 =
    pack(terminal::FOCUSED_PANE_SURFACE_V4.value.body.bottom);
/// V4 focused pane body ramp, lower endpoint channels.
pub const FOCUSED_PANE_BODY_BOTTOM_V4_RGB: (u8, u8, u8) =
    triple(terminal::FOCUSED_PANE_SURFACE_V4.value.body.bottom);

/// V4 unfocused pane header surface.
pub const UNFOCUSED_PANE_HEADER_V4: u32 = pack(terminal::UNFOCUSED_PANE_SURFACE_V4.value.header);
/// V4 unfocused pane header surface channels.
pub const UNFOCUSED_PANE_HEADER_V4_RGB: (u8, u8, u8) =
    triple(terminal::UNFOCUSED_PANE_SURFACE_V4.value.header);
/// V4 unfocused pane body ramp, top endpoint.
pub const UNFOCUSED_PANE_BODY_TOP_V4: u32 =
    pack(terminal::UNFOCUSED_PANE_SURFACE_V4.value.body.top);
/// V4 unfocused pane body ramp, top endpoint channels.
pub const UNFOCUSED_PANE_BODY_TOP_V4_RGB: (u8, u8, u8) =
    triple(terminal::UNFOCUSED_PANE_SURFACE_V4.value.body.top);
/// V4 unfocused pane body ramp, lower endpoint.
pub const UNFOCUSED_PANE_BODY_BOTTOM_V4: u32 =
    pack(terminal::UNFOCUSED_PANE_SURFACE_V4.value.body.bottom);
/// V4 unfocused pane body ramp, lower endpoint channels.
pub const UNFOCUSED_PANE_BODY_BOTTOM_V4_RGB: (u8, u8, u8) =
    triple(terminal::UNFOCUSED_PANE_SURFACE_V4.value.body.bottom);

/// V4 focused pane card border (`tokens::colors::PANE_CARD_BORDER_FOCUSED`).
pub const PANE_BORDER_FOCUSED_V4: u32 = pack(colors::PANE_CARD_BORDER_FOCUSED.value);
/// V4 focused pane card border channels.
pub const PANE_BORDER_FOCUSED_V4_RGB: (u8, u8, u8) = triple(colors::PANE_CARD_BORDER_FOCUSED.value);

/// V4 unfocused pane card border (`tokens::colors::PANE_CARD_BORDER_UNFOCUSED`).
pub const PANE_BORDER_UNFOCUSED_V4: u32 = pack(colors::PANE_CARD_BORDER_UNFOCUSED.value);
/// V4 unfocused pane card border channels.
pub const PANE_BORDER_UNFOCUSED_V4_RGB: (u8, u8, u8) =
    triple(colors::PANE_CARD_BORDER_UNFOCUSED.value);

// --- Text -----------------------------------------------------------------

/// Primary text (`tokens::colors::PRIMARY_TEXT`).
pub const PRIMARY_TEXT: u32 = pack(colors::PRIMARY_TEXT.value);
/// Primary text channels.
pub const PRIMARY_TEXT_RGB: (u8, u8, u8) = triple(colors::PRIMARY_TEXT.value);

/// Secondary text (`tokens::colors::SECONDARY_TEXT`).
pub const SECONDARY_TEXT: u32 = pack(colors::SECONDARY_TEXT.value);
/// Secondary text channels.
pub const SECONDARY_TEXT_RGB: (u8, u8, u8) = triple(colors::SECONDARY_TEXT.value);

// --- Session picker (V1) --------------------------------------------------

/// Session picker background (`tokens::colors::SESSION_PICKER_BACKGROUND`).
pub const SESSION_PICKER_BG: u32 = pack(colors::SESSION_PICKER_BACKGROUND.value);
/// Session picker background channels.
pub const SESSION_PICKER_BG_RGB: (u8, u8, u8) = triple(colors::SESSION_PICKER_BACKGROUND.value);

/// Session picker border (`tokens::colors::SESSION_PICKER_BORDER`).
pub const SESSION_PICKER_BORDER: u32 = pack(colors::SESSION_PICKER_BORDER.value);
/// Session picker border channels.
pub const SESSION_PICKER_BORDER_RGB: (u8, u8, u8) = triple(colors::SESSION_PICKER_BORDER.value);

/// Session picker **blue** selected row
/// (`tokens::colors::SELECTION_SESSION_PICKER_ROW`).
pub const SESSION_PICKER_SELECTED_ROW: u32 = pack(colors::SELECTION_SESSION_PICKER_ROW.value);
/// Session picker blue selected row channels.
pub const SESSION_PICKER_SELECTED_ROW_RGB: (u8, u8, u8) =
    triple(colors::SELECTION_SESSION_PICKER_ROW.value);

// --- Command palette (V4) -------------------------------------------------

/// Command palette background (`tokens::colors::PALETTE_PANEL_BACKGROUND`).
pub const PALETTE_BG: u32 = pack(colors::PALETTE_PANEL_BACKGROUND.value);
/// Command palette background channels.
pub const PALETTE_BG_RGB: (u8, u8, u8) = triple(colors::PALETTE_PANEL_BACKGROUND.value);

/// Command palette search field (`tokens::colors::PALETTE_SEARCH_FIELD`).
pub const PALETTE_SEARCH: u32 = pack(colors::PALETTE_SEARCH_FIELD.value);
/// Command palette search field channels.
pub const PALETTE_SEARCH_RGB: (u8, u8, u8) = triple(colors::PALETTE_SEARCH_FIELD.value);

/// Command palette border (`tokens::colors::PALETTE_BORDER`).
pub const PALETTE_BORDER: u32 = pack(colors::PALETTE_BORDER.value);
/// Command palette border channels.
pub const PALETTE_BORDER_RGB: (u8, u8, u8) = triple(colors::PALETTE_BORDER.value);

/// Command palette **mint** selected row
/// (`tokens::colors::SELECTION_COMMAND_PALETTE_ROW`).
pub const PALETTE_SELECTED_ROW: u32 = pack(colors::SELECTION_COMMAND_PALETTE_ROW.value);
/// Command palette mint selected row channels.
pub const PALETTE_SELECTED_ROW_RGB: (u8, u8, u8) =
    triple(colors::SELECTION_COMMAND_PALETTE_ROW.value);

// --- Typography — Relayable-original, NOT reference ------------------------

/// Preferred monospace families, in order.
///
/// **Relayable-original / non-reference.** `docs/research/design-tokens.md` §4
/// reports font family and weight as **not measurable** from the reference
/// videos (`open-questions.md` Q2), so this is a Relayable-chosen fallback, not
/// a reference match. The members were **verified installed on this host** via
/// `fc-list`; the other candidates (Fira Code, DejaVu Sans Mono, Ubuntu Mono,
/// Source Code Pro, Hack, Cascadia Mono, Iosevka) are *not* installed here and
/// would silently fall back. `fc-match monospace` resolves to
/// `JetBrainsMono Nerd Font`.
pub const FONT_FAMILY_PREFERENCE: &[&str] = &[
    "JetBrains Mono",
    "JetBrainsMono Nerd Font",
    "Noto Sans Mono",
    "Liberation Mono",
];

/// Proposed terminal font size in logical px.
///
/// **Relayable-original / non-reference**: exact type sizes are unmeasured
/// (`docs/research/design-tokens.md` §4; there is no px→logical conversion,
/// §2.3). The UI stage may replace this after its own measurement.
pub const TERMINAL_FONT_SIZE_PROPOSED: f32 = 13.0;

/// Proposed UI label font size in logical px.
///
/// **Relayable-original / non-reference**, as above.
pub const UI_LABEL_FONT_SIZE_PROPOSED: f32 = 12.0;

/// Proposed pane-header label font size in logical px.
///
/// **Relayable-original / non-reference**, as above.
pub const PANE_LABEL_FONT_SIZE_PROPOSED: f32 = 12.0;

// --- Relayable-original workspace chrome (NON-REFERENCE) -------------------
//
// Everything below is a Relayable design decision, not a measurement. The
// reference videos never showed a cursor close-up (`design-tokens.md` §4:
// "scrollbar width/colour, cursor style, selection colour"), and
// `open-questions.md` Q9 records that connecting/reconnecting/disconnected/
// error visuals were **never observed**. These constants exist so the app has
// unambiguous, clearly-labelled states instead of inventing reference tokens.
// UI-002 evidence notes the whole block as non-reference.

/// Block-cursor fill (`0xRRGGBB`).
///
/// **Relayable-original.** Cursor style is observed (block) but its colour is
/// unmeasured; this deliberately reuses the low-confidence V1 local-prompt
/// periwinkle (`colors::TERMINAL_PROMPT_V1_LOCAL`, rgb(125,128,207)) so the
/// cursor is visible against the measured background ramp without introducing a
/// new colour family. Paint it with [`CURSOR_ALPHA_ORIGINAL`] so the glyph under
/// the block stays legible.
pub const CURSOR_FILL_ORIGINAL: u32 = rgb_u32(125, 128, 207);

/// Block-cursor alpha. **Relayable-original.**
pub const CURSOR_ALPHA_ORIGINAL: f32 = 0.55;

/// Scrollbar track colour (`0xRRGGBB`); painted at low alpha.
///
/// **Relayable-original.** A native scrollbar was observed (V1 2:03) but its
/// colours were never measured. Reuses the measured primary-text value.
pub const SCROLLBAR_TRACK_ORIGINAL: u32 = PRIMARY_TEXT;

/// Scrollbar thumb colour (`0xRRGGBB`); painted at higher alpha.
///
/// **Relayable-original**, as above.
pub const SCROLLBAR_THUMB_ORIGINAL: u32 = PRIMARY_TEXT;

/// Scrollbar width in logical px. **Relayable-original** (unmeasured).
pub const SCROLLBAR_WIDTH_ORIGINAL: f32 = 4.0;

/// Pane-card corner radius in logical px.
///
/// **Relayable-original.** The V4 split cards are visibly rounded but their
/// radius is explicitly unmeasured (`design-tokens.md` §4, "pixel-exact corner
/// radii").
pub const PANE_CARD_RADIUS_ORIGINAL: f32 = 6.0;

/// Session-picker corner radius in logical px. **Relayable-original**.
pub const SESSION_PICKER_RADIUS_ORIGINAL: f32 = 6.0;

/// Connection-banner height in logical px. **Relayable-original** (Q9).
pub const CONNECTION_BANNER_HEIGHT_ORIGINAL: f32 = 26.0;

/// Mock banner height in logical px. **Relayable-original**.
pub const MOCK_BANNER_HEIGHT_ORIGINAL: f32 = 20.0;

/// Connection-banner foreground. **Relayable-original.**
pub const BANNER_FG_ORIGINAL: u32 = rgb_u32(230, 238, 243);

/// `Connecting` banner background. **Relayable-original.**
pub const CONNECTING_BG_ORIGINAL: u32 = rgb_u32(18, 48, 68);

/// `Reconnecting` banner background. **Relayable-original.**
pub const RECONNECTING_BG_ORIGINAL: u32 = rgb_u32(74, 58, 16);

/// `Disconnected` banner background. **Relayable-original.**
pub const DISCONNECTED_BG_ORIGINAL: u32 = rgb_u32(42, 42, 42);

/// `Error` banner background. **Relayable-original.**
pub const ERROR_BG_ORIGINAL: u32 = rgb_u32(74, 31, 31);

/// Mock-mode banner background ("this is synthetic data"). **Relayable-original.**
pub const MOCK_BANNER_BG_ORIGINAL: u32 = rgb_u32(138, 109, 26);

/// Mock-mode banner foreground. **Relayable-original.**
pub const MOCK_BANNER_FG_ORIGINAL: u32 = rgb_u32(26, 20, 8);

/// Empty-state title font size in logical px. **Relayable-original / non-reference.**
///
/// Exact reference type sizes are unmeasurable (`design-tokens.md` §4); this is
/// a Relayable choice for the Relayable-original empty state (Q9).
pub const EMPTY_STATE_TITLE_FONT_SIZE_ORIGINAL: f32 = 20.0;

/// Top-bar `+` new-tab glyph font size in logical px. **Relayable-original / non-reference.**
///
/// The `+` control was not present in the measured V1/V4 frames; this is a
/// Relayable affordance, so its glyph size is an app decision.
pub const NEW_TAB_GLYPH_FONT_SIZE_ORIGINAL: f32 = 18.0;

/// Number of horizontal bands used to paint the measured terminal background
/// ramp. **Relayable-original rendering detail.**
///
/// The *ramp endpoints* are measured (`TERMINAL_BG_*`, `FOCUSED_*`,
/// `UNFOCUSED_*`); the band count is a renderer choice. 48 bands is smooth at
/// any realistic pane height and costs 48 quads.
pub const TERMINAL_RAMP_BANDS_ORIGINAL: u32 = 48;

/// Scrollbar track alpha. **Relayable-original** (scrollbar colours unmeasured).
pub const SCROLLBAR_TRACK_ALPHA_ORIGINAL: f32 = 0.18;

/// Scrollbar thumb alpha. **Relayable-original** (scrollbar colours unmeasured).
pub const SCROLLBAR_THUMB_ALPHA_ORIGINAL: f32 = 0.70;

/// Minimum scrollbar thumb length in logical px. **Relayable-original.**
pub const SCROLLBAR_THUMB_MIN_LENGTH_ORIGINAL: f32 = 20.0;

/// Terminal underline thickness in logical px. **Relayable-original**
/// (text-decoration metrics were not measured).
pub const UNDERLINE_THICKNESS_ORIGINAL: f32 = 1.0;

/// Dim (`SGR 2`) text opacity. **Relayable-original** (the reference's dim
/// appearance was not measured).
pub const DIM_OPACITY_ORIGINAL: f32 = 0.6;

#[cfg(test)]
mod tests;
