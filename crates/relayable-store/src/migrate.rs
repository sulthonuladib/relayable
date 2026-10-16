//! Schema versioning: numbered SQL migrations recorded in `schema_migrations`.

use rusqlite::{params, Connection};

use crate::{model::now_millis, StoreError};

/// Highest schema version this build understands.
pub const LATEST_SCHEMA_VERSION: u32 = 1;

/// Migration 1: initial schema.
///
/// Host-independent by construction: no pids, file descriptors, hostnames,
/// peer credentials, host identity, terminal bytes, environment values, or
/// command arguments (`ADR-004` D2.3; `full.md` §11:770, §12:804).
///
/// `tombstones` has **no** foreign key to `sessions`: a destroy record outlives
/// the session row it describes, so pruning a closed session must not delete its
/// tombstone (`ADR-004` D2.3, `ADR-001:286-291`).
const MIGRATION_1_SQL: &str = r#"
CREATE TABLE sessions (
  session_id TEXT PRIMARY KEY NOT NULL,
  label TEXT,
  created_at_millis INTEGER NOT NULL,
  closed_at_millis INTEGER,
  state TEXT NOT NULL CHECK (state IN ('active','closed','interrupted')),
  last_known_revision INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX sessions_state_idx ON sessions (state);
CREATE TABLE tombstones (
  session_id TEXT PRIMARY KEY NOT NULL,
  destroy_sequence INTEGER NOT NULL,
  timestamp_millis INTEGER NOT NULL
);
CREATE INDEX tombstones_timestamp_idx ON tombstones (timestamp_millis);
CREATE TABLE audit_events (
  sequence INTEGER PRIMARY KEY AUTOINCREMENT,
  event_kind TEXT NOT NULL,
  session_id TEXT,
  timestamp_millis INTEGER NOT NULL,
  detail TEXT
);
CREATE INDEX audit_events_timestamp_idx ON audit_events (timestamp_millis);
CREATE INDEX audit_events_kind_idx ON audit_events (event_kind);
"#;

/// Numbered migrations, applied in order, each inside its own transaction.
pub(crate) const MIGRATIONS: &[(u32, &str)] = &[(1, MIGRATION_1_SQL)];

/// Bootstrap the migration table and apply every pending migration.
///
/// Idempotent: already-applied versions are skipped. Rejects a database whose
/// recorded version is newer than [`LATEST_SCHEMA_VERSION`].
pub(crate) fn migrate(conn: &mut Connection) -> Result<(), StoreError> {
    // Bootstrapped, not versioned, so it must exist before the version is read.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            applied_at_millis INTEGER NOT NULL
        );",
    )?;

    let current = read_version(conn)?;
    if current > LATEST_SCHEMA_VERSION {
        return Err(StoreError::SchemaTooNew {
            found: current,
            supported: LATEST_SCHEMA_VERSION,
        });
    }

    for (version, sql) in MIGRATIONS {
        if *version <= current {
            continue;
        }
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at_millis) VALUES (?1, ?2)",
            params![version, now_millis()],
        )?;
        tx.commit()?;
    }

    Ok(())
}

/// Read the highest applied schema version (`0` for an empty database).
pub(crate) fn read_version(conn: &Connection) -> Result<u32, StoreError> {
    let current: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;
    Ok(u32::try_from(current).unwrap_or(0))
}
