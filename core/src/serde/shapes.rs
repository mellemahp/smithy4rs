use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use serde::Deserialize;
use crate::{serde::deserializers::DeserializeWithSchema};
use crate::schema::{SchemaRef, StaticSchemaShape};
use crate::serde::validate::{ListValidator, Validate};
use crate::serde::validation::{DefaultValidator, ValidationErrors, Validator};

/// Builder for a Smithy Shape
///
/// Used during deserialization to accumulate field values
/// before constructing the final shape. Builders implement
/// [`DeserializeWithSchema`] to handle the actual deserialization logic.
pub trait ShapeBuilder<'de, S: StaticSchemaShape>: Sized + DeserializeWithSchema<'de> + Validate<Value=S> {
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
    /// To build a shape using the default validator use [`ShapeBuilder::build`].
    ///
    /// NOTE: Actual validation and build logic is implementated in builder [`Validate`]
    /// implementation.
    fn build_with_validator<V>(self, mut validator: V) -> Result<S, ValidationErrors>
    where for<'a> &'a mut V: Validator {
        let result = self.validate(S::schema(), &mut validator)?;
        validator.results()?;
        Ok(result)
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

/// Simple wrapper type to allow a builder to store _either_ a pre-built
/// struct or a struct builder.
pub enum StructOrBuilder<S, B: Validate<Value=S>> {
    Struct(S),
    Builder(B),
}
impl <S, B: Validate<Value=S>> Validate for StructOrBuilder<S, B> {
    type Value = S;

    fn validate<V: Validator>(self, schema: &SchemaRef, validator: V) -> Result<Self::Value, ValidationErrors> {
        match self {
            // Do not re-validate already built shapes.
            StructOrBuilder::Struct(s) => Ok(s),
            StructOrBuilder::Builder(b) => b.validate(schema, validator)
        }
    }
}

/// A vector of either structs or builders.
pub enum VecOfStructsOrBuilders<S: Hash, B: Validate<Value=S>> {
    Structs(Vec<S>),
    Builders(Vec<B>),
}
impl <S: Hash, B: Validate<Value=S>> Validate for VecOfStructsOrBuilders<S, B> {
    type Value = Vec<S>;

    fn validate<V: Validator>(self, schema: &SchemaRef, validator: V) -> Result<Self::Value, ValidationErrors> {
        let element_schema = schema.expect_member("member");
        match self {
            // We do not re-validate already built shapes. However, we still need to
            // execute a number of top-level checks and check uniqueness.
            VecOfStructsOrBuilders::Structs(s) => {
                let mut list_validator = validator.validate_list(schema, s.len())?;
                // TODO: Do we need to ever check any other member-level constraints against a built struct?
                for item in &s {
                    list_validator.check_uniqueness(element_schema, item)?
                }
                Ok(s)
            }
            VecOfStructsOrBuilders::Builders(b) => {
                let mut list_validator = validator.validate_list(schema, b.len())?;
                let mut results = Vec::with_capacity(b.len());
                for item in b {
                    results.push(list_validator.validate_and_move(element_schema, item)?);
                }
                Ok(results)
            }
        }
    }
}

// impl <'de, S: StaticSchemaShape, B: ShapeBuilder<'de, S>> Validate for StructOrBuilder<'de, S, B> {
//     type Value = S;
//
//     fn validate<V: Validator>(self, schema: &SchemaRef, validator: V) -> Result<Self::Value, ValidationErrors> {
//         match self {
//             // Do not re-validate already built struct
//             StructOrBuilder::Struct(s) => s,
//             StructOrBuilder::Builder(b, _) => b.validate(schema, validator)
//         }
//     }
// }
//
// impl <'de, S: StaticSchemaShape + BuildMarker<false>, B: ShapeBuilder<'de, S>> From<S> for StructOrBuilder<'de, S, B> {
//     fn from(shape: S) -> Self {
//         StructOrBuilder::Struct(shape)
//     }
// }
// impl <'de, S: StaticSchemaShape, B: ShapeBuilder<'de, S>> From<B> for StructOrBuilder<'de, S, B> {
//     fn from(builder: B) -> Self {
//         StructOrBuilder::Builder(builder, PhantomData)
//     }
// }