use gpui::{Keystroke, Modifiers};

use super::helpers::{choose_font_family, disambiguate_labels, panes_needing_attach, shorten_home};
use super::keys::terminal_bytes;
use super::state::{Pane, SplitAxis, Tab};

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
