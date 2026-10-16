use super::{Approx, Confidence, Measured};

/// V1 geometry, measured at the **1170×1080** frame scale (`v1-t0024.png`,
/// `v1-t0047.png`, `v1-t0140.png`).
pub mod v1 {
    use super::{Approx, Confidence, Measured};

    /// Window outer bounds, x: ≈35–1134 px (`≈1099 px` wide).
    pub const WINDOW_OUTER_X_PX: Measured<Approx> = Measured::new(
        Approx::range(35.0, 1134.0),
        "V1 0:24 `v1-t0024.png`",
        Confidence::High,
        "macOS rounded corners + shadow blur ±1–2 px at the corners",
    );

    /// Window outer bounds, y: ≈29–735 px (`≈706 px` tall).
    pub const WINDOW_OUTER_Y_PX: Measured<Approx> = Measured::new(
        Approx::range(29.0, 735.0),
        "V1 0:24 `v1-t0024.png`",
        Confidence::High,
        "macOS rounded corners + shadow blur ±1–2 px at the corners",
    );

    /// Window outer aspect: ≈1.56 (14:9).
    pub const WINDOW_OUTER_ASPECT: Measured<Approx> = Measured::new(
        Approx::point(1.56),
        "V1 0:24 `v1-t0024.png`",
        Confidence::Medium,
        "video-frame aspect is 1170/1080; the window is not the frame",
    );

    /// Window inner content, x: ≈43–1124 px (`≈1081 px`).
    pub const WINDOW_INNER_X_PX: Measured<Approx> = Measured::new(
        Approx::range(43.0, 1124.0),
        "V1 0:24 `v1-t0024.png`",
        Confidence::Medium,
        "1–2 px border ambiguity",
    );

    /// Window inner content, y: ≈32–728 px (`≈697 px`).
    pub const WINDOW_INNER_Y_PX: Measured<Approx> = Measured::new(
        Approx::range(32.0, 728.0),
        "V1 0:24 `v1-t0024.png`",
        Confidence::Medium,
        "1–2 px border ambiguity",
    );

    /// Window inner content aspect: ≈1.55.
    pub const WINDOW_INNER_ASPECT: Measured<Approx> = Measured::new(
        Approx::point(1.55),
        "V1 0:24 `v1-t0024.png`",
        Confidence::Medium,
        "1–2 px border ambiguity",
    );

    /// Top bar height: 40 px (y 32→72).
    pub const TOP_BAR_HEIGHT_PX: Measured<Approx> = Measured::new(
        Approx::point(40.0),
        "V1 0:24/0:47 `v1-t0024.png`, `v1-t0047.png`",
        Confidence::High,
        "measured at 1170-px frame scale",
    );

    /// Top-bar separator thickness: 1–2 px at y 72–73.
    pub const TOP_BAR_SEPARATOR_PX: Measured<Approx> = Measured::new(
        Approx::range(1.0, 2.0),
        "V1 0:24/0:47 `v1-t0024.png`, `v1-t0047.png`",
        Confidence::High,
        "measured at 1170-px frame scale",
    );

    /// Active tab pill width: 205 px (x ≈573–778).
    pub const ACTIVE_TAB_PILL_WIDTH_PX: Measured<Approx> = Measured::new(
        Approx::point(205.0),
        "V1 0:47 `v1-t0047.png`",
        Confidence::High,
        "one frame only; pill width depends on tab count/label",
    );

    /// Active tab pill height: 30 px (y ≈36–66).
    pub const ACTIVE_TAB_PILL_HEIGHT_PX: Measured<Approx> = Measured::new(
        Approx::point(30.0),
        "V1 0:47 `v1-t0047.png`",
        Confidence::High,
        "one frame only; pill width depends on tab count/label",
    );

    /// Tab pill corner radius: ≈8–10 px (visual).
    pub const TAB_PILL_CORNER_RADIUS_PX: Measured<Approx> = Measured::new(
        Approx::range(8.0, 10.0),
        "V1 0:47 `v1-t0047.png`, crop `v1-0047-tabstrip.png`",
        Confidence::Low,
        "compression softens corners",
    );

    /// Inactive-tab vertical divider thickness: 1–2 px.
    pub const INACTIVE_TAB_DIVIDER_PX: Measured<Approx> = Measured::new(
        Approx::range(1.0, 2.0),
        "V1 0:47 `v1-t0047.png`",
        Confidence::MediumHigh,
        "divider may be a per-item separator; not verified for >2 tabs",
    );

