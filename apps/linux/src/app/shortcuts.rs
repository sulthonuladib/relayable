//! The workspace keymap: **one** source of truth for the keys the app handles.
//!
//! Before UI-005 the same rows existed three times — the `match` in
//! `keys.rs::handle_key`, the literal table in `main.rs::print_help`, and
//! `README.md` §11 — with nothing linking them. Everything now derives from
//! [`shortcuts`], and [`route_key`] is the single predicate layer that
//! `handle_key` switches on, so a binding cannot drift between the handler, the
//! help text and the docs without `app/tests.rs` failing.
//!
//! # Relayable-original
//!
//! The reference videos are macOS-only and never show either these chords or a
//! shortcut overlay (`docs/research/open-questions.md` Q4;
//! `docs/research/ui-catalog.md` has **no** catalogue entry for a shortcut
//! overlay), so the bindings and the overlay are a documented Relayable-original
//! deviation. The overlay is **not** the V4 command palette: that surface is
//! unimplemented, has its own measured geometry (442×667 px), and none of it is
//! reused here.
//!
//! # Overlay state
//!
//! The shortcut overlay and the session picker are **mutually exclusive**: the
//! picker gate runs first in [`route_key`], and the overlay can only be opened
//! by [`KeyRoute::Shortcut`], which [`route_key`] never returns while the picker
//! is open. While the overlay is open it is modal — only the toggle and `Esc`
//! are handled and every other keystroke is swallowed, the same contract as
//! `keys.rs::handle_picker_key`. `Esc` closes the overlay **only** while it is
//! open; in the workspace `Esc` still reaches the pane as `0x1b`
//! (`keys.rs::terminal_bytes`).

use gpui::Keystroke;

use super::WorkspaceView;

/// A workspace action that a key chord fires.
///
/// `SelectTab` deliberately carries no index: one table row stands for the
/// `Ctrl+1..9` range, and the digit→index rule lives once in [`tab_index`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShortcutId {
    /// `Ctrl+Shift+E`.
    SplitRight,
    /// `Ctrl+Shift+O`.
    SplitDown,
    /// `Ctrl+Shift+W`.
    ClosePane,
    /// `Ctrl+Shift+Z`.
    ToggleZoom,
    /// `Ctrl+Shift+N`.
    NewSession,
    /// `Ctrl+Shift+S`.
    SessionPicker,
    /// `Ctrl+Shift+T` / `Ctrl+PageDown`.
    NextTab,
    /// `Ctrl+PageUp`.
    PrevTab,
    /// `Ctrl+1..9`.
    SelectTab,
    /// `Ctrl+Shift+Right`.
    FocusNextPane,
    /// `Ctrl+Shift+Left`.
    FocusPrevPane,
    /// `Ctrl+Shift+?` — the toggle for the reference this table describes.
    ToggleShortcuts,
}

/// One row of the keymap: an action, the chord that fires it, and the label both
/// the overlay and `--help` display.
///
/// `keys`/`label` are `pub` because `src/main.rs` is a **separate crate** (the
/// binary target) and prints them in `--help`; the module and the id stay
/// internal to `app`.
pub struct Shortcut {
    pub(super) id: ShortcutId,
    /// The chord as displayed, e.g. `"Ctrl+Shift+E"`. The same string appears
    /// verbatim in `README.md` §11 (enforced by the drift test).
    pub keys: &'static str,
    /// The action as displayed.
    pub label: &'static str,
}

