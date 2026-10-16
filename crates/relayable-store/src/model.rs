//! Public data types for the store: row states, audit kinds, create/close
//! commands, read models, and pruning limits.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    DEFAULT_AUDIT_EVENTS_MAX_AGE_MILLIS, DEFAULT_AUDIT_EVENTS_MAX_ROWS,
    DEFAULT_SESSIONS_MAX_AGE_MILLIS, DEFAULT_SESSIONS_MAX_ROWS, DEFAULT_TOMBSTONES_MAX_AGE_MILLIS,
    DEFAULT_TOMBSTONES_MAX_ROWS,
};

/// Wall-clock milliseconds since the Unix epoch, or `0` if the clock is before
/// the epoch.
pub(crate) fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// Store-local session row state.
///
/// `Interrupted` exists **only** in the DB. The wire/model `SessionState` has no
/// such variant (the frozen proto is unchanged); a row is marked `Interrupted`
/// at startup reconciliation because the process that owned it is gone
/// (`ADR-004` D2.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredSessionState {
    /// Owned by the current server run.
    Active,
    /// Closed by an explicit `session.close`.
    Closed,
    /// Left `active` by a previous run; never silently reported as live.
    Interrupted,
}

impl StoredSessionState {
    /// Stable DB representation stored in `sessions.state`.
    pub fn as_str(&self) -> &'static str {
        match self {
            StoredSessionState::Active => "active",
            StoredSessionState::Closed => "closed",
            StoredSessionState::Interrupted => "interrupted",
        }
    }

    /// Parse a `sessions.state` value. Returns `None` for unknown strings.
    // The name is frozen by the store API; it is intentionally not `FromStr`.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(StoredSessionState::Active),
            "closed" => Some(StoredSessionState::Closed),
            "interrupted" => Some(StoredSessionState::Interrupted),
            _ => None,
        }
    }
}

/// Kind of audit event recorded in `audit_events.event_kind`.
///
/// `Pruned` is **reserved**: it exists so audit readers already recognize the
/// value, but [`crate::Store::prune`] itself does **not** append an audit row.
/// The deleted counts it returns are exactly the rows removed from each table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditEventKind {
    /// A session row was created (or reactivated) as `active`.
    SessionCreated,
    /// A session row was closed and a tombstone written.
    SessionClosed,
    /// An `active` row from a previous run was marked `interrupted` at startup.
    SessionInterrupted,
    /// Reserved; not currently emitted (see the type docs).
    Pruned,
}

impl AuditEventKind {
    /// Stable DB representation stored in `audit_events.event_kind`.
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditEventKind::SessionCreated => "session_created",
            AuditEventKind::SessionClosed => "session_closed",
            AuditEventKind::SessionInterrupted => "session_interrupted",
            AuditEventKind::Pruned => "pruned",
        }
    }
}

/// Parse an `audit_events.event_kind` value. Private: the frozen public API
/// exposes only [`AuditEventKind::as_str`].
pub(crate) fn audit_kind_from_str(s: &str) -> Option<AuditEventKind> {
    match s {
        "session_created" => Some(AuditEventKind::SessionCreated),
        "session_closed" => Some(AuditEventKind::SessionClosed),
        "session_interrupted" => Some(AuditEventKind::SessionInterrupted),
        "pruned" => Some(AuditEventKind::Pruned),
        _ => None,
    }
}

/// Input for [`crate::Store::record_session_created`].
#[derive(Debug, Clone)]
pub struct SessionCreate {
    /// Host-independent session identifier (UUID string).
    pub session_id: String,
    /// Optional user-visible label.
    pub label: Option<String>,
    /// Creation time in milliseconds since the Unix epoch.
    pub created_at_millis: i64,
    /// Last known protocol revision for the session.
    pub revision: i64,
}

/// Input for [`crate::Store::record_session_closed`].
#[derive(Debug, Clone)]
pub struct SessionClose {
    /// Session identifier to close.
    pub session_id: String,
    /// Close time in milliseconds since the Unix epoch.
    pub closed_at_millis: i64,
    /// Last known protocol revision for the session.
    pub revision: i64,
}

/// A persisted session definition (not live process state).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredSession {
    pub session_id: String,
    pub label: Option<String>,
    pub created_at_millis: i64,
    pub closed_at_millis: Option<i64>,
    pub state: StoredSessionState,
    pub last_known_revision: i64,
}

