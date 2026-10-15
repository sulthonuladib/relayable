//! Session target resolution.
//!
//! Target resolution is owned by `relayable-cli`, never by `relayable-model`
//! (contract §1). The documented algorithm (matrix §6) is first-match-wins:
//!
//! 1. full ID, such as `session:01a10cf3-2655-7e48-9b9c-43dd6612bef5`;
//! 2. case-insensitive label;
//! 3. 1-based position within `relayable ls` order;
//! 4. ID suffix of at least four characters.
//!
//! An ambiguous target must stop without mutating anything and exit 3
//! (matrix §6), which is what [`ResolveError::Ambiguous`] signals.

use relayable_model::SessionId;
use relayable_protocol::v1::SessionSummary;

/// One session listed as an ambiguity match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionMatch {
    /// The full session id.
    pub session_id: String,
    /// The mutable human label, when present.
    pub label: Option<String>,
}

/// Why a session target could not be resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// No target was given and there are no sessions to default to.
    Missing,
    /// The target matched nothing.
    NotFound(String),
    /// The target matched more than one session; nothing may be mutated.
    Ambiguous {
        /// The target that was ambiguous, or `None` when no target was given
        /// and several sessions exist.
        target: Option<String>,
        /// The members of the match, in `relayable ls` order.
        matches: Vec<SessionMatch>,
    },
}

/// Resolve `target` (or the single default session) against `sessions`.
///
/// # Errors
///
/// See [`ResolveError`].
pub fn resolve_session(
    target: Option<&str>,
    sessions: &[SessionSummary],
) -> Result<SessionSummary, ResolveError> {
    let Some(target) = target else {
        return match sessions {
            [] => Err(ResolveError::Missing),
            [only] => Ok(only.clone()),
            _ => Err(ResolveError::Ambiguous {
                target: None,
                matches: to_matches(sessions.iter()),
            }),
        };
    };

    // 1. Full ID.
    if let Ok(id) = target.parse::<SessionId>() {
        let wanted = id.to_string();
        if let Some(found) = sessions.iter().find(|s| s.session_id == wanted) {
            return Ok(found.clone());
        }
    }

    // 2. Case-insensitive label.
    let by_label: Vec<&SessionSummary> = sessions
        .iter()
        .filter(|s| {
            s.label
                .as_deref()
                .is_some_and(|label| label.eq_ignore_ascii_case(target))
        })
        .collect();
    match only(by_label) {
        Match::One(found) => return Ok(found.clone()),
        Match::Many(found) => {
            return Err(ResolveError::Ambiguous {
                target: Some(target.to_string()),
                matches: to_matches(found.into_iter()),
            });
        }
        Match::None => {}
    }

    // 3. 1-based position in listing order.
    if let Ok(position) = target.parse::<usize>() {
        if position >= 1 && position <= sessions.len() {
            return Ok(sessions[position - 1].clone());
        }
    }

    // 4. ID suffix of at least four characters.
    if target.len() >= 4 {
        let by_suffix: Vec<&SessionSummary> = sessions
            .iter()
            .filter(|s| s.session_id.ends_with(target))
            .collect();
        match only(by_suffix) {
            Match::One(found) => return Ok(found.clone()),
            Match::Many(found) => {
                return Err(ResolveError::Ambiguous {
                    target: Some(target.to_string()),
                    matches: to_matches(found.into_iter()),
                });
            }
            Match::None => {}
        }
    }

    Err(ResolveError::NotFound(target.to_string()))
}

/// The trailing characters of a full ID, used for ambiguity listings
/// (matrix §6: "one line per match with short IDs").
#[must_use]
pub fn short_id(id: &str) -> &str {
    const SHORT: usize = 6;
    if id.len() > SHORT {
        &id[id.len() - SHORT..]
    } else {
        id
    }
}

enum Match<'a> {
    None,
    One(&'a SessionSummary),
    Many(Vec<&'a SessionSummary>),
}

fn only(candidates: Vec<&SessionSummary>) -> Match<'_> {
    match candidates.len() {
        0 => Match::None,
        1 => Match::One(candidates[0]),
        _ => Match::Many(candidates),
    }
}

