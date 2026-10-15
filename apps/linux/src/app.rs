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
use std::time::Duration;

use crate::api::{ApiCommand, ApiEvent, ApiHandle, ConnectionState, SessionSummaryView};
use crate::geometry;
use crate::term::{TerminalModel, TerminalSize};
use crate::theme;
use crate::ui::terminal::TerminalCanvas;
use gpui::{
    div, prelude::*, px, rgb, AnyElement, Context, Div, ElementId, FocusHandle, Font, KeyDownEvent,
    Keystroke, MouseButton, Pixels, ScrollDelta, SharedString, Window, WindowBackgroundAppearance,
    WindowDecorations,
};

/// Drained-event poll interval (foreground task). 16 ms ≈ 60 fps.
const DRAIN_INTERVAL: Duration = Duration::from_millis(16);

/// Upper bound on events applied in one frame so a flooding server cannot make a
/// single frame unbounded; the remainder is left for the next 16 ms tick.
const MAX_EVENTS_PER_FRAME: usize = 256;

/// Maximum panes across all tabs.
///
/// This mirrors the server's advertised per-connection `max_concurrent_streams`
/// (`crates/relayable-protocol/src/limits.rs::DEFAULT_LIMITS.max_concurrent_streams`
/// = 8, enforced in `crates/relayable-server/src/dispatch.rs`); the server
/// rejects a 9th `block.attach` on one connection. It is **not** a writer-lease
/// limit: each pane is its own block/session, so there is no shared lease.
/// Reading the advertised value from the handshake at runtime would require
/// widening the frozen `ApiHandle` surface, so the default is mirrored here and
/// the advertised value is logged instead.
const MAX_PANES: usize = 8;

/// Which way a tab's panes are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SplitAxis {
    /// Panes side by side (`Split Pane Right`).
    Horizontal,
    /// Panes stacked (`Split Pane Down`).
    Vertical,
}

/// What the next `session.create` reply should build.
#[derive(Debug, Clone)]
enum CreateKind {
    /// A new tab (its own session).
    NewTab,
    /// A new pane in the current tab (a new session — no `block.create`).
    Split { axis: SplitAxis },
}

/// A pending create request.
#[derive(Debug, Clone)]
struct CreateIntent {
    kind: CreateKind,
    label: Option<String>,
}

/// Pane-header button actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaneAction {
    /// `split-right`.
    SplitRight,
    /// `split-down`.
    SplitDown,
    /// `zoom`.
    Zoom,
    /// `close ×`.
    Close,
}

/// One terminal pane: a block attached as writer.
struct Pane {
    pane_id: u64,
    session_id: String,
    block_id: String,
    /// Label requested at session creation (the only label `session.create`
    /// echoes back to the client).
    requested_label: Option<String>,
    /// Label from `session.list`.
    server_label: Option<String>,
    /// OSC 0/2 title from the live output stream.
    program: Option<String>,
    /// OSC 7 cwd from the live output stream.
    cwd: Option<String>,
    stream_id: Option<u64>,
    term: TerminalModel,
    cols: u16,
    rows: u16,
    sent_cols: u16,
    sent_rows: u16,
    history_offset: isize,
    scroll_accum: f32,
}

impl Pane {
    fn new(
        pane_id: u64,
        session_id: String,
        block_id: String,
        requested_label: Option<String>,
    ) -> Self {
        Self {
            pane_id,
            session_id,
            block_id,
            requested_label,
            server_label: None,
            program: None,
            cwd: None,
            stream_id: None,
            term: TerminalModel::new(TerminalSize::cells(80, 24)),
            cols: 80,
            rows: 24,
            sent_cols: 0,
            sent_rows: 0,
            history_offset: 0,
            scroll_accum: 0.0,
        }
    }

    fn view_snapshot(&self) -> crate::term::ScreenSnapshot {
        if self.history_offset > 0 {
            self.term.snapshot_at(self.history_offset)
        } else {
            self.term.snapshot()
        }
    }

    /// The pane-header label: live OSC program/cwd first, then labels, then a
    /// documented fallback (the reference showed `~` / `shell`).
    fn header_label(&self, home: &str) -> String {
        if let Some(program) = self.program.as_deref().filter(|p| !p.is_empty()) {
            return program.to_string();
        }
        if let Some(cwd) = self.cwd.as_deref().filter(|c| !c.is_empty()) {
            return shorten_home(cwd, home);
        }
        if let Some(label) = self.requested_label.as_deref().filter(|l| !l.is_empty()) {
            return label.to_string();
        }
        "shell".to_string()
    }

    /// The tab label: an explicitly requested session label wins, then the
    /// live program/cwd, then the server-side session label, then the fallback.
    fn tab_label(&self, home: &str) -> String {
        if let Some(label) = self.requested_label.as_deref().filter(|l| !l.is_empty()) {
            return label.to_string();
        }
        if let Some(program) = self.program.as_deref().filter(|p| !p.is_empty()) {
            return program.to_string();
        }
        if let Some(cwd) = self.cwd.as_deref().filter(|c| !c.is_empty()) {
            return shorten_home(cwd, home);
        }
        if let Some(label) = self.server_label.as_deref().filter(|l| !l.is_empty()) {
            return label.to_string();
        }
        "shell".to_string()
    }
}

/// One tab: a session's terminal block plus any split panes.
struct Tab {
    panes: Vec<Pane>,
    focus: usize,
    axis: SplitAxis,
}

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
    /// Zoomed pane (tab-local) renders alone.
    zoomed: bool,
    /// Whether the real window geometry has been logged once.
    logged_geometry: bool,
}

impl WorkspaceView {
    /// Build the workspace view around `api` and start the event drain loop.
    pub fn new(api: ApiHandle, cx: &mut Context<Self>) -> Self {
        let text_system = cx.text_system();
        let names = text_system.all_font_names();
        let family = choose_font_family(&names);
        let font = gpui::font(family.clone());
        let font_id = text_system.resolve_font(&font);
        let font_size = px(theme::TERMINAL_FONT_SIZE_PROPOSED);
        let cell_width = text_system
            .ch_advance(font_id, font_size)
            .map(f32::from)
            .unwrap_or(8.0)
            .max(1.0);
        let row_pitch = geometry::v1::TERMINAL_ROW_PITCH.value;
        let is_mock = api.is_mock();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        let focus_handle = cx.focus_handle();

        eprintln!(
            "relayable: font chosen={family:?} (of {} installed families) cell_width={cell_width:.3} row_pitch={row_pitch} font_size={}",
            names.len(),
            f32::from(font_size)
        );

        let mut view = Self {
            api,
            is_mock,
            connection: ConnectionState::Connecting,
            tabs: Vec::new(),
            active_tab: 0,
            next_pane_id: 1,
            sessions: Vec::new(),
            pending_create: None,
            pending_attach: VecDeque::new(),
            last_title_stream: None,
            stream_output_bytes: HashMap::new(),
            status: Vec::new(),
            font,
            font_size,
            cell_width,
            row_pitch,
            home,
            window_width: geometry::BASELINE_CANVAS_OUTER.0,
            window_height: geometry::BASELINE_CANVAS_OUTER.1,
            focus_handle,
            picker_open: false,
            picker_query: String::new(),
            picker_selected: 0,
            zoomed: false,
            logged_geometry: false,
        };
        view.spawn_drain(cx);
        view.register_shutdown(cx);
        view
    }

    /// Best-effort `Shutdown` so the API worker threads stop on app quit.
    fn register_shutdown(&mut self, cx: &mut Context<Self>) {
        let subscription = cx.on_app_quit(|view, _cx| {
            view.api.send(ApiCommand::Shutdown);
            async {}
        });
        subscription.detach();
    }

