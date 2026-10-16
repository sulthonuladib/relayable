//! The [`Store`] itself: open/lock, pragmas, migrations, writes, reads, prune.

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};

use crate::error::StoreError;
use crate::migrate::{migrate, read_version};
use crate::model::{
    audit_kind_from_str, AuditEvent, AuditEventKind, PruneLimits, PruneReport, ReconcileReport,
    SessionClose, SessionCreate, StoredSession, StoredSessionState, Tombstone,
};

/// Durable, single-writer SQLite metadata/audit store.
///
/// The store owns one connection behind a `Mutex`; every write is one explicit
/// transaction. While a `Store` is alive it holds an OS file lock on
/// `<state_dir>/relayable-store.lock`, so a second `Store::open` on the same
/// directory fails with [`StoreError::StateDirLocked`].
pub struct Store {
    conn: Mutex<Connection>,
    /// `None` for [`Store::open_in_memory`]; otherwise held for the store's
    /// lifetime so the OS lock releases on drop.
    guard: Option<File>,
    state_dir: PathBuf,
    db_path: PathBuf,
}

/// Raw `sessions` row prior to typed parsing.
struct RawSession {
    session_id: String,
    label: Option<String>,
    created_at_millis: i64,
    closed_at_millis: Option<i64>,
    state: String,
    last_known_revision: i64,
}

/// Raw `audit_events` row prior to typed parsing.
struct RawAudit {
    sequence: i64,
    event_kind: String,
    session_id: Option<String>,
    timestamp_millis: i64,
    detail: Option<String>,
}