/// A destroy record. `destroy_sequence` is `MAX(destroy_sequence)+1` at write
/// time, computed inside the close transaction (`COALESCE(MAX(...),0)+1`). It is
/// **not** globally monotonic across pruning: once the highest-sequence rows are
/// pruned the next write can reuse a lower sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tombstone {
    pub session_id: String,
    pub destroy_sequence: i64,
    pub timestamp_millis: i64,
}

/// One row of the store-local audit trail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    /// Store-local audit order (`AUTOINCREMENT`), not the protocol `sequence`.
    pub sequence: i64,
    pub kind: AuditEventKind,
    pub session_id: Option<String>,
    pub timestamp_millis: i64,
    pub detail: Option<String>,
}

/// Retention limits for [`crate::Store::prune`].
///
/// Cutoffs are `now_millis - <max_age_millis>`; a row strictly older than the
/// cutoff is deleted. Row limits keep the newest rows by the table's monotonic
/// order (`audit_events.sequence` / `tombstones.destroy_sequence`) or, for
/// sessions, by `created_at_millis` within the non-active set. A non-positive
/// row limit keeps no rows.
#[derive(Debug, Clone)]
pub struct PruneLimits {
    /// "Current" time used to compute all age cutoffs.
    pub now_millis: i64,
    /// Maximum `audit_events` rows to keep.
    pub audit_max_rows: i64,
    /// Maximum age of an `audit_events` row, in milliseconds.
    pub audit_max_age_millis: i64,
    /// Maximum `tombstones` rows to keep.
    pub tombstones_max_rows: i64,
    /// Maximum age of a `tombstones` row, in milliseconds.
    pub tombstones_max_age_millis: i64,
    /// Maximum `sessions` rows to keep in the non-active set
    /// (`closed`/`interrupted`), newest by `created_at_millis`.
    pub sessions_max_rows: i64,
    /// Maximum age of a `closed`/`interrupted` session row, in milliseconds.
    /// The age is measured from `closed_at_millis` when present, else
    /// `created_at_millis`.
    pub sessions_max_age_millis: i64,
}

impl PruneLimits {
    /// Defaults with `now_millis` taken from the system clock.
    pub fn defaults() -> Self {
        Self::with_now(now_millis())
    }

    /// Default retention values with an explicit `now_millis` (for hermetic
    /// tests and deterministic callers).
    pub fn with_now(now_millis: i64) -> Self {
        Self {
            now_millis,
            audit_max_rows: DEFAULT_AUDIT_EVENTS_MAX_ROWS,
            audit_max_age_millis: DEFAULT_AUDIT_EVENTS_MAX_AGE_MILLIS,
            tombstones_max_rows: DEFAULT_TOMBSTONES_MAX_ROWS,
            tombstones_max_age_millis: DEFAULT_TOMBSTONES_MAX_AGE_MILLIS,
            sessions_max_rows: DEFAULT_SESSIONS_MAX_ROWS,
            sessions_max_age_millis: DEFAULT_SESSIONS_MAX_AGE_MILLIS,
        }
    }

    /// Build limits from the `RELAYABLE_STORE_*` environment variables.
    ///
    /// Names (implementation-chosen under the ADR-005 `RELAYABLE_*` namespace;
    /// documented in `docs/operations/persistence.md`):
    /// `RELAYABLE_STORE_AUDIT_MAX_ROWS`,
    /// `RELAYABLE_STORE_AUDIT_MAX_AGE_MILLIS`,
    /// `RELAYABLE_STORE_TOMBSTONES_MAX_ROWS`,
    /// `RELAYABLE_STORE_TOMBSTONES_MAX_AGE_MILLIS`,
    /// `RELAYABLE_STORE_SESSIONS_MAX_ROWS`,
    /// `RELAYABLE_STORE_SESSIONS_MAX_AGE_MILLIS`.
    ///
    /// Unset, empty, or unparsable values fall back to the default.
    pub fn from_env() -> Self {
        Self::from_lookup(|name| std::env::var(name).ok(), now_millis())
    }

