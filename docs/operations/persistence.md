# Relayable — persistence store operations

How the `relayable-store` SQLite metadata/audit store is laid out on disk, how
its schema migrates, how retention works, and what it explicitly does **not**
guarantee.

- **Task:** STORE-001 (slice A) — `crates/relayable-store` + `docs/operations/persistence.md`.
- **Branch / worktree:** `store/store-001-sqlite-metadata` @ `/home/sulthonuladib/.herdr/worktrees/relayable/store-store-001-sqlite-metadata`.
- **Date:** 2026-10-16.
- **Scope:** the store crate's on-disk contract and operational behavior. Server
  integration (startup reconciliation wiring, prune scheduling) is slice B and
  is **not** covered here.
- Related: [`../../full.md`](../../full.md) §6.4 (state and persistence),
  §7.1 (filesystem convention), §12 (memory/resource rules),
  [`../adr/ADR-004-process-ownership-and-persistence.md`](../adr/ADR-004-process-ownership-and-persistence.md)
  Decision 2, [`user-install.md`](user-install.md) (paths and service model).

Runtime behavior and the guarantee set live in the ADR; this page is the
operator-facing summary of the store implementation.

---

## 1. State-directory resolution

Resolution is performed at server startup and returns `None` when no source is
usable. Empty values are treated as unset. A set-but-non-absolute value is
**unresolvable** — the store never invents a cwd-relative path.

| Order | Source | Result | Notes |
|---|---|---|---|
| 1 | `RELAYABLE_STATE_DIR` | the value, verbatim | Must be absolute. Highest precedence. |
| 2 | `$XDG_STATE_HOME` | `$XDG_STATE_HOME/relayable` | Must be absolute. `full.md` §7.1:513. |
| 3 | `$HOME` | `$HOME/.local/state/relayable` | Must be absolute. `full.md` §7.1:513. |
| — | none usable | `None` | **Run without persistence** and log a warning. |

`$HOME` and `$XDG_*` are never assumed to exist (`full.md` §7.1:519). When
resolution returns `None`, the server has no DB and no lock file; metadata and
audit are not durable for that run. The warning must tell the user persistence
is disabled and how to enable it.

`crates/relayable-store` exposes this as `resolve_state_dir()` /
`resolve_state_dir_from(lookup)` (pure, for tests) and reports the winning
source (`RelayableStateDir`, `XdgStateHome`, `HomeFallback`).

## 2. Files

| Artifact | Path | Notes |
|---|---|---|
| State directory | resolved above | Created on first open with mode **0700** on Unix. |
| Database | `<state_dir>/relayable.db` | `Store::db_file_name()`. |
| SQLite WAL | `<state_dir>/relayable.db-wal` | Present while WAL is active. |
| SQLite SHM | `<state_dir>/relayable.db-shm` | Present while WAL is active. |
| Single-writer lock | `<state_dir>/relayable-store.lock` | `Store::lock_file_name()`; empty marker file. |

## 3. Schema (version 1)

All columns are host-independent. There are deliberately **no** pids, file
descriptors, hostnames, peer credentials, host identity, terminal bytes,
environment values, or command arguments (`ADR-004` D2.3; `full.md` §11:770,
§12:804).

| Table | Columns |
|---|---|
| `sessions` | `session_id` TEXT PK, `label` TEXT, `created_at_millis` INTEGER NOT NULL, `closed_at_millis` INTEGER, `state` TEXT NOT NULL CHECK (`active`\|`closed`\|`interrupted`), `last_known_revision` INTEGER NOT NULL DEFAULT 1. Index: `sessions_state_idx (state)`. |
| `tombstones` | `session_id` TEXT PK, `destroy_sequence` INTEGER NOT NULL, `timestamp_millis` INTEGER NOT NULL. Index: `tombstones_timestamp_idx (timestamp_millis)`. |
| `audit_events` | `sequence` INTEGER PK AUTOINCREMENT, `event_kind` TEXT NOT NULL, `session_id` TEXT, `timestamp_millis` INTEGER NOT NULL, `detail` TEXT. Indexes: `audit_events_timestamp_idx (timestamp_millis)`, `audit_events_kind_idx (event_kind)`. |
| `schema_migrations` | `version` INTEGER PK, `applied_at_millis` INTEGER NOT NULL. |

