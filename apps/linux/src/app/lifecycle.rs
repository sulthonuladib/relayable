use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use crate::api::{ApiCommand, ApiHandle, ConnectionState};
use crate::geometry;
use crate::theme;
use gpui::{px, Context, FocusHandle};

use super::helpers::choose_font_family;
use super::WorkspaceView;

/// Drained-event poll interval (foreground task). 16 ms ≈ 60 fps.
const DRAIN_INTERVAL: Duration = Duration::from_millis(16);

/// Upper bound on events applied in one frame so a flooding server cannot make a
/// single frame unbounded; the remainder is left for the next 16 ms tick.
const MAX_EVENTS_PER_FRAME: usize = 256;

impl WorkspaceView {
    /// Build the workspace view around `api` and start the event drain loop.
    pub fn new(api: ApiHandle, cx: &mut Context<Self>) -> Self {
        let text_system = cx.text_system();
        let names = text_system.all_font_names();
        let family = choose_font_family(&names);
        let font = gpui::font(family.clone());
        let font_id = text_system.resolve_font(&font);
        let font_size = px(theme::TERMINAL_FONT_SIZE_PROPOSED);
        let cell_width = text_system
            .ch_advance(font_id, font_size)
            .map(f32::from)
            .unwrap_or(8.0)
            .max(1.0);
        let row_pitch = geometry::v1::TERMINAL_ROW_PITCH.value;
        let is_mock = api.is_mock();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        let focus_handle = cx.focus_handle();

        eprintln!(
            "relayable: font chosen={family:?} (of {} installed families) cell_width={cell_width:.3} row_pitch={row_pitch} font_size={}",
            names.len(),
            f32::from(font_size)
        );

        let mut view = Self {
            api,
            is_mock,
            connection: ConnectionState::Connecting,
            tabs: Vec::new(),
            active_tab: 0,
            next_pane_id: 1,
            sessions: Vec::new(),
            pending_create: None,
            pending_attach: VecDeque::new(),
            last_title_stream: None,
            stream_output_bytes: HashMap::new(),
            status: Vec::new(),
            font,
            font_size,
            cell_width,
            row_pitch,
            home,
            window_width: geometry::BASELINE_CANVAS_OUTER.0,
            window_height: geometry::BASELINE_CANVAS_OUTER.1,
            focus_handle,
            picker_open: false,
            picker_query: String::new(),
            picker_selected: 0,
            zoomed: false,
            logged_geometry: false,
        };
        view.spawn_drain(cx);
        view.register_shutdown(cx);
        view
    }

    /// Best-effort `Shutdown` so the API worker threads stop on app quit.
    fn register_shutdown(&mut self, cx: &mut Context<Self>) {
        let subscription = cx.on_app_quit(|view, _cx| {
            view.api.send(ApiCommand::Shutdown);
            async {}
        });
        subscription.detach();
    }

    /// The root keyboard focus handle (main focuses it when the window opens).
    #[must_use]
    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    fn spawn_drain(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| loop {
            cx.background_executor().timer(DRAIN_INTERVAL).await;
            let alive = this.update(cx, |view, cx| {
                // Bounded per frame: a flooding server cannot make one frame
                // unbounded. The remainder stays queued for the next tick.
                let mut events = Vec::new();
                for _ in 0..MAX_EVENTS_PER_FRAME {
                    match view.api.try_recv() {
                        Some(event) => events.push(event),
                        None => break,
                    }
                }
                if !events.is_empty() {
                    view.apply(events);
                    cx.notify();
                }
            });
            if alive.is_err() {
                break;
            }
        })
        .detach();
    }
}
