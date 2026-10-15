//! Relayable Linux desktop app — UI-001 minimal GPUI window.
//!
//! Scope for UI-001 is deliberately minimal: prove that the ADR-003
//! `gpui = "=0.2.2"` pin actually compiles and opens a real window on this
//! host, after the contained TERM-002 `libc` remedy. This is not product UI.
//!
//! The structure follows the crate's own `examples/hello_world.rs` (read from
//! `~/.cargo/registry/src/.../gpui-0.2.2/examples/hello_world.rs`); no GPUI API
//! here is invented.
//!
//! The window exercises two tokens from the (GPUI-free) library token sheet
//! [`relayable_linux::tokens`]: the documented V1 pane-header/surface background
//! (`colors::PANE_HEADER_SURFACE_V1`, confidence **high**) and the documented
//! primary-text colour (`colors::PRIMARY_TEXT`, confidence **medium**). This is
//! a deliberate token smoke test, not the UI-002 workspace UI.

use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, Render, SharedString,
    Window, WindowBounds, WindowOptions,
};

use relayable_linux::tokens::colors;

/// Root view: a single branded placeholder label.
struct RelayableRoot {
    label: SharedString,
}

impl Render for RelayableRoot {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .justify_center()
            .items_center()
            // Documented V1 pane-header/surface colour (confidence high).
            .bg(rgb(colors::PANE_HEADER_SURFACE_V1
                .value
                .to_packed_rgb_u32()))
            .text_xl()
            // Documented primary-text colour (confidence medium).
            .text_color(rgb(colors::PRIMARY_TEXT.value.to_packed_rgb_u32()))
            .child(self.label.clone())
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(960.), px(600.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Relayable".into()),
                    ..Default::default()
                }),
                app_id: Some("relayable".to_string()),
                ..Default::default()
            },
            |window, cx| {
                // On Wayland the `WindowOptions.titlebar` title is not applied by
                // gpui 0.2.2 (only macOS uses it); the xdg_toplevel title is set
                // through `Window::set_window_title`
                // (`gpui-0.2.2/src/window.rs:1779` -> wayland `set_title`,
                // `.../wayland/window.rs:939` -> `toplevel.set_title`).
                window.set_window_title("Relayable");
                cx.new(|_cx| RelayableRoot {
                    label: "Relayable".into(),
                })
            },
        )
        .expect("failed to open Relayable window");
        cx.activate(true);
    });
}
