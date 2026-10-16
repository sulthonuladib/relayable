//! Relayable domain model.
//!
//! This crate owns the entity types, tagged identifiers, revisions, and
//! snapshot/value types described by `docs/architecture/interface-contract.md`
//! §1–§2 and `docs/adr/ADR-001-domain-model-and-dependency-boundaries.md`.
//!
//! # Purity rule (ADR-001)
//!
//! Model types must not depend on GPUI, Android, OS views, or transport
//! (`full.md` §5.2). Concretely this crate has **no** dependency on a
//! serialization framework, async runtime, socket/transport type, or any
//! `std::os::*` API, so it compiles unchanged for both the Linux and the
//! future Android client.
//!
//! # Identity
//!
//! IDs are opaque, stable, never reused, and encoded as `<kind>:<uuid>` with a
//! singular lowercase kind (`full.md` §4:329, §4:343; ADR-001 Decision B).
//! The UUID portion is UUIDv7 (RFC 9562 §5.7). IDs are **not** security
//! capabilities (RFC 9562 §8): authorization must never rely on possessing one.
//!
//! Target resolution (labels, positions, ID suffixes) is deliberately **not**
//! implemented here; `docs/architecture/interface-contract.md` §1 assigns it to
//! `relayable-cli` and the docs layer.

#![deny(unsafe_code)]

mod entities;
mod ids;
mod size;
mod state;

#[cfg(test)]
mod tests;

pub use entities::{Block, Client, ServerInfo, Session, Window};
pub use ids::{
    BlockId, ClientId, HostProfileId, IdError, PtyProcessId, Revision, ScriptId, Sequence,
    ServerId, SessionId, Timestamp, WindowId,
};
pub use size::TerminalSize;
pub use state::{BlockKind, ConnectionState, ProgramState, ProgramStatus, SessionState};
