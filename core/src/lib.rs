extern crate core;

pub mod errors;
pub mod schema;
pub use schema::prelude as prelude;

pub mod serde;
pub mod macros;

// =========== Common Types ==========
use std::sync::Arc;

/// Common cheaply-copyable reference type.
/// Defined as a common type so Arc type could be swapped out.
pub type Ref<T> = Arc<T>;

pub use bigdecimal::BigDecimal;
pub use num_bigint::BigInt;
pub use bytebuffer::ByteBuffer;
pub use std::time::Instant;
