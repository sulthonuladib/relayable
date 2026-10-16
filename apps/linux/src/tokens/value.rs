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
/// [`super::geometry`]. No unit conversion is applied.
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
    /// `src/tokens/tests.rs`'s unit tests enforce that for every token in this module.
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
