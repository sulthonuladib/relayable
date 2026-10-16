//! Integration tests for `relayable-store`.
//!
//! Every test is hermetic: it uses a `tempfile` directory (or an in-memory
//! store) and never reads the ambient `HOME` / `XDG_*` / `RELAYABLE_*`
//! environment.

use relayable_store::{
    AuditEventKind, PruneLimits, SessionClose, SessionCreate, StateDirSource, Store, StoreError,
    StoredSessionState,
};

fn create(session_id: &str, label: Option<&str>, created_at_millis: i64) -> SessionCreate {
    SessionCreate {
        session_id: session_id.to_string(),
        label: label.map(str::to_string),
        created_at_millis,
        revision: 1,
    }
}

fn close(session_id: &str, closed_at_millis: i64) -> SessionClose {
    SessionClose {
        session_id: session_id.to_string(),
        closed_at_millis,
        revision: 2,
    }
}

/// 1. Open/migrations/idempotent.
#[test]
fn open_migrates_to_version_1_and_is_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");

    let store = Store::open(dir.path()).expect("first open");
    assert_eq!(store.schema_version().expect("schema version"), 1);
    assert_eq!(store.state_dir(), dir.path());
    assert_eq!(store.db_path_ref(), Store::db_path(dir.path()));
    drop(store);

    let reopened = Store::open(dir.path()).expect("second open");
    assert_eq!(reopened.schema_version().expect("schema version"), 1);
    drop(reopened);

    let in_memory = Store::open_in_memory().expect("in-memory open");
    assert_eq!(in_memory.schema_version().expect("schema version"), 1);
}

/// 1b. A non-absolute state directory is rejected.
#[test]
fn non_absolute_state_dir_is_rejected() {
    let err = match Store::open(std::path::Path::new("relative/state")) {
        Ok(_) => panic!("relative state dir must be rejected"),
        Err(err) => err,
    };
    assert!(matches!(err, StoreError::NonAbsoluteStateDir(_)));
}

/// 2. Single-writer guard via `flock`.
#[test]
fn second_open_is_locked_until_the_first_drops() {
    let dir = tempfile::tempdir().expect("tempdir");

    let first = Store::open(dir.path()).expect("first open");
    let err = match Store::open(dir.path()) {
        Ok(_) => panic!("second concurrent open must be locked out"),
        Err(err) => err,
    };
    assert!(matches!(err, StoreError::StateDirLocked(_)));
    drop(first);

    let second = Store::open(dir.path()).expect("open after drop");
    assert_eq!(second.schema_version().expect("schema version"), 1);
}

/// 3. Startup reconciliation marks stale `active` rows `interrupted`.
#[test]
fn reconcile_marks_active_rows_and_is_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");

    {
        let store = Store::open(dir.path()).expect("open");
        store
            .record_session_created(&create("stale", Some("stale"), 10))
            .expect("create");
        assert_eq!(
            store.session("stale").expect("get").expect("row").state,
            StoredSessionState::Active
        );
    }

    let store = Store::open(dir.path()).expect("reopen");
    let report = store.reconcile_on_startup().expect("reconcile");
    assert_eq!(report.interrupted, 1);

    let row = store.session("stale").expect("get").expect("row");
    assert_eq!(row.state, StoredSessionState::Interrupted);

    let events = store.audit_events().expect("audit");
    assert!(events
        .iter()
        .any(|event| event.kind == AuditEventKind::SessionInterrupted
            && event.session_id.as_deref() == Some("stale")));

    let second = store.reconcile_on_startup().expect("second reconcile");
    assert_eq!(second.interrupted, 0);

    // A closed row is never touched by reconciliation.
    store
        .record_session_created(&create("done", None, 20))
        .expect("create");
    store
        .record_session_closed(&close("done", 30))
        .expect("close");
    assert_eq!(
        store.reconcile_on_startup().expect("reconcile").interrupted,
        0
    );
    assert_eq!(
        store.session("done").expect("get").expect("row").state,
        StoredSessionState::Closed
    );
}

