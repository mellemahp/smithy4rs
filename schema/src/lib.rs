pub mod prelude;

mod traits;
pub use traits::*;

mod documents;
pub use documents::*;

pub mod macros;
//pub use macros::*;

mod shapes;
pub use shapes::*;

mod schema;
pub use schema::*;

// =========== Common Types ==========
use std::sync::Arc;

/// Common cheaply-copyable reference type.
/// Defined as a common type so Arc type could be swapped out.
pub type Ref<T> = Arc<T>;

pub use bigdecimal::BigDecimal;
pub use num_bigint::BigInt;
pub use bytebuffer::ByteBuffer;
pub use std::time::Instant;