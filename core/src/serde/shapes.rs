//use crate::schema::SchemaShape;
//use crate::serde::documents::SerializableShape;
//use crate::serde::validation::{DefaultValidator, ValidationErrors, Validator};

use crate::{
    serde::{
        deserializers::Deserializer, documents::{DeserializableShape, SerializableShape}, validation::{DefaultValidator, ValidationErrors, Validator}
    },
};

///// Smithy Shapes should implement Serializable, Deserializable, and Schema
///// MARKER TRAIT
// TODO: Is this necessary?
pub trait SmithyShape: SerializableShape + DeserializableShape {}

/// Builder for a Smithy Shape
pub trait ShapeBuilder<'de, S: SerializableShape>: Sized {
    fn new() -> Self;

    /// Implements [Smithy Error Correction](https://smithy.io/2.0/spec/aggregate-types.html#client-error-correction)
    fn error_correction(self) -> Self {
        todo!("Should be code generated")
    }
    
    /// Deserializes data from the given deserializer into the builder.
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self, D::Error>;

    /// Build a shape. Default implementation uses the [`DefaultValidator`] for validation,
    /// which checks the basic Smithy constraints for types.
    ///
    /// Possibly raises errors if built shape is invalid.
    fn build(self) -> Result<S, ValidationErrors> {
        self.build_with_validator(&mut DefaultValidator)
    }

    /// Build a shape, using the specified validator to validate values.
    fn build_with_validator<V: Validator>(self, validator: V) -> Result<S, ValidationErrors>;
}
