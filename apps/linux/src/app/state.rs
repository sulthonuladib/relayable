use crate::term::{TerminalModel, TerminalSize};

use super::helpers::shorten_home;

/// Which way a tab's panes are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SplitAxis {
    /// Panes side by side (`Split Pane Right`).
    Horizontal,
    /// Panes stacked (`Split Pane Down`).
    Vertical,
}

/// What the next `session.create` reply should build.
#[derive(Debug, Clone)]
pub(super) enum CreateKind {
    /// A new tab (its own session).
    NewTab,
    /// A new pane in the current tab (a new session — no `block.create`).
    Split { axis: SplitAxis },
}

/// A pending create request.
#[derive(Debug, Clone)]
pub(super) struct CreateIntent {
    pub(super) kind: CreateKind,
    pub(super) label: Option<String>,
}

/// Pane-header button actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PaneAction {
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
pub(super) struct Pane {
    pub(super) pane_id: u64,
    pub(super) session_id: String,
    pub(super) block_id: String,
    /// Label requested at session creation (the only label `session.create`
    /// echoes back to the client).
    requested_label: Option<String>,
    /// Label from `session.list`.
    server_label: Option<String>,
    /// OSC 0/2 title from the live output stream.
    pub(super) program: Option<String>,
    /// OSC 7 cwd from the live output stream.
    pub(super) cwd: Option<String>,
    pub(super) stream_id: Option<u64>,
    pub(super) term: TerminalModel,
    pub(super) cols: u16,
    pub(super) rows: u16,
    pub(super) sent_cols: u16,
    pub(super) sent_rows: u16,
    pub(super) history_offset: isize,
    pub(super) scroll_accum: f32,
}

impl Pane {
    pub(super) fn new(
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

    pub(super) fn view_snapshot(&self) -> crate::term::ScreenSnapshot {
        if self.history_offset > 0 {
            self.term.snapshot_at(self.history_offset)
        } else {
            self.term.snapshot()
        }
    }

    /// The pane-header label: live OSC program/cwd first, then labels, then a
    /// documented fallback (the reference showed `~` / `shell`).
    pub(super) fn header_label(&self, home: &str) -> String {
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
    pub(super) fn tab_label(&self, home: &str) -> String {
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
pub(super) struct Tab {
    pub(super) panes: Vec<Pane>,
    pub(super) focus: usize,
    pub(super) axis: SplitAxis,
}
