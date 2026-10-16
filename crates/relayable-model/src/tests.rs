use uuid::Uuid;

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
