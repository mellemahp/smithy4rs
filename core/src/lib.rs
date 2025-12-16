pub mod schema;

pub use schema::prelude;

#[doc(hidden)]
pub mod macros;
pub mod serde;

use std::sync::Arc;
// =========== Common Types ==========
// Re-export for use in macros
#[doc(hidden)]
pub use std::sync::LazyLock;

#[doc(hidden)]
pub use pastey;

/// Common cheaply-copyable reference type.
/// Defined as a common type so Arc type could be swapped out.
pub type Ref<T> = Arc<T>;

#[doc(hidden)]
pub use bigdecimal::BigDecimal;
#[doc(hidden)]
pub use bytebuffer::ByteBuffer;
#[doc(hidden)]
pub use num_bigint::BigInt;
#[doc(hidden)]
pub use temporal_rs::Instant;
