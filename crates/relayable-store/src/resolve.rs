//! State-directory resolution (`full.md` §7.1, `ADR-004` D2.8).
//!
//! Resolution order is `RELAYABLE_STATE_DIR` → `$XDG_STATE_HOME/relayable` →
//! `$HOME/.local/state/relayable` → `None`. Empty values are treated as unset.
//! A set-but-non-absolute value (for any of the three) is **unresolvable**:
//! this function returns `None` rather than inventing a cwd-relative path.
//! `$HOME` and `$XDG_*` are never assumed to exist (`full.md` §7.1:519).

use std::path::PathBuf;

/// Which variable produced a resolved state directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateDirSource {
    /// `RELAYABLE_STATE_DIR` (highest precedence).
    RelayableStateDir,
    /// `$XDG_STATE_HOME/relayable`.
    XdgStateHome,
    /// `$HOME/.local/state/relayable`.
    HomeFallback,
}

/// Resolve the state directory from the process environment.
///
/// Returns `None` when unresolvable; the caller then runs without persistence
/// and should surface a warning (`docs/operations/persistence.md`).
pub fn resolve_state_dir() -> Option<(PathBuf, StateDirSource)> {
    resolve_state_dir_from(|name| std::env::var(name).ok())
}

/// Pure variant of [`resolve_state_dir`] for hermetic tests: `lookup` maps a
/// variable name to its value (`None` means unset or empty).
pub fn resolve_state_dir_from<F: Fn(&str) -> Option<String>>(
    lookup: F,
) -> Option<(PathBuf, StateDirSource)> {
    let non_empty = |name: &str| -> Option<PathBuf> {
        lookup(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    };

    if let Some(candidate) = non_empty("RELAYABLE_STATE_DIR") {
        return candidate
            .is_absolute()
            .then_some((candidate, StateDirSource::RelayableStateDir));
    }

    if let Some(base) = non_empty("XDG_STATE_HOME") {
        if !base.is_absolute() {
            return None;
        }
        return Some((base.join("relayable"), StateDirSource::XdgStateHome));
    }

    if let Some(home) = non_empty("HOME") {
        if !home.is_absolute() {
            return None;
        }
        return Some((
            home.join(".local/state/relayable"),
            StateDirSource::HomeFallback,
        ));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn resolve(pairs: &[(&str, &str)]) -> Option<(PathBuf, StateDirSource)> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        resolve_state_dir_from(|name| map.get(name).cloned())
    }

    #[test]
    fn relayable_state_dir_wins() {
        let resolved = resolve(&[
            ("RELAYABLE_STATE_DIR", "/srv/relayable"),
            ("XDG_STATE_HOME", "/home/u/.state"),
            ("HOME", "/home/u"),
        ]);
        assert_eq!(
            resolved,
            Some((
                PathBuf::from("/srv/relayable"),
                StateDirSource::RelayableStateDir
            ))
        );
    }

    #[test]
    fn xdg_state_home_fallback_appends_relayable() {
        let resolved = resolve(&[("XDG_STATE_HOME", "/home/u/.state"), ("HOME", "/home/u")]);
        assert_eq!(
            resolved,
            Some((
                PathBuf::from("/home/u/.state/relayable"),
                StateDirSource::XdgStateHome
            ))
        );
    }

    #[test]
    fn home_fallback_uses_local_state() {
        let resolved = resolve(&[("HOME", "/home/u")]);
        assert_eq!(
            resolved,
            Some((
                PathBuf::from("/home/u/.local/state/relayable"),
                StateDirSource::HomeFallback
            ))
        );
    }

    #[test]
    fn all_unset_is_unresolvable() {
        assert_eq!(resolve(&[]), None);
    }

    #[test]
    fn empty_values_are_ignored() {
        // Empty high-precedence values fall through to the next source.
        assert_eq!(
            resolve(&[
                ("RELAYABLE_STATE_DIR", ""),
                ("XDG_STATE_HOME", ""),
                ("HOME", "/home/u"),
            ]),
            Some((
                PathBuf::from("/home/u/.local/state/relayable"),
                StateDirSource::HomeFallback
            ))
        );
        // All empty is unresolvable.
        assert_eq!(
            resolve(&[
                ("RELAYABLE_STATE_DIR", ""),
                ("XDG_STATE_HOME", ""),
                ("HOME", ""),
            ]),
            None
        );
    }

    #[test]
    fn relative_values_are_unresolvable() {
        assert_eq!(resolve(&[("RELAYABLE_STATE_DIR", "relative/state")]), None);
        assert_eq!(
            resolve(&[("XDG_STATE_HOME", "relative/state"), ("HOME", "/home/u")]),
            None
        );
        assert_eq!(resolve(&[("HOME", "relative/home")]), None);
    }
}
