use relayable_protocol::v1::response::Result as ResponseResult;

use crate::error::{variant_name, ClientError};

pub(crate) fn parse_id<T>(
    parsed: Result<T, relayable_model::IdError>,
    kind: &str,
) -> Result<T, ClientError> {
    parsed.map_err(|err| ClientError::Protocol(format!("invalid {kind} id in response: {err}")))
}

pub(crate) fn missing(field: &str) -> ClientError {
    ClientError::Protocol(format!("response is missing {field}"))
}

pub(crate) fn unexpected(method: &str, result: &ResponseResult) -> ClientError {
    ClientError::Protocol(format!(
        "unexpected response variant for {method}: {}",
        variant_name(result)
    ))
}

pub(crate) fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack.len() >= needle.len()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}
