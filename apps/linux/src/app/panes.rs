use crate::geometry;
use crate::theme;
use crate::ui::terminal::TerminalCanvas;
use gpui::{
    div, prelude::*, px, rgb, AnyElement, Context, Div, ElementId, MouseButton, ScrollDelta,
    SharedString,
};

use super::helpers::empty_state_message;
use super::state::{PaneAction, SplitAxis};
use super::WorkspaceView;

impl WorkspaceView {
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

    pub(super) fn render_body(&self, cx: &mut Context<Self>) -> AnyElement {
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
        // Message selection only: the empty surface keeps its measured shape, so
        // the body geometry cannot move. The copy is Relayable-original — the
        // reference videos show no empty/loading/error/disconnected surface (see
        // `apps/linux/evidence/ui-004-notes.md`).
        let message = empty_state_message(&self.connection, self.pending_create.is_some());
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
