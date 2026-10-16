//! UI-002 reference-workspace view: the GPUI root entity for `apps/linux`.
//!
//! This is the committed Stage-B implementation of the reference workspace. It
//! renders the measured V1/V4 chrome (top bar, horizontal tab strip, pane
//! headers, split panes, terminal bodies) against **real** data from a live
//! `relayable-server` through [`crate::api`], with an explicitly labelled mock
//! for offline work.
//!
//! # State model vs. the wire model
//!
//! CORE-002 has **no `block.create`** and `session.create` returns exactly one
//! `{session, window, block}`. A tab is therefore the terminal block of one
//! session, and a split adds a **second session** (its own block) to the same
//! tab. This is a wire-model limitation, not a UI choice; it is recorded in the
//! UI-002 evidence notes.
//!
//! The bundled [`ApiHandle`] keeps every pane's block attached as its own live
//! writer stream, so panes run independently and none freezes when focus moves.
//! Focus is purely local visual state. The server permits up to
//! `max_concurrent_streams` (default 8) streams per connection, which is why the
//! pane cap is 8; the advertised value is logged from the handshake.
//!
//! Attach replies are correlated with the pane that requested them through a
//! FIFO queue of awaiting pane ids, so a focus change cannot bind a snapshot to
//! the wrong pane.
//!
//! # Threading
//!
//! `ApiHandle` is `Send + Sync` and lives in the entity. A foreground task
//! drains events every 16 ms and calls `cx.notify()` only when something
//! changed; the socket threads never touch the app (the `AsyncApp` is not
//! `Send`).

use std::collections::{HashMap, VecDeque};

use crate::api::{ApiHandle, ConnectionState, SessionSummaryView};
use gpui::{FocusHandle, Font, Pixels};

mod events;
mod helpers;
mod keys;
mod lifecycle;
mod panes;
mod picker;
mod render;
mod resize;
/// The workspace keymap, shared by the handler, the overlay and `--help`.
///
/// `pub` because the `relayable` binary is a separate crate and prints
/// [`shortcuts::shortcuts`] in `--help`; every predicate it holds stays
/// internal to this module.
pub mod shortcuts;
mod state;
mod streams;
mod tabs;
mod top_bar;

use state::{CreateIntent, Tab};

#[cfg(test)]
mod tests;

/// The root workspace view.
pub struct WorkspaceView {
    api: ApiHandle,
    is_mock: bool,
    connection: ConnectionState,
    tabs: Vec<Tab>,
    active_tab: usize,
    next_pane_id: u64,
    sessions: Vec<SessionSummaryView>,
    pending_create: Option<CreateIntent>,
    /// Pane ids awaiting an `Attached` reply, in request order (FIFO). The reply
    /// is bound to the front of this queue, never to "whatever is focused".
    pending_attach: VecDeque<u64>,
    /// Stream of the most recent `Output`/`Attached`; used to attribute the
    /// stream-less `PaneTitleChanged` event to the pane it belongs to.
    last_title_stream: Option<u64>,
    /// Cumulative output bytes observed per stream id (diagnostic/evidence only).
    stream_output_bytes: HashMap<u64, u64>,
    /// Recent server/status messages (newest last).
    status: Vec<String>,
    font: Font,
    font_size: Pixels,
    cell_width: f32,
    row_pitch: f32,
    home: String,
    window_width: f32,
    window_height: f32,
    focus_handle: FocusHandle,
    /// Session picker overlay.
    picker_open: bool,
    picker_query: String,
    picker_selected: usize,
    /// Shortcut reference overlay (`Ctrl+Shift+?`). Mutually exclusive with the
    /// picker; see `shortcuts.rs`.
    shortcuts_open: bool,
    /// Zoomed pane (tab-local) renders alone.
    zoomed: bool,
    /// Whether the real window geometry has been logged once.
    logged_geometry: bool,
}