/// The keymap, in reference-table order (README §11 / `--help` / the overlay).
///
/// The last row is the overlay's own toggle, so the reference documents how to
/// close itself.
const SHORTCUTS: &[Shortcut] = &[
    Shortcut {
        id: ShortcutId::SplitRight,
        keys: "Ctrl+Shift+E",
        label: "split pane right",
    },
    Shortcut {
        id: ShortcutId::SplitDown,
        keys: "Ctrl+Shift+O",
        label: "split pane down",
    },
    Shortcut {
        id: ShortcutId::ClosePane,
        keys: "Ctrl+Shift+W",
        label: "close focused pane",
    },
    Shortcut {
        id: ShortcutId::ToggleZoom,
        keys: "Ctrl+Shift+Z",
        label: "zoom / unzoom focused pane",
    },
    Shortcut {
        id: ShortcutId::NewSession,
        keys: "Ctrl+Shift+N",
        label: "new session (new tab)",
    },
    Shortcut {
        id: ShortcutId::SessionPicker,
        keys: "Ctrl+Shift+S",
        label: "session picker",
    },
    Shortcut {
        id: ShortcutId::NextTab,
        keys: "Ctrl+Shift+T / Ctrl+PageDown",
        label: "next tab",
    },
    Shortcut {
        id: ShortcutId::PrevTab,
        keys: "Ctrl+PageUp",
        label: "previous tab",
    },
    Shortcut {
        id: ShortcutId::SelectTab,
        keys: "Ctrl+1..9",
        label: "select tab",
    },
    Shortcut {
        id: ShortcutId::FocusNextPane,
        keys: "Ctrl+Shift+Right",
        label: "focus next pane",
    },
    Shortcut {
        id: ShortcutId::FocusPrevPane,
        keys: "Ctrl+Shift+Left",
        label: "focus previous pane",
    },
    Shortcut {
        id: ShortcutId::ToggleShortcuts,
        keys: "Ctrl+Shift+?",
        label: "toggle this shortcut reference",
    },
];

/// The workspace keymap.
///
/// `keys` and `label` of every row are the displayed strings, so the overlay,
/// `--help` and `README.md` §11 cannot disagree.
#[must_use]
pub fn shortcuts() -> &'static [Shortcut] {
    SHORTCUTS
}

/// The displayed chord of an action, looked up in the table.
///
/// Used where a *displayed* chord must not be spelled a second time (the
/// overlay's open/close edge log). An id absent from the table is a bug the
/// tests catch; it renders as an empty chord rather than panicking.
#[must_use]
pub(super) fn keys_for(id: ShortcutId) -> &'static str {
    SHORTCUTS
        .iter()
        .find(|shortcut| shortcut.id == id)
        .map_or("", |shortcut| shortcut.keys)
}

/// The id a keystroke fires, or `None` when the key is not a binding.
///
/// Re-encodes the predicates the workspace has always used, with no additions:
/// the `Ctrl+Shift` namespace (`gpui` clears `shift` for shifted symbols — see
/// [`ShortcutId::ToggleShortcuts`]), and `Ctrl`-without-`Shift`
/// PageUp/PageDown/digits. Keys are matched in the spelling the Linux platform
/// reports (`"pagedown"`, `"right"`, `"?"`), never in their displayed form.
#[must_use]
pub(super) fn shortcut_for(keystroke: &Keystroke) -> Option<ShortcutId> {
    let modifiers = &keystroke.modifiers;
    let key = keystroke.key.as_str();
    if modifiers.control && modifiers.shift {
        return match key {
            "e" => Some(ShortcutId::SplitRight),
            "o" => Some(ShortcutId::SplitDown),
            "w" => Some(ShortcutId::ClosePane),
            "z" => Some(ShortcutId::ToggleZoom),
            "s" => Some(ShortcutId::SessionPicker),
            "n" => Some(ShortcutId::NewSession),
            "right" => Some(ShortcutId::FocusNextPane),
            "left" => Some(ShortcutId::FocusPrevPane),
            "t" => Some(ShortcutId::NextTab),
            _ => None,
        };
    }
    if modifiers.control && !modifiers.shift {
        match key {
            "pagedown" => return Some(ShortcutId::NextTab),
            "pageup" => return Some(ShortcutId::PrevTab),
            // `Ctrl+Shift+/`: gpui's Linux backend reports `key == "?"` and
            // **clears** `shift` for shift+`/`
            // (`gpui-0.2.2/src/platform/linux/platform.rs:910-917`), so "?" is
            // the whole predicate — requiring `shift` would never match, and a
            // bare `?` (no control) is excluded below because this arm needs
            // `control`. A literal bare `?` binding would steal a character
            // from the terminal and is deliberately not used.
            "?" => return Some(ShortcutId::ToggleShortcuts),
            _ => {}
        }
        if tab_index(keystroke).is_some() {
            return Some(ShortcutId::SelectTab);
        }
    }
    None
}

