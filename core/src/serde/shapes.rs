use crate::{schema::SchemaShape, serde::deserializers::DeserializeWithSchema};

/// Builder for a Smithy Shape
///
/// Builders are used during deserialization to accumulate field values
/// before constructing the final shape. They implement DeserializeWithSchema
/// to handle the actual deserialization logic.
pub trait ShapeBuilder<'de, S>: Sized + DeserializeWithSchema<'de> {
    /// Error type returned by build
    type Error: std::fmt::Display;

    /// Create a new builder with all fields unset
    fn new() -> Self;

    /// Build the final shape from the builder
    ///
    /// Returns an error if required fields are missing or validation fails
    fn build(self) -> Result<S, Self::Error>;
}

/// Shape that can create a builder for deserialization
pub trait Buildable<'de, B>
where
    Self: Sized + SchemaShape,
    B: ShapeBuilder<'de, Self>,
{
    /// Get a new builder for this shape
    fn builder() -> B {
        B::new()
    }
}
