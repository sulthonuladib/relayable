use crate::api::ApiCommand;
use gpui::{KeyDownEvent, Keystroke, Window};

use super::shortcuts::{route_key, tab_index, KeyRoute, ShortcutId};
use super::state::SplitAxis;
use super::WorkspaceView;

impl WorkspaceView {
    // --- keyboard ----------------------------------------------------------

    /// Route a keystroke and run the action it selects.
    ///
    /// The decision itself lives in `shortcuts.rs` ([`route_key`]), which owns
    /// the keymap, the overlay gates and the `Esc` contract; this function only
    /// maps an action id onto a method. Dispatch is by [`ShortcutId`], never by
    /// re-matching key strings, so the keymap has exactly one definition and a
    /// new binding cannot be handled in one place and documented in another.
    pub(super) fn handle_key(&mut self, event: &KeyDownEvent, _window: &mut Window) {
        let keystroke = &event.keystroke;
        match route_key(self.picker_open, self.shortcuts_open, keystroke) {
            KeyRoute::Picker => self.handle_picker_key(keystroke),
            KeyRoute::CloseShortcuts => self.set_shortcuts_open(false),
            // The overlay is modal: an unhandled key goes nowhere, exactly like
            // an unhandled key in the session picker.
            KeyRoute::SwallowShortcuts => {}
            KeyRoute::Shortcut(id) => self.handle_shortcut(id, keystroke),
            KeyRoute::Terminal => {
                if let Some(bytes) = terminal_bytes(keystroke) {
                    if let Some(stream_id) = self.focused_stream() {
                        self.api.send(ApiCommand::Input { stream_id, bytes });
                    }
                }
            }
        }
    }

    /// Run the action a [`ShortcutId`] names.
    ///
    /// Exhaustive by construction: the compiler rejects a new [`ShortcutId`]
    /// that is not handled here, which is the one guarantee a display-free test
    /// cannot make about this table.
    fn handle_shortcut(&mut self, id: ShortcutId, keystroke: &Keystroke) {
        match id {
            ShortcutId::SplitRight => self.split_focused(SplitAxis::Horizontal),
            ShortcutId::SplitDown => self.split_focused(SplitAxis::Vertical),
            ShortcutId::ClosePane => self.close_focused(),
            ShortcutId::ToggleZoom => self.zoomed = !self.zoomed,
            ShortcutId::NewSession => self.request_new_tab(),
            ShortcutId::SessionPicker => {
                self.picker_open = true;
                self.picker_query.clear();
                self.picker_selected = 0;
                self.refresh_sessions();
            }
            ShortcutId::NextTab => self.next_tab(),
            ShortcutId::PrevTab => self.prev_tab(),
            ShortcutId::SelectTab => {
                // The digit rule is `shortcuts::tab_index`, shared with the
                // predicate that matched this key.
                if let Some(index) = tab_index(keystroke) {
                    self.select_tab(index);
                }
            }
            ShortcutId::FocusNextPane => self.focus_next_pane(),
            ShortcutId::FocusPrevPane => self.focus_prev_pane(),
            ShortcutId::ToggleShortcuts => self.set_shortcuts_open(true),
        }
    }

    fn handle_picker_key(&mut self, keystroke: &Keystroke) {
        match keystroke.key.as_str() {
            "escape" => {
                self.picker_open = false;
                return;
            }
            "enter" => {
                self.picker_accept();
                return;
            }
            "up" => {
                self.picker_selected = self.picker_selected.saturating_sub(1);
                return;
            }
            "down" => {
                self.picker_selected = self.picker_selected.saturating_add(1);
                return;
            }
            "backspace" => {
                self.picker_query.pop();
                return;
            }
            _ => {}
        }
        if let Some(ch) = keystroke
            .key_char
            .as_deref()
            .filter(|c| !c.is_empty() && !keystroke.modifiers.control)
        {
            self.picker_query.push_str(ch);
        }
    }
}

/// Map a keystroke to terminal input bytes.
///
/// This is standard terminal behaviour (not reference UI), so it is
/// Relayable-original. `Ctrl+Shift+<key>` combos are reserved for the app's own
/// bindings and never reach the terminal.
pub(super) fn terminal_bytes(keystroke: &Keystroke) -> Option<Vec<u8>> {
    if keystroke.modifiers.platform {
        return None;
    }
    if keystroke.modifiers.control && keystroke.modifiers.shift {
        return None;
    }
    match keystroke.key.as_str() {
        "enter" => return Some(b"\r".to_vec()),
        "backspace" => return Some(vec![0x7f]),
        "tab" => return Some(b"\t".to_vec()),
        "escape" => return Some(vec![0x1b]),
        "up" => return Some(b"\x1b[A".to_vec()),
        "down" => return Some(b"\x1b[B".to_vec()),
        "right" => return Some(b"\x1b[C".to_vec()),
        "left" => return Some(b"\x1b[D".to_vec()),
        "home" => return Some(b"\x1b[H".to_vec()),
        "end" => return Some(b"\x1b[F".to_vec()),
        "delete" => return Some(b"\x1b[3~".to_vec()),
        "pageup" => return Some(b"\x1b[5~".to_vec()),
        "pagedown" => return Some(b"\x1b[6~".to_vec()),
        _ => {}
    }
    let text = keystroke.key_char.as_deref().filter(|c| !c.is_empty())?;
    if keystroke.modifiers.control {
        let ch = text.chars().next()?;
        if ch.is_ascii_alphabetic() {
            return Some(vec![(ch.to_ascii_lowercase() as u8) - b'a' + 1]);
        }
        return None;
    }
    if keystroke.modifiers.alt {
        let mut bytes = vec![0x1b];
        bytes.extend_from_slice(text.as_bytes());
        return Some(bytes);
    }
    Some(text.as_bytes().to_vec())
}
