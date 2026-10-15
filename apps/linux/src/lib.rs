//! Relayable Linux app library crate (`relayable-linux`).
//!
//! This crate holds the Linux desktop app's modules. `full.md` §5.3 requires
//! GPUI-specific code to live only in `apps/linux` and forbids leaking GPUI types
//! into shared crates. Within this crate the `api`, `geometry`, `term`, `theme`
//! and `tokens` modules are framework-free, while `app` and `ui` are GPUI view
//! modules (`Cargo.toml` enables GPUI for the library target); `src/main.rs` is
//! still the only place that opens a window.
//!
//! Contents:
//! - [`api`] — the live two-plane server client (data + control) and a clearly
//!   labelled mock, exposing a non-blocking event/command API to the UI.
//! - [`geometry`] — measured reference geometry (pure data) and layout helpers.
//! - [`term`] — the headless `alacritty_terminal` 0.26.0 screen model.
//! - [`theme`] — packed `0xRRGGBB` colours mapped from [`tokens`].
//! - [`tokens`] — the design-token sheet measured from the reference videos.
//! - [`app`] / [`ui`] — the UI-002 GPUI workspace view and its terminal canvas.
//!
//! UI-001 was the minimal window proof plus the token sheet; UI-002 is the
//! reference workspace UI built on these modules.

pub mod api;
pub mod app;
pub mod geometry;
pub mod term;
pub mod theme;
pub mod tokens;
pub mod ui;
