# Relayable — persistence store operations

How the `relayable-store` SQLite metadata/audit store is laid out on disk, how
its schema migrates, how retention works, and what it explicitly does **not**
guarantee.

- **Task:** STORE-001 (slice A) — `crates/relayable-store` + `docs/operations/persistence.md`.
- **Branch / worktree:** `store/store-001-sqlite-metadata` @ `/home/sulthonuladib/.herdr/worktrees/relayable/store-store-001-sqlite-metadata`.
- **Date:** 2026-10-16.
- **Scope:** the store crate's on-disk contract and operational behavior, plus
  the server integration (startup resolution/reconciliation, the write path,
  the failure policy, and shutdown ordering) in section 10. Server-side
  "restart session" product surface is still future work.
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
| State directory | resolved above | Created on first open; only the leaf is mode **0700** on Unix. Any parent directories are created with the process default permissions (they are shared application directories, not Relayable's). |
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
- `tombstones.destroy_sequence` is `MAX(destroy_sequence) + 1` computed inside
  the close transaction (`COALESCE(MAX(...), 0) + 1`). It is **not** globally
  monotonic across pruning: once the highest-sequence tombstones are pruned, the
  next write can reuse a lower sequence.
- Closing a session writes the tombstone with `INSERT OR REPLACE` keyed by
  `session_id`. If a destroy record for that session already exists it is
  **replaced** and receives a new `destroy_sequence`; the prior sequence value
  is not preserved.
- `interrupted` is a **store-local row state**. The wire/model `SessionState`
  has **no** `interrupted` variant yet (the frozen proto is unchanged). It marks
  a row left `active` by a previous run whose owning process is gone; it is
  never reported as live (`ADR-004` D2.5). It is a terminal-ish state retained
  for a future restart surface: the row survives until retention prunes it
  (section 6), and nothing resurrects it into a live session.
- Row state strings: `active`, `closed`, `interrupted`. Audit kinds:
  `session_created`, `session_closed`, `session_interrupted`, and `pruned`
  (the last is **reserved** and not currently emitted by `Store::prune`).

### Deliberately absent (ADR-004-implied future work)

`preferences`, `session_layout` (windows/blocks/split ratios), and `identities`
are **not** implemented. No such durable data exists in the server model today
(no split ratios, no preferences, no durable client records), so no schema is
invented for them. They remain ADR-004-implied future work (D2.3) and need their
own slice once the data exists.

**ADR amendment required.** `ADR-004` D2.3 still lists this broader table set
(`identities`, `preferences`, `session_layout`) as part of the store. This slice
implements only `sessions`, `tombstones`, `audit_events`, and
`schema_migrations`, so D2.3 needs a decision amendment to record the narrowed
scope; the ADR itself is intentionally **not** edited by this slice.

**Further ADR/status amendment required (post-integration, lead/Manager).** This
slice also contradicts `ADR-004`'s non-guarantees row ("SQLite persistence store
— Designed here; not implemented") and its "Implied future work (not done)"
checklist (crate creation, workspace wiring, reconcile+flock, write path, prune,
startup/shutdown ordering), as well as `ADR-001`'s `server → store` allowed-edge
list. The stale `docs/agent-loop/STATUS.md` / "blocked" entries record the same
gap. Those files are intentionally **not** edited here; they need a
lead/Manager amendment after integration.

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
| `journal_mode` | `WAL` (`MEMORY` in-memory) | Crash-consistent writes; readers do not block the writer. The returned mode is **verified**: on an on-disk store, a mode other than `wal` logs a loud `eprintln!` warning naming the actual mode and stating that crash-consistency now relies on the rollback journal (startup continues). In-memory stores are exempt. |
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
- **Local-filesystem assumption.** Both the `flock` single-writer guard and the
  WAL fallback/consistency guarantee assume the state directory is on a local
  filesystem. On NFS or another network filesystem, advisory locking may not be
  honored between hosts and WAL may be refused or silently degraded — weakening
  **both** guards. Keep the state directory local.
- `Store::open_in_memory()` skips WAL and the lock; it is for hermetic tests
  only.

## 6. Retention and pruning

Pruning runs in one transaction and returns exact per-table deleted counts
(`PruneReport`). Order: (1) `audit_events` older than the age cutoff; (2)
`audit_events` beyond the newest-by-`sequence` row cap; (3) `tombstones` older
than the age cutoff; (4) `tombstones` beyond the newest-by-`destroy_sequence`
row cap; (5) `closed`/`interrupted` sessions older than the age cutoff, where
the age is measured from `closed_at_millis` when present and otherwise from
`created_at_millis`; (6) `closed`/`interrupted` sessions beyond the
newest-by-`created_at_millis` row cap (the same non-active set). A row strictly
older than `now_millis - max_age_millis` is deleted; a non-positive row cap
keeps no rows.

**Bounded session states.** Only `closed` and `interrupted` rows are prunable.
A row with `state = 'active'` is **never** deleted — the server owns live rows
and prune runs while other sessions may be active. `interrupted` is the
store-local terminal-ish state described in section 3: it is retained for a
future restart surface and is bounded by exactly the same age and row-cap rules
as `closed`.

Defaults (Relayable-defined, no REX evidence; `ADR-004` D2.4):

| Bound | Default |
|---|---|
| `audit_events` max rows | 10 000 |
| `audit_events` max age | 30 days |
| `tombstones` max rows | 10 000 |
| `tombstones` max age | 90 days |
| non-active (`closed`/`interrupted`) sessions max rows | 10 000 |
| non-active (`closed`/`interrupted`) sessions max age | 90 days |

Environment overrides (implementation-chosen under the ADR-005 `RELAYABLE_*`
namespace; **pending confirmation**, ADR-005 as accepted fixes only
`RELAYABLE_SESSION`/`BLOCK`/`SERVER`/`CLIENT`). Unset, empty, or non-numeric
values fall back to the default. `PruneLimits::from_env()` resolves **all six**
once at server startup and the server uses that one set for every prune
(startup and close), so these overrides are actually honored.

| Variable | Effect |
|---|---|
| `RELAYABLE_STORE_AUDIT_MAX_ROWS` | `audit_events` max rows. |
| `RELAYABLE_STORE_AUDIT_MAX_AGE_MILLIS` | `audit_events` max age (ms). |
| `RELAYABLE_STORE_TOMBSTONES_MAX_ROWS` | `tombstones` max rows. |
| `RELAYABLE_STORE_TOMBSTONES_MAX_AGE_MILLIS` | `tombstones` max age (ms). |
| `RELAYABLE_STORE_SESSIONS_MAX_ROWS` | non-active session max rows. |
| `RELAYABLE_STORE_SESSIONS_MAX_AGE_MILLIS` | non-active session max age (ms). |

Tombstones remain independent of sessions (no foreign key), so pruning a
session row never deletes its tombstone (section 3).

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

## 10. Server integration (slice B)

How `crates/relayable-server` wires the store into its lifecycle. This is the
"slice B" of STORE-001; the on-disk/store contract above is slice A.

### 10.1 Startup resolution

`ServerConfig::from_env()` resolves the state directory once, at startup, via
`relayable_store::resolve_state_dir()` (section 1 order) and stores it as
`ServerConfig::state_dir: Option<PathBuf>`. `ServerConfig::from_parts` takes the
state dir as an explicit parameter so the hermetic config unit tests can pass
`None`.

`Server::bind` opens persistence **before** binding the socket, so a refusal
leaves no socket artifact. When a directory resolves:

1. `Store::open(dir)` creates the leaf directory `0700` (parents use default
   permissions), takes the `flock` single-writer lock, sets pragmas, and applies
   migrations. The `journal_mode` returned by SQLite is verified; a non-`wal`
   mode logs the fallback warning from section 5 and startup continues.
2. `Store::reconcile_on_startup()` marks every row left `active` by a previous
   run as `interrupted` (one `session_interrupted` audit event per row). A
   non-zero count is logged; the stale rows are **never** reported live.
3. `PruneLimits::from_env()` is resolved **once** and kept with the store; the
   server applies it with `Store::prune(&limits)` (section 6). Every later
   server-side prune uses the same set.

The store is held in `ServerShared` and is reachable from `create_session` and
`close_session`. Memory stays authoritative for live state: the server **never**
creates in-memory sessions from DB rows. `store-local` state `interrupted` has
no model/proto representation, so a stale (interrupted/closed) session id gets
the normal unknown-entity behaviour and is not resurrected.

### 10.2 Failure and degradation policy

| Startup condition | Behaviour |
|---|---|
| No usable state dir (`RELAYABLE_STATE_DIR`, `XDG_STATE_HOME`, `HOME` all unset/empty/relative) | `eprintln!` warning naming those variables; run with in-memory metadata only (no DB, no lock). **Not** a startup error. |
| `Store::open` fails with `StoreError::StateDirLocked(path)` | **Refuse to start**: `anyhow::bail!` naming the path; `main.rs` exits `1`. A second server must never become a second writer. |
| Any other `Store::open`/migrate error | Loud `eprintln!` warning; continue **without** persistence. Store errors are never allowed to abort a protocol operation. |
| `reconcile_on_startup()` fails | Loud warning; drop the store and continue without persistence. |
| Startup `prune(&limits)` fails | Warning only; the store stays open (it opened and reconciled). |

**Lock failure is the one asymmetric case.** Only `StateDirLocked` refuses
startup (a second writer must never exist). Every other open/migrate/reconcile
error — including a state path that is an existing regular file, or an
unusable/corrupt database — degrades to in-memory operation after a warning.

All diagnostics use `eprintln!` on stderr; the repository has no logging
framework and none was added.

### 10.3 Write path

Writes are issued from `registry.rs` **outside every entity lock**, matching the
existing event-append discipline (a store call must never hold the `sessions` or
`block` mutex):

- `create_session` success -> `record_session_created` with the new
  `session_id`, label, `created_at_millis`, and the initial revision. This
  writes the `active` row and its `session_created` audit event in one
  transaction.
- `close_session`, only on the reaped `CloseSessionOutcome::Closed` path (not
  `NotConfirmed`, not `Unknown`) -> `record_session_closed` writes the `closed`
  row, the tombstone, and the `session_closed` audit event in **one**
  transaction, then `prune(&limits)` runs with the startup-resolved limits
  (retention runs on startup and on close, `ADR-004` D2.4). The idempotent
  re-close path and the shutdown-time close do not write again (section 10.4).

Only those fields are passed. No pids, file descriptors, hostnames, env values,
argv, or terminal bytes ever reach the store.

**Failure policy.** Store errors never fail a protocol operation and are never
allowed to panic the server: a failing `record_session_created`,
`record_session_closed`, or `prune` is caught at the call site and logged as
`relayable-server: persistence <operation> failed for session <id>: <err>` (or
`persistence prune failed ...`), and the create/close response still succeeds.
Diagnostics use `eprintln!` on stderr per repository convention. This is the
actual guarantee; it is not a blanket "no code path can panic" claim.

**Writer-lease audit is deferred.** `client`/`block` lease transitions are
retained in the in-memory event log but are **not** written to `audit_events` in
this slice (there is no durable lease table); only session lifecycle events are
persisted.

### 10.4 Shutdown ordering

On the `Server::run` shutdown path (SIGINT/SIGTERM, or a programmatic
`shutdown()`):

1. The accept loop exits and live sessions are closed; the child trees are
   reaped.
2. Connection threads are joined.
3. Best-effort `Store::checkpoint()` (`PRAGMA wal_checkpoint(TRUNCATE)`) runs,
   then the store is dropped, closing the database cleanly and releasing the
   `flock` single-writer lock. A checkpoint/close error is logged, never fatal.
4. The socket file is unlinked last, preserving the existing ordering.

A server that ended without an explicit `session.close` leaves its session rows
`active`; the next startup's reconciliation marks them `interrupted` (never
live).

**Shutdown-time termination writes no tombstone, by design.** When `run`
terminates live children during graceful shutdown it does **not** call
`record_session_closed`: shutdown is not a user-authorized destroy, so no
tombstone and no `session_closed` audit event is written. The row stays `active`
until the next startup, which marks it `interrupted` (and it is then subject to
the section 6 retention bounds).

### 10.5 Audit sequence reminder

`audit_events.sequence` written by the server integration is the **store's local
audit order**, not the protocol/server `sequence`. The server `sequence` is
per-run and non-durable (`ADR-004` D2.6); the two are never conflated.
