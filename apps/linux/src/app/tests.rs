use crate::api::ConnectionState;
use gpui::{Keystroke, Modifiers};

use super::helpers::{
    abandon_create, choose_font_family, disambiguate_labels, empty_state_message, needs_session,
    panes_needing_attach, shorten_home,
};
use super::keys::terminal_bytes;
use super::shortcuts::{
    keys_for, route_key, shortcut_for, shortcuts, tab_index, KeyRoute, ShortcutId, OVERLAY_FOOTER,
    OVERLAY_TITLE,
};
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

// --- UI-005: single-source keymap + shortcut reference ----------------------

/// A keystroke with the given modifiers, in the spelling gpui's Linux backend
/// produces for it.
fn chord(key: &str, key_char: Option<&str>, control: bool, shift: bool) -> Keystroke {
    let mut ks = keystroke(key, key_char);
    ks.modifiers.control = control;
    ks.modifiers.shift = shift;
    ks
}

/// Every workspace shortcut as the **platform** shapes it.
///
/// gpui's Linux backend reports a shifted letter lowercased (`key == "e"` with
/// `shift` still set) and reports shift+`/` as `key == "?"` with `shift`
/// **cleared** (`gpui-0.2.2/src/platform/linux/platform.rs:910-917`), which is
/// why the toggle row carries `shift: false`. `Ctrl+Shift+?` was driven live on
/// this host (UI-005 evidence notes) and produced exactly that shape.
fn platform_shapes() -> Vec<(Keystroke, ShortcutId)> {
    let mut shapes = vec![
        (chord("e", Some("E"), true, true), ShortcutId::SplitRight),
        (chord("o", Some("O"), true, true), ShortcutId::SplitDown),
        (chord("w", Some("W"), true, true), ShortcutId::ClosePane),
        (chord("z", Some("Z"), true, true), ShortcutId::ToggleZoom),
        (chord("n", Some("N"), true, true), ShortcutId::NewSession),
        (chord("s", Some("S"), true, true), ShortcutId::SessionPicker),
        (chord("t", Some("T"), true, true), ShortcutId::NextTab),
        (chord("pagedown", None, true, false), ShortcutId::NextTab),
        (chord("right", None, true, true), ShortcutId::FocusNextPane),
        (chord("left", None, true, true), ShortcutId::FocusPrevPane),
        (chord("pageup", None, true, false), ShortcutId::PrevTab),
        (
            chord("?", Some("?"), true, false),
            ShortcutId::ToggleShortcuts,
        ),
    ];
    for digit in 1..=9u8 {
        let digit = digit.to_string();
        shapes.push((
            chord(&digit, Some(digit.as_str()), true, false),
            ShortcutId::SelectTab,
        ));
    }
    shapes
}

/// Every id in `shortcuts()` must be reachable from a platform keystroke, and
/// every platform keystroke must resolve to its id through the router.
///
/// Catches: a predicate typo in `shortcut_for` (`"E"` for `"e"`, a dropped arm,
/// a shifted `Ctrl+PageDown`); a row added to the table with no predicate; a
/// binding removed from the handler but left in the table.
#[test]
fn every_shortcut_id_is_reachable_from_its_platform_keystroke() {
    let reachable: Vec<ShortcutId> = platform_shapes().iter().map(|(_, id)| *id).collect();
    for (ks, expected) in platform_shapes() {
        assert_eq!(
            shortcut_for(&ks),
            Some(expected),
            "control={} shift={} key={:?} must fire {expected:?}",
            ks.modifiers.control,
            ks.modifiers.shift,
            ks.key
        );
        assert_eq!(
            route_key(false, false, &ks),
            KeyRoute::Shortcut(expected),
            "the router must not send {expected:?} to the pane"
        );
    }
    for shortcut in shortcuts() {
        assert!(
            reachable.contains(&shortcut.id),
            "no platform keystroke produces {:?} ({}); the table and the handler have drifted",
            shortcut.id,
            shortcut.keys
        );
    }
}

