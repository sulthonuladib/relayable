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