    /// The root keyboard focus handle (main focuses it when the window opens).
    #[must_use]
    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    fn spawn_drain(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| loop {
            cx.background_executor().timer(DRAIN_INTERVAL).await;
            let alive = this.update(cx, |view, cx| {
                // Bounded per frame: a flooding server cannot make one frame
                // unbounded. The remainder stays queued for the next tick.
                let mut events = Vec::new();
                for _ in 0..MAX_EVENTS_PER_FRAME {
                    match view.api.try_recv() {
                        Some(event) => events.push(event),
                        None => break,
                    }
                }
                if !events.is_empty() {
                    view.apply(events);
                    cx.notify();
                }
            });
            if alive.is_err() {
                break;
            }
        })
        .detach();
    }

    // --- event application -------------------------------------------------

    fn apply(&mut self, events: Vec<ApiEvent>) {
        for event in events {
            self.apply_one(event);
        }
    }

    fn apply_one(&mut self, event: ApiEvent) {
        match event {
            ApiEvent::ConnectionStateChanged(state) => {
                eprintln!("relayable-ui: connection state = {state:?}");
                self.connection = state.clone();
                if matches!(state, ConnectionState::Connected) {
                    self.api.send(ApiCommand::RefreshSessions);
                    if self.tabs.is_empty() && self.pending_create.is_none() {
                        self.pending_create = Some(CreateIntent {
                            kind: CreateKind::NewTab,
                            label: None,
                        });
                        self.api.send(ApiCommand::CreateSession { label: None });
                    }
                    // After a reconnect, existing panes have no live stream (they
                    // were invalidated on disconnect); attach them again so they
                    // re-seed from the returned snapshots.
                    self.reattach_missing_panes();
                }
            }
            ApiEvent::SessionsListed(sessions) => self.sessions = sessions,
            ApiEvent::SessionCreated {
                session_id,
                block_id,
                ..
            } => {
                eprintln!("relayable-ui: session.created session={session_id} block={block_id}");
                let intent = self.pending_create.take().unwrap_or(CreateIntent {
                    kind: CreateKind::NewTab,
                    label: None,
                });
                let pane_id = self.next_pane_id;
                self.next_pane_id += 1;
                let pane = Pane::new(pane_id, session_id.clone(), block_id, intent.label.clone());
                match intent.kind {
                    CreateKind::NewTab => {
                        self.tabs.push(Tab {
                            panes: vec![pane],
                            focus: 0,
                            axis: SplitAxis::Horizontal,
                        });
                        self.active_tab = self.tabs.len() - 1;
                    }
                    CreateKind::Split { axis } => {
                        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                            tab.axis = axis;
                            tab.panes.push(pane);
                            tab.focus = tab.panes.len() - 1;
                        }
                    }
                }
                // Match the server-side label by session id once the list lands.
                self.attach_pane(pane_id);
                self.api.send(ApiCommand::RefreshSessions);
            }
            ApiEvent::Attached {
                stream_id,
                cols,
                rows,
                screen_bytes,
            } => {
                eprintln!(
                    "relayable-ui: block.attached stream_id={stream_id} cols={cols} rows={rows} snapshot_bytes={}",
                    screen_bytes.len()
                );
                // Deterministic correlation: replies are processed in request
                // order, so the front of the queue is the pane that asked.
                let pane_id = self.pending_attach.pop_front();
                match pane_id {
                    Some(pane_id) => {
                        if let Some(pane) = self.pane_mut(pane_id) {
                            pane.stream_id = Some(stream_id);
                            pane.cols = cols;
                            pane.rows = rows;
                            pane.sent_cols = cols;
                            pane.sent_rows = rows;
                            pane.term = TerminalModel::new(TerminalSize::cells(cols, rows));
                            pane.term.process(&screen_bytes);
                            pane.history_offset = 0;
                            self.last_title_stream = Some(stream_id);
                        } else {
                            // The pane was closed while its attach was in
                            // flight. The FIFO slot was still consumed, so just
                            // release the stream it acquired.
                            eprintln!(
                                "relayable-ui: pane {pane_id} closed mid-attach; detaching stream_id={stream_id}"
                            );
                            self.api.send(ApiCommand::Detach { stream_id });
                        }
                    }
                    None => eprintln!(
                        "relayable-ui: Attached stream_id={stream_id} had no awaiting pane; ignoring"
                    ),
                }
            }
            ApiEvent::Output {
                stream_id, bytes, ..
            } => {
                self.last_title_stream = Some(stream_id);
                let total = self.stream_output_bytes.entry(stream_id).or_insert(0);
                *total += bytes.len() as u64;
                eprintln!(
                    "relayable-ui: output stream_id={stream_id} bytes={} total={total}",
                    bytes.len()
                );
                if let Some(pane) = self.pane_by_stream_mut(stream_id) {
                    pane.term.process(&bytes);
                }
            }
            ApiEvent::Resized {
                stream_id,
                cols,
                rows,
            } => {
                eprintln!(
                    "relayable-ui: terminal.resize accepted stream_id={stream_id} cols={cols} rows={rows}"
                );
                if let Some(pane) = self.pane_by_stream_mut(stream_id) {
                    pane.cols = cols;
                    pane.rows = rows;
                }
            }
            ApiEvent::PaneTitleChanged { program, cwd } => {
                eprintln!("relayable-ui: pane title/cwd program={program:?} cwd={cwd:?}");
                // `PaneTitleChanged` carries no stream id; the API emits it
                // immediately after the `Output`/`Attached` that produced it, so
                // attribute it to that stream.
                if let Some(stream_id) = self.last_title_stream {
                    if let Some(pane) = self.pane_by_stream_mut(stream_id) {
                        if program.is_some() {
                            pane.program = program;
                        }
                        if cwd.is_some() {
                            pane.cwd = cwd;
                        }
                    }
                }
            }
            ApiEvent::ServerEvent { kind, detail } => {
                // The API emits this when the data plane (re)connects. Re-attach
                // any pane whose stream was invalidated by a data-plane-only
                // disconnect, where the control plane stays connected and so no
                // `ConnectionStateChanged(Connected)` arrives.
                if kind == "data_plane_connected" {
                    // The terminal data path is usable again; clear a
                    // data-plane-only Disconnected banner so the UI does not stay
                    // stuck while panes work. The control plane re-emits its own
                    // state if it is still down.
                    if matches!(self.connection, ConnectionState::Disconnected) {
                        self.connection = ConnectionState::Connected;
                        eprintln!(
                            "relayable-ui: data plane reconnected; connection state restored to Connected"
                        );
                    }
                    self.reattach_missing_panes();
                }
                self.push_status(format!("{kind}: {detail}"));
            }
            ApiEvent::Disconnected { reason } => {
                eprintln!("relayable-ui: disconnected reason={reason}");
                self.connection = ConnectionState::Disconnected;
                self.push_status(format!("disconnected: {reason}"));
                self.invalidate_streams();
                // The connection is gone: a queued attach reply will never
                // arrive, so drop the queue to avoid a stale FIFO head.
                self.pending_attach.clear();
            }
            ApiEvent::Error { message } => {
                eprintln!("relayable-ui: error {message}");
                self.connection = ConnectionState::Error(message.clone());
                // A failed `block.attach` produced no `Attached`, so consume its
                // queue slot to keep the correlation FIFO aligned.
                if message.starts_with("block.attach") {
                    self.pending_attach.pop_front();
                }
                self.push_status(message);
                // Treat a request error like a disconnect: the stream the failed
                // request targeted may be dead, so suppress input/resize to it
                // until the next successful (re)attach.
                self.invalidate_streams();
            }
        }
    }

    fn push_status(&mut self, message: String) {
        self.status.push(message);
        if self.status.len() > 64 {
            self.status.remove(0);
        }
    }

    // --- pane/tab plumbing -------------------------------------------------

    fn pane(&self, pane_id: u64) -> Option<&Pane> {
        self.tabs
            .iter()
            .flat_map(|tab| tab.panes.iter())
            .find(|pane| pane.pane_id == pane_id)
    }

    fn pane_mut(&mut self, pane_id: u64) -> Option<&mut Pane> {
        self.tabs
            .iter_mut()
            .flat_map(|tab| tab.panes.iter_mut())
            .find(|pane| pane.pane_id == pane_id)
    }

    fn pane_by_stream_mut(&mut self, stream_id: u64) -> Option<&mut Pane> {
        self.tabs
            .iter_mut()
            .flat_map(|tab| tab.panes.iter_mut())
            .find(|pane| pane.stream_id == Some(stream_id))
    }

    fn focused_pane_id(&self) -> Option<u64> {
        let tab = self.tabs.get(self.active_tab)?;
        tab.panes.get(tab.focus).map(|pane| pane.pane_id)
    }

    fn focused_stream(&self) -> Option<u64> {
        let pane_id = self.focused_pane_id()?;
        self.pane(pane_id).and_then(|pane| pane.stream_id)
    }

    fn pane_count(&self) -> usize {
        self.tabs.iter().map(|tab| tab.panes.len()).sum()
    }

    /// Attach one pane's block as its own live writer stream.
    ///
    /// Focus does **not** detach anything: every pane keeps its stream until it
    /// is explicitly closed. `takeover: true` makes a re-attach robust against a
    /// lease still held by the previous (now dead) connection; Relayable is the
    /// only writer client for these sessions.
    fn attach_pane(&mut self, pane_id: u64) {
        let Some(block_id) = self.pane(pane_id).map(|pane| pane.block_id.clone()) else {
            return;
        };
        if self.pending_attach.contains(&pane_id) {
            return;
        }
        self.pending_attach.push_back(pane_id);
        self.api.send(ApiCommand::AttachBlock {
            block_id,
            takeover: true,
        });
    }

    /// Re-attach every pane with no live stream (used after a reconnect).
    fn reattach_missing_panes(&mut self) {
        for pane_id in panes_needing_attach(&self.tabs) {
            self.attach_pane(pane_id);
        }
    }

    /// Drop every pane's stream association. Used when the connection is lost
    /// or errored so the UI never sends input or resize to a dead stream; the
    /// next `Connected` re-attaches and re-seeds. The pending-attach queue is
    /// left alone: a fresh attach may still be in flight and its reply must
    /// still bind to the pane that requested it.
    fn invalidate_streams(&mut self) {
        self.last_title_stream = None;
        for tab in &mut self.tabs {
            for pane in &mut tab.panes {
                pane.stream_id = None;
            }
        }
    }

    fn select_tab(&mut self, index: usize) {
        if index >= self.tabs.len() || index == self.active_tab {
            return;
        }
        self.active_tab = index;
        self.zoomed = false;
    }

    fn next_tab(&mut self) {
        if self.tabs.len() < 2 {
            return;
        }
        let next = (self.active_tab + 1) % self.tabs.len();
        self.select_tab(next);
    }

    fn prev_tab(&mut self) {
        if self.tabs.len() < 2 {
            return;
        }
        let prev = (self.active_tab + self.tabs.len() - 1) % self.tabs.len();
        self.select_tab(prev);
    }

    /// Move focus. Purely local: the newly focused pane is already attached, so
    /// no attach/detach is issued here.
    fn focus_pane(&mut self, tab_index: usize, pane_index: usize) {
        if let Some(tab) = self.tabs.get_mut(tab_index) {
            if pane_index < tab.panes.len() {
                tab.focus = pane_index;
            }
        }
        self.active_tab = tab_index;
    }

    fn focus_next_pane(&mut self) {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            if tab.panes.len() > 1 {
                let next = (tab.focus + 1) % tab.panes.len();
                self.focus_pane(self.active_tab, next);
            }
        }
    }

    fn focus_prev_pane(&mut self) {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            if tab.panes.len() > 1 {
                let prev = (tab.focus + tab.panes.len() - 1) % tab.panes.len();
                self.focus_pane(self.active_tab, prev);
            }
        }
    }

    fn create_session(&mut self, intent: CreateIntent) {
        if self.pending_create.is_some() {
            return;
        }
        self.api.send(ApiCommand::CreateSession {
            label: intent.label.clone(),
        });
        self.pending_create = Some(intent);
    }

    fn request_new_tab(&mut self) {
        self.create_session(CreateIntent {
            kind: CreateKind::NewTab,
            label: None,
        });
    }

    fn split_focused(&mut self, axis: SplitAxis) {
        if self.pane_count() >= MAX_PANES {
            self.push_status(format!(
                "pane limit reached ({MAX_PANES}); the server advertises max_concurrent_streams per connection"
            ));
            return;
        }
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.axis = axis;
        }
        self.create_session(CreateIntent {
            kind: CreateKind::Split { axis },
            label: None,
        });
    }

    fn close_focused(&mut self) {
        let Some(tab_index) = self.tabs.get(self.active_tab).map(|_| self.active_tab) else {
            return;
        };
        let pane_index = match self.tabs.get(tab_index) {
            Some(tab) => tab.focus,
            None => return,
        };
        let removed = self.tabs.get_mut(tab_index).and_then(|tab| {
            if pane_index < tab.panes.len() {
                Some(tab.panes.remove(pane_index))
            } else {
                None
            }
        });
        let Some(removed) = removed else {
            return;
        };
        if let Some(stream_id) = removed.stream_id {
            self.api.send(ApiCommand::Detach { stream_id });
        }
        // The removed pane's stream was explicitly detached above. Keep its
        // `pending_attach` slot (if any) so an in-flight reply still consumes the
        // correct FIFO position; the `Attached` handler detaches that stream when
        // it finds the pane gone.
        if self.last_title_stream == removed.stream_id {
            self.last_title_stream = None;
        }
        if let Some(tab) = self.tabs.get_mut(tab_index) {
            if tab.panes.is_empty() {
                self.tabs.remove(tab_index);
                if self.tabs.is_empty() {
                    self.active_tab = 0;
                    return;
                }
                if self.active_tab >= self.tabs.len() {
                    self.active_tab = self.tabs.len() - 1;
                }
            } else if tab.focus >= tab.panes.len() {
                tab.focus = tab.panes.len() - 1;
            }
        }
        // Remaining panes stay attached; focus is purely local, so no re-attach.
    }

    fn pane_action(&mut self, action: PaneAction) {
        match action {
            PaneAction::SplitRight => self.split_focused(SplitAxis::Horizontal),
            PaneAction::SplitDown => self.split_focused(SplitAxis::Vertical),
            PaneAction::Zoom => self.zoomed = !self.zoomed,
            PaneAction::Close => self.close_focused(),
        }
    }

    fn scroll_pane(&mut self, pane_id: u64, delta: ScrollDelta) {
        let lines = match delta {
            ScrollDelta::Lines(point) => point.y,
            ScrollDelta::Pixels(point) => f32::from(point.y) / self.row_pitch,
        };
        let Some(pane) = self.pane_mut(pane_id) else {
            return;
        };
        let history = pane.term.history_len() as isize;
        pane.scroll_accum += lines;
        let whole = pane.scroll_accum.trunc();
        pane.scroll_accum -= whole;
        pane.history_offset = (pane.history_offset + whole as isize).clamp(0, history);
    }

    fn refresh_sessions(&self) {
        self.api.send(ApiCommand::RefreshSessions);
    }

    // --- resize reconciliation --------------------------------------------

    /// Derive every attached pane's grid from the window layout and send
    /// `terminal.resize` only when cols/rows change (a natural debounce: one
    /// command per distinct size, driven by render).
    ///
    /// The flex layout (`flex: 1`) divides the row/column into equal shares with
    /// the measured inter-pane gap between them; exactly two panes additionally
    /// honour the measured creation ratio. Every pane with a live stream is
    /// reconciled, not just the focused one.
    fn reconcile_resizes(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        let mut chrome =
            geometry::v1::TOP_BAR_HEIGHT.value + geometry::v1::TOP_BAR_SEPARATOR_RENDER_PX.value;
        if !matches!(self.connection, ConnectionState::Connected) {
            chrome += theme::CONNECTION_BANNER_HEIGHT_ORIGINAL;
        }
        if self.is_mock {
            chrome += theme::MOCK_BANNER_HEIGHT_ORIGINAL;
        }
        let body_height = (self.window_height - chrome).max(0.0);
        let width = self.window_width;

        let index = self.active_tab.min(self.tabs.len() - 1);
        let pane_count = self.tabs[index].panes.len();
        let axis = self.tabs[index].axis;
        let focus = self.tabs[index].focus;
        let zoomed = self.zoomed && pane_count > 1;
        let show = if zoomed { 1 } else { pane_count };
        let split = show > 1;
        let gap = if split {
            geometry::v4::INTER_PANE_GAP.value
        } else {
            0.0
        };
        let border = if split {
            geometry::v4::PANE_BORDER_FOCUSED.value
        } else {
            0.0
        };
        let header = geometry::v4::PANE_HEADER_HEIGHT.value;
        let ratio = geometry::v4::SPLIT_RATIO.value;

        // `extent` is the axis the panes are divided along; `other_extent` is the
        // full length of the other axis.
        let (extent, other_extent) = match axis {
            SplitAxis::Horizontal => (width, body_height),
            SplitAxis::Vertical => (body_height, width),
        };
        let shown: Vec<usize> = if zoomed {
            vec![focus]
        } else {
            (0..pane_count).collect()
        };

        for (slot, pane_index) in shown.iter().enumerate() {
            let share = geometry::pane_extent(extent, gap, show, ratio, slot);
            let (pane_width, pane_height) = match axis {
                SplitAxis::Horizontal => (
                    (share - 2.0 * border).max(0.0),
                    (other_extent - header - 2.0 * border).max(0.0),
                ),
                SplitAxis::Vertical => (
                    (other_extent - 2.0 * border).max(0.0),
                    (share - header - 2.0 * border).max(0.0),
                ),
            };
            let pane = &mut self.tabs[index].panes[*pane_index];
            let Some(stream_id) = pane.stream_id else {
                continue;
            };
            let cols = ((pane_width / self.cell_width).floor() as i64).clamp(1, 1000) as u16;
            let rows = ((pane_height / self.row_pitch).floor() as i64).clamp(1, 500) as u16;
            if pane.cols != cols || pane.rows != rows {
                pane.cols = cols;
                pane.rows = rows;
                pane.term.resize(TerminalSize {
                    cols,
                    rows,
                    pixel_width: pane_width as u32,
                    pixel_height: pane_height as u32,
                });
            }
            if pane.sent_cols != cols || pane.sent_rows != rows {
                pane.sent_cols = cols;
                pane.sent_rows = rows;
                eprintln!(
                    "relayable-ui: sending terminal.resize stream_id={stream_id} cols={cols} rows={rows} pane_px={}x{}",
                    pane_width as u32, pane_height as u32
                );
                self.api.send(ApiCommand::Resize {
                    stream_id,
                    cols,
                    rows,
                    pixel_width: Some(pane_width as u32),
                    pixel_height: Some(pane_height as u32),
                });
            }
        }
    }

    // --- keyboard ----------------------------------------------------------

    fn handle_key(&mut self, event: &KeyDownEvent, _window: &mut Window) {
        let keystroke = &event.keystroke;
        if self.picker_open {
            self.handle_picker_key(keystroke);
            return;
        }

        let control = keystroke.modifiers.control;
        let shift = keystroke.modifiers.shift;
        if control && shift {
            match keystroke.key.as_str() {
                "e" => return self.split_focused(SplitAxis::Horizontal),
                "o" => return self.split_focused(SplitAxis::Vertical),
                "w" => return self.close_focused(),
                "z" => {
                    self.zoomed = !self.zoomed;
                    return;
                }
                "s" => {
                    self.picker_open = true;
                    self.picker_query.clear();
                    self.picker_selected = 0;
                    self.refresh_sessions();
                    return;
                }
                "n" => return self.request_new_tab(),
                "right" => return self.focus_next_pane(),
                "left" => return self.focus_prev_pane(),
                "t" => return self.next_tab(),
                _ => {}
            }
        }
        if control && !shift {
            match keystroke.key.as_str() {
                "pagedown" => return self.next_tab(),
                "pageup" => return self.prev_tab(),
                _ => {}
            }
            if let Some(digit) = keystroke.key.chars().next().and_then(|c| c.to_digit(10)) {
                if digit >= 1 {
                    self.select_tab(digit as usize - 1);
                    return;
                }
            }
        }

        if let Some(bytes) = terminal_bytes(keystroke) {
            if let Some(stream_id) = self.focused_stream() {
                self.api.send(ApiCommand::Input { stream_id, bytes });
            }
        }
    }

    fn handle_picker_key(&mut self, keystroke: &Keystroke) {
        match keystroke.key.as_str() {
            "escape" => {
                self.picker_open = false;
                return;
            }
            "enter" => {
                self.picker_accept();
                return;
            }
            "up" => {
                self.picker_selected = self.picker_selected.saturating_sub(1);
                return;
            }
            "down" => {
                self.picker_selected = self.picker_selected.saturating_add(1);
                return;
            }
            "backspace" => {
                self.picker_query.pop();
                return;
            }
            _ => {}
        }
        if let Some(ch) = keystroke
            .key_char
            .as_deref()
            .filter(|c| !c.is_empty() && !keystroke.modifiers.control)
        {
            self.picker_query.push_str(ch);
        }
    }

    fn picker_matches(&self) -> Vec<usize> {
        let query = self.picker_query.to_lowercase();
        self.sessions
            .iter()
            .enumerate()
            .filter(|(_, session)| {
                query.is_empty()
                    || session
                        .label
                        .as_deref()
                        .unwrap_or(&session.session_id)
                        .to_lowercase()
                        .contains(&query)
                    || session.session_id.to_lowercase().contains(&query)
            })
            .map(|(index, _)| index)
            .collect()
    }

    fn picker_accept(&mut self) {
        if !self.picker_query.trim().is_empty() {
            let label = self.picker_query.trim().to_string();
            self.picker_open = false;
            self.create_session(CreateIntent {
                kind: CreateKind::NewTab,
                label: Some(label),
            });
            return;
        }
        let matches = self.picker_matches();
        if let Some(index) = matches.get(self.picker_selected).copied() {
            if let Some(session) = self.sessions.get(index) {
                let target = self.tabs.iter().position(|tab| {
                    tab.panes
                        .iter()
                        .any(|pane| pane.session_id == session.session_id)
                });
                if let Some(tab) = target {
                    self.select_tab(tab);
                }
            }
        }
        self.picker_open = false;
    }

    fn tab_labels(&self) -> Vec<String> {
        let labels: Vec<String> = self
            .tabs
            .iter()
            .map(|tab| {
                tab.panes
                    .get(tab.focus)
                    .map(|pane| pane.tab_label(&self.home))
                    .unwrap_or_else(|| "shell".to_string())
            })
            .collect();
        disambiguate_labels(labels)
    }

    // --- rendering ---------------------------------------------------------

    fn render_top_bar(&self, cx: &mut Context<Self>) -> Div {
        let labels = self.tab_labels();
        let active = self.active_tab;
        let mut tabs = div().flex().flex_row().items_start();
        for index in 0..self.tabs.len() {
            let is_active = index == active;
            let label = labels
                .get(index)
                .cloned()
                .unwrap_or_else(|| "shell".to_string());
            let next_is_active = index + 1 == active;

            let mut pill = div()
                .w(px(geometry::v1::ACTIVE_TAB_PILL_WIDTH.value))
                .h(px(geometry::v1::ACTIVE_TAB_PILL_HEIGHT.value))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(geometry::invented::TAB_PILL_CONTENT_GAP_ORIGINAL))
                .px(px(geometry::invented::TAB_PILL_PADDING_X_ORIGINAL))
                .text_size(px(theme::UI_LABEL_FONT_SIZE_PROPOSED))
                .cursor_pointer()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |view, _event, _window, cx| {
                        view.select_tab(index);
                        cx.notify();
                    }),
                );

            if is_active {
                pill = pill
                    .bg(rgb(theme::ACTIVE_TAB_PILL_FILL))
                    .rounded(px(geometry::v1::TAB_PILL_CORNER_RADIUS.value));
            } else {
                pill = pill.text_color(rgb(theme::SECONDARY_TEXT));
                if !next_is_active && index + 1 < self.tabs.len() {
                    pill = pill
                        .border_r(px(geometry::v1::TAB_DIVIDER_RENDER_PX.value))
                        .border_color(rgb(theme::INACTIVE_TAB_DIVIDER));
                }
            }

            let glyph_color = if is_active {
                theme::PRIMARY_TEXT
            } else {
                theme::SECONDARY_TEXT
            };
            pill = pill
                .child(
                    div()
                        .text_color(rgb(glyph_color))
                        .child(SharedString::new_static(">_")),
                )
                .child(
                    div()
                        .text_color(rgb(if is_active {
                            theme::PRIMARY_TEXT
                        } else {
                            theme::SECONDARY_TEXT
                        }))
                        .child(label),
                );
            tabs = tabs.child(pill);
        }

        div()
            .flex()
            .flex_row()
            .items_start()
            .h(px(geometry::v1::TOP_BAR_HEIGHT.value))
            .w_full()
            .bg(rgb(theme::TOP_BAR_BG))
            // Empty traffic-light region: the reference's macOS chrome is not an
            // app token, so Relayable leaves the measured 128 px strip blank.
            .child(
                div()
                    .w(px(geometry::v1::TAB_LEADING_INSET.value))
                    .flex_shrink_0(),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .pt(px(geometry::invented::TAB_STRIP_PADDING_TOP_ORIGINAL))
                    .child(tabs),
            )
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(geometry::invented::NEW_TAB_CONTROL_SIZE_ORIGINAL))
                    .h(px(geometry::v1::TOP_BAR_HEIGHT.value))
                    .flex_shrink_0()
                    .cursor_pointer()
                    .text_size(px(theme::NEW_TAB_GLYPH_FONT_SIZE_ORIGINAL))
                    .text_color(rgb(theme::PRIMARY_TEXT))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|view, _event, _window, cx| {
                            view.request_new_tab();
                            cx.notify();
                        }),
                    )
                    .child(SharedString::new_static("+")),
            )
    }

    fn render_connection_banner(&self) -> Option<Div> {
        let (background, text) = match &self.connection {
            ConnectionState::Connected => return None,
            ConnectionState::Connecting => (
                theme::CONNECTING_BG_ORIGINAL,
                "CONNECTING — waiting for the Relayable server".to_string(),
            ),
            ConnectionState::Reconnecting { attempt } => (
                theme::RECONNECTING_BG_ORIGINAL,
                format!("RECONNECTING — attempt {attempt}"),
            ),
            ConnectionState::Disconnected => (
                theme::DISCONNECTED_BG_ORIGINAL,
                "DISCONNECTED — no live server connection".to_string(),
            ),
            ConnectionState::Error(message) => {
                (theme::ERROR_BG_ORIGINAL, format!("ERROR — {message}"))
            }
        };
        Some(
            div()
                .h(px(theme::CONNECTION_BANNER_HEIGHT_ORIGINAL))
                .w_full()
                .flex()
                .items_center()
                .gap(px(geometry::invented::BANNER_CONTENT_GAP_ORIGINAL))
                .px(px(geometry::invented::BANNER_PADDING_X_ORIGINAL))
                .bg(rgb(background))
                .text_size(px(theme::UI_LABEL_FONT_SIZE_PROPOSED))
                .text_color(rgb(theme::BANNER_FG_ORIGINAL))
                .child(
                    div()
                        .w(px(geometry::invented::BANNER_DOT_SIZE_ORIGINAL))
                        .h(px(geometry::invented::BANNER_DOT_SIZE_ORIGINAL))
                        .rounded(px(geometry::invented::BANNER_DOT_RADIUS_ORIGINAL))
                        .bg(rgb(theme::BANNER_FG_ORIGINAL)),
                )
                .child(text),
        )
    }

    fn render_mock_banner(&self) -> Div {
        div()
            .h(px(theme::MOCK_BANNER_HEIGHT_ORIGINAL))
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(theme::MOCK_BANNER_BG_ORIGINAL))
            .text_size(px(theme::UI_LABEL_FONT_SIZE_PROPOSED))
            .text_color(rgb(theme::MOCK_BANNER_FG_ORIGINAL))
            .child(SharedString::new_static(
                "MOCK — synthetic in-process data; not a live relayable-server",
            ))
    }

    fn render_body(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.tabs.is_empty() {
            return self.render_empty_state();
        }
        let tab_index = self.active_tab.min(self.tabs.len() - 1);
        let tab = &self.tabs[tab_index];
        let pane_count = tab.panes.len();
        let split = pane_count > 1;
        let zoomed = self.zoomed && split;

        let mut container = div().flex().flex_1().w_full().min_h(px(0.));
        container = match tab.axis {
            SplitAxis::Horizontal => container.flex_row(),
            SplitAxis::Vertical => container.flex_col(),
        };
        if split && !zoomed {
            container = container.gap(px(geometry::v4::INTER_PANE_GAP.value));
        }

        let indices: Vec<usize> = if zoomed {
            vec![tab.focus]
        } else {
            (0..pane_count).collect()
        };
        let mut body = container;
        for pane_index in indices {
            let focused = pane_index == tab.focus;
            body =
                body.child(self.render_pane(tab_index, pane_index, focused, split && !zoomed, cx));
        }
        body.into_any_element()
    }

    fn render_empty_state(&self) -> AnyElement {
        let message = match &self.connection {
            ConnectionState::Connected => "No sessions yet — creating one…",
            ConnectionState::Connecting => "Connecting to the Relayable server…",
            ConnectionState::Reconnecting { .. } => "Reconnecting…",
            ConnectionState::Disconnected => "Disconnected — no live server connection",
            ConnectionState::Error(_) => "Server error — see the banner",
        };
        div()
            .flex()
            .flex_1()
            .w_full()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(geometry::invented::EMPTY_STATE_GAP_ORIGINAL))
            .bg(rgb(theme::TERMINAL_BG_TOP_V1))
            .child(
                div()
                    .text_size(px(theme::EMPTY_STATE_TITLE_FONT_SIZE_ORIGINAL))
                    .text_color(rgb(theme::PRIMARY_TEXT))
                    .child(SharedString::new_static("Relayable")),
            )
            .child(
                div()
                    .text_size(px(theme::UI_LABEL_FONT_SIZE_PROPOSED))
                    .text_color(rgb(theme::SECONDARY_TEXT))
                    .child(SharedString::new(message.to_string())),
            )
            .into_any_element()
    }

    fn render_pane(
        &self,
        tab_index: usize,
        pane_index: usize,
        focused: bool,
        show_border: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(tab) = self.tabs.get(tab_index) else {
            return div().into_any_element();
        };
        let Some(pane) = tab.panes.get(pane_index) else {
            return div().into_any_element();
        };

        let header_bg = if !show_border {
            theme::PANE_HEADER_SURFACE_V1
        } else if focused {
            theme::FOCUSED_PANE_HEADER_V4
        } else {
            theme::UNFOCUSED_PANE_HEADER_V4
        };
        let (ramp_top, ramp_bottom) = if !show_border {
            (theme::TERMINAL_BG_TOP_V1, theme::TERMINAL_BG_BOTTOM_V1)
        } else if focused {
            (
                theme::FOCUSED_PANE_BODY_TOP_V4,
                theme::FOCUSED_PANE_BODY_BOTTOM_V4,
            )
        } else {
            (
                theme::UNFOCUSED_PANE_BODY_TOP_V4,
                theme::UNFOCUSED_PANE_BODY_BOTTOM_V4,
            )
        };

        let header =
            self.render_pane_header(tab_index, pane_index, focused, show_border, header_bg, cx);

        let pane_id = pane.pane_id;
        let scroll_listener = cx.listener(move |view, delta: &ScrollDelta, _window, cx| {
            view.scroll_pane(pane_id, *delta);
            cx.notify();
        });
        let terminal = TerminalCanvas {
            snapshot: pane.view_snapshot(),
            font: self.font.clone(),
            font_size: self.font_size,
            cell_width: self.cell_width,
            row_pitch: self.row_pitch,
            ramp_top,
            ramp_bottom,
            default_fg: theme::PRIMARY_TEXT,
            default_bg: ramp_top,
            cursor_fill: theme::CURSOR_FILL_ORIGINAL,
            cursor_alpha: theme::CURSOR_ALPHA_ORIGINAL,
            history_offset: pane.history_offset,
            history_len: pane.term.history_len(),
            scrollbar_track: theme::SCROLLBAR_TRACK_ORIGINAL,
            scrollbar_thumb: theme::SCROLLBAR_THUMB_ORIGINAL,
            scrollbar_width: theme::SCROLLBAR_WIDTH_ORIGINAL,
        }
        .into_element(
            ElementId::NamedInteger(SharedString::new_static("terminal"), pane_id),
            move |delta, window, cx| scroll_listener(&delta, window, cx),
        );

        let mut card = div()
            .flex_1()
            .min_w(px(0.))
            .min_h(px(0.))
            .flex()
            .flex_col()
            .overflow_hidden();
        if show_border {
            let border = if focused {
                theme::PANE_BORDER_FOCUSED_V4
            } else {
                theme::PANE_BORDER_UNFOCUSED_V4
            };
            card = card
                .border(px(geometry::v4::PANE_BORDER_FOCUSED.value))
                .border_color(rgb(border))
                .rounded(px(theme::PANE_CARD_RADIUS_ORIGINAL));
        }
        card = card
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |view, _event, _window, cx| {
                    view.focus_pane(tab_index, pane_index);
                    cx.notify();
                }),
            )
            .child(header)
            .child(terminal);
        card.into_any_element()
    }

    fn render_pane_header(
        &self,
        tab_index: usize,
        pane_index: usize,
        focused: bool,
        show_buttons: bool,
        background: u32,
        cx: &mut Context<Self>,
    ) -> Div {
        let label = self
            .tabs
            .get(tab_index)
            .and_then(|tab| tab.panes.get(pane_index))
            .map(|pane| pane.header_label(&self.home))
            .unwrap_or_else(|| "shell".to_string());

        let mut header = div()
            .h(px(geometry::v4::PANE_HEADER_HEIGHT.value))
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(geometry::invented::PANE_HEADER_GAP_ORIGINAL))
            .px(px(geometry::invented::PANE_HEADER_PADDING_X_ORIGINAL))
            .flex_shrink_0()
            .bg(rgb(background))
            .child(
                div()
                    .text_size(px(theme::PANE_LABEL_FONT_SIZE_PROPOSED))
                    .text_color(rgb(theme::SECONDARY_TEXT))
                    .child(SharedString::new_static(">_")),
            )
            .child(
                div()
                    .text_size(px(theme::PANE_LABEL_FONT_SIZE_PROPOSED))
                    .text_color(rgb(theme::PRIMARY_TEXT))
                    .child(label),
            )
            .child(div().flex_1());

        // The reference only showed the four buttons on the focused pane of a
        // *split* workspace (V4); V1's single-pane header has none.
        if focused && show_buttons {
            for action in [
                PaneAction::SplitRight,
                PaneAction::SplitDown,
                PaneAction::Zoom,
                PaneAction::Close,
            ] {
                header = header.child(self.render_icon_button(tab_index, pane_index, action, cx));
            }
        }
        header
    }

    fn render_icon_button(
        &self,
        _tab_index: usize,
        _pane_index: usize,
        action: PaneAction,
        cx: &mut Context<Self>,
    ) -> Div {
        div()
            .w(px(geometry::invented::PANE_ICON_BUTTON_SIZE_ORIGINAL))
            .h(px(geometry::invented::PANE_ICON_BUTTON_SIZE_ORIGINAL))
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .rounded(px(geometry::invented::PANE_ICON_BUTTON_RADIUS_ORIGINAL))
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |view, _event, _window, cx| {
                    // gpui bubbles mouse events deepest-first: without this the
                    // enclosing pane card's `on_mouse_down` would also fire and
                    // re-focus (and, for Close, re-attach) on the same click.
                    cx.stop_propagation();
                    // Button identity is low-medium confidence in the evidence;
                    // the mapping below is documented in the UI-002 notes.
                    view.pane_action(action);
                    cx.notify();
                }),
            )
            .child(icon_glyph(action, theme::PRIMARY_TEXT))
    }

    fn render_picker(&self, cx: &mut Context<Self>) -> Div {
        let matches = self.picker_matches();
        let current_session = self
            .tabs
            .get(self.active_tab)
            .and_then(|tab| tab.panes.get(tab.focus))
            .map(|pane| pane.session_id.clone());

        let mut rows = Vec::new();
        for (row_index, session_index) in matches.iter().enumerate() {
            let Some(session) = self.sessions.get(*session_index) else {
                continue;
            };
            let label = session
                .label
                .clone()
                .unwrap_or_else(|| session.session_id.clone());
            let is_current = current_session.as_deref() == Some(session.session_id.as_str());
            let is_selected = row_index == self.picker_selected;
            let session_id = session.session_id.clone();
            rows.push(
                div()
                    .h(px(geometry::v1::SESSION_PICKER_LIST_ROW.value))
                    .w_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(geometry::invented::PICKER_ROW_GAP_ORIGINAL))
                    .px(px(geometry::invented::PICKER_ROW_PADDING_X_ORIGINAL))
                    .rounded(px(geometry::invented::PICKER_ROW_RADIUS_ORIGINAL))
                    .when(is_selected, |row| {
                        row.bg(rgb(theme::SESSION_PICKER_SELECTED_ROW))
                    })
                    .text_size(px(theme::UI_LABEL_FONT_SIZE_PROPOSED))
                    .text_color(rgb(theme::PRIMARY_TEXT))
                    .child(if is_current {
                        SharedString::new_static("✓")
                    } else {
                        SharedString::new_static(" ")
                    })
                    .child(label)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |view, _event, _window, cx| {
                            view.picker_query = session_id.clone();
                            view.picker_accept();
                            cx.notify();
                        }),
                    ),
            );
        }
        if rows.is_empty() {
            rows.push(
                div()
                    .h(px(geometry::v1::SESSION_PICKER_LIST_ROW.value))
                    .w_full()
                    .flex()
                    .items_center()
                    .px(px(geometry::invented::PICKER_ROW_PADDING_X_ORIGINAL))
                    .text_size(px(theme::UI_LABEL_FONT_SIZE_PROPOSED))
                    .text_color(rgb(theme::SECONDARY_TEXT))
                    .child(format!("New Session \"{}\"", self.picker_query)),
            );
        }

        div()
            .absolute()
            .top(px(geometry::v1::TOP_BAR_HEIGHT.value
                + geometry::v1::TOP_BAR_SEPARATOR_RENDER_PX.value
                + geometry::invented::PICKER_OFFSET_BELOW_SEPARATOR_ORIGINAL))
            .left(px(geometry::v1::TAB_LEADING_INSET.value))
            .w(px(geometry::v1::SESSION_PICKER_WIDTH.value))
            .h(px(geometry::v1::SESSION_PICKER_HEIGHT.value))
            .flex()
            .flex_col()
            .gap(px(geometry::invented::PICKER_COLUMN_GAP_ORIGINAL))
            .p(px(geometry::invented::PICKER_PADDING_ORIGINAL))
            .bg(rgb(theme::SESSION_PICKER_BG))
            .border_1()
            .border_color(rgb(theme::SESSION_PICKER_BORDER))
            .rounded(px(theme::SESSION_PICKER_RADIUS_ORIGINAL))
            .child(
                div()
                    .h(px(geometry::v1::SESSION_PICKER_SEARCH_FIELD.value))
                    .w_full()
                    .flex()
                    .items_center()
                    .px(px(geometry::invented::PICKER_SEARCH_PADDING_X_ORIGINAL))
                    .rounded(px(geometry::invented::PICKER_SEARCH_RADIUS_ORIGINAL))
                    .bg(rgb(theme::PALETTE_SEARCH))
                    .text_size(px(theme::UI_LABEL_FONT_SIZE_PROPOSED))
                    .text_color(rgb(theme::SECONDARY_TEXT))
                    .child(if self.picker_query.is_empty() {
                        SharedString::new_static("Filter or create...")
                    } else {
                        SharedString::new(self.picker_query.clone())
                    }),
            )
            .children(rows)
    }
}

