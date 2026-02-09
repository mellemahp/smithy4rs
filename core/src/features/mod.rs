#[cfg(feature = "serde-adapters")]
/// Adapters to bridge between `serde` and schema-guided (de)serialization
/// for Smithy shapes.
pub mod adapters;

#[cfg(feature = "arbitrary")]
/// Tools to implement `Arbitrary` trait for generated shapes.
/// This allows generated shapes to support structured fuzzing.
pub mod arbitrary;
