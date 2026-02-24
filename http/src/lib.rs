#![deny(missing_docs)]

//! HTTP binding support for the `smithy4rs` framework.
//!
//! This crate provides the foundation for HTTP protocol bindings in smithy4rs.
//! It partitions struct members across HTTP binding locations (URI labels, query
//! parameters, headers, body) based on Smithy HTTP traits, and delegates body
//! serialization to a [`Codec`](codec::Codec) implementation.

/// Member partitioning by HTTP binding location.
pub mod binding;
/// Error types for HTTP binding operations.
pub mod error;

/// HTTP request and response deserialization.
pub(crate) mod deserialize;
/// HTTP header serialization and deserialization.
pub(crate) mod header;
/// HTTP URI label serialization and deserialization.
pub(crate) mod label;
/// HTTP query parameter serialization and deserialization.
pub(crate) mod query;
/// HTTP request and response serialization.
pub(crate) mod serialize;
/// URI pattern parsing.
pub(crate) mod uri;
