pub mod prelude;

pub mod traits;

pub use traits::*;

pub mod documents;

pub mod macros;
//pub use macros::*;

pub mod shapes;
pub use shapes::*;

pub mod schema;
pub use schema::*;

// =========== Common Types ==========
use std::sync::Arc;

/// Common cheaply-copyable reference type.
/// Defined as a common type so Arc type could be swapped out.
pub(crate) type Ref<T> = Arc<T>;

pub use bigdecimal::BigDecimal;
pub use num_bigint::BigInt;
pub use bytebuffer::ByteBuffer;