Notes:

- **`audit_events.sequence` is the store's local audit order**, not the
  protocol/server `sequence`. The server `sequence` is per-run and non-durable
  and starts a new epoch on restart (`ADR-004` D2.6).
- **`tombstones` has no foreign key to `sessions` by design.** A destroy record
  outlives the session row it describes, so pruning a closed session must not
  delete its tombstone (`ADR-001:286-291`).
- `tombstones.destroy_sequence` is monotonic per store: computed inside the
  close transaction as `COALESCE(MAX(destroy_sequence), 0) + 1`.
- Closing a session writes the tombstone with `INSERT OR REPLACE` keyed by
  `session_id`. If a destroy record for that session already exists it is
  **replaced** and receives a new `destroy_sequence`; the prior sequence value
  is not preserved.
- `interrupted` is a **store-local row state**. The wire/model `SessionState`
  has **no** `interrupted` variant yet (the frozen proto is unchanged). It marks
  a row left `active` by a previous run whose owning process is gone; it is
  never reported as live (`ADR-004` D2.5).
- Row state strings: `active`, `closed`, `interrupted`. Audit kinds:
  `session_created`, `session_closed`, `session_interrupted`, `pruned`.

### Deliberately absent (ADR-004-implied future work)

`preferences`, `session_layout` (windows/blocks/split ratios), and `identities`
are **not** implemented. No such durable data exists in the server model today
(no split ratios, no preferences, no durable client records), so no schema is
invented for them. They remain ADR-004-implied future work (D2.3) and need their
own slice once the data exists.

## 4. Migrations

- Numbered SQL migrations in a `schema_migrations` table (not
  `PRAGMA user_version`): each pending migration runs in **its own
  transaction**, together with its `schema_migrations` row carrying
  `applied_at_millis`.
- `schema_migrations` is bootstrapped (unversioned) before any migration runs,
  so an empty database reads as version `0`.
- Opening is **idempotent**: already-applied versions are skipped.
- `LATEST_SCHEMA_VERSION` is `1`. A database whose recorded maximum version is
  newer than that is rejected with `SchemaTooNew { found, supported }`; it is
  never silently downgraded.

## 5. Pragmas, one writer, and the lock

Set on every open:

| Pragma | Value | Why |
|---|---|---|
| `journal_mode` | `WAL` (`MEMORY` in-memory) | Crash-consistent writes; readers do not block the writer. |
| `synchronous` | `NORMAL` | WAL durability/performance default (`FULL` available later if needed). |
| `busy_timeout` | `5000` ms | Contention cannot fail spuriously. |
| `foreign_keys` | `ON` | Enforced if FKs are ever added. |

- **One writer.** A `Store` owns exactly one `rusqlite::Connection` behind a
  `Mutex`; every logical operation is one explicit transaction. Memory remains
  authoritative for live state — the DB never substitutes for a live child.
- **`flock` guard.** `Store::open` opens/creates `<state_dir>/relayable-store.lock`
  and calls `std::fs::File::try_lock()` (on Linux `flock(2)`
  `LOCK_EX | LOCK_NB`) **before** running migrations. A second open while the
  first is alive returns `StateDirLocked`. The `File` is held for the store's
  lifetime; the lock releases when the store drops.
- `Store::open_in_memory()` skips WAL and the lock; it is for hermetic tests
  only.

## 6. Retention and pruning

Pruning runs in one transaction and returns exact per-table deleted counts
(`PruneReport`). Order: (1) `audit_events` older than the age cutoff; (2)
`audit_events` beyond the newest-by-`sequence` row cap; (3) `tombstones` older
than the age cutoff; (4) `tombstones` beyond the newest-by-`destroy_sequence`
row cap; (5) `closed` sessions older than the age cutoff. A row strictly older
than `now_millis - max_age_millis` is deleted; a non-positive row cap keeps no
rows. `active`/`interrupted` sessions are never pruned.

