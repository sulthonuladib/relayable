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

use std::fmt;
use std::str::FromStr;

use uuid::Uuid;

/// Failure modes for parsing a tagged entity ID from its `<kind>:<uuid>` text
/// form.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdError {
    /// No `:` separator was present.
    #[error("entity ID is missing a `kind:` prefix")]
    MissingKind,
    /// The kind tag is not one of the known lowercase kinds.
    #[error("unknown entity ID kind: {0}")]
    UnknownKind(String),
    /// The UUID portion is not a canonical lowercase hyphenated UUID.
    #[error("malformed UUID in entity ID: {0}")]
    BadUuid(String),
}

/// Parse the UUID portion of a tagged ID, requiring the canonical lowercase
/// hyphenated 36-character form.
fn parse_canonical_uuid(text: &str) -> Result<Uuid, IdError> {
    if text.len() != 36 {
        return Err(IdError::BadUuid(text.to_string()));
    }
    let uuid = Uuid::parse_str(text).map_err(|_| IdError::BadUuid(text.to_string()))?;
    // Reject non-canonical spellings accepted by `parse_str` (uppercase hex,
    // alternate hyphenation) so the round-trip is exact.
    if uuid.hyphenated().to_string() == text {
        Ok(uuid)
    } else {
        Err(IdError::BadUuid(text.to_string()))
    }
}

macro_rules! define_entity_id {
    ($name:ident, $kind:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Uuid);

        impl $name {
            /// Generate a fresh UUIDv7-backed identifier.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            /// Wrap an already-generated UUID.
            #[must_use]
            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// The underlying UUID.
            #[must_use]
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}:{}", $kind, self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let (kind, rest) = s.split_once(':').ok_or(IdError::MissingKind)?;
                if kind != $kind {
                    return Err(IdError::UnknownKind(kind.to_string()));
                }
                parse_canonical_uuid(rest).map(Self)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = IdError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                value.parse()
            }
        }
    };
}

define_entity_id!(
    SessionId,
    "session",
    "Identifier for a server-owned session; text form `session:<uuid>`."
);
define_entity_id!(
    WindowId,
    "window",
    "Identifier for a window; text form `window:<uuid>`."
);
define_entity_id!(
    BlockId,
    "block",
    "Identifier for a block; text form `block:<uuid>`."
);
define_entity_id!(
    ClientId,
    "client",
    "Identifier for an attached client record; text form `client:<uuid>`."
);
define_entity_id!(
    ServerId,
    "server",
    "Identifier for a [`ServerInfo`]; text form `server:<uuid>`."
);
define_entity_id!(
    HostProfileId,
    "host",
    "Client-local host-profile identifier; text form `host:<uuid>`. Never accepted as a server identity (ADR-001 Decision D)."
);
define_entity_id!(
    PtyProcessId,
    "pty",
    "Identifier for the server-owned PTY/process tree; text form `pty:<uuid>`."
);
define_entity_id!(
    ScriptId,
    "script",
    "Identifier for an automation script registry entry; text form `script:<uuid>`."
);

macro_rules! define_counter {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u64);

        impl $name {
            /// Construct from a raw value.
            #[must_use]
            pub fn new(value: u64) -> Self {
                Self(value)
            }

            /// The raw value.
            #[must_use]
            pub fn get(&self) -> u64 {
                self.0
            }
        }

        impl From<u64> for $name {
            fn from(value: u64) -> Self {
                Self(value)
            }
        }

        impl From<$name> for u64 {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_counter!(
    Revision,
    "Per-entity optimistic-concurrency token; starts at 1 and increments on every committed mutation (ADR-001 Decision F)."
);
define_counter!(
    Sequence,
    "Server-wide monotonic event order and resynchronization cursor (ADR-001 Decision F)."
);
define_counter!(
    Timestamp,
    "A Unix-epoch timestamp in milliseconds. Informational; never identity."
);

/// Lifecycle state of a session (contract §2 `SessionRecord.state`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionState {
    /// The session is live.
    Active,
    /// The session has been closed.
    Closed,
}

/// Kind of a block. Only `Terminal` exists in the first release (contract §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockKind {
    /// A terminal block.
    Terminal,
}

/// Lifecycle state of the program running inside a terminal block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProgramState {
    /// The program is running.
    Running,
    /// The program has exited.
    Exited,
}

/// Normalized program status reported by a terminal.
///
/// This is a value type; it is **not** an OSC parser. Ingestion of terminal
/// status bytes belongs to the terminal layer (ADR-001 "ProgramStatus").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramStatus {
    /// Current program state.
    pub state: ProgramState,
    /// Child exit code, when the program has exited normally.
    pub exit_code: Option<u32>,
    /// Terminating signal name, when the program was signalled.
    pub signal: Option<String>,
    /// Last known working directory.
    pub pwd: Option<String>,
    /// Last known title.
    pub title: Option<String>,
}

