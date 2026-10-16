//! Store error type.

use std::path::PathBuf;

/// Errors returned by [`crate::Store`].
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Filesystem error creating the state directory, lock file, or DB file.
    #[error("state directory error: {0}")]
    Io(#[from] std::io::Error),
    /// Another process (or `Store`) already holds the single-writer lock for
    /// this state directory.
    #[error("another relayable instance already owns the state directory: {}", .0.display())]
    StateDirLocked(PathBuf),
    /// SQLite error.
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// The database records a schema version newer than this build supports.
    #[error("database schema version {found} is newer than supported version {supported}")]
    SchemaTooNew { found: u32, supported: u32 },
    /// The requested state directory is not absolute.
    #[error("state directory path must be absolute: {}", .0.display())]
    NonAbsoluteStateDir(PathBuf),
    /// A stored value could not be interpreted (for example an unknown
    /// `state` or `event_kind` string).
    #[error("invalid stored value: {0}")]
    InvalidData(String),
}