impl Store {
    /// Database file name inside the state directory.
    pub fn db_file_name() -> &'static str {
        "relayable.db"
    }

    /// Single-writer lock file name inside the state directory.
    pub fn lock_file_name() -> &'static str {
        "relayable-store.lock"
    }

    /// Conventional database path for a state directory.
    pub fn db_path(state_dir: &Path) -> PathBuf {
        state_dir.join(Self::db_file_name())
    }

    /// Open (creating if needed) the store under `state_dir`.
    ///
    /// The directory is created with mode `0700` on Unix, the single-writer lock
    /// is acquired **before** migrations, then pragmas are set and migrations
    /// applied. `state_dir` must be absolute.
    pub fn open(state_dir: &Path) -> Result<Store, StoreError> {
        if !state_dir.is_absolute() {
            return Err(StoreError::NonAbsoluteStateDir(state_dir.to_path_buf()));
        }
        create_state_dir(state_dir)?;

        let lock_path = state_dir.join(Self::lock_file_name());
        let guard = lock_state_dir(&lock_path, state_dir)?;

        let db_path = Self::db_path(state_dir);
        let mut conn = Connection::open(&db_path)?;
        configure_pragmas(&conn, false)?;
        migrate(&mut conn)?;

        Ok(Store {
            conn: Mutex::new(conn),
            guard: Some(guard),
            state_dir: state_dir.to_path_buf(),
            db_path,
        })
    }

    /// Open a throwaway in-memory store for unit tests.
    ///
    /// WAL is disabled (`journal_mode = MEMORY`), no lock file is created, and
    /// [`Store::checkpoint`] is a no-op.
    pub fn open_in_memory() -> Result<Store, StoreError> {
        let mut conn = Connection::open_in_memory()?;
        configure_pragmas(&conn, true)?;
        migrate(&mut conn)?;

        Ok(Store {
            conn: Mutex::new(conn),
            guard: None,
            state_dir: PathBuf::new(),
            db_path: PathBuf::new(),
        })
    }

    /// The state directory this store was opened with (empty for in-memory).
    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    /// The database file path (empty for in-memory).
    pub fn db_path_ref(&self) -> &Path {
        &self.db_path
    }

    /// Current applied schema version.
    pub fn schema_version(&self) -> Result<u32, StoreError> {
        read_version(&self.conn())
    }

    /// Mark every `active` row `interrupted` and append one
    /// `SessionInterrupted` audit event per changed row.
    ///
    /// Called once at startup: the process that owned an `active` row is gone
    /// (`ADR-004` D2.5). A second call is a no-op returning `interrupted == 0`.
    /// Closed rows are never touched.
    pub fn reconcile_on_startup(&self) -> Result<ReconcileReport, StoreError> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        let now = crate::model::now_millis();

        let changed: Vec<String> = {
            let mut stmt = tx.prepare("SELECT session_id FROM sessions WHERE state = 'active'")?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<rusqlite::Result<Vec<String>>>()?
        };

        let interrupted = tx.execute(
            "UPDATE sessions SET state = 'interrupted' WHERE state = 'active'",
            [],
        )?;

        for session_id in &changed {
            insert_audit(
                &tx,
                AuditEventKind::SessionInterrupted,
                Some(session_id),
                now,
                None,
            )?;
        }

        tx.commit()?;
        Ok(ReconcileReport {
            interrupted: interrupted as u64,
        })
    }

    /// Record a created (or reactivated) session and its audit event in one
    /// transaction.
    ///
    /// The upsert sets `state = 'active'` and clears any previous
    /// `closed_at_millis`; the audit event is timestamped with
    /// `create.created_at_millis` and carries the label as its detail.
    pub fn record_session_created(&self, create: &SessionCreate) -> Result<(), StoreError> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT INTO sessions \
                 (session_id, label, created_at_millis, closed_at_millis, state, last_known_revision) \
             VALUES (?1, ?2, ?3, NULL, 'active', ?4) \
             ON CONFLICT(session_id) DO UPDATE SET \
                 label = excluded.label, \
                 created_at_millis = excluded.created_at_millis, \
                 closed_at_millis = NULL, \
                 state = 'active', \
                 last_known_revision = excluded.last_known_revision",
            params![
                create.session_id,
                create.label,
                create.created_at_millis,
                create.revision
            ],
        )?;

        insert_audit(
            &tx,
            AuditEventKind::SessionCreated,
            Some(&create.session_id),
            create.created_at_millis,
            create.label.as_deref(),
        )?;

        tx.commit()?;
        Ok(())
    }

    /// Close a session, write its tombstone, and append the audit event in one
    /// transaction.
    ///
    /// Defensive: if no `sessions` row exists it is inserted directly in
    /// `closed` state with `created_at_millis = closed_at_millis`. The tombstone
    /// is `INSERT OR REPLACE` keyed by `session_id`: an existing destroy record
    /// for the same session is **replaced**, receiving a new
    /// `destroy_sequence = COALESCE(MAX(destroy_sequence),0)+1`.
    pub fn record_session_closed(&self, close: &SessionClose) -> Result<(), StoreError> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;

        let exists: Option<i64> = tx
            .query_row(
                "SELECT 1 FROM sessions WHERE session_id = ?1",
                [&close.session_id],
                |row| row.get(0),
            )
            .optional()?;

        if exists.is_some() {
            tx.execute(
                "UPDATE sessions \
                 SET state = 'closed', closed_at_millis = ?2, last_known_revision = ?3 \
                 WHERE session_id = ?1",
                params![close.session_id, close.closed_at_millis, close.revision],
            )?;
        } else {
            tx.execute(
                "INSERT INTO sessions \
                     (session_id, label, created_at_millis, closed_at_millis, state, last_known_revision) \
                 VALUES (?1, NULL, ?2, ?2, 'closed', ?3)",
                params![close.session_id, close.closed_at_millis, close.revision],
            )?;
        }

        let destroy_sequence: i64 = tx.query_row(
            "SELECT COALESCE(MAX(destroy_sequence), 0) + 1 FROM tombstones",
            [],
            |row| row.get(0),
        )?;
        tx.execute(
            "INSERT OR REPLACE INTO tombstones (session_id, destroy_sequence, timestamp_millis) \
             VALUES (?1, ?2, ?3)",
            params![close.session_id, destroy_sequence, close.closed_at_millis],
        )?;

        insert_audit(
            &tx,
            AuditEventKind::SessionClosed,
            Some(&close.session_id),
            close.closed_at_millis,
            None,
        )?;

        tx.commit()?;
        Ok(())
    }

    /// Apply retention limits in one transaction and return exact per-table
    /// deleted counts.
    ///
    /// Order: (1) `audit_events` older than the age cutoff; (2) `audit_events`
    /// beyond the newest-by-`sequence` row limit; (3) `tombstones` older than
    /// the age cutoff; (4) `tombstones` beyond the newest-by-`destroy_sequence`
    /// row limit; (5) `closed`/`interrupted` sessions older than the age cutoff
    /// (age measured from `closed_at_millis`, else `created_at_millis`);
    /// (6) `closed`/`interrupted` sessions beyond the newest-by
    /// `created_at_millis` row limit.
    ///
    /// A row with `state = 'active'` is **never** deleted: the server owns live
    /// rows and may be serving other sessions while prune runs.
    pub fn prune(&self, limits: &PruneLimits) -> Result<PruneReport, StoreError> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;

        let audit_cutoff = limits
            .now_millis
            .saturating_sub(limits.audit_max_age_millis);
        let audit_events_deleted = tx.execute(
            "DELETE FROM audit_events WHERE timestamp_millis < ?1",
            [audit_cutoff],
        )?;
        let audit_keep = limits.audit_max_rows.max(0);
        let audit_events_deleted = audit_events_deleted
            + tx.execute(
                "DELETE FROM audit_events \
                 WHERE sequence NOT IN \
                     (SELECT sequence FROM audit_events ORDER BY sequence DESC LIMIT ?1)",
                [audit_keep],
            )?;

        let tombstone_cutoff = limits
            .now_millis
            .saturating_sub(limits.tombstones_max_age_millis);
        let tombstones_deleted = tx.execute(
            "DELETE FROM tombstones WHERE timestamp_millis < ?1",
            [tombstone_cutoff],
        )?;
        let tombstones_keep = limits.tombstones_max_rows.max(0);
        let tombstones_deleted = tombstones_deleted
            + tx.execute(
                "DELETE FROM tombstones \
                 WHERE session_id NOT IN \
                     (SELECT session_id FROM tombstones ORDER BY destroy_sequence DESC LIMIT ?1)",
                [tombstones_keep],
            )?;

        let sessions_age_cutoff = limits
            .now_millis
            .saturating_sub(limits.sessions_max_age_millis);
        let sessions_deleted = tx.execute(
            "DELETE FROM sessions \
             WHERE state IN ('closed','interrupted') \
               AND COALESCE(closed_at_millis, created_at_millis) < ?1",
            [sessions_age_cutoff],
        )?;
        // Row-cap the same non-active set, newest by `created_at_millis`. The
        // subquery is restricted to non-active rows so an `active` row can never
        // be selected for deletion.
        let sessions_keep = limits.sessions_max_rows.max(0);
        let sessions_deleted = sessions_deleted
            + tx.execute(
                "DELETE FROM sessions \
                 WHERE state IN ('closed','interrupted') \
                   AND session_id NOT IN (\
                       SELECT session_id FROM sessions \
                       WHERE state IN ('closed','interrupted') \
                       ORDER BY created_at_millis DESC LIMIT ?1)",
                [sessions_keep],
            )?;

        tx.commit()?;
        Ok(PruneReport {
            audit_events_deleted: audit_events_deleted as u64,
            tombstones_deleted: tombstones_deleted as u64,
            sessions_deleted: sessions_deleted as u64,
        })
    }

    /// [`Store::prune`] with [`PruneLimits::defaults`].
    pub fn prune_with_defaults(&self) -> Result<PruneReport, StoreError> {
        self.prune(&PruneLimits::defaults())
    }

    /// Fetch one session definition.
    pub fn session(&self, session_id: &str) -> Result<Option<StoredSession>, StoreError> {
        let conn = self.conn();
        let raw = conn
            .query_row(
                "SELECT session_id, label, created_at_millis, closed_at_millis, state, \
                        last_known_revision \
                 FROM sessions WHERE session_id = ?1",
                [session_id],
                session_row,
            )
            .optional()?;
        raw.map(session_into).transpose()
    }

    /// All session definitions, oldest first.
    pub fn sessions(&self) -> Result<Vec<StoredSession>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT session_id, label, created_at_millis, closed_at_millis, state, \
                    last_known_revision \
             FROM sessions ORDER BY created_at_millis ASC, session_id ASC",
        )?;
        let rows = stmt.query_map([], session_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(session_into(row?)?);
        }
        Ok(out)
    }

    /// All tombstones, oldest `destroy_sequence` first.
    pub fn tombstones(&self) -> Result<Vec<Tombstone>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT session_id, destroy_sequence, timestamp_millis \
             FROM tombstones ORDER BY destroy_sequence ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Tombstone {
                session_id: row.get(0)?,
                destroy_sequence: row.get(1)?,
                timestamp_millis: row.get(2)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// The full audit trail in store-local `sequence` order.
    pub fn audit_events(&self) -> Result<Vec<AuditEvent>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT sequence, event_kind, session_id, timestamp_millis, detail \
             FROM audit_events ORDER BY sequence ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(RawAudit {
                sequence: row.get(0)?,
                event_kind: row.get(1)?,
                session_id: row.get(2)?,
                timestamp_millis: row.get(3)?,
                detail: row.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(audit_into(row?)?);
        }
        Ok(out)
    }

    /// Best-effort `PRAGMA wal_checkpoint(TRUNCATE)`; a no-op for in-memory
    /// stores.
    pub fn checkpoint(&self) -> Result<(), StoreError> {
        if self.guard.is_none() {
            return Ok(());
        }
        let conn = self.conn();
        conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))?;
        Ok(())
    }

    /// Lock the connection, recovering from mutex poisoning so a panicking
    /// writer cannot wedge every later read.
    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Create the state directory: parents use the process default permissions and