    /// Session picker popup width: ≈183 px (x 117–300).
    pub const SESSION_PICKER_WIDTH_PX: Measured<Approx> = Measured::new(
        Approx::point(183.0),
        "V1 1:40 `v1-t0140.png`",
        Confidence::Medium,
        "bottom edge soft (shadow)",
    );

    /// Session picker popup height: ≈137–142 px (y ≈68–210).
    pub const SESSION_PICKER_HEIGHT_PX: Measured<Approx> = Measured::new(
        Approx::range(137.0, 142.0),
        "V1 1:40 `v1-t0140.png`",
        Confidence::Medium,
        "bottom edge soft (shadow)",
    );

    /// Session picker search field height: ≈27–30 px.
    pub const SESSION_PICKER_SEARCH_FIELD_PX: Measured<Approx> = Measured::new(
        Approx::range(27.0, 30.0),
        "V1 1:40 `v1-t0140.png`",
        Confidence::Medium,
        "cursor position changes row contents frame-to-frame",
    );

    /// Session picker list row height: ≈35 px.
    pub const SESSION_PICKER_LIST_ROW_PX: Measured<Approx> = Measured::new(
        Approx::point(35.0),
        "V1 1:40 `v1-t0140.png`",
        Confidence::Medium,
        "cursor position changes row contents frame-to-frame",
    );

    /// Session picker row pitch: ≈35–40 px.
    pub const SESSION_PICKER_ROW_PITCH_PX: Measured<Approx> = Measured::new(
        Approx::range(35.0, 40.0),
        "V1 1:40 `v1-t0140.png`",
        Confidence::Medium,
        "cursor position changes row contents frame-to-frame",
    );

    /// Terminal row pitch (cell height): ≈17.0–17.5 px (mean ≈17.25,
    /// 49 rows sampled — the mean is the document's, not computed here).
    pub const TERMINAL_ROW_PITCH_PX: Measured<Approx> = Measured::new(
        Approx::range(17.0, 17.5),
        "V1 1:40 `v1-t0140.png`",
        Confidence::High,
        "native terminal grid, stable across 40+ rows",
    );
}

/// V4 geometry, measured at the **1624×1080** frame scale (`v4-t0058.png`,
/// `v4-t0030.png`).
pub mod v4 {
    use super::{Approx, Confidence, Measured};

    /// Window width: x ≈34–1590 px (`≈1556 px`).
    pub const WINDOW_WIDTH_X_PX: Measured<Approx> = Measured::new(
        Approx::range(34.0, 1590.0),
        "V4 0:58 `v4-t0058.png`",
        Confidence::Medium,
        "right edge over wallpaper; bottom edge not measured (presenter PiP + rounded corner)",
    );

    /// Header region height: ≈57–61 px.
    pub const HEADER_REGION_PX: Measured<Approx> = Measured::new(
        Approx::range(57.0, 61.0),
        "V4 0:58 `v4-t0058.png`",
        Confidence::Medium,
        "cannot tell bar height vs bar+outer padding without a V1-equivalent card",
    );

    /// Pane header height: ≈40–42 px.
    pub const PANE_HEADER_HEIGHT_PX: Measured<Approx> = Measured::new(
        Approx::range(40.0, 42.0),
        "V4 0:58 `v4-t0058.png`, crop `v4-0058-paneheader.png`",
        Confidence::Medium,
        "no colour step at the boundary; the line is faint",
    );

    /// Pane card border (focused): 2–3 px.
    pub const PANE_CARD_BORDER_FOCUSED_PX: Measured<Approx> = Measured::new(
        Approx::range(2.0, 3.0),
        "V4 0:58 `v4-t0058.png`",
        Confidence::MediumHigh,
        "includes antialias",
    );

    /// Pane card border (unfocused): 2–3 px.
    pub const PANE_CARD_BORDER_UNFOCUSED_PX: Measured<Approx> = Measured::new(
        Approx::range(2.0, 3.0),
        "V4 0:58 `v4-t0058.png`",
        Confidence::Medium,
        "one frame",
    );

    /// Inter-pane gap: ≈8–11 px.
    pub const INTER_PANE_GAP_PX: Measured<Approx> = Measured::new(
        Approx::range(8.0, 11.0),
        "V4 0:58 `v4-t0058.png`",
        Confidence::High,
        "equals gap between card outer edges; not the drag target",
    );

