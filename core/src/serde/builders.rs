use std::hash::Hash;
use indexmap::IndexMap;
use serde::Serialize;
use crate::{serde::deserializers::DeserializeWithSchema};
use crate::schema::{SchemaRef, StaticSchemaShape};
use crate::serde::se::{SerializeWithSchema, Serializer};
use crate::serde::validate::{SmithyConstraints, Validate};
use crate::serde::validation::{ErrorCorrection, ValidationErrors, Validator};

/// Builder for a Smithy Shape
///
/// Used during deserialization to accumulate field values
/// before constructing the final shape. Builders implement
/// [`DeserializeWithSchema`] to handle the actual deserialization logic.
pub trait ShapeBuilder<'de, S: StaticSchemaShape>: Sized + DeserializeWithSchema<'de> + Validate + BuildWithCorrection<S> {
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
        Ok(self.build_with_correction())
    }
}

/// Basic builder trait that is unaware of Validation or Deserialization
pub trait BuildWithCorrection<S> {
    /// Build the structure, ignoring any validation and filling any required values.
    ///
    /// **IMPL NOTE**: Implementations should be infallible and should fill any
    /// required values in order to successfully build the output shape.
    fn build_with_correction(self) -> S;
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
pub enum Required<T: ErrorCorrection> {
    Set(T),
    Unset
}
impl <T: SerializeWithSchema + ErrorCorrection> SerializeWithSchema for Required<T> {
    fn serialize_with_schema<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Required::Set(value) => value.serialize_with_schema(schema, serializer),
            Required::Unset => serializer.write_missing(schema)
        }
    }
}
impl <T: ErrorCorrection> Required<T> {
    pub fn get_or_resolve(self) -> T {
        match self {
            Required::Unset => T::default(),
            Required::Set(v) => v
        }
    }
}

pub enum MaybeBuilt<S: Validate + ErrorCorrection, B: Validate + BuildWithCorrection<S>> {
    Struct(S),
    Builder(B),
}
impl <SHAPE: Validate + ErrorCorrection + SerializeWithSchema, B: Validate + BuildWithCorrection<SHAPE> + SerializeWithSchema> SerializeWithSchema for MaybeBuilt<SHAPE, B> {
    fn serialize_with_schema<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            MaybeBuilt::Struct(value) => value.serialize_with_schema(schema, serializer),
            MaybeBuilt::Builder(value) => value.serialize_with_schema(schema, serializer),
        }
    }
}
// Fill a missing required builder
impl<'de, S: Validate + ErrorCorrection, B: Validate + BuildWithCorrection<S>> ErrorCorrection for MaybeBuilt<S, B> {
    fn default() -> Self {
        MaybeBuilt::Struct(S::default())
    }
}

// Get the contained struct or convert the contained builder
impl <S: Validate + ErrorCorrection, B: Validate + BuildWithCorrection<S>> BuildWithCorrection<S> for MaybeBuilt<S, B> {
    fn build_with_correction(self) -> S {
        match self {
            MaybeBuilt::Struct(s) => s,
            MaybeBuilt::Builder(b) => b.build_with_correction(),
        }
    }
}

// Convert and optional of a builder to an optional of the built shape
impl <S, B: BuildWithCorrection<S>> BuildWithCorrection<Option<S>> for Option<B> {
    fn build_with_correction(self) -> Option<S> {
        match self {
            None => None,
            Some(b) => Some(b.build_with_correction())
        }
    }
}

// Convert a vector of builders into a vector of built structures
impl <S, B: BuildWithCorrection<S>> BuildWithCorrection<Vec<S>> for Vec<B> {
    fn build_with_correction(self) -> Vec<S> {
        let mut results = Vec::with_capacity(self.len());
        for builder in self.into_iter() {
            results.push(builder.build_with_correction())
        }
        results
    }
}

// Convert a vector of builders into a vector of built structures
impl <S, B: BuildWithCorrection<S>> BuildWithCorrection<IndexMap<String, S>> for IndexMap<String, B> {
    fn build_with_correction(self) -> IndexMap<String, S> {
        let mut results = IndexMap::with_capacity(self.len());
        for (key, builder) in self.into_iter() {
            let _ = results.insert(key, builder.build_with_correction());
        }
        results
    }
}