/// only the leaf is restricted to `0700` where supported.
fn create_state_dir(state_dir: &Path) -> Result<(), StoreError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        // Parents (e.g. `~/.local/state`) are created with default permissions;
        // they are shared with other applications and are not ours to lock down.
        if let Some(parent) = state_dir.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        // Only the Relayable leaf is `0700`. An existing leaf (regular directory
        // or any other file) is left untouched; `Store::open` then fails on the
        // lock/database path and the caller degrades to in-memory operation.
        if !state_dir.exists() {
            std::fs::DirBuilder::new().mode(0o700).create(state_dir)?;
        }
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(state_dir)?;
    }
    Ok(())
}

/// Open and exclusively lock the single-writer lock file.
///
/// Uses `std::fs::File::try_lock`, which on Linux is `flock(2)` with
/// `LOCK_EX | LOCK_NB`. The call is non-blocking: a held lock yields
/// [`StoreError::StateDirLocked`].
fn lock_state_dir(lock_path: &Path, state_dir: &Path) -> Result<File, StoreError> {
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        // Keep any existing lock file contents; the lock, not the bytes, is
        // what matters.
        .truncate(false)
        .open(lock_path)?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(std::fs::TryLockError::WouldBlock) => {
            Err(StoreError::StateDirLocked(state_dir.to_path_buf()))
        }
        Err(std::fs::TryLockError::Error(err)) => Err(StoreError::Io(err)),
    }
}

