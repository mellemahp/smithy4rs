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
    schema::{Document, DocumentError, Schema, StaticSchemaShape},
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
pub trait BuildWithSchema<S>: Sized + SerializeWithSchema + ErrorCorrection<Value = S> {
    /// Create a new builder with all fields unset
    fn new() -> Self;

    /// Build the final shape from the builder using an explicit schema.
    ///
    /// Builds shape with the [`DefaultValidator`] if required fields
    /// are missing or validation fails.
    ///
    /// # Errors
    /// Returns validation errors detected by the `DefaultValidator`
    #[inline]
    fn build_with_schema(self, schema: &Schema) -> Result<S, ValidationErrors> {
        self.build_with_schema_and_validator(schema, &mut DefaultValidator::new())
    }

    /// Build the final shape from the builder using an explicit schema,
    /// checking fields using a custom [`Validator`] implementation.
    ///
    /// To build a shape using the default validator use [`Self::build_with_schema()`].
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
    fn build_with_schema_and_validator(
        self,
        schema: &Schema,
        validator: impl Validator,
    ) -> Result<S, ValidationErrors> {
        validator.validate(schema, &self)?;
        Ok(self.correct())
    }

    /// Deserialize a document into this builder using an explicit schema.
    ///
    /// Note that the builder still needs to be built and validated
    /// after conversion from a document.
    ///
    /// # Errors
    /// If the builder could not be converted into a valid document implementation.
    #[inline]
    fn from_document_with_schema(
        document: Box<dyn Document>,
        schema: &Schema,
    ) -> Result<Self, DocumentError>
    where
        Self: for<'de> DeserializeWithSchema<'de>,
    {
        document.into_builder_with_schema::<Self>(schema)
    }

    /// Build the final shape using the shape's static schema.
    ///
    /// Convenience method equivalent to
    /// `self.build_with_schema(S::schema())`.
    ///
    /// # Errors
    /// Returns validation errors detected by the [`DefaultValidator`].
    #[inline]
    fn build(self) -> Result<S, ValidationErrors>
    where
        S: StaticSchemaShape,
    {
        self.build_with_schema(S::schema())
    }

    /// Build the final shape using the shape's static schema with a custom validator.
    ///
    /// Convenience method equivalent to
    /// `self.build_with_schema_and_validator(S::schema(), validator)`.
    ///
    /// # Errors
    /// Returns aggregate validation errors detected by the selected validator.
    #[inline]
    fn build_with_validator(self, validator: impl Validator) -> Result<S, ValidationErrors>
    where
        S: StaticSchemaShape,
    {
        self.build_with_schema_and_validator(S::schema(), validator)
    }

    /// Deserialize a document into this builder using the shape's static schema.
    ///
    /// Convenience method equivalent to
    /// `Self::from_document_with_schema(document, S::schema())`.
    ///
    /// # Errors
    /// If the builder could not be converted into a valid document implementation.
    #[inline]
    fn from_document(document: Box<dyn Document>) -> Result<Self, DocumentError>
    where
        S: StaticSchemaShape,
        Self: for<'de> DeserializeWithSchema<'de>,
    {
        Self::from_document_with_schema(document, S::schema())
    }
}

/// Connects a shape to its builder type.
///
/// Implemented on the **shape** (not the builder) by the `SmithyShape` derive macro.
/// Provides a type-level mapping from any derived shape to its builder.
pub trait BuildableShape: Sized {
    /// The builder type that can construct this shape.
    type Builder: BuildWithSchema<Self> + for<'de> DeserializeWithSchema<'de>;

    /// Get a new builder for this shape.
    #[inline]
    #[must_use]
    fn builder() -> Self::Builder {
        Self::Builder::new()
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
        schema: &Schema,
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
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match self {
            MaybeBuilt::Struct(value) => value.serialize_with_schema(schema, serializer),
            MaybeBuilt::Builder(value) => value.serialize_with_schema(schema, serializer),
        }
    }
}
