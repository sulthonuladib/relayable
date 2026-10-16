use crate::api::{ApiCommand, ConnectionState};
use crate::geometry;
use crate::term::TerminalSize;
use crate::theme;

use super::state::SplitAxis;
use super::WorkspaceView;

impl WorkspaceView {
    // --- resize reconciliation --------------------------------------------

    /// Derive every attached pane's grid from the window layout and send
    /// `terminal.resize` only when cols/rows change (a natural debounce: one
    /// command per distinct size, driven by render).
    ///
    /// The flex layout (`flex: 1`) divides the row/column into equal shares with
    /// the measured inter-pane gap between them; exactly two panes additionally
    /// honour the measured creation ratio. Every pane with a live stream is
    /// reconciled, not just the focused one.
    pub(super) fn reconcile_resizes(&mut self) {
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
}