/// Connection lifecycle state (contract §4).
///
/// The state names are Relayable-defined (no REX evidence); the field name
/// `connection_state` is reused verbatim from the reference docs (matrix §12,
/// §13). `as_str` returns exactly the spelling in the contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    /// Initial state, or reached when the transport drops.
    Disconnected,
    /// Bytes flow but no semantics are negotiated yet.
    TransportConnected,
    /// Handshake reply exchanged; version and capabilities agree.
    Negotiated,
    /// Authentication accepted.
    Authenticated,
    /// Snapshot requested and in flight.
    Synchronizing,
    /// Live state is usable.
    Ready,
    /// A stale sequence or reconnect forced resynchronization.
    Resynchronizing,
    /// Client shutdown in progress; draining only.
    Closing,
    /// Terminal state reached after transport close.
    Closed,
    /// Terminal failure with a recorded reason.
    Failed(String),
}

impl ConnectionState {
    /// The exact contract §4 spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Disconnected => "Disconnected",
            Self::TransportConnected => "TransportConnected",
            Self::Negotiated => "Negotiated",
            Self::Authenticated => "Authenticated",
            Self::Synchronizing => "Synchronizing",
            Self::Ready => "Ready",
            Self::Resynchronizing => "Resynchronizing",
            Self::Closing => "Closing",
            Self::Closed => "Closed",
            Self::Failed(_) => "Failed",
        }
    }
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failed(reason) => write!(f, "Failed({reason})"),
            other => f.write_str(other.as_str()),
        }
    }
}

/// Terminal dimensions in character cells and (optionally) pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerminalSize {
    /// Terminal columns.
    pub cols: u16,
    /// Terminal rows.
    pub rows: u16,
    /// Pixel width, when known.
    pub pixel_width: u16,
    /// Pixel height, when known.
    pub pixel_height: u16,
}

impl TerminalSize {
    /// The conventional default terminal size (80×24, zero pixels).
    pub const DEFAULT_SIZE: TerminalSize = TerminalSize {
        cols: 80,
        rows: 24,
        pixel_width: 0,
        pixel_height: 0,
    };
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self::DEFAULT_SIZE
    }
}

/// A server-owned session group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// Stable identity.
    pub id: SessionId,
    /// Mutable human label; never identity.
    pub label: Option<String>,
    /// Optimistic-concurrency revision.
    pub revision: Revision,
    /// Lifecycle state.
    pub state: SessionState,
    /// Creation time.
    pub created_at: Timestamp,
}

/// A window within a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    /// Stable identity.
    pub id: WindowId,
    /// Owning session.
    pub session_id: SessionId,
    /// Mutable human label; never identity.
    pub label: Option<String>,
    /// Optimistic-concurrency revision.
    pub revision: Revision,
    /// Whether the window is active.
    pub active: bool,
}

/// A block within a window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// Stable identity.
    pub id: BlockId,
    /// Owning window.
    pub window_id: WindowId,
    /// Owning session (scoping; ADR-001 Decision C).
    pub session_id: SessionId,
    /// Block kind.
    pub kind: BlockKind,
    /// Mutable human label; never identity.
    pub label: Option<String>,
    /// Optimistic-concurrency revision.
    pub revision: Revision,
    /// Terminal size.
    pub size: TerminalSize,
    /// Normalized program status.
    pub program: ProgramStatus,
    /// Server-owned PTY/process identity.
    pub pty_process_id: PtyProcessId,
    /// The OS process id of the program, when known.
    pub program_pid: Option<u32>,
    /// Whether a writer lease is currently attached.
    pub writer_attached: bool,
}

/// An attached client record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Client {
    /// Stable identity.
    pub id: ClientId,
    /// Client kind (for example a platform-agnostic name).
    pub kind: String,
    /// Client platform.
    pub platform: String,
    /// Client version.
    pub version: String,
    /// Connection lifecycle state.
    pub connection_state: ConnectionState,
    /// Authenticated principal, when known (mechanism deferred).
    pub principal: Option<String>,
    /// When the client connected.
    pub connected_at: Timestamp,
}

