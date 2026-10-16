use crate::geometry;
use crate::theme;
use gpui::{
    div, prelude::*, px, rgb, Context, Window, WindowBackgroundAppearance, WindowDecorations,
};

use super::WorkspaceView;

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