/// The tab a `Ctrl+<digit>` chord selects, 0-based (`Ctrl+1` → `0`).
///
/// The digit rule exists once and is shared with [`shortcut_for`]. Digits only
/// count under `Ctrl` without `Shift` (the `Ctrl+Shift` namespace is reserved for
/// the app's own bindings) and only 1–9 count, so `Ctrl+0` stays with the pane.
#[must_use]
pub(super) fn tab_index(keystroke: &Keystroke) -> Option<usize> {
    if !keystroke.modifiers.control || keystroke.modifiers.shift {
        return None;
    }
    let digit = keystroke.key.chars().next()?.to_digit(10)?;
    (digit >= 1).then(|| digit as usize - 1)
}

/// The key that closes the overlay besides its toggle, in the spelling gpui
/// reports for it.
const OVERLAY_CLOSE_KEY: &str = "escape";

/// Whether `keystroke` closes the open shortcut overlay.
///
/// That is the toggle and `Esc` — nothing else. In the workspace (overlay
/// closed) `Esc` is not a binding: [`route_key`] returns [`KeyRoute::Terminal`]
/// and the pane receives `0x1b`.
#[must_use]
pub(super) fn closes_overlay(keystroke: &Keystroke) -> bool {
    shortcut_for(keystroke) == Some(ShortcutId::ToggleShortcuts)
        || keystroke.key == OVERLAY_CLOSE_KEY
}

/// Which surface consumes a workspace keystroke.
///
/// This is the complete keyboard routing rule; `keys.rs::handle_key` switches on
/// exactly this value, so the routing is testable without a display.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum KeyRoute {
    /// The session picker is open and owns the keyboard (`handle_picker_key`).
    /// It wins over the shortcut overlay: the two surfaces are mutually
    /// exclusive, and this arm is the reason the overlay cannot open on top of
    /// the picker.
    Picker,
    /// The shortcut overlay is open and this key closes it.
    CloseShortcuts,
    /// The shortcut overlay is open and swallowed this key (it is modal).
    SwallowShortcuts,
    /// A workspace shortcut fires; the id names the action.
    Shortcut(ShortcutId),
    /// No binding: the keystroke belongs to the focused pane as terminal input.
    Terminal,
}

/// Route a keystroke over the workspace's overlay state.
#[must_use]
pub(super) fn route_key(
    picker_open: bool,
    shortcuts_open: bool,
    keystroke: &Keystroke,
) -> KeyRoute {
    if picker_open {
        return KeyRoute::Picker;
    }
    if shortcuts_open {
        return if closes_overlay(keystroke) {
            KeyRoute::CloseShortcuts
        } else {
            KeyRoute::SwallowShortcuts
        };
    }
    match shortcut_for(keystroke) {
        Some(id) => KeyRoute::Shortcut(id),
        None => KeyRoute::Terminal,
    }
}

// --- overlay ---------------------------------------------------------------

impl WorkspaceView {
    /// Open or close the shortcut reference, logging the edge once per change.
    ///
    /// The overlay and the picker are mutually exclusive; [`route_key`] only
    /// yields [`KeyRoute::Shortcut`] — the sole caller with `open == true` —
    /// while the picker is closed, and the assertion states that invariant where
    /// it is enforced.
    pub(super) fn set_shortcuts_open(&mut self, open: bool) {
        debug_assert!(
            !open || !self.picker_open,
            "overlay and picker are exclusive"
        );
        if self.shortcuts_open == open {
            return;
        }
        self.shortcuts_open = open;
        eprintln!(
            "relayable: shortcut reference {} ({})",
            if open { "opened" } else { "closed" },
            keys_for(ShortcutId::ToggleShortcuts)
        );
    }
}
