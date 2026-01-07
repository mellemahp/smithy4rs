//! # Shape Builders
//!
//! Enable the construction of shape manually or through a deserializer.
//!
//! ## Derived Builders
//!
//! The `SmithyShape` derive macro will automatically generate a builder
//! for a Smithy shape.
//!
//! For example:
//! ```rust,ignore
//!  #[derive(SmithyShape)]
//!  #[smithy_schema(SCHEMA)]
//!  pub struct Test {
//!      #[smithy_schema(A)]
//!      a: String,
//!  }
//! ```
//!
//! Will automatically generate a `TestBuilder` structure that can be used to
//! construct and instance of the `Test` shape:
//! ```rust,ignore
//! let built: Test = Test::builder().a("stuff".into()).build()
//! ```

use crate::{
    schema::{Document, DocumentError, SchemaRef, StaticSchemaShape},
    serde::{
        correction::{ErrorCorrection, ErrorCorrectionDefault},
        deserializers::DeserializeWithSchema,
        se::{SerializeWithSchema, Serializer},
        validation::{DefaultValidator, ValidationErrors, Validator},
    },
};
//============================================================================
// Builder Traits
//============================================================================

/// Builder for a Smithy Shape
///
/// Used during deserialization to accumulate field values
/// before constructing the final shape. Builders implement
/// [`DeserializeWithSchema`] to handle the actual deserialization logic.
///
pub trait ShapeBuilder<'de, S: StaticSchemaShape>:
    Sized + DeserializeWithSchema<'de> + SerializeWithSchema + ErrorCorrection<Value = S>
{
    /// Create a new builder with all fields unset
    fn new() -> Self;

    /// Build the final shape from the builder
    ///
    /// Builds shape with the [`DefaultValidator`] if required fields
    /// are missing or validation fails
    ///
    /// # Errors
    /// Returns validation errors detected by the `DefaultValidator`
    #[inline]
    fn build(self) -> Result<S, ValidationErrors> {
        self.build_with_validator(&mut DefaultValidator::new())
    }

    /// Build the final shape from the builder, checking fields using a
    /// custom [`Validator`] implementation.
    ///
    /// To build a shape using the default validator use [`Self::build()`].
    ///
    /// <div class="note">
    /// **NOTE**: Validation is supported by the [`SerializeWithSchema`] implementation
    /// for the builder.
    /// </div>
    ///
    /// # Errors
    /// Returns aggregate validation error containing all validation errors detected by the
    /// selected validator.
    #[inline]
    fn build_with_validator(self, validator: impl Validator) -> Result<S, ValidationErrors> {
        validator.validate(S::schema(), &self)?;
        Ok(self.correct())
    }

    /// Deserialize a document into this builder.
    ///
    /// Note that the builder still needs to be built and validated
    /// after conversion from a document.
    ///
    /// # Errors
    /// If the builder could not be converted into a valid document implementation.
    #[inline]
    fn from_document(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.into_builder::<Self, S>()
    }
}

/// Shape that can create a builder for deserialization
pub trait Buildable<'de, B>
where
    Self: Sized + StaticSchemaShape,
    B: ShapeBuilder<'de, Self>,
{
    /// Get a new builder for this shape
    #[must_use]
    fn builder() -> B {
        B::new()
    }
}

//============================================================================
// Builder Adapter Types
//============================================================================

/// Indicates that a field is required to be set in a builder.
///
/// This type allows us to track if a type was set or not
/// during construction of a shape.
#[derive(Clone)]
pub enum Required<T: ErrorCorrectionDefault> {
    /// A value that has been set
    Set(T),
    /// An unset value
    Unset,
}
impl<T: ErrorCorrectionDefault> Required<T> {
    /// Resolves the required value, returning an error correction default if unset.
    #[inline]
    pub fn get(self) -> T {
        match self {
            Required::Unset => T::default(),
            Required::Set(v) => v,
        }
    }
}
impl<T: SerializeWithSchema + ErrorCorrectionDefault> SerializeWithSchema for Required<T> {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match self {
            Required::Set(value) => value.serialize_with_schema(schema, serializer),
            Required::Unset => serializer.write_missing(schema),
        }
    }
}

/// A builder field that could contain either fully built shapes or shape builders.
///
/// This type allows us to deserialize into builders and convert lazily to built shapes
/// after validation.
#[derive(Clone)]
pub enum MaybeBuilt<
    S: ErrorCorrectionDefault + SerializeWithSchema,
    B: ErrorCorrection<Value = S> + SerializeWithSchema,
> {
    /// A built structure
    Struct(S),
    /// A builder for a structure
    Builder(B),
}
impl<
    E: ErrorCorrectionDefault + SerializeWithSchema,
    B: ErrorCorrection<Value = E> + SerializeWithSchema,
> SerializeWithSchema for MaybeBuilt<E, B>
{
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match self {
            MaybeBuilt::Struct(value) => value.serialize_with_schema(schema, serializer),
            MaybeBuilt::Builder(value) => value.serialize_with_schema(schema, serializer),
        }
    }
}