/// The four pane-header icons, drawn with layout (font-independent).
fn icon_glyph(action: PaneAction, color: u32) -> AnyElement {
    let line = rgb(color);
    match action {
        PaneAction::SplitRight => div()
            .w(px(geometry::invented::ICON_GLYPH_SPLIT_SIZE_ORIGINAL))
            .h(px(geometry::invented::ICON_GLYPH_SPLIT_SIZE_ORIGINAL))
            .border_1()
            .border_color(line)
            .flex()
            .flex_row()
            .justify_center()
            .child(
                div()
                    .w(px(geometry::invented::ICON_GLYPH_SPLIT_LINE_ORIGINAL))
                    .h_full()
                    .bg(line),
            )
            .into_any_element(),
        PaneAction::SplitDown => div()
            .w(px(geometry::invented::ICON_GLYPH_SPLIT_SIZE_ORIGINAL))
            .h(px(geometry::invented::ICON_GLYPH_SPLIT_SIZE_ORIGINAL))
            .border_1()
            .border_color(line)
            .flex()
            .flex_col()
            .justify_center()
            .child(
                div()
                    .w_full()
                    .h(px(geometry::invented::ICON_GLYPH_SPLIT_LINE_ORIGINAL))
                    .bg(line),
            )
            .into_any_element(),
        PaneAction::Zoom => div()
            .w(px(geometry::invented::ICON_GLYPH_ZOOM_SIZE_ORIGINAL))
            .h(px(geometry::invented::ICON_GLYPH_ZOOM_SIZE_ORIGINAL))
            .border_1()
            .border_color(line)
            .into_any_element(),
        PaneAction::Close => div()
            .text_size(px(geometry::invented::ICON_GLYPH_CLOSE_FONT_SIZE_ORIGINAL))
            .text_color(line)
            .child(SharedString::new_static("×"))
            .into_any_element(),
    }
}

