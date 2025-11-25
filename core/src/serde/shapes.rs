use crate::{schema::SchemaShape, serde::deserializers::DeserializeWithSchema};
use crate::serde::validation::{DefaultValidator, ValidationErrors, Validator};

/// Builder for a Smithy Shape
///
/// Used during deserialization to accumulate field values
/// before constructing the final shape. Builders implement
/// [`DeserializeWithSchema`] to handle the actual deserialization logic.
pub trait ShapeBuilder<'de, S>: Sized + DeserializeWithSchema<'de> {
    /// Create a new builder with all fields unset
    fn new() -> Self;

    /// Build the final shape from the builder
    ///
    /// Builds shape with the default validator if required fields
    /// are missing or validation fails
    fn build(self) -> Result<S, ValidationErrors> {
        self.build_with_validator(DefaultValidator::new())
    }

    /// Build the final shape from the builder, checking fields using a
    /// custom [`Validator`] implementation.
    ///
    /// To build a shape using the default validator use [`ShapeBuilder::build`]
    fn build_with_validator<V>(self, validator: V) -> Result<S, ValidationErrors>
    where for<'a> &'a mut V: Validator {
        todo!()
    }
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