/// The table itself: one row per id, one row per chord string, and the id
/// lookup returns the row's own chord.
///
/// Catches: two rows claiming the same `keys` (copy-paste in the table), the
/// same id (a second spelling of one action), and a broken lookup (which would
/// make the overlay's open/close log name the wrong chord).
#[test]
fn the_keymap_table_has_no_duplicate_row() {
    let mut keys: Vec<&'static str> = Vec::new();
    let mut ids: Vec<ShortcutId> = Vec::new();
    for shortcut in shortcuts() {
        assert!(
            !keys.contains(&shortcut.keys),
            "duplicate keys {:?} in the keymap",
            shortcut.keys
        );
        assert!(
            !ids.contains(&shortcut.id),
            "duplicate id {:?} in the keymap",
            shortcut.id
        );
        assert_eq!(
            keys_for(shortcut.id),
            shortcut.keys,
            "the lookup for {:?} must return its own row",
            shortcut.id
        );
        keys.push(shortcut.keys);
        ids.push(shortcut.id);
    }
    // The table is the documented surface: it must list the toggle that opens
    // it, and the overlay must say it is Relayable-original.
    assert!(
        keys.contains(&"Ctrl+Shift+?"),
        "the reference must document how to open it"
    );
    assert!(OVERLAY_TITLE.contains("Relayable-original"));
    assert!(OVERLAY_FOOTER.contains("Esc"));
}

/// README §11 of the app documents the same chords, verbatim, as **table rows**.
///
/// The section is sliced out of the file, so a row that exists elsewhere in the
/// README does not satisfy the check, and each chord must appear in the
/// `| \`chord\`` form the table uses — prose that merely mentions it does not.
/// Catches: editing a chord in the table without the docs (`Ctrl+Shift+O` →
/// `Ctrl+Shift+Shift+O`), or deleting a row from either side. The mutation
/// record in `evidence/ui-005-notes.md` is the proof that both fail.
#[test]
fn readme_lists_every_shortcut_verbatim() {
    let readme = include_str!("../../README.md");
    let start = readme
        .find("## 11. Keybindings")
        .expect("README §11 must exist");
    let section = &readme[start..];
    let end = section.find("\n## 12.").unwrap_or(section.len());
    let section = &section[..end];
    for shortcut in shortcuts() {
        let row = format!("| `{}`", shortcut.keys);
        assert!(
            section.contains(&row),
            "README §11 ({} chars) has no table row for {:?} ({})",
            section.len(),
            row,
            shortcut.label
        );
    }
    assert!(section.contains("Relayable-original"));
}

/// `Ctrl+1..9` selects tabs; `Ctrl+0` never did and the `Ctrl+Shift` namespace
/// stays reserved.
///
/// Catches: an off-by-one in the digit→index rule (`Ctrl+1` selecting the
/// second tab), and a widening of the rule to `Ctrl+0` or `Ctrl+Shift+<digit>`.
#[test]
fn control_digits_select_tabs_and_only_one_to_nine() {
    for digit in 1..=9u8 {
        let key = digit.to_string();
        let ks = chord(&key, Some(key.as_str()), true, false);
        assert_eq!(tab_index(&ks), Some(usize::from(digit) - 1));
        assert_eq!(shortcut_for(&ks), Some(ShortcutId::SelectTab));
    }
    assert_eq!(tab_index(&chord("0", Some("0"), true, false)), None);
    assert_eq!(shortcut_for(&chord("0", Some("0"), true, false)), None);
    assert_eq!(tab_index(&chord("1", Some("1"), true, true)), None);
    assert_eq!(shortcut_for(&chord("1", Some("1"), true, true)), None);
    assert_eq!(tab_index(&keystroke("1", Some("1"))), None);
    assert_eq!(shortcut_for(&keystroke("1", Some("1"))), None);
}

/// The toggle is `Ctrl+Shift+?` and nothing else: a bare `?` is terminal input,
/// and no `Ctrl+Shift+?` byte ever reaches the pane.
///
/// Catches: the stolen-character deviation (binding a bare `?`, which would make
/// the `assert_eq!(shortcut_for(&bare), None)` fail) and a toggle that leaks a
/// byte into the pane.
#[test]
fn the_toggle_key_never_reaches_the_pane_and_a_bare_question_mark_does() {
    let toggle = chord("?", Some("?"), true, false);
    assert_eq!(shortcut_for(&toggle), Some(ShortcutId::ToggleShortcuts));
    assert_eq!(
        terminal_bytes(&toggle),
        None,
        "the toggle must not leak a byte"
    );

    let bare = keystroke("?", Some("?"));
    assert_eq!(shortcut_for(&bare), None);
    assert_eq!(route_key(false, false, &bare), KeyRoute::Terminal);
    assert_eq!(terminal_bytes(&bare), Some(b"?".to_vec()));
}

