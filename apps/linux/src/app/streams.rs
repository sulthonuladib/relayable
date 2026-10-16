use crate::api::ApiCommand;

use super::helpers::panes_needing_attach;
use super::state::Pane;
use super::WorkspaceView;

impl WorkspaceView {
    // --- pane/tab plumbing -------------------------------------------------

    fn pane(&self, pane_id: u64) -> Option<&Pane> {
        self.tabs
            .iter()
            .flat_map(|tab| tab.panes.iter())
            .find(|pane| pane.pane_id == pane_id)
    }

    pub(super) fn pane_mut(&mut self, pane_id: u64) -> Option<&mut Pane> {
        self.tabs
            .iter_mut()
            .flat_map(|tab| tab.panes.iter_mut())
            .find(|pane| pane.pane_id == pane_id)
    }

    pub(super) fn pane_by_stream_mut(&mut self, stream_id: u64) -> Option<&mut Pane> {
        self.tabs
            .iter_mut()
            .flat_map(|tab| tab.panes.iter_mut())
            .find(|pane| pane.stream_id == Some(stream_id))
    }

    fn focused_pane_id(&self) -> Option<u64> {
        let tab = self.tabs.get(self.active_tab)?;
        tab.panes.get(tab.focus).map(|pane| pane.pane_id)
    }

    pub(super) fn focused_stream(&self) -> Option<u64> {
        let pane_id = self.focused_pane_id()?;
        self.pane(pane_id).and_then(|pane| pane.stream_id)
    }

    pub(super) fn pane_count(&self) -> usize {
        self.tabs.iter().map(|tab| tab.panes.len()).sum()
    }

    /// Attach one pane's block as its own live writer stream.
    ///
    /// Focus does **not** detach anything: every pane keeps its stream until it
    /// is explicitly closed. `takeover: true` makes a re-attach robust against a
    /// lease still held by the previous (now dead) connection; Relayable is the
    /// only writer client for these sessions. The session id travels with the
    /// block id because the data plane attaches through the public client API,
    /// which names both (`relayable_client::Client::attach_with_takeover`).
    pub(super) fn attach_pane(&mut self, pane_id: u64) {
        let Some((session_id, block_id)) = self
            .pane(pane_id)
            .map(|pane| (pane.session_id.clone(), pane.block_id.clone()))
        else {
            return;
        };
        if self.pending_attach.contains(&pane_id) {
            return;
        }
        self.pending_attach.push_back(pane_id);
        self.api.send(ApiCommand::AttachBlock {
            session_id,
            block_id,
            takeover: true,
        });
    }

    /// Re-attach every pane with no live stream (used after a reconnect).
    pub(super) fn reattach_missing_panes(&mut self) {
        for pane_id in panes_needing_attach(&self.tabs) {
            self.attach_pane(pane_id);
        }
    }

    /// Drop every pane's stream association. Used when the connection is lost
    /// or errored so the UI never sends input or resize to a dead stream; the
    /// next `Connected` re-attaches and re-seeds. The pending-attach queue is
    /// left alone: a fresh attach may still be in flight and its reply must
    /// still bind to the pane that requested it.
    pub(super) fn invalidate_streams(&mut self) {
        self.last_title_stream = None;
        for tab in &mut self.tabs {
            for pane in &mut tab.panes {
                pane.stream_id = None;
            }
        }
    }
}
