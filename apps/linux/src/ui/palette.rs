//! ANSI terminal palette — **NON-REFERENCE / Relayable-original**.
//!
//! `docs/research/design-tokens.md` §4 lists *"exact terminal colour values under
//! translucency"* (and the ANSI palette generally) as **not measurable** from the
//! reference videos; `docs/research/open-questions.md` Q1 records the terminal
//! theme as unmeasured. The reference frames only ever showed a handful of
//! prompt colours, never a full palette.
//!
//! Therefore Relayable renders non-default cell colours with the **standard
//! xterm 256-colour palette** (the de-facto default used by xterm, VTE,
//! alacritty and most Linux terminals). This is a deliberate, documented
//! non-reference fallback: no reference ANSI value is invented, and the palette
//! is only consulted for cells whose colour the terminal model reports as
//! `TermColor::Indexed`/`Rgb`. Default-background cells stay transparent so the
//! measured background ramp shows through (`crate::term` guarantees `bg: None`).
//!
//! Sources for the standard palette values: the xterm `256colres.h` colour cube
//! (`16..=231`, levels `0/95/135/175/215/255`) and greyscale ramp (`232..=255`,
//! `8 + 10·n`). These are the conventional, widely reproduced values.

use gpui::{rgb, Rgba};

/// The conventional xterm 16 base colours (`0..=15`), packed `0xRRGGBB`.
pub const ANSI_16: [u32; 16] = [
    0x000000, // 0 black
    0xCD0000, // 1 red
    0x00CD00, // 2 green
    0xCDCD00, // 3 yellow
    0x0000EE, // 4 blue
    0xCD00CD, // 5 magenta
    0x00CDCD, // 6 cyan
    0xE5E5E5, // 7 white
    0x7F7F7F, // 8 bright black
    0xFF0000, // 9 bright red
    0x00FF00, // 10 bright green
    0xFFFF00, // 11 bright yellow
    0x5C5CFF, // 12 bright blue
    0xFF00FF, // 13 bright magenta
    0x00FFFF, // 14 bright cyan
    0xFFFFFF, // 15 bright white
];

/// The six levels of the xterm 6×6×6 colour cube.
const CUBE_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// Resolve a 256-colour-palette index to a concrete colour.
#[must_use]
pub fn ansi(index: u8) -> Rgba {
    match index {
        0..=15 => rgb(ANSI_16[usize::from(index)]),
        16..=231 => {
            let i = u32::from(index - 16);
            let r = CUBE_LEVELS[(i / 36) as usize];
            let g = CUBE_LEVELS[((i % 36) / 6) as usize];
            let b = CUBE_LEVELS[(i % 6) as usize];
            rgb(pack(r, g, b))
        }
        _ => {
            let v = 8u8.saturating_add(10u8.saturating_mul(index - 232));
            rgb(pack(v, v, v))
        }
    }
}

/// Pack three channels into `0xRRGGBB`.
#[must_use]
const fn pack(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `0xRRGGBB` for a resolved colour (drops alpha).
    fn hex(c: Rgba) -> u32 {
        u32::from(c) >> 8
    }

    #[test]
    fn base_colours_match_xterm() {
        assert_eq!(hex(ansi(0)), 0x000000);
        assert_eq!(hex(ansi(1)), 0xCD0000);
        assert_eq!(hex(ansi(15)), 0xFFFFFF);
    }

    #[test]
    fn cube_endpoints_are_black_and_white() {
        // 16 = (0,0,0); 231 = (255,255,255).
        assert_eq!(hex(ansi(16)), 0x000000);
        assert_eq!(hex(ansi(231)), 0xFFFFFF);
    }

    #[test]
    fn greyscale_ramp_endpoints() {
        assert_eq!(hex(ansi(232)), 0x080808);
        assert_eq!(hex(ansi(255)), 0xEEEEEE);
    }

    #[test]
    fn every_index_is_defined_and_opaque() {
        for index in 0..=255u8 {
            let c = ansi(index);
            assert!(c.a > 0.99, "index {index} must be opaque");
        }
    }
}