/// Apply the crash-consistency pragmas (`ADR-004` D2.5).
fn configure_pragmas(conn: &Connection, in_memory: bool) -> Result<(), StoreError> {
    let requested = if in_memory { "MEMORY" } else { "WAL" };
    let actual: String =
        conn.query_row(&format!("PRAGMA journal_mode = {requested}"), [], |row| {
            row.get(0)
        })?;
    // SQLite can silently refuse WAL (e.g. on some network/read-only filesystems)
    // and fall back to a rollback journal. Verify instead of trusting the request
    // and tell the operator; startup continues either way.
    if !in_memory && !actual.eq_ignore_ascii_case("wal") {
        eprintln!(
            "relayable-store: WARNING: PRAGMA journal_mode = WAL was not honored (journal_mode = \
             {actual}); crash-consistency now relies on the rollback journal"
        );
    }
    conn.busy_timeout(Duration::from_millis(5000))?;
    conn.execute_batch("PRAGMA synchronous = NORMAL; PRAGMA foreign_keys = ON;")?;
    Ok(())
}

/// Append one audit row using the shared transaction.
fn insert_audit(
    tx: &Transaction<'_>,
    kind: AuditEventKind,
    session_id: Option<&str>,
    timestamp_millis: i64,
    detail: Option<&str>,
) -> Result<(), StoreError> {
    tx.execute(
        "INSERT INTO audit_events (event_kind, session_id, timestamp_millis, detail) \
         VALUES (?1, ?2, ?3, ?4)",
        params![kind.as_str(), session_id, timestamp_millis, detail],
    )?;
    Ok(())
}

fn session_row(row: &Row<'_>) -> rusqlite::Result<RawSession> {
    Ok(RawSession {
        session_id: row.get(0)?,
        label: row.get(1)?,
        created_at_millis: row.get(2)?,
        closed_at_millis: row.get(3)?,
        state: row.get(4)?,
        last_known_revision: row.get(5)?,
    })
}

fn session_into(raw: RawSession) -> Result<StoredSession, StoreError> {
    let state = StoredSessionState::from_str(&raw.state)
        .ok_or_else(|| StoreError::InvalidData(format!("unknown session state {:?}", raw.state)))?;
    Ok(StoredSession {
        session_id: raw.session_id,
        label: raw.label,
        created_at_millis: raw.created_at_millis,
        closed_at_millis: raw.closed_at_millis,
        state,
        last_known_revision: raw.last_known_revision,
    })
}

fn audit_into(raw: RawAudit) -> Result<AuditEvent, StoreError> {
    let kind = audit_kind_from_str(&raw.event_kind).ok_or_else(|| {
        StoreError::InvalidData(format!("unknown audit kind {:?}", raw.event_kind))
    })?;
    Ok(AuditEvent {
        sequence: raw.sequence,
        kind,
        session_id: raw.session_id,
        timestamp_millis: raw.timestamp_millis,
        detail: raw.detail,
    })
}
