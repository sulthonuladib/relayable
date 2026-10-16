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
/// Width of the shortcut reference panel, in logical px.
///
/// **Relayable-original** (no shortcut overlay was observed in the reference;
/// `docs/research/open-questions.md` Q4). Wide enough for the longest row
/// (`Ctrl+Shift+T / Ctrl+PageDown` plus `next tab`) at
/// [`crate::theme::UI_LABEL_FONT_SIZE_PROPOSED`]; re-checked against the
/// UI-005 overlay capture.
pub const SHORTCUTS_PANEL_WIDTH_ORIGINAL: f32 = 460.0;
/// Width of the key-chord column in the shortcut reference, in logical px.
///
/// **Relayable-original.** Must fit `Ctrl+Shift+T / Ctrl+PageDown` (29 chars)
/// at the 12 px label size, so keys and labels stay in two aligned columns.
pub const SHORTCUTS_KEYS_COLUMN_WIDTH_ORIGINAL: f32 = 232.0;
/// Vertical gap between rows of the shortcut reference, in logical px.
///
/// **Relayable-original.**
pub const SHORTCUTS_ROW_GAP_ORIGINAL: f32 = 6.0;
/// Padding inside the shortcut reference panel, in logical px.
///
/// **Relayable-original.**
pub const SHORTCUTS_PADDING_ORIGINAL: f32 = 12.0;
/// Offset of the shortcut reference below the top-bar separator, in logical px.
///
/// **Relayable-original**; matches the picker's offset so the two overlays hang
/// from the same edge.
pub const SHORTCUTS_OFFSET_BELOW_SEPARATOR_ORIGINAL: f32 = 4.0;
/// Side length of the split-right/split-down icon glyph, in logical px.
pub const ICON_GLYPH_SPLIT_SIZE_ORIGINAL: f32 = 12.0;
/// Thickness of the divider inside the split icon glyph, in logical px.
pub const ICON_GLYPH_SPLIT_LINE_ORIGINAL: f32 = 1.0;
/// Side length of the zoom icon glyph, in logical px.
pub const ICON_GLYPH_ZOOM_SIZE_ORIGINAL: f32 = 11.0;
/// Font size of the `\u{d7}` close glyph, in logical px.
pub const ICON_GLYPH_CLOSE_FONT_SIZE_ORIGINAL: f32 = 14.0;
