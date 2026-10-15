//! Relayable versioned protocol.
//!
//! This crate owns the wire schema, the `model <-> wire` conversion layer, the
//! length-prefixed frame codec, the method-name/error-code vocabularies, and
//! (behind the non-default `transport-unix` feature) the local Unix-domain
//! socket transport.
//!
//! It depends on [`relayable_model`] but the dependency is one-way: the domain
//! never depends on the protocol (ADR-001 Decision M).
//!
//! The generated protobuf types live in the [`v1`] module.
//! `protocol/compatibility/core-001-method-set.md` is the frozen CORE-001
//! method/message set; `docs/architecture/interface-contract.md` is the parent
//! contract, which this crate does not change.

#![deny(unsafe_code)]

pub mod codec;
pub mod error_code;
pub mod ids;
pub mod limits;
pub mod methods;
#[cfg(feature = "transport-unix")]
pub mod transport;

/// Generated types for protocol package `relayable.protocol.v1`.
pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/relayable.protocol.v1.rs"));
}

/// Advertised protocol version, `major.minor`.
///
/// Minor changes are additive only; a client whose major is unsupported
/// receives the structured `incompatible_version` error (contract §3, §6).
pub const PROTOCOL_VERSION: &str = "1.0";

/// The control stream id: every connection multiplexes logical streams and
/// `stream_id = 0` is always control (contract §3).
pub const CONTROL_STREAM_ID: u64 = 0;
