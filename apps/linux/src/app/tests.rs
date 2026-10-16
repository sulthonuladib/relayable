use crate::api::ConnectionState;
use gpui::{Keystroke, Modifiers};

use super::helpers::{
    abandon_create, choose_font_family, disambiguate_labels, empty_state_message, needs_session,
    panes_needing_attach, shorten_home,
};
use super::keys::terminal_bytes;
use super::state::{CreateIntent, CreateKind, Pane, SplitAxis, Tab};

fn keystroke(key: &str, key_char: Option<&str>) -> Keystroke {
    Keystroke {
        modifiers: Modifiers::default(),
        key: key.to_string(),
        key_char: key_char.map(str::to_string),
    }
}

#[test]
fn duplicate_tab_labels_get_the_reference_suffix() {
    let labels = disambiguate_labels(vec![
        "~".to_string(),
        "~/Documents/ghostty".to_string(),
        "~/Documents/ghostty".to_string(),
        "~/Documents/ghostty".to_string(),
    ]);
    assert_eq!(
        labels,
        vec![
            "~",
            "~/Documents/ghostty",
            "~/Documents/ghostty-1",
            "~/Documents/ghostty-2"
        ]
    );
}

#[test]
fn home_prefix_is_shortened_to_tilde() {
    assert_eq!(shorten_home("/home/me", "/home/me"), "~");
    assert_eq!(shorten_home("/home/me/project", "/home/me"), "~/project");
    assert_eq!(shorten_home("/tmp", "/home/me"), "/tmp");
}

#[test]
fn terminal_bytes_maps_special_keys() {
    assert_eq!(
        terminal_bytes(&keystroke("enter", None)),
        Some(b"\r".to_vec())
    );
    assert_eq!(
        terminal_bytes(&keystroke("backspace", None)),
        Some(vec![0x7f])
    );
    assert_eq!(
        terminal_bytes(&keystroke("up", None)),
        Some(b"\x1b[A".to_vec())
    );
    assert_eq!(
        terminal_bytes(&keystroke("a", Some("a"))),
        Some(b"a".to_vec())
    );
}

#[test]
fn control_letters_become_control_bytes() {
    let mut ks = keystroke("c", Some("c"));
    ks.modifiers.control = true;
    assert_eq!(terminal_bytes(&ks), Some(vec![0x03]));
}

#[test]
fn control_shift_is_reserved_for_the_app() {
    let mut ks = keystroke("e", Some("e"));
    ks.modifiers.control = true;
    ks.modifiers.shift = true;
    assert_eq!(terminal_bytes(&ks), None);
}

#[test]
fn font_family_is_always_chosen_from_installed_names() {
    let installed = vec!["Noto Sans Mono".to_string(), "Arial".to_string()];
    assert_eq!(choose_font_family(&installed), "Noto Sans Mono");
    let preferred = vec!["JetBrainsMono Nerd Font".to_string(), "Arial".to_string()];
    assert_eq!(choose_font_family(&preferred), "JetBrainsMono Nerd Font");
    // Otherwise any monospace-looking installed name.
    let quirky = vec!["Arial".to_string(), "Iosevka Term Mono".to_string()];
    assert_eq!(choose_font_family(&quirky), "Iosevka Term Mono");
    // Then the first installed name (never the literal "monospace").
    let no_mono = vec!["Arial".to_string(), "Comic Sans".to_string()];
    assert_eq!(choose_font_family(&no_mono), "Arial");
    // Empty list falls back to the gpui alias that its fallback stack maps.
    assert_eq!(choose_font_family(&[]), ".ZedMono");
}

#[test]
fn reconnect_targets_only_panes_without_a_live_stream() {
    let mut attached = Pane::new(1, "session:a".to_string(), "block:a".to_string(), None);
    attached.stream_id = Some(9);
    let detached = Pane::new(2, "session:b".to_string(), "block:b".to_string(), None);
    let tabs = vec![Tab {
        panes: vec![attached, detached],
        focus: 1,
        axis: SplitAxis::Horizontal,
    }];
    assert_eq!(panes_needing_attach(&tabs), vec![2]);
    assert_eq!(panes_needing_attach(&[]), Vec::<u64>::new());
}

// --- UI-004 stage 1: the auto-session invariant (DEC-019) -------------------

