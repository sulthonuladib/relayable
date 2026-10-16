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
