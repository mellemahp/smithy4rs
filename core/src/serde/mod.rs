#[cfg(feature = "serde-adapters")]
/// Adapters to bridge between `serde` and schema-guided (de)serialization
/// for Smithy shapes.
pub mod adapters;

mod builders;
pub use builders::*;

pub mod correction;
pub mod debug;
pub mod deserializers;
mod documents;
pub use documents::*;
pub mod serializers;
mod unit;

pub mod validation;
pub use deserializers as de;
pub use serializers as se;
