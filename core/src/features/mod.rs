#[cfg(feature = "serde-adapters")]
/// Adapters to bridge between `serde` and schema-guided (de)serialization
/// for Smithy shapes.
pub mod adapters;

#[cfg(feature = "arbitrary")]
/// Tools to implement `Arbitrary` trait for generated shapes.
/// This allows generated shapes to support structured fuzzing.
pub mod arbitrary;

#[cfg(feature = "documents")]
/// Support for document types
///
/// See: <https://smithy.io/2.0/spec/simple-types.html#document>
pub mod documents;
