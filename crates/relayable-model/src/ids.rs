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
