#![allow(unused_variables, dead_code)]

use crate::{
    schema::SchemaRef,
    serde::{
        deserializers::Deserializer,
        documents::{DeserializableShape, SerializableShape},
        validation::{DefaultValidator, ValidationErrors, Validator},
    },
};

///// Smithy Shapes should implement Serializable, Deserializable, and Schema

///// MARKER TRAIT

// TODO: Is this necessary?

pub trait SmithyShape: SerializableShape + DeserializableShape {}

/// Shape that can be built with a [`ShapeBuilder`].

pub trait Builder
where
    Self: Sized,
{
    type Builder: ShapeBuilder<Self>;

    /// Get a new builder for this shape.

    ///

    /// By default, delegates to [`Self::Builder::new()`]

    fn builder() -> Self::Builder {
        <Self::Builder as ShapeBuilder<Self>>::new()
    }

    /// Convert a shape to a builder

    fn to_builder(&self) -> Self::Builder {
        todo!("This should be implemented for each type by codegen")
    }
}

/// Builder for a Smithy Shape

pub trait ShapeBuilder<S>: Sized {
    fn new() -> Self;

    /// Implements [Smithy Error Correction](https://smithy.io/2.0/spec/aggregate-types.html#client-error-correction)

    fn error_correction(self) -> Self {
        todo!("Should be code generated")
    }

    /// Deserializes data into the builder for a single member.

    fn deserialize_member<'de, D: Deserializer<'de>>(
        &mut self,

        member_schema: &SchemaRef,

        deserializer: D,
    ) -> Result<(), D::Error>;

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

/// Enum representing either a

///

/// Used to allow the input of both structures and their builders

pub enum StructOrBuilder<S: Builder> {
    Structure(S),

    Builder(S::Builder),
}

impl<S: Builder> From<S> for StructOrBuilder<S> {
    fn from(builder: S) -> Self {
        StructOrBuilder::Structure(builder)
    }
}