/// 4. Write path: create + close, tombstone sequence, audit order.
#[test]
fn create_and_close_write_session_tombstone_and_audit() {
    let store = Store::open_in_memory().expect("open");

    store
        .record_session_created(&create("a", Some("alpha"), 100))
        .expect("create a");
    store
        .record_session_closed(&close("a", 200))
        .expect("close a");

    let session = store.session("a").expect("get").expect("row");
    assert_eq!(session.state, StoredSessionState::Closed);
    assert_eq!(session.closed_at_millis, Some(200));
    assert_eq!(session.label.as_deref(), Some("alpha"));

    let tombstones = store.tombstones().expect("tombstones");
    assert_eq!(tombstones.len(), 1);
    assert_eq!(tombstones[0].session_id, "a");
    assert_eq!(tombstones[0].destroy_sequence, 1);
    assert_eq!(tombstones[0].timestamp_millis, 200);

    let kinds: Vec<AuditEventKind> = store
        .audit_events()
        .expect("audit")
        .into_iter()
        .map(|event| event.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            AuditEventKind::SessionCreated,
            AuditEventKind::SessionClosed
        ]
    );

    // A second close of a different session gets the next destroy sequence.
    store
        .record_session_created(&create("b", None, 300))
        .expect("create b");
    store
        .record_session_closed(&close("b", 400))
        .expect("close b");
    let tombstones = store.tombstones().expect("tombstones");
    assert_eq!(tombstones.len(), 2);
    assert_eq!(tombstones[1].destroy_sequence, 2);

    // Closing with no prior row is defensive and still closes + tombstones.
    store
        .record_session_closed(&close("orphan", 500))
        .expect("close orphan");
    let orphan = store.session("orphan").expect("get").expect("row");
    assert_eq!(orphan.state, StoredSessionState::Closed);
    assert_eq!(orphan.created_at_millis, 500);
    assert_eq!(orphan.closed_at_millis, Some(500));
}

/// 5. Prune: exact counts, age + row limits, recent rows survive.
#[test]
fn prune_applies_age_and_row_limits_with_exact_counts() {
    let store = Store::open_in_memory().expect("open");
    let now = 1_000_000_000_i64;

    // Two old sessions (age-pruned) and two recent sessions (one tombstone
    // row-pruned, one surviving).
    for (id, created, closed) in [
        ("a", 100, 200),
        ("b", 150, 250),
        ("c", now - 5, now - 3),
        ("d", now - 4, now - 2),
    ] {
        store
            .record_session_created(&create(id, None, created))
            .expect("create");
        store
            .record_session_closed(&close(id, closed))
            .expect("close");
    }

    let limits = PruneLimits {
        now_millis: now,
        audit_max_rows: 2,
        audit_max_age_millis: 1_000,
        tombstones_max_rows: 1,
        tombstones_max_age_millis: 1_000,
        sessions_max_rows: 10,
        sessions_max_age_millis: 1_000,
    };
    let report = store.prune(&limits).expect("prune");

    // 4 audit rows pruned by age (a/b create+close) + 2 by row limit = 6.
    assert_eq!(report.audit_events_deleted, 6);
    // 2 tombstones pruned by age (a, b) + 1 by row limit (c) = 3.
    assert_eq!(report.tombstones_deleted, 3);
    // 2 old closed sessions pruned (a, b) by age.
    assert_eq!(report.sessions_deleted, 2);

    assert_eq!(store.audit_events().expect("audit").len(), 2);

    let tombstones = store.tombstones().expect("tombstones");
    assert_eq!(tombstones.len(), 1);
    assert_eq!(tombstones[0].session_id, "d");
    assert_eq!(tombstones[0].destroy_sequence, 4);

    // A recent closed session and its tombstone survive.
    let sessions = store.sessions().expect("sessions");
    assert_eq!(sessions.len(), 2);
    let d = store.session("d").expect("get").expect("row");
    assert_eq!(d.state, StoredSessionState::Closed);
    assert_eq!(d.closed_at_millis, Some(now - 2));

    // Pruning again is a no-op on already-bounded data.
    let second = store.prune(&limits).expect("second prune");
    assert_eq!(second, relayable_store::PruneReport::default());
}

