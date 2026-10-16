use crate::api::ConnectionState;
use crate::geometry;
use crate::theme;
use gpui::{div, prelude::*, px, rgb, Context, Div, MouseButton, SharedString};

use super::helpers::disambiguate_labels;
use super::WorkspaceView;

impl WorkspaceView {
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

    pub(super) fn render_top_bar(&self, cx: &mut Context<Self>) -> Div {
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

    pub(super) fn render_connection_banner(&self) -> Option<Div> {
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

    pub(super) fn render_mock_banner(&self) -> Div {
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
}