fn to_matches<'a>(sessions: impl Iterator<Item = &'a SessionSummary>) -> Vec<SessionMatch> {
    sessions
        .map(|s| SessionMatch {
            session_id: s.session_id.clone(),
            label: s.label.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use relayable_protocol::v1::SessionState;

    fn summary(id: &str, label: Option<&str>) -> SessionSummary {
        SessionSummary {
            session_id: id.to_string(),
            label: label.map(str::to_owned),
            state: SessionState::Active as i32,
            revision: 1,
        }
    }

    fn sessions() -> Vec<SessionSummary> {
        vec![
            summary(
                "session:01a10cf3-2655-7e48-9b9c-43dd6612bef5",
                Some("Research"),
            ),
            summary(
                "session:01a10d97-aaaa-7bbb-8ccc-43dd6612be11",
                Some("SL Website"),
            ),
            summary("session:01a10d99-cccc-7ddd-8eee-43dd66120099", None),
        ]
    }

    #[test]
    fn resolves_a_full_id() {
        let resolved = resolve_session(
            Some("session:01a10cf3-2655-7e48-9b9c-43dd6612bef5"),
            &sessions(),
        )
        .unwrap();
        assert_eq!(resolved.label.as_deref(), Some("Research"));
    }

    #[test]
    fn resolves_a_label_case_insensitively() {
        let resolved = resolve_session(Some("sl website"), &sessions()).unwrap();
        assert_eq!(
            resolved.session_id,
            "session:01a10d97-aaaa-7bbb-8ccc-43dd6612be11"
        );
    }

    #[test]
    fn resolves_a_position() {
        let resolved = resolve_session(Some("2"), &sessions()).unwrap();
        assert_eq!(resolved.label.as_deref(), Some("SL Website"));
    }

    #[test]
    fn resolves_a_four_character_suffix() {
        let resolved = resolve_session(Some("be11"), &sessions()).unwrap();
        assert_eq!(
            resolved.session_id,
            "session:01a10d97-aaaa-7bbb-8ccc-43dd6612be11"
        );
    }

    #[test]
    fn an_ambiguous_label_stops_without_mutation() {
        let sessions = vec![
            summary("session:01a10cf3-2655-7e48-9b9c-43dd6612be01", Some("work")),
            summary("session:01a10d97-aaaa-7bbb-8ccc-43dd6612be02", Some("WORK")),
        ];
        let err = resolve_session(Some("work"), &sessions).unwrap_err();
        match err {
            ResolveError::Ambiguous { target, matches } => {
                assert_eq!(target.as_deref(), Some("work"));
                assert_eq!(matches.len(), 2);
            }
            other => panic!("expected ambiguity, got {other:?}"),
        }
    }

    #[test]
    fn no_match_is_not_found() {
        assert_eq!(
            resolve_session(Some("does-not-exist"), &sessions()),
            Err(ResolveError::NotFound("does-not-exist".to_string()))
        );
    }

    #[test]
    fn short_targets_do_not_suffix_match() {
        // `be1` is only three characters, so rule 4 does not apply.
        assert_eq!(
            resolve_session(Some("be1"), &sessions()),
            Err(ResolveError::NotFound("be1".to_string()))
        );
    }

    #[test]
    fn no_target_defaults_to_the_only_session() {
        let one = vec![summary(
            "session:01a10cf3-2655-7e48-9b9c-43dd6612bef5",
            None,
        )];
        assert!(resolve_session(None, &one).is_ok());
        assert_eq!(resolve_session(None, &[]), Err(ResolveError::Missing));
        assert!(matches!(
            resolve_session(None, &sessions()),
            Err(ResolveError::Ambiguous { target: None, .. })
        ));
    }

    #[test]
    fn full_id_wins_over_other_matches() {
        // The full ID also ends with `2bef5`; rule 1 must win.
        let resolved = resolve_session(
            Some("session:01a10cf3-2655-7e48-9b9c-43dd6612bef5"),
            &sessions(),
        )
        .unwrap();
        assert_eq!(resolved.label.as_deref(), Some("Research"));
    }

    #[test]
    fn short_id_is_the_trailing_six_characters() {
        assert_eq!(
            short_id("session:01a10cf3-2655-7e48-9b9c-43dd6612bef5"),
            "12bef5"
        );
    }
}