/// 5b. The sessions row cap keeps the newest non-active rows and never deletes
/// an `active` row, even while other sessions are live.
#[test]
fn session_prune_row_cap_keeps_newest_non_active_and_never_deletes_active() {
    let store = Store::open_in_memory().expect("open");
    let now = 1_000_000_000_i64;

    // One interrupted row (created earliest) and five closed rows.
    store
        .record_session_created(&create("interrupted", None, 50))
        .expect("create interrupted");
    assert_eq!(
        store.reconcile_on_startup().expect("reconcile").interrupted,
        1
    );
    for (id, created) in [
        ("c1", 100),
        ("c2", 200),
        ("c3", 300),
        ("c4", 400),
        ("c5", 500),
    ] {
        store
            .record_session_created(&create(id, None, created))
            .expect("create closed");
        store
            .record_session_closed(&close(id, created + 1))
            .expect("close");
    }
    // A live row that prune runs alongside; it must survive.
    store
        .record_session_created(&create("live", None, now - 1))
        .expect("create live");

    let limits = PruneLimits {
        now_millis: now,
        audit_max_rows: 1_000,
        audit_max_age_millis: i64::MAX,
        tombstones_max_rows: 1_000,
        tombstones_max_age_millis: i64::MAX,
        sessions_max_rows: 2,
        sessions_max_age_millis: i64::MAX,
    };
    let report = store.prune(&limits).expect("prune");

    // Non-active set is {interrupted(50), c1(100)..c5(500)}; keep newest two
    // (c5, c4), delete the other four. Tombstones/audit are untouched.
    assert_eq!(report.sessions_deleted, 4);
    assert_eq!(report.audit_events_deleted, 0);
    assert_eq!(report.tombstones_deleted, 0);

    assert_eq!(
        store.session("live").expect("get live").expect("row").state,
        StoredSessionState::Active,
        "an active row must never be pruned"
    );
    assert!(store.session("interrupted").expect("get").is_none());
    assert!(store.session("c1").expect("get").is_none());
    assert!(store.session("c2").expect("get").is_none());
    assert!(store.session("c3").expect("get").is_none());
    assert!(store.session("c4").expect("get").is_some());
    assert!(store.session("c5").expect("get").is_some());

    let sessions = store.sessions().expect("sessions");
    assert_eq!(sessions.len(), 3, "live + the two newest closed rows");
}

/// 6a. Exact column allowlist for every table (fails if a host-state column is
/// ever added).
#[test]
fn schema_has_no_host_state_columns() {
    let dir = tempfile::tempdir().expect("tempdir");
    {
        let store = Store::open(dir.path()).expect("open");
        assert_eq!(store.schema_version().expect("schema version"), 1);
    }

    let conn = rusqlite::Connection::open(Store::db_path(dir.path())).expect("raw open");
    let allowlist: &[(&str, &[&str])] = &[
        (
            "sessions",
            &[
                "session_id",
                "label",
                "created_at_millis",
                "closed_at_millis",
                "state",
                "last_known_revision",
            ],
        ),
        (
            "tombstones",
            &["session_id", "destroy_sequence", "timestamp_millis"],
        ),
        (
            "audit_events",
            &[
                "sequence",
                "event_kind",
                "session_id",
                "timestamp_millis",
                "detail",
            ],
        ),
        ("schema_migrations", &["version", "applied_at_millis"]),
    ];

    for (table, expected) in allowlist {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({table})"))
            .expect("prepare table_info");
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query table_info")
            .collect::<rusqlite::Result<Vec<String>>>()
            .expect("collect columns");
        assert_eq!(&columns, expected, "unexpected columns for {table}");
    }
}

