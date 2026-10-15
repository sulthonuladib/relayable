//! Relayable Linux app library crate (`relayable-linux`).
//!
//! This crate holds the reusable, framework-independent parts of the Linux
//! desktop app. `full.md` §5.3 requires GPUI-specific code to live only in
//! `apps/linux` and forbids leaking GPUI types into shared crates; the same
//! discipline applies *within* this crate, so the library target is kept
//! **GPUI-free** and only the `relayable` binary (`src/main.rs`) depends on GPUI.
//!
//! Current contents:
//! - [`tokens`] — the design-token sheet measured from the reference videos.
//!
//! UI-002 (the reference workspace UI) is future work; UI-001 is only the
//! minimal window proof plus this token sheet.

pub mod tokens;
