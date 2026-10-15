//! Relayable reference design tokens — measured from the reference videos.
//!
//! **Single source:** `docs/research/design-tokens.md` (222 lines, read in full
//! by the UI-001 phase-2 author). Every value, source stamp, confidence label
//! and caveat below is taken from that file. **No value is invented**, and no
//! px→point conversion, cross-video comparison or other derived value is
//! computed here — the document states that no such conversion was performed
//! (`design-tokens.md` §2.3).
//!
//! ## What this module deliberately does *not* contain
//!
//! The document reports the following as `not measurable` (see [`unmeasured`]):
//! typography (font family/weight), exact type sizes, motion/interaction
//! timings, pixel-exact corner radii, V1 pane-header height, V4 window vertical
//! bounds, terminal colour under possible translucency, scrollbar/cursor/
//! selection colour, and Video 5 CLI text. None of these is filled in here.
//!
//! ## Shape
//!
//! * [`Measured<T>`] keeps provenance in the type system: value, source stamp,
//!   [`Confidence`] and caveat travel together, so a reviewer cannot lose them.
//! * [`Approx`] keeps measurement *spread* honest: values the document gives as
//!   ranges stay ranges; single measurements use [`Approx::point`].
//! * [`Rgb8::to_packed_rgb_u32`] lets the GPUI binary call `gpui::rgb(...)`
//!   without this module depending on GPUI.
//!
//! This file must stay **GPUI-free** (no `use gpui` anywhere): it is part of the
//! library target and is unit-tested without a display server.

use core::fmt;

/// Confidence vocabulary used by `docs/research/design-tokens.md` (§3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Confidence {
    /// "high" — clean boundary/fill in a legible frame, reproducible in a
    /// second frame.
    High,
    /// "medium-high".
    MediumHigh,
    /// "medium" — one frame or compression-limited.
    Medium,
    /// "low-medium".
    LowMedium,
    /// "low" — visual judgement.
    Low,
}

impl Confidence {
    /// The exact lowercase label used in the source document.
    pub const fn label(self) -> &'static str {
        match self {
            Confidence::High => "high",
            Confidence::MediumHigh => "medium-high",
            Confidence::Medium => "medium",
            Confidence::LowMedium => "low-medium",
            Confidence::Low => "low",
        }
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// A measured quantity that the document reports as a range (or as a single
/// approximate point via [`Approx::point`]).
///
/// All values are **raw video pixels of the cited frame** — see the warning on
/// [`geometry`]. No unit conversion is applied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Approx {
    /// Lower end of the reported range (inclusive).
    pub min: f32,
    /// Upper end of the reported range (inclusive).
    pub max: f32,
}

impl Approx {
    /// A reported range, e.g. `≈8–11 px`.
    pub const fn range(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    /// A single reported approximate value (range with zero span).
    pub const fn point(value: f32) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    /// `max - min`.
    pub const fn span(self) -> f32 {
        self.max - self.min
    }

    /// `true` when the range is well formed (`min <= max`).
    pub const fn is_ordered(self) -> bool {
        self.min <= self.max
    }

    /// Does `v` fall inside `[min, max]`?
    pub const fn contains(self, v: f32) -> bool {
        v >= self.min && v <= self.max
    }
}

/// A 24-bit sRGB colour with 8 bits per channel.
///
/// The GPUI binary converts this to the packed `0xRRGGBB` form that
/// `gpui::rgb` expects via [`Rgb8::to_packed_rgb_u32`], so this module never
/// imports GPUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb8 {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

impl Rgb8 {
    /// Construct a colour from its channels.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Pack the channels into the `0xRRGGBB` form expected by `gpui::rgb(..)`.
    pub const fn to_packed_rgb_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

/// A vertical colour ramp with two measured endpoints. Used where the document
/// explicitly says a single value must not be hard-coded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ramp<T> {
    /// Top endpoint.
    pub top: T,
    /// Lower endpoint.
    pub bottom: T,
}

/// A measured range between two samples, with **no direction implied**
/// (contrast [`Ramp<T>`], which is directional/vertical). Used for values the
/// document reports as a spread of samples, e.g. glyph bright-sample ranges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span<T> {
    /// Lower/first sample.
    pub low: T,
    /// Upper/second sample.
    pub high: T,
}

/// A pane surface: a flat header colour plus a body ramp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaneSurface {
    /// Header (flat) colour.
    pub header: Rgb8,
    /// Body vertical ramp.
    pub body: Ramp<Rgb8>,
}

/// A value plus its provenance. `source`, `confidence` and `caveat` are
/// mandatory so no measured token can be recorded without them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measured<T> {
    /// The measured value.
    pub value: T,
    /// Source stamp exactly as the document cites it (video + timestamp +
    /// frame), e.g. `"V1 0:24 `v1-t0024.png`"`.
    pub source: &'static str,
    /// The document's confidence label for this value.
    pub confidence: Confidence,
    /// The document's caveat text for this value (verbatim; where the document
    /// records no per-row caveat, this states that explicitly).
    pub caveat: &'static str,
}

impl<T> Measured<T> {
    /// Assemble a measured token. `source` and `caveat` must be non-empty;
    /// `src/tokens.rs`'s unit tests enforce that for every token in this module.
    pub const fn new(
        value: T,
        source: &'static str,
        confidence: Confidence,
        caveat: &'static str,
    ) -> Self {
        Self {
            value,
            source,
            confidence,
            caveat,
        }
    }
}