/// Headless model of the view's auto-session wiring.
///
/// `app/tests.rs` is gpui-free, so `WorkspaceView` (which needs an `ApiHandle`
/// and a `FocusHandle`) cannot be built here. This probe mirrors the call sites
/// — the `Connected`/`Error`/`Disconnected` arms in `events.rs` and
/// `close_focused`/`create_session` in `tabs.rs` — and drives them through the
/// **real** `needs_session` and `abandon_create` decisions, so a regression in
/// either shows up as a wrong `creates_sent` below. The call sites themselves
/// are not executed here (no display, no gpui context).
struct AutoSessionProbe {
    connection: ConnectionState,
    tabs_empty: bool,
    pending_create: Option<CreateIntent>,
    creates_sent: u32,
}

impl AutoSessionProbe {
    /// A fresh view: connecting, no tabs, nothing in flight.
    fn new() -> Self {
        Self {
            connection: ConnectionState::Connecting,
            tabs_empty: true,
            pending_create: None,
            creates_sent: 0,
        }
    }

    /// `ApiEvent::ConnectionStateChanged`: the view records the state, and the
    /// `Connected` arm runs the invariant.
    fn connection_state_changed(&mut self, state: ConnectionState) {
        self.connection = state;
        self.ensure_session();
    }

    /// `WorkspaceView::ensure_session` + `request_new_tab`.
    fn ensure_session(&mut self) {
        if needs_session(
            &self.connection,
            self.tabs_empty,
            self.pending_create.is_some(),
        ) {
            self.creates_sent += 1;
            self.pending_create = Some(CreateIntent {
                kind: CreateKind::NewTab,
                label: None,
            });
        }
    }

    /// `ApiEvent::SessionCreated`: the reply that builds the tab.
    fn session_created(&mut self) {
        self.pending_create = None;
        self.tabs_empty = false;
    }

    /// `tabs.rs::close_focused` closing the last pane of the last tab.
    fn close_last_tab(&mut self) {
        self.tabs_empty = true;
        self.ensure_session();
    }

    /// `ApiEvent::Error`: state first, then re-arm, then the invariant.
    fn error(&mut self, message: &str) {
        self.connection = ConnectionState::Error(message.to_string());
        self.abandon_pending_create();
        self.ensure_session();
    }

    /// `ApiEvent::Disconnected`: an in-flight create is void.
    fn disconnected(&mut self) {
        self.connection = ConnectionState::Disconnected;
        self.abandon_pending_create();
    }

    /// `tabs.rs::create_session` as reached by `Ctrl+Shift+N`: it only refuses
    /// while a create is already in flight.
    fn manual_new_tab(&mut self) -> bool {
        if self.pending_create.is_some() {
            return false;
        }
        self.creates_sent += 1;
        self.pending_create = Some(CreateIntent {
            kind: CreateKind::NewTab,
            label: None,
        });
        true
    }

    /// `WorkspaceView::abandon_pending_create` — the real re-arm rule.
    fn abandon_pending_create(&mut self) -> bool {
        abandon_create(&mut self.pending_create)
    }
}

#[test]
fn zero_tabs_and_connected_requests_a_session_exactly_once() {
    let mut probe = AutoSessionProbe::new();
    probe.connection_state_changed(ConnectionState::Connected);
    assert_eq!(
        probe.creates_sent, 1,
        "the Connected edge must request one session"
    );
    assert!(
        probe.pending_create.is_some(),
        "the request must be in flight"
    );
    // A repeated `Connected`, and a reconnect sweep that lands back on
    // `Connected`, must not stack a second request.
    probe.connection_state_changed(ConnectionState::Reconnecting { attempt: 1 });
    probe.connection_state_changed(ConnectionState::Connected);
    probe.connection_state_changed(ConnectionState::Connected);
    assert_eq!(probe.creates_sent, 1);
    // Once the session exists, no further request is made.
    probe.session_created();
    for _ in 0..3 {
        probe.connection_state_changed(ConnectionState::Connected);
    }
    assert_eq!(probe.creates_sent, 1);
}

