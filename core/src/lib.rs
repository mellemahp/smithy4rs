#![cfg_attr(rustfmt, rustfmt_skip)]

//! Core library for the `Smithy4rs` framework for [Smithy] in Rust.
//!
//! This library provides the common functionality required for
//! all generate clients, servers, and shapes using the `smithy4rs` framework.
//!
//! # Usage
//! TODO
//!
//! #
//!
//!
//! [Smithy]: https://smithy.io/

pub mod schema;

use std::sync::Arc;
pub use schema::prelude;



pub mod serde;

// Don't list macros as a module in generated docs
#[doc(hidden)]
pub mod macros;

/// Common cheaply-copyable reference type.
/// Defined as a common type so Arc type could be swapped out.
pub type Ref<T> = Arc<T>;

// =================================================================
// Re-exports of depenency types
// =================================================================

// Base types
#[doc(hidden)]
pub use bigdecimal::BigDecimal;
#[doc(hidden)]
pub use bytebuffer::ByteBuffer;
#[doc(hidden)]
pub use indexmap::IndexMap;
#[doc(hidden)]
pub use num_bigint::BigInt;
#[doc(hidden)]
pub use temporal_rs::Instant;

// For public macros
#[doc(hidden)]
pub use std::sync::LazyLock;
#[doc(hidden)]
pub use pastey;

// =================================================================
// High performance hashmaps
// -------------------------
// Faster Map and Set implementations used for internal types and Schemas.
//
// NOTE: These should _not_ be used in serialized/deserialized types
// as they are not resistant to DOS attacks.
// =================================================================
use rustc_hash::FxBuildHasher;
use indexmap::IndexSet;

pub(crate) type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;
pub(crate) type FxIndexSet<T> = IndexSet<T, FxBuildHasher>;