/// Pane ids that have no live stream and therefore must be (re-)attached.
///
/// Pure so the reconnect policy can be unit-tested without a gpui context.
fn panes_needing_attach(tabs: &[Tab]) -> Vec<u64> {
    tabs.iter()
        .flat_map(|tab| tab.panes.iter())
        .filter(|pane| pane.stream_id.is_none())
        .map(|pane| pane.pane_id)
        .collect()
}

/// Choose a font family that is guaranteed to be present in `all_font_names`.
///
/// Preference order first; then any installed name that looks monospace; then
/// the first installed name. Only when the list is empty (gpui in practice
/// appends its fallback stack, so this is defensive) fall back to gpui's own
/// `".ZedMono"` alias, which the crate maps into its fallback stack
/// (`gpui-0.2.2/src/text_system.rs:908-918`). Never returns an unvalidated
/// literal such as `"monospace"`, which `TextSystem::resolve_font` would panic
/// on if no fallback resolved.
fn choose_font_family(names: &[String]) -> String {
    for candidate in theme::FONT_FAMILY_PREFERENCE {
        if names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(candidate))
        {
            return (*candidate).to_string();
        }
    }
    if let Some(name) = names
        .iter()
        .find(|name| name.to_ascii_lowercase().contains("mono"))
    {
        return name.clone();
    }
    if let Some(name) = names.first() {
        return name.clone();
    }
    ".ZedMono".to_string()
}