#[test]
fn closing_the_last_tab_requests_a_session_again() {
    let mut probe = AutoSessionProbe::new();
    probe.connection_state_changed(ConnectionState::Connected);
    probe.session_created();
    assert!(!probe.tabs_empty);
    probe.close_last_tab();
    assert_eq!(
        probe.creates_sent, 2,
        "closing the last tab must re-request a session, without a manual `new`"
    );
    assert!(probe.pending_create.is_some());
}

#[test]
fn closing_the_last_tab_while_not_connected_does_not_request_a_session() {
    let mut probe = AutoSessionProbe::new();
    probe.connection_state_changed(ConnectionState::Connected);
    probe.session_created();
    probe.connection_state_changed(ConnectionState::Disconnected);
    probe.close_last_tab();
    assert_eq!(
        probe.creates_sent, 1,
        "offline, the invariant must wait for a Connected edge"
    );
}

#[test]
fn a_failed_create_re_arms_the_manual_shortcut_and_the_next_connect_edge() {
    let mut probe = AutoSessionProbe::new();
    probe.connection_state_changed(ConnectionState::Connected);
    assert_eq!(probe.creates_sent, 1);
    // `session.create` fails: the API answers with `ApiEvent::Error`.
    probe.error("session create unavailable while disconnected");
    assert!(
        probe.pending_create.is_none(),
        "a failed create must clear the in-flight intent"
    );
    assert_eq!(
        probe.creates_sent, 1,
        "a failure must not retry immediately (no create loop)"
    );
    // The manual shortcut works again...
    assert!(
        probe.manual_new_tab(),
        "Ctrl+Shift+N must work after a failed create"
    );
    // ...and the next `Connected` edge retries exactly once.
    probe.error("session.create failed");
    probe.connection_state_changed(ConnectionState::Connected);
    let after_retry = probe.creates_sent;
    probe.connection_state_changed(ConnectionState::Connected);
    assert_eq!(probe.creates_sent, after_retry);
    assert_eq!(probe.creates_sent, 3);
}

#[test]
fn a_disconnect_voids_an_in_flight_create() {
    let mut probe = AutoSessionProbe::new();
    probe.connection_state_changed(ConnectionState::Connected);
    probe.disconnected();
    assert!(
        probe.pending_create.is_none(),
        "a create in flight when the transport dies is never answered"
    );
    probe.connection_state_changed(ConnectionState::Connected);
    assert_eq!(
        probe.creates_sent, 2,
        "the next Connected edge must create again"
    );
}

#[test]
fn the_invariant_never_fires_while_not_connected() {
    assert!(needs_session(&ConnectionState::Connected, true, false));
    assert!(!needs_session(&ConnectionState::Connected, true, true));
    assert!(!needs_session(&ConnectionState::Connected, false, false));
    for state in [
        ConnectionState::Connecting,
        ConnectionState::Reconnecting { attempt: 2 },
        ConnectionState::Disconnected,
        ConnectionState::Error("boom".to_string()),
    ] {
        assert!(
            !needs_session(&state, true, false),
            "{state:?} must not start a create"
        );
    }
}

#[test]
fn empty_state_messages_are_distinct_and_truthful() {
    let creating = empty_state_message(&ConnectionState::Connected, true);
    let idle = empty_state_message(&ConnectionState::Connected, false);
    assert!(creating.contains("creating one"));
    assert!(
        !idle.contains("creating"),
        "the idle empty surface must not claim a create is in flight: {idle}"
    );
    assert!(
        idle.contains("Ctrl+Shift+N"),
        "the idle surface must say how to create one: {idle}"
    );

    let messages = [
        creating,
        idle,
        empty_state_message(&ConnectionState::Connecting, false),
        empty_state_message(&ConnectionState::Reconnecting { attempt: 1 }, false),
        empty_state_message(&ConnectionState::Disconnected, false),
        empty_state_message(&ConnectionState::Error("boom".to_string()), false),
    ];
    for (index, first) in messages.iter().enumerate() {
        for second in messages.iter().skip(index + 1) {
            assert_ne!(first, second, "two states share one message: {first}");
        }
    }
    // The attempt counter is not surfaced in the copy (the top bar owns it).
    assert_eq!(
        empty_state_message(&ConnectionState::Reconnecting { attempt: 1 }, false),
        empty_state_message(&ConnectionState::Reconnecting { attempt: 3 }, false)
    );
}
