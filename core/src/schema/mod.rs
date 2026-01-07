/// Core Smithy shape and trait definitions
pub mod prelude;

mod traits;
pub use traits::*;

mod documents;
pub use documents::*;

mod shapes;
pub use shapes::*;

mod schemas;
pub use schemas::*;

mod unit;
// Do not include the unit trait as it can remain private.
pub use unit::{UNIT, Unit};