/// Drop a `$HOME` prefix to `~` (the reference showed `~`, not a full path).
fn shorten_home(path: &str, home: &str) -> String {
    if !home.is_empty() && path == home {
        return "~".to_string();
    }
    if !home.is_empty() {
        if let Some(rest) = path.strip_prefix(home) {
            if rest.starts_with('/') {
                return format!("~{rest}");
            }
        }
    }
    path.to_string()
}

/// Apply the reference's duplicate-label rule: second occurrence gets `-1`,
/// third `-2` (medium confidence, V1 1:33).
fn disambiguate_labels(labels: Vec<String>) -> Vec<String> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut out = Vec::with_capacity(labels.len());
    for label in labels {
        let count = seen.entry(label.clone()).or_insert(0);
        if *count == 0 {
            out.push(label);
        } else {
            out.push(format!("{label}-{}", *count));
        }
        *count += 1;
    }
    out
}

/// Map a keystroke to terminal input bytes.
///
/// This is standard terminal behaviour (not reference UI), so it is
/// Relayable-original. `Ctrl+Shift+<key>` combos are reserved for the app's own
/// bindings and never reach the terminal.
fn terminal_bytes(keystroke: &Keystroke) -> Option<Vec<u8>> {
    if keystroke.modifiers.platform {
        return None;
    }
    if keystroke.modifiers.control && keystroke.modifiers.shift {
        return None;
    }
    match keystroke.key.as_str() {
        "enter" => return Some(b"\r".to_vec()),
        "backspace" => return Some(vec![0x7f]),
        "tab" => return Some(b"\t".to_vec()),
        "escape" => return Some(vec![0x1b]),
        "up" => return Some(b"\x1b[A".to_vec()),
        "down" => return Some(b"\x1b[B".to_vec()),
        "right" => return Some(b"\x1b[C".to_vec()),
        "left" => return Some(b"\x1b[D".to_vec()),
        "home" => return Some(b"\x1b[H".to_vec()),
        "end" => return Some(b"\x1b[F".to_vec()),
        "delete" => return Some(b"\x1b[3~".to_vec()),
        "pageup" => return Some(b"\x1b[5~".to_vec()),
        "pagedown" => return Some(b"\x1b[6~".to_vec()),
        _ => {}
    }
    let text = keystroke.key_char.as_deref().filter(|c| !c.is_empty())?;
    if keystroke.modifiers.control {
        let ch = text.chars().next()?;
        if ch.is_ascii_alphabetic() {
            return Some(vec![(ch.to_ascii_lowercase() as u8) - b'a' + 1]);
        }
        return None;
    }
    if keystroke.modifiers.alt {
        let mut bytes = vec![0x1b];
        bytes.extend_from_slice(text.as_bytes());
        return Some(bytes);
    }
    Some(text.as_bytes().to_vec())
}