/// A server instance descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerInfo {
    /// Stable identity.
    pub id: ServerId,
    /// Advertised protocol version, `major.minor`.
    pub protocol_version: String,
    /// Current server-wide sequence.
    pub sequence: Sequence,
    /// When the server started.
    pub started_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_round_trips_through_text() {
        let id = SessionId::new();
        let text = id.to_string();
        assert!(text.starts_with("session:"));
        assert_eq!(text.parse::<SessionId>().unwrap(), id);
    }

    #[test]
    fn every_kind_has_the_expected_prefix() {
        assert!(WindowId::new().to_string().starts_with("window:"));
        assert!(BlockId::new().to_string().starts_with("block:"));
        assert!(ClientId::new().to_string().starts_with("client:"));
        assert!(ServerId::new().to_string().starts_with("server:"));
        assert!(HostProfileId::new().to_string().starts_with("host:"));
        assert!(PtyProcessId::new().to_string().starts_with("pty:"));
        assert!(ScriptId::new().to_string().starts_with("script:"));
    }

    #[test]
    fn new_ids_are_uuid_v7() {
        assert_eq!(SessionId::new().as_uuid().get_version_num(), 7);
        assert_eq!(PtyProcessId::new().as_uuid().get_version_num(), 7);
    }

    #[test]
    fn id_is_opaque_but_traceable_to_its_uuid() {
        let uuid = Uuid::now_v7();
        let id = BlockId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), uuid);
        assert_eq!(id.to_string(), format!("block:{uuid}"));
    }

    #[test]
    fn missing_colon_is_rejected() {
        assert_eq!(
            "session01a10cf3-2655-7e48-9b9c-43dd6612bef5"
                .parse::<SessionId>()
                .unwrap_err(),
            IdError::MissingKind
        );
    }

    #[test]
    fn unknown_and_uppercase_kinds_are_rejected() {
        let rest = "01a10cf3-2655-7e48-9b9c-43dd6612bef5";
        assert_eq!(
            format!("nosuch:{rest}").parse::<SessionId>().unwrap_err(),
            IdError::UnknownKind("nosuch".to_string())
        );
        // Uppercase kind is not the lowercase kind, so it must not parse.
        assert_eq!(
            format!("Session:{rest}").parse::<SessionId>().unwrap_err(),
            IdError::UnknownKind("Session".to_string())
        );
    }

    #[test]
    fn cross_kind_parse_is_rejected() {
        let block_text = BlockId::new().to_string();
        assert!(matches!(
            block_text.parse::<SessionId>(),
            Err(IdError::UnknownKind(_))
        ));
    }

    #[test]
    fn bad_uuids_are_rejected() {
        assert!(matches!(
            "session:not-a-uuid".parse::<SessionId>(),
            Err(IdError::BadUuid(_))
        ));
        // Uppercase hex is not canonical lowercase.
        assert!(matches!(
            "session:01A10CF3-2655-7E48-9B9C-43DD6612BEF5".parse::<SessionId>(),
            Err(IdError::BadUuid(_))
        ));
    }

    #[test]
    fn try_from_str_matches_from_str() {
        let id = WindowId::new();
        assert_eq!(WindowId::try_from(id.to_string().as_str()).unwrap(), id);
    }

    #[test]
    fn counters_behave_as_ordered_u64_wrappers() {
        assert_eq!(Revision::new(3).get(), 3);
        assert_eq!(u64::from(Sequence::from(9)), 9);
        assert_eq!(Timestamp::new(5).to_string(), "5");
        assert!(Revision::new(1) < Revision::new(2));
    }

    #[test]
    fn connection_state_spelling_is_exact() {
        assert_eq!(ConnectionState::Disconnected.as_str(), "Disconnected");
        assert_eq!(
            ConnectionState::TransportConnected.as_str(),
            "TransportConnected"
        );
        assert_eq!(ConnectionState::Negotiated.as_str(), "Negotiated");
        assert_eq!(ConnectionState::Authenticated.as_str(), "Authenticated");
        assert_eq!(ConnectionState::Synchronizing.as_str(), "Synchronizing");
        assert_eq!(ConnectionState::Ready.as_str(), "Ready");
        assert_eq!(ConnectionState::Resynchronizing.as_str(), "Resynchronizing");
        assert_eq!(ConnectionState::Closing.as_str(), "Closing");
        assert_eq!(ConnectionState::Closed.as_str(), "Closed");
        assert_eq!(
            ConnectionState::Failed("boom".to_string()).as_str(),
            "Failed"
        );
        assert_eq!(
            ConnectionState::Failed("boom".to_string()).to_string(),
            "Failed(boom)"
        );
    }

    #[test]
    fn terminal_size_default_is_80x24() {
        assert_eq!(TerminalSize::default(), TerminalSize::DEFAULT_SIZE);
        assert_eq!(TerminalSize::DEFAULT_SIZE.cols, 80);
        assert_eq!(TerminalSize::DEFAULT_SIZE.rows, 24);
    }
}