    /// Pure variant of [`PruneLimits::from_env`] for hermetic tests.
    fn from_lookup<F: Fn(&str) -> Option<String>>(lookup: F, now_millis: i64) -> Self {
        let value = |name: &str| {
            lookup(name)
                .map(|raw| raw.trim().to_string())
                .filter(|raw| !raw.is_empty())
                .and_then(|raw| raw.parse::<i64>().ok())
        };
        Self {
            now_millis,
            audit_max_rows: value("RELAYABLE_STORE_AUDIT_MAX_ROWS")
                .unwrap_or(DEFAULT_AUDIT_EVENTS_MAX_ROWS),
            audit_max_age_millis: value("RELAYABLE_STORE_AUDIT_MAX_AGE_MILLIS")
                .unwrap_or(DEFAULT_AUDIT_EVENTS_MAX_AGE_MILLIS),
            tombstones_max_rows: value("RELAYABLE_STORE_TOMBSTONES_MAX_ROWS")
                .unwrap_or(DEFAULT_TOMBSTONES_MAX_ROWS),
            tombstones_max_age_millis: value("RELAYABLE_STORE_TOMBSTONES_MAX_AGE_MILLIS")
                .unwrap_or(DEFAULT_TOMBSTONES_MAX_AGE_MILLIS),
            sessions_max_rows: value("RELAYABLE_STORE_SESSIONS_MAX_ROWS")
                .unwrap_or(DEFAULT_SESSIONS_MAX_ROWS),
            sessions_max_age_millis: value("RELAYABLE_STORE_SESSIONS_MAX_AGE_MILLIS")
                .unwrap_or(DEFAULT_SESSIONS_MAX_AGE_MILLIS),
        }
    }
}

/// Exact number of rows removed by [`crate::Store::prune`], per table.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PruneReport {
    pub audit_events_deleted: u64,
    pub tombstones_deleted: u64,
    pub sessions_deleted: u64,
}

/// Number of `active` rows marked `interrupted` by
/// [`crate::Store::reconcile_on_startup`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReconcileReport {
    pub interrupted: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits_from(pairs: &[(&str, &str)]) -> PruneLimits {
        let map: std::collections::HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        PruneLimits::from_lookup(|name| map.get(name).cloned(), 1234)
    }

    #[test]
    fn from_env_falls_back_to_defaults() {
        let limits = limits_from(&[]);
        assert_eq!(limits.now_millis, 1234);
        assert_eq!(limits.audit_max_rows, DEFAULT_AUDIT_EVENTS_MAX_ROWS);
        assert_eq!(
            limits.audit_max_age_millis,
            DEFAULT_AUDIT_EVENTS_MAX_AGE_MILLIS
        );
        assert_eq!(limits.tombstones_max_rows, DEFAULT_TOMBSTONES_MAX_ROWS);
        assert_eq!(
            limits.tombstones_max_age_millis,
            DEFAULT_TOMBSTONES_MAX_AGE_MILLIS
        );
        assert_eq!(limits.sessions_max_rows, DEFAULT_SESSIONS_MAX_ROWS);
        assert_eq!(
            limits.sessions_max_age_millis,
            DEFAULT_SESSIONS_MAX_AGE_MILLIS
        );
    }

    #[test]
    fn from_env_reads_overrides_and_ignores_garbage() {
        let limits = limits_from(&[
            ("RELAYABLE_STORE_AUDIT_MAX_ROWS", " 25 "),
            ("RELAYABLE_STORE_AUDIT_MAX_AGE_MILLIS", ""),
            ("RELAYABLE_STORE_TOMBSTONES_MAX_ROWS", "not-a-number"),
            ("RELAYABLE_STORE_TOMBSTONES_MAX_AGE_MILLIS", "77"),
            ("RELAYABLE_STORE_SESSIONS_MAX_ROWS", "9"),
            ("RELAYABLE_STORE_SESSIONS_MAX_AGE_MILLIS", "99"),
        ]);
        assert_eq!(limits.audit_max_rows, 25);
        assert_eq!(
            limits.audit_max_age_millis,
            DEFAULT_AUDIT_EVENTS_MAX_AGE_MILLIS
        );
        assert_eq!(limits.tombstones_max_rows, DEFAULT_TOMBSTONES_MAX_ROWS);
        assert_eq!(limits.tombstones_max_age_millis, 77);
        assert_eq!(limits.sessions_max_rows, 9);
        assert_eq!(limits.sessions_max_age_millis, 99);
    }

    #[test]
    fn session_state_round_trips() {
        for state in [
            StoredSessionState::Active,
            StoredSessionState::Closed,
            StoredSessionState::Interrupted,
        ] {
            assert_eq!(StoredSessionState::from_str(state.as_str()), Some(state));
        }
        assert_eq!(StoredSessionState::from_str("nonsense"), None);
    }

    #[test]
    fn audit_kind_round_trips_privately() {
        for kind in [
            AuditEventKind::SessionCreated,
            AuditEventKind::SessionClosed,
            AuditEventKind::SessionInterrupted,
            AuditEventKind::Pruned,
        ] {
            assert_eq!(audit_kind_from_str(kind.as_str()), Some(kind));
        }
        assert_eq!(audit_kind_from_str("nonsense"), None);
    }
}
