use std::collections::HashMap;

use crate::theme;

use super::state::Tab;

/// Pane ids that have no live stream and therefore must be (re-)attached.
///
/// Pure so the reconnect policy can be unit-tested without a gpui context.
pub(super) fn panes_needing_attach(tabs: &[Tab]) -> Vec<u64> {
    tabs.iter()
        .flat_map(|tab| tab.panes.iter())
        .filter(|pane| pane.stream_id.is_none())
        .map(|pane| pane.pane_id)
        .collect()
}

/// Choose a font family that is guaranteed to be present in `all_font_names`.
///
/// Preference order first; then any installed name that looks monospace; then
/// the first installed name. Only when the list is empty (gpui in practice
/// appends its fallback stack, so this is defensive) fall back to gpui's own
/// `".ZedMono"` alias, which the crate maps into its fallback stack
/// (`gpui-0.2.2/src/text_system.rs:908-918`). Never returns an unvalidated
/// literal such as `"monospace"`, which `TextSystem::resolve_font` would panic
/// on if no fallback resolved.
pub(super) fn choose_font_family(names: &[String]) -> String {
    for candidate in theme::FONT_FAMILY_PREFERENCE {
        if names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(candidate))
        {
            return (*candidate).to_string();
        }
    }
    if let Some(name) = names
        .iter()
        .find(|name| name.to_ascii_lowercase().contains("mono"))
    {
        return name.clone();
    }
    if let Some(name) = names.first() {
        return name.clone();
    }
    ".ZedMono".to_string()
}

/// Drop a `$HOME` prefix to `~` (the reference showed `~`, not a full path).
pub(super) fn shorten_home(path: &str, home: &str) -> String {
    if !home.is_empty() && path == home {
        return "~".to_string();
    }
    if !home.is_empty() {
        if let Some(rest) = path.strip_prefix(home) {
            if rest.starts_with('/') {
                return format!("~{rest}");
            }
        }
    }
    path.to_string()
}

/// Apply the reference's duplicate-label rule: second occurrence gets `-1`,
/// third `-2` (medium confidence, V1 1:33).
pub(super) fn disambiguate_labels(labels: Vec<String>) -> Vec<String> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut out = Vec::with_capacity(labels.len());
    for label in labels {
        let count = seen.entry(label.clone()).or_insert(0);
        if *count == 0 {
            out.push(label);
        } else {
            out.push(format!("{label}-{}", *count));
        }
        *count += 1;
    }
    out
}