/// 6b. Raw DB round-trip: the scan itself works (positive control).
///
/// The strong end-to-end "no terminal bytes are ever persisted" proof lives in
/// the server PTY test
/// `relayable-server/tests/store_wiring.rs::create_and_close_persist_metadata_without_terminal_bytes`,
/// which pushes a unique sentinel through a real PTY and scans the same raw DB
/// bytes. At the store level there is no API that accepts terminal bytes, so a
/// negative assertion here would be vacuous; the structural guard that no
/// host-state column can exist is the `PRAGMA table_info` allowlist in
/// `schema_has_no_host_state_columns`.
#[test]
fn raw_database_round_trips_a_stored_label() {
    const LABEL: &str = "MARKER-LABEL-abc123";

    let dir = tempfile::tempdir().expect("tempdir");
    {
        let store = Store::open(dir.path()).expect("open");
        store
            .record_session_created(&create("sess-marker", Some(LABEL), 10))
            .expect("create");
        store
            .record_session_closed(&close("sess-marker", 20))
            .expect("close");
        store.checkpoint().expect("checkpoint");
    }

    // Bound the scan (the test DB is tiny; the cap keeps it bounded anyway).
    const MAX_SCAN: usize = 8 * 1024 * 1024;
    let bytes = std::fs::read(Store::db_path(dir.path())).expect("read db");
    assert!(bytes.len() <= MAX_SCAN, "unexpectedly large test DB");
    let haystack = &bytes[..bytes.len().min(MAX_SCAN)];

    assert!(
        contains(haystack, LABEL.as_bytes()),
        "legitimately stored label missing from raw DB (scan is broken)"
    );
}

/// 6c. Store write failures surface as `StoreError::Sqlite` rather than being
/// silently swallowed. The `sessions` table is destroyed and recreated in a
/// shape whose `INSERT` cannot satisfy, then the typed write is exercised.
#[test]
fn broken_sessions_table_surfaces_sqlite_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    {
        let store = Store::open(dir.path()).expect("open");
        assert_eq!(store.schema_version().expect("schema version"), 1);
    }

    // Recreate `sessions` with an extra NOT NULL column and no default, so the
    // typed `INSERT` must fail, while `schema_migrations` still reads version 1
    // (migrations are skipped) and reconciliation still works.
    {
        let conn = rusqlite::Connection::open(Store::db_path(dir.path())).expect("raw open");
        conn.execute_batch(
            "DROP TABLE sessions;\n\
             CREATE TABLE sessions (\n\
                 session_id TEXT PRIMARY KEY NOT NULL,\n\
                 label TEXT,\n\
                 created_at_millis INTEGER NOT NULL,\n\
                 closed_at_millis INTEGER,\n\
                 state TEXT NOT NULL CHECK (state IN ('active','closed','interrupted')),\n\
                 last_known_revision INTEGER NOT NULL DEFAULT 1,\n\
                 broken_col TEXT NOT NULL\n\
             );",
        )
        .expect("recreate sessions in a broken shape");
    }

    let store = Store::open(dir.path()).expect("reopen after breakage");
    assert_eq!(store.schema_version().expect("schema version"), 1);
    store.reconcile_on_startup().expect("reconcile still works");

    let error = store
        .record_session_created(&create("broken", None, 1))
        .expect_err("a broken table must surface an error");
    assert!(
        matches!(error, StoreError::Sqlite(_)),
        "expected StoreError::Sqlite, got {error:?}"
    );
}

/// 7. State-directory resolution never guesses; source is reported.
#[test]
fn resolve_state_dir_is_environment_driven() {
    // `resolve_state_dir_from` is the pure lookup variant; the ambient
    // `resolve_state_dir()` wrapper is exercised only indirectly (through the
    // server tests) to keep this test hermetic.
    let resolved = relayable_store::resolve_state_dir_from(|name| match name {
        "RELAYABLE_STATE_DIR" => Some("/tmp/relayable-state".to_string()),
        _ => None,
    });
    assert_eq!(
        resolved,
        Some((
            std::path::PathBuf::from("/tmp/relayable-state"),
            StateDirSource::RelayableStateDir
        ))
    );

    let unresolved = relayable_store::resolve_state_dir_from(|_| None);
    assert_eq!(unresolved, None);
}

/// Bounded substring search over raw bytes.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}
