use crate::api::ApiCommand;
use gpui::{KeyDownEvent, Keystroke, Window};

use super::state::SplitAxis;
use super::WorkspaceView;

impl WorkspaceView {
    // --- keyboard ----------------------------------------------------------

    pub(super) fn handle_key(&mut self, event: &KeyDownEvent, _window: &mut Window) {
        let keystroke = &event.keystroke;
        if self.picker_open {
            self.handle_picker_key(keystroke);
            return;
        }

        let control = keystroke.modifiers.control;
        let shift = keystroke.modifiers.shift;
        if control && shift {
            match keystroke.key.as_str() {
                "e" => return self.split_focused(SplitAxis::Horizontal),
                "o" => return self.split_focused(SplitAxis::Vertical),
                "w" => return self.close_focused(),
                "z" => {
                    self.zoomed = !self.zoomed;
                    return;
                }
                "s" => {
                    self.picker_open = true;
                    self.picker_query.clear();
                    self.picker_selected = 0;
                    self.refresh_sessions();
                    return;
                }
                "n" => return self.request_new_tab(),
                "right" => return self.focus_next_pane(),
                "left" => return self.focus_prev_pane(),
                "t" => return self.next_tab(),
                _ => {}
            }
        }
        if control && !shift {
            match keystroke.key.as_str() {
                "pagedown" => return self.next_tab(),
                "pageup" => return self.prev_tab(),
                _ => {}
            }
            if let Some(digit) = keystroke.key.chars().next().and_then(|c| c.to_digit(10)) {
                if digit >= 1 {
                    self.select_tab(digit as usize - 1);
                    return;
                }
            }
        }

        if let Some(bytes) = terminal_bytes(keystroke) {
            if let Some(stream_id) = self.focused_stream() {
                self.api.send(ApiCommand::Input { stream_id, bytes });
            }
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