/// The overlay is modal: the toggle and `Esc` close it, every other chord is
/// swallowed, and the picker gate wins over it.
///
/// Catches: an overlay that lets `Ctrl+Shift+E` split a pane while the reference
/// is open, one that closes on any key, `Esc` closing it while it is *closed*
/// (which would swallow the pane's `0x1b`), and the two overlays opening
/// together.
#[test]
fn the_overlay_is_modal_and_escape_only_closes_it_while_it_is_open() {
    let toggle = chord("?", Some("?"), true, false);
    let escape = keystroke("escape", None);
    let plain_e = keystroke("e", Some("e"));

    // Overlay closed: the workspace behaves exactly as before UI-005.
    assert_eq!(
        route_key(false, false, &toggle),
        KeyRoute::Shortcut(ShortcutId::ToggleShortcuts)
    );
    assert_eq!(
        route_key(false, false, &chord("e", Some("E"), true, true)),
        KeyRoute::Shortcut(ShortcutId::SplitRight)
    );
    assert_eq!(route_key(false, false, &escape), KeyRoute::Terminal);
    assert_eq!(route_key(false, false, &plain_e), KeyRoute::Terminal);

    // Overlay open: closed by the toggle or `Esc`, everything else swallowed.
    assert_eq!(route_key(false, true, &toggle), KeyRoute::CloseShortcuts);
    assert_eq!(route_key(false, true, &escape), KeyRoute::CloseShortcuts);
    assert_eq!(route_key(false, true, &plain_e), KeyRoute::SwallowShortcuts);
    assert_eq!(
        route_key(false, true, &keystroke("?", Some("?"))),
        KeyRoute::SwallowShortcuts
    );
    for (ks, id) in platform_shapes() {
        if id == ShortcutId::ToggleShortcuts {
            continue;
        }
        assert_eq!(
            route_key(false, true, &ks),
            KeyRoute::SwallowShortcuts,
            "{id:?} must be swallowed while the reference is open"
        );
    }

    // The picker gate runs first: the two overlays cannot be open together.
    assert_eq!(route_key(true, false, &toggle), KeyRoute::Picker);
    assert_eq!(
        route_key(true, false, &chord("e", Some("E"), true, true)),
        KeyRoute::Picker
    );
    assert_eq!(route_key(true, true, &escape), KeyRoute::Picker);
}

/// The pane's own keys are untouched by the keymap change: `Esc` is still
/// `0x1b`, a bare `?` and plain letters/digits still type, arrows still move,
/// and no `Ctrl+Shift` chord reaches the pane.
///
/// Catches: a terminal-byte regression introduced while adding the toggle (for
/// example `escape` no longer mapping to `0x1b`, or the reserved namespace
/// leaking a letter as a control byte).
#[test]
fn terminal_bytes_still_maps_the_panes_own_keys() {
    assert_eq!(terminal_bytes(&keystroke("escape", None)), Some(vec![0x1b]));
    assert_eq!(
        terminal_bytes(&keystroke("right", None)),
        Some(b"\x1b[C".to_vec())
    );
    assert_eq!(
        terminal_bytes(&keystroke("2", Some("2"))),
        Some(b"2".to_vec())
    );
    assert_eq!(
        terminal_bytes(&keystroke("?", Some("?"))),
        Some(b"?".to_vec())
    );
    assert_eq!(
        terminal_bytes(&keystroke("e", Some("e"))),
        Some(b"e".to_vec())
    );
    for key in ["e", "o", "w", "z", "s", "n", "t", "right", "left"] {
        assert_eq!(
            terminal_bytes(&chord(key, None, true, true)),
            None,
            "Ctrl+Shift+{key} is reserved for the app"
        );
    }
}
