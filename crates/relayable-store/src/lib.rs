//! `relayable-store` — Relayable's durable metadata/audit store.
//!
//! Implements the persistence design fixed by
//! `docs/adr/ADR-004-process-ownership-and-persistence.md` Decision 2
//! (`full.md` §6.4, §7.1, §12). The store is **metadata/audit only**: it never
//! holds terminal bytes, scrollback, environment values, or full command
//! arguments (`full.md` §6.4:490, §12:804; `ADR-001:239,286-291`).
//!
//! This crate contains no `unsafe` code; the process-level `unsafe` blocks live
//! in the crates that own OS boundaries (`relayable-pty`, `relayable-server`).
//!
//! # Migration mechanism
//!
//! Migrations are **numbered SQL migrations** applied exactly once each inside
//! its own transaction and recorded in a `schema_migrations` table
//! (`version INTEGER PRIMARY KEY`, `applied_at_millis INTEGER`). The table is
//! bootstrapped (not versioned) before any migration runs, so an unversioned or
//! empty database reads as version `0`. [`LATEST_SCHEMA_VERSION`] is `1`; a
//! database whose recorded maximum version is newer than that is rejected with
//! [`StoreError::SchemaTooNew`] rather than silently downgraded. Opening is
//! idempotent: already-applied migrations are skipped.
//!
//! # Single writer
//!
//! One [`Store`] owns one SQLite connection behind a `Mutex`; every write is a
//! single explicit transaction. A second [`Store::open`] on the same state
//! directory fails with [`StoreError::StateDirLocked`] because the directory is
//! guarded by an OS file lock (`std::fs::File::try_lock`, `flock(2)`
//! `LOCK_EX | LOCK_NB` on Linux) held for the store's lifetime. The lock is
//! acquired **before** migrations run. [`Store::open_in_memory`] is for
//! hermetic tests and takes no lock.
//!
//! # Audit sequence
//!
//! `audit_events.sequence` is the **store's local audit order** (SQLite
//! `AUTOINCREMENT`). It is not the protocol/server `sequence`, which is per-run
//! and non-durable (`ADR-004` D2.6).

#![deny(unsafe_code)]

mod error;
mod migrate;
mod model;
mod resolve;
mod store;

pub use error::StoreError;
pub use model::{
    AuditEvent, AuditEventKind, PruneLimits, PruneReport, ReconcileReport, SessionClose,
    SessionCreate, StoredSession, StoredSessionState, Tombstone,
};
pub use resolve::{resolve_state_dir, resolve_state_dir_from, StateDirSource};
pub use store::Store;

/// Default maximum number of `audit_events` rows retained (newest by local
/// `sequence`). Relayable-defined (no REX evidence); `ADR-004` D2.4.
pub const DEFAULT_AUDIT_EVENTS_MAX_ROWS: i64 = 10_000;

/// Default maximum age of an `audit_events` row: 30 days, in milliseconds.
pub const DEFAULT_AUDIT_EVENTS_MAX_AGE_MILLIS: i64 = 30 * 24 * 60 * 60 * 1000;

/// Default maximum number of `tombstones` rows retained (newest by
/// `destroy_sequence`).
pub const DEFAULT_TOMBSTONES_MAX_ROWS: i64 = 10_000;

/// Default maximum age of a `tombstones` row: 90 days, in milliseconds.
pub const DEFAULT_TOMBSTONES_MAX_AGE_MILLIS: i64 = 90 * 24 * 60 * 60 * 1000;

/// Default maximum number of `sessions` rows retained in the non-active set
/// (newest by `created_at_millis`). Relayable-defined (no REX evidence);
/// `ADR-004` D2.4.
pub const DEFAULT_SESSIONS_MAX_ROWS: i64 = 10_000;

/// Default maximum age of a `closed`/`interrupted` session row: 90 days, in
/// milliseconds.
pub const DEFAULT_SESSIONS_MAX_AGE_MILLIS: i64 = 90 * 24 * 60 * 60 * 1000;