impl Render for WorkspaceView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // gpui 0.2.2 + Hyprland interaction: Hyprland forces client-side
        // decorations, which makes gpui's `is_transparent()` true and leaves the
        // surface with a small residual alpha. Re-asserting opaque server-side
        // decorations immediately before the appearance update keeps
        // `update_window` on the opaque path for this frame.
        window.request_decorations(WindowDecorations::Server);
        window.set_background_appearance(WindowBackgroundAppearance::Opaque);

        let bounds = window.bounds();
        self.window_width = f32::from(bounds.size.width);
        self.window_height = f32::from(bounds.size.height);
        if !self.logged_geometry {
            self.logged_geometry = true;
            eprintln!(
                "relayable: window bounds {}x{} at ({}, {}) scale_factor={}",
                f32::from(bounds.size.width),
                f32::from(bounds.size.height),
                f32::from(bounds.origin.x),
                f32::from(bounds.origin.y),
                window.scale_factor()
            );
        }
        self.reconcile_resizes();

        let top_bar = self.render_top_bar(cx);
        let body = self.render_body(cx);

        let mut root = div()
            .flex()
            .flex_col()
            .size_full()
            .relative()
            .bg(rgb(theme::TOP_BAR_BG))
            .key_context("Workspace")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|view, event, _window, cx| {
                view.handle_key(event, _window);
                cx.notify();
            }))
            .child(top_bar)
            .child(
                div()
                    .h(px(geometry::v1::TOP_BAR_SEPARATOR_RENDER_PX.value))
                    .w_full()
                    .flex_shrink_0()
                    .bg(rgb(theme::TOP_BAR_SEPARATOR)),
            );
        if let Some(banner) = self.render_connection_banner() {
            root = root.child(banner);
        }
        root = root.child(body);
        if self.is_mock {
            root = root.child(self.render_mock_banner());
        }
        if self.picker_open {
            root = root.child(self.render_picker(cx));
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Modifiers;

    fn keystroke(key: &str, key_char: Option<&str>) -> Keystroke {
        Keystroke {
            modifiers: Modifiers::default(),
            key: key.to_string(),
            key_char: key_char.map(str::to_string),
        }
    }

    #[test]
    fn duplicate_tab_labels_get_the_reference_suffix() {
        let labels = disambiguate_labels(vec![
            "~".to_string(),
            "~/Documents/ghostty".to_string(),
            "~/Documents/ghostty".to_string(),
            "~/Documents/ghostty".to_string(),
        ]);
        assert_eq!(
            labels,
            vec![
                "~",
                "~/Documents/ghostty",
                "~/Documents/ghostty-1",
                "~/Documents/ghostty-2"
            ]
        );
    }

    #[test]
    fn home_prefix_is_shortened_to_tilde() {
        assert_eq!(shorten_home("/home/me", "/home/me"), "~");
        assert_eq!(shorten_home("/home/me/project", "/home/me"), "~/project");
        assert_eq!(shorten_home("/tmp", "/home/me"), "/tmp");
    }

    #[test]
    fn terminal_bytes_maps_special_keys() {
        assert_eq!(
            terminal_bytes(&keystroke("enter", None)),
            Some(b"\r".to_vec())
        );
        assert_eq!(
            terminal_bytes(&keystroke("backspace", None)),
            Some(vec![0x7f])
        );
        assert_eq!(
            terminal_bytes(&keystroke("up", None)),
            Some(b"\x1b[A".to_vec())
        );
        assert_eq!(
            terminal_bytes(&keystroke("a", Some("a"))),
            Some(b"a".to_vec())
        );
    }

    #[test]
    fn control_letters_become_control_bytes() {
        let mut ks = keystroke("c", Some("c"));
        ks.modifiers.control = true;
        assert_eq!(terminal_bytes(&ks), Some(vec![0x03]));
    }

    #[test]
    fn control_shift_is_reserved_for_the_app() {
        let mut ks = keystroke("e", Some("e"));
        ks.modifiers.control = true;
        ks.modifiers.shift = true;
        assert_eq!(terminal_bytes(&ks), None);
    }

    #[test]
    fn font_family_is_always_chosen_from_installed_names() {
        let installed = vec!["Noto Sans Mono".to_string(), "Arial".to_string()];
        assert_eq!(choose_font_family(&installed), "Noto Sans Mono");
        let preferred = vec!["JetBrainsMono Nerd Font".to_string(), "Arial".to_string()];
        assert_eq!(choose_font_family(&preferred), "JetBrainsMono Nerd Font");
        // Otherwise any monospace-looking installed name.
        let quirky = vec!["Arial".to_string(), "Iosevka Term Mono".to_string()];
        assert_eq!(choose_font_family(&quirky), "Iosevka Term Mono");
        // Then the first installed name (never the literal "monospace").
        let no_mono = vec!["Arial".to_string(), "Comic Sans".to_string()];
        assert_eq!(choose_font_family(&no_mono), "Arial");
        // Empty list falls back to the gpui alias that its fallback stack maps.
        assert_eq!(choose_font_family(&[]), ".ZedMono");
    }

    #[test]
    fn reconnect_targets_only_panes_without_a_live_stream() {
        let mut attached = Pane::new(1, "session:a".to_string(), "block:a".to_string(), None);
        attached.stream_id = Some(9);
        let detached = Pane::new(2, "session:b".to_string(), "block:b".to_string(), None);
        let tabs = vec![Tab {
            panes: vec![attached, detached],
            focus: 1,
            axis: SplitAxis::Horizontal,
        }];
        assert_eq!(panes_needing_attach(&tabs), vec![2]);
        assert_eq!(panes_needing_attach(&[]), Vec::<u64>::new());
    }
}
