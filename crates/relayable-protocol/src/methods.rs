//! Method-name vocabulary for the CORE-001 slice, extended by CORE-002.
//!
//! The frozen contract (`docs/architecture/interface-contract.md` §3) fixes the
//! shape of `Request` but deliberately leaves the first-release method list to
//! the server-API task — which is this slice. These constants are that list.
//!
//! Evidence per constant:
//!
//! - `session.create`, `session.list`, `session.view` are documented server
//!   methods (matrix §8.2 Schemas and examples, §8.3 Calling methods).
//! - `session.close` mirrors the documented CLI family `session close`
//!   (`full.md` §8.1) and the documented `block.close` semantics.
//! - `block.attach`, `block.detach`, `terminal.input`, `terminal.resize` are
//!   Relayable-defined (no REX evidence) for CORE-001.
//! - `session.resume` is Relayable-defined (no REX evidence); it implements the
//!   contract §2 resume cursor `(scope, last_sequence_applied)` (`full.md`
//!   §6.1:455) and is frozen in `protocol/compatibility/core-002-method-set.md`.
//!
//! The shipped binary/CLI is `relayable` (`full.md` §0.1); the matrix's `rex`
//! spellings are reference evidence only.

/// Create a session: `session.create`.
pub const SESSION_CREATE: &str = "session.create";
/// List sessions: `session.list`.
pub const SESSION_LIST: &str = "session.list";
/// Fetch a session snapshot: `session.view`.
pub const SESSION_VIEW: &str = "session.view";
/// Close a session: `session.close`.
pub const SESSION_CLOSE: &str = "session.close";
/// Attach to a terminal block: `block.attach`.
pub const BLOCK_ATTACH: &str = "block.attach";
/// Detach from a terminal block: `block.detach`.
pub const BLOCK_DETACH: &str = "block.detach";
/// Send terminal input: `terminal.input`.
pub const TERMINAL_INPUT: &str = "terminal.input";
/// Resize a terminal: `terminal.resize`.
pub const TERMINAL_RESIZE: &str = "terminal.resize";
/// Resume from a server-wide event-log cursor: `session.resume` (CORE-002).
pub const SESSION_RESUME: &str = "session.resume";

/// Every implemented method name (CORE-001 plus the CORE-002 `session.resume`).
pub const ALL: [&str; 9] = [
    SESSION_CREATE,
    SESSION_LIST,
    SESSION_VIEW,
    SESSION_CLOSE,
    BLOCK_ATTACH,
    BLOCK_DETACH,
    TERMINAL_INPUT,
    TERMINAL_RESIZE,
    SESSION_RESUME,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_strings_are_exact() {
        assert_eq!(SESSION_CREATE, "session.create");
        assert_eq!(SESSION_LIST, "session.list");
        assert_eq!(SESSION_VIEW, "session.view");
        assert_eq!(SESSION_CLOSE, "session.close");
        assert_eq!(BLOCK_ATTACH, "block.attach");
        assert_eq!(BLOCK_DETACH, "block.detach");
        assert_eq!(TERMINAL_INPUT, "terminal.input");
        assert_eq!(TERMINAL_RESIZE, "terminal.resize");
        assert_eq!(SESSION_RESUME, "session.resume");
        assert_eq!(ALL.len(), 9);
    }

    #[test]
    fn session_resume_resolves_via_the_method_registry() {
        assert!(ALL.contains(&SESSION_RESUME));
    }
}
