use crate::{
    schema::{Document, DocumentError, SchemaRef, StaticSchemaShape},
    serde::{
        correction::{ErrorCorrection, ErrorCorrectionDefault},
        deserializers::DeserializeWithSchema,
        documents::DocumentDeserializer,
        se::{SerializeWithSchema, Serializer},
        validate::DefaultValidator,
        validation::{ValidationErrors, Validator},
    },
};

/// Builder for a Smithy Shape
///
/// Used during deserialization to accumulate field values
/// before constructing the final shape. Builders implement
/// [`DeserializeWithSchema`] to handle the actual deserialization logic.
///
/// ### Derived Builders
/// The `SmithyShape` derive macro will automatically generate a builder
/// for Smithy shape.
///
/// For example:
/// ```rust,ignore
///  #[derive(SmithyShape)]
///  #[smithy_schema(SCHEMA)]
///  pub struct Test {
///      #[smithy_schema(A)]
///      a: String,
///  }
/// ```
///
/// Will automatically generate a `TestBuilder` structure that can be used to
/// construct and instance of the `Test` shape:
/// ```rust,ignore
/// let built: Test = Test::builder().a("stuff".into()).build()
/// ```
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
    #[inline]
    fn build_with_validator(self, validator: impl Validator) -> Result<S, ValidationErrors> {
        validator.validate(S::schema(), &self)?;
        Ok(self.correct())
    }

    /// Deserialize a document into this builder.
    ///
    /// Note that the builder still needs to be built and validated
    /// after conversion from a document.
    #[inline]
    fn from_document(document: Document) -> Result<Self, DocumentError> {
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
    fn builder() -> B {
        B::new()
    }
}

// Conversion convenience method as we cannot have generic `TryFrom` impl
// due to orphan rules
impl Document {
    /// Convert a document into a builder
    ///
    /// Note that the builder still needs to be built and validated
    /// after conversion from a document.
    #[inline]
    pub(crate) fn into_builder<'de, B: ShapeBuilder<'de, S>, S: StaticSchemaShape>(
        self,
    ) -> Result<B, DocumentError> {
        B::deserialize_with_schema(S::schema(), &mut DocumentDeserializer::new(self))
    }
}

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