/// Geometry tokens, **namespaced per source video**.
///
/// # Warning — values are not comparable across sources
///
/// `design-tokens.md` §2.3: all values are raw **video pixels of the cited
/// frame**. The recordings are macOS Retina captures downscaled into 1080p
/// exports, no logical-pixel/point conversion was performed, and V1's frame is
/// 1170×1080 while V4's is 1624×1080 — so **absolute pixel values are not
/// comparable between V1 and V4**. This module therefore never mixes them; use
/// each source's values only within that source.
pub mod geometry {
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
}

/// Colour tokens (video-compression approximations; `design-tokens.md` §2.2).
pub mod colors {
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
}

/// Terminal / pane surfaces, including the ramps the document says must not be
/// collapsed to one value.
pub mod terminal {
    use super::{Confidence, Measured, PaneSurface, Ramp, Rgb8};

    /// Main terminal background (V1): a **vertical ramp** — top
    /// ≈rgb(10,29,43) → lower ≈rgb(17,27,35).
    ///
    /// The document says "do **not** hard-code one value without deciding
    /// opacity", so both endpoints are represented.
    pub const MAIN_TERMINAL_BACKGROUND_RAMP_V1: Measured<Ramp<Rgb8>> = Measured::new(
        Ramp {
            top: Rgb8::new(10, 29, 43),
            bottom: Rgb8::new(17, 27, 35),
        },
        "V1 0:24 `v1-t0024.png`",
        Confidence::Medium,
        "possibly wallpaper bleed-through (window may be translucent); do not hard-code one value without deciding opacity. V4 panes show a similar but not identical ramp",
    );

    /// Focused pane surface (V4): header rgb(0,36,53); body ramp
    /// rgb(8,34,45)→rgb(7,31,36).
    pub const FOCUSED_PANE_SURFACE_V4: Measured<PaneSurface> = Measured::new(
        PaneSurface {
            header: Rgb8::new(0, 36, 53),
            body: Ramp {
                top: Rgb8::new(8, 34, 45),
                bottom: Rgb8::new(7, 31, 36),
            },
        },
        "V4 0:58 `v4-t0058.png`",
        Confidence::MediumHigh,
        "teal-dominant; one frame (4:49/4:11 used only as qualitative cross-check)",
    );

    /// Unfocused pane surface (V4): header rgb(6,27,38); body ramp
    /// rgb(7,26,34)→rgb(9,25,28).
    pub const UNFOCUSED_PANE_SURFACE_V4: Measured<PaneSurface> = Measured::new(
        PaneSurface {
            header: Rgb8::new(6, 27, 38),
            body: Ramp {
                top: Rgb8::new(7, 26, 34),
                bottom: Rgb8::new(9, 25, 28),
            },
        },
        "V4 0:58 `v4-t0058.png`",
        Confidence::MediumHigh,
        "focus shown by ~+7 G / +8–15 B vs unfocused",
    );
}

/// Tokens the document **measures** but that must **not** be adopted as app
/// tokens. Reasons are the document's own wording.
pub mod not_app_tokens {
    use super::{Confidence, Measured, Rgb8, Span};

    /// macOS traffic-light colours: rgb(254,79,84) / (248,190,0) / (42,190,75).
    ///
    /// Reason (document, verbatim): "OS chrome; Relayable Linux must not copy
    /// these" — marked "**not app tokens**".
    pub const MACOS_TRAFFIC_LIGHTS: Measured<[Rgb8; 3]> = Measured::new(
        [
            Rgb8::new(254, 79, 84),
            Rgb8::new(248, 190, 0),
            Rgb8::new(42, 190, 75),
        ],
        "V1 0:47 `v1-t0047.png`",
        Confidence::High,
        "OS chrome; Relayable Linux must not copy these",
    );

    /// Accent — terminal glyph in tabs/header: mixed green, bright samples
    /// rgb(33,94,39)→(208,255,214).
    ///
    /// Reason (document, verbatim): "small glyphs, heavy antialias; not a safe
    /// token".
    pub const ACCENT_TERMINAL_GLYPH_RANGE: Measured<Span<Rgb8>> = Measured::new(
        Span {
            low: Rgb8::new(33, 94, 39),
            high: Rgb8::new(208, 255, 214),
        },
        "V1 0:47 `v1-t0047.png`; V1 1:40 `v1-t0140.png`",
        Confidence::Low,
        "small glyphs, heavy antialias; not a safe token",
    );

    /// Traffic-light-adjacent toolbar button: a macOS chrome item seen at
    /// V1 0:47 left of the tabs with no confirmed role — "not a Relayable
    /// token".
    pub const TRAFFIC_LIGHT_ADJACENT_TOOLBAR_BUTTON: Measured<&'static str> = Measured::new(
        "macOS chrome item left of the tabs; role unconfirmed",
        "V1 0:47 `v1-t0047.png`",
        Confidence::Low,
        "a macOS chrome item (seen at V1 0:47 left of the tabs) has no confirmed role; not a Relayable token",
    );
}

/// Categories the document explicitly reports as **not measurable**. This module
/// carries the document's own wording; nothing here is filled in with a value.
pub mod unmeasured {
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
}

#[cfg(test)]
mod tests {
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
}
