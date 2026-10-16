use crate::geometry;
use crate::theme;
use gpui::{div, prelude::*, px, rgb, Context, Div, MouseButton, SharedString};

use super::state::{CreateIntent, CreateKind};
use super::WorkspaceView;

impl WorkspaceView {
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

    pub(super) fn picker_accept(&mut self) {
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

    pub(super) fn render_picker(&self, cx: &mut Context<Self>) -> Div {
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
