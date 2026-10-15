//! Named/indexed → RGB resolution for the neutral model.
//!
//! `alacritty_terminal 0.26.0` exposes `Term::colors() -> &Colors`, but the
//! palette array is filled by the *frontend* from user config; a bare
//! `Term::new()` leaves every slot `None` (verified: `Colors::default()` is
//! `[None; COUNT]`, `src/term/color.rs:22`). The engine therefore does **not**
//! resolve named/indexed colours by itself.
//!
//! The values below are the standard xterm 256-colour palette. They were **not**
//! read from the pinned crate source, so any consumer that uses them must treat
//! them as [`PALETTE_STATUS`] = `UNVERIFIED`. The phase-B checks assert the
//! *parsed* [`crate::model::Color`] values (which are exact) and record palette
//! resolution as UNVERIFIED; this helper exists for the future UI-001 renderer.

/// Provenance marker for the table below.
pub const PALETTE_STATUS: &str =
    "UNVERIFIED: xterm-standard 256-colour values, not read from the pinned crate source";

/// The 16 base ANSI colours (xterm defaults).
const BASE: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (205, 0, 0),
    (0, 205, 0),
    (205, 205, 0),
    (0, 0, 238),
    (205, 0, 205),
    (0, 205, 205),
    (229, 229, 229),
    (127, 127, 127),
    (255, 0, 0),
    (0, 255, 0),
    (255, 255, 0),
    (92, 92, 255),
    (255, 0, 255),
    (0, 255, 255),
    (255, 255, 255),
];

/// Resolve a palette index to xterm RGB, or `None` if the value is not a
/// standard xterm colour (it always is for `u8`, so this only fails for
/// completeness).
#[must_use]
pub fn xterm256_rgb(index: u8) -> Option<(u8, u8, u8)> {
    let i = index as usize;
    if i < 16 {
        return Some(BASE[i]);
    }
    if i < 232 {
        let n = i - 16;
        let r = (n / 36) % 6;
        let g = (n / 6) % 6;
        let b = n % 6;
        let chan = |c: usize| -> u8 {
            if c == 0 {
                0
            } else {
                (55 + 40 * c) as u8
            }
        };
        return Some((chan(r), chan(g), chan(b)));
    }
    let level = 8 + 10 * (i - 232) as u16;
    Some((level as u8, level as u8, level as u8))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_and_cube_and_gray() {
        assert_eq!(xterm256_rgb(0), Some((0, 0, 0)));
        assert_eq!(xterm256_rgb(15), Some((255, 255, 255)));
        assert_eq!(xterm256_rgb(16), Some((0, 0, 0)));
        assert_eq!(xterm256_rgb(196), Some((255, 0, 0)));
        assert_eq!(xterm256_rgb(21), Some((0, 0, 255)));
        assert_eq!(xterm256_rgb(232), Some((8, 8, 8)));
        assert_eq!(xterm256_rgb(255), Some((238, 238, 238)));
    }
}