    /// Outer margin, window→card, left: ≈0–5 px.
    pub const OUTER_MARGIN_LEFT_PX: Measured<Approx> = Measured::new(
        Approx::range(0.0, 5.0),
        "V4 0:58 `v4-t0058.png`",
        Confidence::Low,
        "asymmetric reading may be window-edge shading/scrollbar",
    );

    /// Outer margin, window→card, right: ≈10–12 px.
    pub const OUTER_MARGIN_RIGHT_PX: Measured<Approx> = Measured::new(
        Approx::range(10.0, 12.0),
        "V4 0:58 `v4-t0058.png`",
        Confidence::Low,
        "asymmetric reading may be window-edge shading/scrollbar",
    );

    /// Split creation: left pane ≈757 px.
    pub const SPLIT_PANE_LEFT_PX: Measured<Approx> = Measured::new(
        Approx::point(757.0),
        "V4 0:58 `v4-t0058.png`",
        Confidence::MediumHigh,
        "consistent with the 480p observation and the spoken \"split\" action; exact ratio depends on padding definition",
    );

    /// Split creation: right pane ≈767 px (≈50/50 with the left pane).
    pub const SPLIT_PANE_RIGHT_PX: Measured<Approx> = Measured::new(
        Approx::point(767.0),
        "V4 0:58 `v4-t0058.png`",
        Confidence::MediumHigh,
        "consistent with the 480p observation and the spoken \"split\" action; exact ratio depends on padding definition",
    );

    /// Top-bar→palette offset: ≈99 px.
    pub const TOP_BAR_TO_PALETTE_OFFSET_PX: Measured<Approx> = Measured::new(
        Approx::point(99.0),
        "V4 0:30 `v4-t0030.png`",
        Confidence::Medium,
        "panel float offset; moveable?",
    );

    /// Command palette width: ≈442 px.
    pub const COMMAND_PALETTE_WIDTH_PX: Measured<Approx> = Measured::new(
        Approx::point(442.0),
        "V4 0:30 `v4-t0030.png`",
        Confidence::High,
        "outer bounds incl. border",
    );

    /// Command palette height: ≈667 px.
    pub const COMMAND_PALETTE_HEIGHT_PX: Measured<Approx> = Measured::new(
        Approx::point(667.0),
        "V4 0:30 `v4-t0030.png`",
        Confidence::High,
        "outer bounds incl. border",
    );

    /// Command palette row pitch: ≈37.5 px, reported range 36.0–38.0 px.
    ///
    /// # Document conflict — recorded, not silently resolved
    ///
    /// `docs/research/design-tokens.md` is internally inconsistent about the
    /// palette-row figure:
    /// - **line 204** (reconciliation table) marks the earlier
    ///   `ui-catalog.md` estimate `"row pitch ≈29 px"` as **wrong: 37.5 px**;
    /// - **line 210** (`**GoRex values not used.**`) lists `29 px palette
    ///   rows` among *clone-derived* numbers;
    /// - **line 212** in the same paragraph gives the GoRex palette-row
    ///   value as `34 px` (`palette rows 37.5 px vs GoRex 34 px`).
    ///
    /// So the "29 px" on line 210 is attributed to GoRex there, yet line 212
    /// gives GoRex's value as 34 px and line 204 says the 29 px figure was a
    /// non-GoRex earlier estimate that is wrong. This is the open correction
    /// item **REF-004** (`docs/agent-loop/TASKS.md:31`,
    /// `docs/agent-loop/WORKLOG.md:496`). `docs/**` is Manager-owned and is
    /// **not** edited by UI-001; this module encodes only the measured video
    /// values and adopts **no** clone-derived number.
    pub const COMMAND_PALETTE_ROW_PITCH_PX: Measured<Approx> = Measured::new(
        Approx::range(36.0, 38.0),
        "V4 0:30 `v4-t0030.png`",
        Confidence::High,
        "group header `Pane` breaks the rhythm (43/34 px gaps); document states ≈37.5 px",
    );

    /// Command palette selected row height: ≈35 px.
    pub const COMMAND_PALETTE_SELECTED_ROW_PX: Measured<Approx> = Measured::new(
        Approx::point(35.0),
        "V4 0:30 `v4-t0030.png`",
        Confidence::High,
        "no per-row caveat recorded in design-tokens.md §3.1",
    );
}
