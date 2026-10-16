use crate::api::ApiCommand;

use super::state::{CreateIntent, CreateKind, PaneAction, SplitAxis};
use super::WorkspaceView;

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

impl WorkspaceView {
    pub(super) fn select_tab(&mut self, index: usize) {
        if index >= self.tabs.len() || index == self.active_tab {
            return;
        }
        self.active_tab = index;
        self.zoomed = false;
    }

    pub(super) fn next_tab(&mut self) {
        if self.tabs.len() < 2 {
            return;
        }
        let next = (self.active_tab + 1) % self.tabs.len();
        self.select_tab(next);
    }

    pub(super) fn prev_tab(&mut self) {
        if self.tabs.len() < 2 {
            return;
        }
        let prev = (self.active_tab + self.tabs.len() - 1) % self.tabs.len();
        self.select_tab(prev);
    }

    /// Move focus. Purely local: the newly focused pane is already attached, so
    /// no attach/detach is issued here.
    pub(super) fn focus_pane(&mut self, tab_index: usize, pane_index: usize) {
        if let Some(tab) = self.tabs.get_mut(tab_index) {
            if pane_index < tab.panes.len() {
                tab.focus = pane_index;
            }
        }
        self.active_tab = tab_index;
    }

    pub(super) fn focus_next_pane(&mut self) {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            if tab.panes.len() > 1 {
                let next = (tab.focus + 1) % tab.panes.len();
                self.focus_pane(self.active_tab, next);
            }
        }
    }

    pub(super) fn focus_prev_pane(&mut self) {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            if tab.panes.len() > 1 {
                let prev = (tab.focus + tab.panes.len() - 1) % tab.panes.len();
                self.focus_pane(self.active_tab, prev);
            }
        }
    }

    pub(super) fn create_session(&mut self, intent: CreateIntent) {
        if self.pending_create.is_some() {
            return;
        }
        self.api.send(ApiCommand::CreateSession {
            label: intent.label.clone(),
        });
        self.pending_create = Some(intent);
    }

    pub(super) fn request_new_tab(&mut self) {
        self.create_session(CreateIntent {
            kind: CreateKind::NewTab,
            label: None,
        });
    }

    pub(super) fn split_focused(&mut self, axis: SplitAxis) {
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

    pub(super) fn close_focused(&mut self) {
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

    pub(super) fn pane_action(&mut self, action: PaneAction) {
        match action {
            PaneAction::SplitRight => self.split_focused(SplitAxis::Horizontal),
            PaneAction::SplitDown => self.split_focused(SplitAxis::Vertical),
            PaneAction::Zoom => self.zoomed = !self.zoomed,
            PaneAction::Close => self.close_focused(),
        }
    }

    pub(super) fn refresh_sessions(&self) {
        self.api.send(ApiCommand::RefreshSessions);
    }
}