Defaults (Relayable-defined, no REX evidence; `ADR-004` D2.4):

| Bound | Default |
|---|---|
| `audit_events` max rows | 10 000 |
| `audit_events` max age | 30 days |
| `tombstones` max rows | 10 000 |
| `tombstones` max age | 90 days |
| `closed` sessions max age | 90 days |

Environment overrides (implementation-chosen under the ADR-005 `RELAYABLE_*`
namespace; **pending confirmation**, ADR-005 as accepted fixes only
`RELAYABLE_SESSION`/`BLOCK`/`SERVER`/`CLIENT`). Unset, empty, or non-numeric
values fall back to the default.

| Variable | Effect |
|---|---|
| `RELAYABLE_STORE_AUDIT_MAX_ROWS` | `audit_events` max rows. |
| `RELAYABLE_STORE_AUDIT_MAX_AGE_MILLIS` | `audit_events` max age (ms). |
| `RELAYABLE_STORE_TOMBSTONES_MAX_ROWS` | `tombstones` max rows. |
| `RELAYABLE_STORE_TOMBSTONES_MAX_AGE_MILLIS` | `tombstones` max age (ms). |
| `RELAYABLE_STORE_CLOSED_SESSIONS_MAX_AGE_MILLIS` | `closed` session max age (ms). |

`PruneLimits::from_env()` reads these; slice B initially calls
`prune_with_defaults()`.

## 7. Backup and export

The DB is plain SQLite; any SQLite backup tool works while the server is
running, because WAL lets a reader take a consistent copy.

```sh
# Online-consistent copy (sqlite3 CLI; `VACUUM INTO` requires SQLite >= 3.27):
sqlite3 "$STATE_DIR/relayable.db" "VACUUM INTO '$BACKUP_DIR/relayable-$(date -u +%Y%m%dT%H%M%SZ).db'"

# Or the backup API via the sqlite3 shell:
sqlite3 "$STATE_DIR/relayable.db" ".backup '$BACKUP_DIR/relayable.db'"
```

A schema-versioned, host-independent JSON/NDJSON export is described in
`ADR-004` D2.8 as Relayable-defined future work; it is **not implemented** here.
**Import/restore is out of scope for the first release.** Exports contain no
terminal bytes because none are stored.

## 8. What is restored after a restart

A restored session definition **is a restart, not a resurrected process**
(`full.md` §6.3:486; `docs/architecture/interface-contract.md:93-95`). The
server presents restoring a definition as an explicit restart action; it never
silently reattaches.

| Restored | Not restored |
|---|---|
| Session definitions: `session_id`, `label`, timestamps, `state` | The live child/PTY and its pids |
| Tombstones | The writer lease |
| Audit trail | Connected clients |
| — | Output tail / scrollback / byte-exact terminal screen |
| — | Event-sequence continuity (new per-run epoch) |

At startup, every row left `active` is marked `interrupted` (one
`session_interrupted` audit event per changed row). A second reconciliation is a
no-op returning `interrupted == 0`.

## 9. Explicit non-guarantees

| Property | Status | Authority |
|---|---|---|
| Session/PTY survival across server crash or host reboot | **NOT provided, NOT claimed** | `full.md` §1.3:120, §6.3:480, §6.3:486; `ADR-004` D1.7/D2.7 |
| Byte-exact terminal stream/screen recovery | **Not claimed** | `full.md` §6.1:460; `ADR-001:239`; `interface-contract.md:88-92` |
| Event-sequence continuity across a restart | **Not claimed** (new per-run epoch) | `ADR-004` D2.6 |
| Live child/PTY state in the DB | **Never** — the DB is not a live child and is not authoritative for liveness | `ADR-004` D2.5/D2.7 |
| Terminal bytes, scrollback, env values, or full argv in the DB | **Never** — no such column exists | `full.md` §6.4:490, §12:804; `ADR-001:239,286-291` |
| Multi-user isolation / authn / authz | **NOT implemented** (one user per server) | `full.md` §11:766; `ADR-004` D3 |
| Silent reattach of a restored session | **Never** — restart only | `interface-contract.md:93-95` |
