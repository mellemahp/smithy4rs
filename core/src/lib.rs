extern crate core;

pub mod errors;
pub mod schema;
pub use schema::prelude;

pub mod macros;
pub mod serde;

// =========== Common Types ==========
use std::sync::Arc;

/// Common cheaply-copyable reference type.
/// Defined as a common type so Arc type could be swapped out.
pub type Ref<T> = Arc<T>;

pub use bigdecimal::BigDecimal;
pub use bytebuffer::ByteBuffer;
pub use num_bigint::BigInt;
pub use std::time::Instant;
