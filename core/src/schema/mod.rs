pub mod prelude;

pub mod traits;

use triomphe::Arc;
pub use traits::core::*;

pub mod documents;

mod schema;
pub use schema::*;

pub mod macros;
//pub use macros::*;

pub mod shapes;
pub use shapes::*;

/// Common cheaply-copyable reference type.
/// Defined as a common type so Arc type could be swapped out.
pub(crate) type Ref<T> = Arc<T>;