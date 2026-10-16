//! CLI failure type and its exit-code / JSON mapping.
//!
//! A [`CliError`] maps a client or target-resolution failure to a documented
//! exit code (see [`crate::exit`]) and to the stable `--json` error shape (see
//! [`crate::json`]).

use relayable_client::ClientError;
use serde_json::{Map, Value};

use crate::exit;
use crate::json;
use crate::resolve::ResolveError;

/// A failure that maps to a documented exit code.
#[derive(Debug)]
pub(crate) enum CliError {
    Client(ClientError),
    Resolve(ResolveError),
}

impl CliError {
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            Self::Client(err) => exit::for_client_error(err),
            Self::Resolve(ResolveError::NotFound(_) | ResolveError::Ambiguous { .. }) => {
                exit::TARGET
            }
            Self::Resolve(ResolveError::Missing) => exit::USAGE,
        }
    }

    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::Client(err) => exit::code_for_client_error(err),
            Self::Resolve(ResolveError::NotFound(_)) => "unknown_entity",
            Self::Resolve(ResolveError::Ambiguous { .. }) => "ambiguous_target",
            Self::Resolve(ResolveError::Missing) => "bad_request",
        }
    }

    pub(crate) fn message(&self) -> String {
        match self {
            Self::Client(ClientError::Server(err)) => err.message.clone(),
            Self::Client(err) => err.to_string(),
            Self::Resolve(ResolveError::Missing) => {
                "no session target given and no sessions exist".to_string()
            }
            Self::Resolve(ResolveError::NotFound(target)) => {
                format!("session target \"{target}\" not found")
            }
            Self::Resolve(ResolveError::Ambiguous {
                target: Some(target),
                ..
            }) => format!("session target \"{target}\" is ambiguous"),
            Self::Resolve(ResolveError::Ambiguous { target: None, .. }) => {
                "several sessions exist; specify one with -s/--session".to_string()
            }
        }
    }

    fn details(&self) -> Value {
        match self {
            Self::Client(ClientError::Server(err)) => {
                let details: Map<String, Value> = err
                    .details
                    .iter()
                    .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                    .collect();
                Value::Object(details)
            }
            Self::Resolve(ResolveError::Ambiguous { matches, .. }) => {
                json::ambiguous_details(matches)
            }
            _ => Value::Object(Map::new()),
        }
    }

    pub(crate) fn render(&self) -> Value {
        json::error_json(self.code(), &self.message(), self.details())
    }
}
