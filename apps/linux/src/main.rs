//! Relayable Linux desktop app — UI-002 reference workspace entry point.
//!
//! Opens the measured 1099×706 baseline window, focuses the workspace view and
//! lets [`relayable_linux::app::WorkspaceView`] render the reference UI against
//! a live `relayable-server` (default) or the clearly-labelled mock (`--mock`).
//!
//! ADR-003 pins `gpui = "=0.2.2"`; GPUI 0.2.2 cannot resize a Wayland toplevel
//! (`platform/linux/wayland/window.rs` never calls `xdg_toplevel.set_size`), so
//! the initial `window_bounds` is the only size control — a Hyprland window rule
//! pins it for the baseline screenshot.
//!
//! ## Options
//!
//! ```text
//! relayable [--mock] [--socket PATH] [--canvas W H]
//! ```
//!
//! * `--mock` — in-process synthetic data (never the default; the UI shows a
//!   visible `MOCK` banner).
//! * `--socket PATH` — override the server socket (also `RELAYABLE_SERVER`).
//! * `--canvas W H` — override the initial window size (also
//!   `RELAYABLE_CANVAS=WxH`); the baseline is `geometry::BASELINE_CANVAS_OUTER`.
//!
//! ## Relayable-original keybindings (Linux; the reference showed macOS only)
//!
//! The keymap has **one** definition: `relayable_linux::app::shortcuts`
//! (`shortcuts()`). `--help` prints it, the in-app `Ctrl+Shift+?` reference
//! renders it, and README §11 mirrors the same `keys` strings. This file
//! deliberately holds no second table.

use std::path::PathBuf;

use gpui::{
    prelude::*, px, size, App, Application, Bounds, WindowBackgroundAppearance, WindowBounds,
    WindowDecorations, WindowOptions,
};
use relayable_linux::api::ApiHandle;
use relayable_linux::app::WorkspaceView;
use relayable_linux::geometry;

/// Parsed command-line options.
struct Options {
    mock: bool,
    socket: PathBuf,
    canvas: (f32, f32),
}

impl Options {
    fn parse() -> Self {
        let mut mock = false;
        let mut socket: Option<PathBuf> = std::env::var_os("RELAYABLE_SERVER").map(PathBuf::from);
        let mut canvas = geometry::BASELINE_CANVAS_OUTER;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--mock" => mock = true,
                "--live" => mock = false,
                "--socket" => {
                    if let Some(value) = args.next() {
                        socket = Some(PathBuf::from(value));
                    }
                }
                "--canvas" => {
                    let width = args.next().and_then(|value| value.parse::<f32>().ok());
                    let height = args.next().and_then(|value| value.parse::<f32>().ok());
                    if let (Some(width), Some(height)) = (width, height) {
                        canvas = (width, height);
                    } else {
                        eprintln!("relayable: --canvas needs two numbers W H; ignoring");
                    }
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => eprintln!("relayable: ignoring unknown argument {other:?}"),
            }
        }
        if let Ok(spec) = std::env::var("RELAYABLE_CANVAS") {
            let split = spec.split_once('x').or_else(|| spec.split_once('X'));
            if let Some((width, height)) = split {
                if let (Ok(width), Ok(height)) = (width.trim().parse(), height.trim().parse()) {
                    canvas = (width, height);
                } else {
                    eprintln!("relayable: RELAYABLE_CANVAS={spec:?} is not WxH; ignoring");
                }
            }
        }
        let socket = socket.unwrap_or_else(relayable_protocol::transport::default_socket_path);
        Self {
            mock,
            socket,
            canvas,
        }
    }
}

fn print_help() {
    println!(
        "relayable — Relayable reference workspace\n\n\
         USAGE:\n  relayable [--mock] [--socket PATH] [--canvas W H]\n\n\
         OPTIONS:\n\
         \x20 --mock            use clearly-labelled synthetic data instead of a live server\n\
         \x20 --live            force live mode (default)\n\
         \x20 --socket PATH     server socket (default: $RELAYABLE_SERVER or the protocol default)\n\
         \x20 --canvas W H      initial window size (default: {}x{}; or RELAYABLE_CANVAS=WxH)",
        geometry::BASELINE_CANVAS_OUTER.0, geometry::BASELINE_CANVAS_OUTER.1
    );
    // One definition only: the rows come from `app::shortcuts`, the same table
    // the in-app `Ctrl+Shift+?` reference renders and README §11 mirrors, so
    // `--help` cannot drift from the handler.
    println!("\nKEYBINDINGS (Relayable-original; the reference showed macOS only):");
    for shortcut in relayable_linux::app::shortcuts::shortcuts() {
        println!("  {:<32} {}", shortcut.keys, shortcut.label);
    }
}

fn main() {
    let options = Options::parse();
    if options.mock {
        eprintln!("relayable: MOCK mode — synthetic in-process data, no live server");
    } else {
        eprintln!(
            "relayable: live mode — socket = {}",
            options.socket.display()
        );
    }

    let (width, height) = options.canvas;
    let mock = options.mock;
    let socket = options.socket;

    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(width), px(height)), cx);
        let result = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                focus: true,
                show: true,
                app_id: Some("relayable".to_string()),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Relayable".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |window, cx| {
                // The `WindowOptions.titlebar` title is macOS-only in gpui 0.2.2;
                // the xdg_toplevel title goes through `set_window_title`.
                window.set_window_title("Relayable");
                // gpui 0.2.2's Linux backend creates an ARGB surface and treats
                // client-side decorations as transparent
                // (`platform/linux/wayland/window.rs`: `is_transparent()` is true
                // when `decorations == Client`). Hyprland draws no server
                // decorations, so requesting them makes the surface opaque
                // without adding a titlebar; the appearance is set explicitly
                // too. Without this the desktop shows faintly through the ramp.
                window.request_decorations(WindowDecorations::Server);
                window.set_background_appearance(WindowBackgroundAppearance::Opaque);
                let api = if mock {
                    ApiHandle::connect_mock()
                } else {
                    ApiHandle::connect_live(socket)
                };
                let view = cx.new(|cx| WorkspaceView::new(api, cx));
                window.focus(&view.read(cx).focus_handle());
                view
            },
        );
        if let Err(error) = result {
            eprintln!("relayable: failed to open the workspace window: {error}");
            std::process::exit(1);
        }
        cx.activate(true);
    });
}
