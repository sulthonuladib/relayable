//! `model <-> wire` conversions for tagged entity IDs.
//!
//! Public APIs of this crate take and return the `relayable-model` newtypes,
//! never bare `String` (contract §1). The `*_from_wire` functions fail with
//! [`relayable_model::IdError`] rather than silently accepting a malformed or
//! cross-kind ID.

use relayable_model::{
    BlockId, ClientId, HostProfileId, IdError, PtyProcessId, ScriptId, ServerId, SessionId,
    WindowId,
};

macro_rules! wire_id {
    ($to_fn:ident, $from_fn:ident, $ty:ty, $doc:literal) => {
        #[doc = concat!("Render ", $doc, " in its `<kind>:<uuid>` wire/CLI form.")]
        #[must_use]
        pub fn $to_fn(id: &$ty) -> String {
            id.to_string()
        }

        #[doc = concat!("Parse ", $doc, " from its `<kind>:<uuid>` wire/CLI form.")]
        pub fn $from_fn(text: &str) -> Result<$ty, IdError> {
            text.parse()
        }
    };
}

wire_id!(
    session_id_to_wire,
    session_id_from_wire,
    SessionId,
    "a session ID"
);
wire_id!(
    window_id_to_wire,
    window_id_from_wire,
    WindowId,
    "a window ID"
);
wire_id!(block_id_to_wire, block_id_from_wire, BlockId, "a block ID");
wire_id!(
    client_id_to_wire,
    client_id_from_wire,
    ClientId,
    "a client ID"
);
wire_id!(
    server_id_to_wire,
    server_id_from_wire,
    ServerId,
    "a server ID"
);
wire_id!(
    pty_process_id_to_wire,
    pty_process_id_from_wire,
    PtyProcessId,
    "a PTY/process ID"
);
wire_id!(
    host_profile_id_to_wire,
    host_profile_id_from_wire,
    HostProfileId,
    "a client-local host-profile ID"
);
wire_id!(
    script_id_to_wire,
    script_id_from_wire,
    ScriptId,
    "a script ID"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_id_kinds_round_trip() {
        let session = SessionId::new();
        assert_eq!(
            session_id_from_wire(&session_id_to_wire(&session)).unwrap(),
            session
        );

        let window = WindowId::new();
        assert_eq!(
            window_id_from_wire(&window_id_to_wire(&window)).unwrap(),
            window
        );

        let block = BlockId::new();
        assert_eq!(
            block_id_from_wire(&block_id_to_wire(&block)).unwrap(),
            block
        );

        let client = ClientId::new();
        assert_eq!(
            client_id_from_wire(&client_id_to_wire(&client)).unwrap(),
            client
        );

        let server = ServerId::new();
        assert_eq!(
            server_id_from_wire(&server_id_to_wire(&server)).unwrap(),
            server
        );

        let pty = PtyProcessId::new();
        assert_eq!(
            pty_process_id_from_wire(&pty_process_id_to_wire(&pty)).unwrap(),
            pty
        );

        let host = HostProfileId::new();
        assert_eq!(
            host_profile_id_from_wire(&host_profile_id_to_wire(&host)).unwrap(),
            host
        );

        let script = ScriptId::new();
        assert_eq!(
            script_id_from_wire(&script_id_to_wire(&script)).unwrap(),
            script
        );
    }

    #[test]
    fn malformed_ids_are_rejected() {
        assert!(session_id_from_wire("block:01a10d99-0c06-7795-bf67-798ce4c36a33").is_err());
        assert!(block_id_from_wire("nope").is_err());
    }
}
