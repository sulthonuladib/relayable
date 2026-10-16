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

mod value;

pub use value::{Approx, Confidence, Measured, PaneSurface, Ramp, Rgb8, Span};

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
pub mod geometry;

/// Colour tokens (video-compression approximations; `design-tokens.md` §2.2).
pub mod colors;

/// Terminal / pane surfaces, including the ramps the document says must not be
/// collapsed to one value.
pub mod terminal;

/// Tokens the document **measures** but that must **not** be adopted as app
/// tokens. Reasons are the document's own wording.
pub mod not_app_tokens;

/// Categories the document explicitly reports as **not measurable**. This module
/// carries the document's own wording; nothing here is filled in with a value.
pub mod unmeasured;

#[cfg(test)]
mod tests;
