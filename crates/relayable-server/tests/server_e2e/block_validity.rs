//! A closed session's block is unresolvable and stale stream entries are
//! rejected.

use relayable_protocol::methods;
use relayable_protocol::v1::{
    request, response, AttachMode, BlockAttachRequest, BlockDetachRequest, ErrorCode,
    SessionCloseRequest, TerminalInput,
};

use crate::harness::*;

/// m3/m7: a closed session's block is unresolvable, and stale stream entries
/// pointing at the dead block answer `unknown_entity`.
#[test]
fn closed_session_block_and_streams_are_unknown() {
    let mut server = ServerGuard::spawn();
    let mut client = Client::new(server.connect());
    client.handshake();
    let created = create(&mut client, "m3");
    let session_id = created
        .session
        .as_ref()
        .expect("session")
        .session_id
        .clone();
    let block_id = created.block_id.clone();
    let child_pid = created.program_pid;
    server.track(child_pid);
    let stream_id = attach(&mut client, &block_id, AttachMode::Writer).stream_id;

    match client.request(
        methods::SESSION_CLOSE,
        request::Operation::SessionClose(SessionCloseRequest {
            session_id: session_id.clone(),
        }),
    ) {
        response::Result::SessionClose(_) => {}
        other => panic!("session.close failed: {other:?}"),
    }
    assert!(wait_gone(child_pid, RETRY));

    // m3: the closed block cannot be attached.
    expect_error(
        client.request(
            methods::BLOCK_ATTACH,
            request::Operation::BlockAttach(BlockAttachRequest {
                block_id: block_id.clone(),
                mode: AttachMode::Writer as i32,
                takeover: false,
            }),
        ),
        ErrorCode::UnknownEntity,
    );
    // m7: stale stream entries answer unknown_entity.
    expect_error(
        client.request(
            methods::TERMINAL_INPUT,
            request::Operation::TerminalInput(TerminalInput {
                stream_id,
                bytes: b"x".to_vec(),
            }),
        ),
        ErrorCode::UnknownEntity,
    );
    expect_error(
        client.request(
            methods::BLOCK_DETACH,
            request::Operation::BlockDetach(BlockDetachRequest { stream_id }),
        ),
        ErrorCode::UnknownEntity,
    );
}
