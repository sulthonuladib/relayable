//! GPUI view layer for the UI-002 reference workspace.
//!
//! # Why GPUI lives in this library module
//!
//! `full.md` §5.3 requires GPUI code to live only inside `apps/linux`; it does
//! **not** require it to live only in the binary. UI-002's brief explicitly
//! assigns `src/ui/**` and `src/app.rs` as the implementation paths for the
//! reference workspace, so both modules carry GPUI types. `src/tokens.rs`,
//! `src/geometry.rs` and `src/theme.rs` remain GPUI-free data/measurement
//! modules (only `theme.rs` names `gpui`-free packed `u32`s), and the headless
//! `src/term/**` and `src/api/**` modules remain GPUI-free too. `src/main.rs`
//! is still the only place that opens a window.
//!
//! # Layout of the reference workspace
//!
//! Everything is measured or explicitly marked Relayable-original:
//!
//! * top bar (40 px) + 1 px separator — [`crate::geometry::workspace_columns`]
//! * horizontal tab strip starting at x = 128 px, 205×30 px pills, no gap
//! * one pane header (41 px, V4 inference) per pane, four icon buttons on the
//!   focused pane when a tab is split
//! * terminal body: measured V1/V4 vertical background ramps, monospace grid
//!   with a 17.25 px row pitch, block cursor and a thin scrollback scrollbar
//!
//! [`palette`] is deliberately **non-reference** (the ANSI palette is not
//! measurable from the videos); [`terminal`] paints the grid.

pub mod palette;
pub mod terminal;

use gpui::{rgb, Rgba};

/// Linear interpolation between two packed `0xRRGGBB` colours.
///
/// Used for the measured terminal background ramps (`theme::TERMINAL_BG_*`),
/// which `docs/research/design-tokens.md` §3.2 records as smooth gradients, not
/// two flat bands. `t` is clamped to `0.0..=1.0`.
#[must_use]
pub fn lerp_packed(top: u32, bottom: u32, t: f32) -> Rgba {
    let t = t.clamp(0.0, 1.0);
    let lerp = |shift: u32| {
        let a = ((top >> shift) & 0xFF) as f32;
        let b = ((bottom >> shift) & 0xFF) as f32;
        ((a + (b - a) * t).round() as u32) & 0xFF
    };
    rgb((lerp(16) << 16) | (lerp(8) << 8) | lerp(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_endpoints_are_exact() {
        assert_eq!(
            u32::from(lerp_packed(0x0A1D2B, 0x111B23, 0.0)) >> 8,
            0x0A1D2B
        );
        assert_eq!(
            u32::from(lerp_packed(0x0A1D2B, 0x111B23, 1.0)) >> 8,
            0x111B23
        );
    }

    #[test]
    fn lerp_clamps_and_is_monotonic_on_one_channel() {
        let low = u32::from(lerp_packed(0x000000, 0xFFFFFF, -5.0)) >> 8;
        let high = u32::from(lerp_packed(0x000000, 0xFFFFFF, 5.0)) >> 8;
        assert_eq!(low, 0x000000);
        assert_eq!(high, 0xFFFFFF);
        let mid = u32::from(lerp_packed(0x000000, 0xFFFFFF, 0.5)) >> 8;
        assert_eq!(mid, 0x808080);
    }
}
