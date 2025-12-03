use crate::{serde::deserializers::DeserializeWithSchema};
use crate::schema::{SchemaRef, StaticSchemaShape};
use crate::serde::correction::{ErrorCorrection, ErrorCorrectionDefault};
use crate::serde::se::{SerializeWithSchema, Serializer};
use crate::serde::validate::Validate;
use crate::serde::validation::{ValidationErrors, Validator};

/// Builder for a Smithy Shape
///
/// Used during deserialization to accumulate field values
/// before constructing the final shape. Builders implement
/// [`DeserializeWithSchema`] to handle the actual deserialization logic.
pub trait ShapeBuilder<'de, S: StaticSchemaShape>: Sized + DeserializeWithSchema<'de> + Validate + ErrorCorrection<Value=S> {
    /// Create a new builder with all fields unset
    fn new() -> Self;

    /// Build the final shape from the builder
    ///
    /// Builds shape with the default validator if required fields
    /// are missing or validation fails
    fn build(self) -> Result<S, ValidationErrors> {
        self.build_with_validator(Validator::new())
    }

    /// Build the final shape from the builder, checking fields using a
    /// custom [`Validator`] implementation.
    ///
    /// To build a shape using the default validator use [`ShapeBuilder::build`].
    ///
    /// NOTE: Actual validation and build logic is implementated in builder [`Validate`]
    /// implementation.
    fn build_with_validator(self, mut validator: Validator) -> Result<S, ValidationErrors> {
        validator.validate(S::schema(), &self)?;
        Ok(self.correct())
    }
}

/// Shape that can create a builder for deserialization
pub trait Buildable<'de, B>
where
    Self: Sized + StaticSchemaShape,
    B: ShapeBuilder<'de, Self>,
{
    /// Get a new builder for this shape
    fn builder() -> B {
        B::new()
    }
}

/// Indicates that a field is required to be set in a builder.
///
/// This type allows us to track if a type was set or not
/// during construction of a shape.
#[derive(Clone)]
pub enum Required<T: ErrorCorrectionDefault> {
    Set(T),
    Unset
}
impl <T: ErrorCorrectionDefault> Required<T> {
    #[inline]
    pub fn get(self) -> T {
        match self {
            Required::Unset => T::default(),
            Required::Set(v) => v
        }
    }
}
impl <T: SerializeWithSchema + ErrorCorrectionDefault> SerializeWithSchema for Required<T> {
    fn serialize_with_schema<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Required::Set(value) => value.serialize_with_schema(schema, serializer),
            Required::Unset => serializer.write_missing(schema)
        }
    }
}

/// A builder field that could contain either fully built shapes or shape builders.
///
/// This type allows us to deserialize into builders and convert lazily to built shapes
/// after validation.
pub enum MaybeBuilt<S: ErrorCorrectionDefault + SerializeWithSchema, B: ErrorCorrection<Value=S> + SerializeWithSchema> {
    Struct(S),
    Builder(B),
}
impl <E: ErrorCorrectionDefault + SerializeWithSchema, B: ErrorCorrection<Value=E> + SerializeWithSchema> SerializeWithSchema for MaybeBuilt<E, B> {
    fn serialize_with_schema<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            MaybeBuilt::Struct(value) => value.serialize_with_schema(schema, serializer),
            MaybeBuilt::Builder(value) => value.serialize_with_schema(schema, serializer),
        }
    }
}