pub mod schema;

pub use schema::prelude;

pub mod macros;
pub mod serde;

// =========== Common Types ==========
use std::sync::Arc;
// Re-export for use in macros
#[doc(hidden)]
pub use std::sync::LazyLock;
#[doc(hidden)]
pub use paste;

/// Common cheaply-copyable reference type.
/// Defined as a common type so Arc type could be swapped out.
pub type Ref<T> = Arc<T>;

/// Faster Map and Set implementations used for internal types and Schemas.
///
/// NOTE: These should not be used in serialized/deserialized types as they are not
/// resistant to DOS attacks.
pub type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;
pub type FxIndexSet<T> = IndexSet<T, FxBuildHasher>;

pub use bigdecimal::BigDecimal;
pub use bytebuffer::ByteBuffer;
use indexmap::{IndexMap, IndexSet};
pub use num_bigint::BigInt;
use rustc_hash::FxBuildHasher;
pub use temporal_rs::Instant;
