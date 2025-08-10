// #![allow(unused_variables, unused_imports)]
//
// //! # Validation
// //!
// //! Validation allows us to compare a shape against a set of constraints.
// //!
// //! ## Some basic requirements
// //!
// //! Validation in `smithy4rs` has a few fundamental requirements:
// //! - Validation needs to occur AFTER deserialization -- Validation should occur only
// //!   once all data has been properly unmarshalled into the ShapeBuilder. Multiple different
// //!   sources could be deserialized into a single shape definition, so ALL deserialization
// //!   must be completed before validation can occur. This also avoids validating multiple times
// //!   if multiple sources are used. It does unfortunately mean that a second pass over the
// //!   data is required, but I believe that is a worthwhile tradeoff.
// //! - Validation and deserialization errors should be distinct (i.e. no shared trait) -- This
// //!   allows users to clearly distinguish where issues occured in a processing pipeline
// //! - Validation should aggregate all errors from all nested types -- Users should get a single
// //!   error for _ALL_ of their validation errors so they can fix them all at once.
// //!
// //! ## Build and Validation
// //!
// //! For `smithy4rs`, [`Builder`] implementations are built when validated. We do not
// //! want to allow a user to construct an invalid shape.
// //! If you have a different definition of "invalid" that is fine, but you have to encode that in a
// //! [`Validator`] implementation.  For example, if you don't care if a response from a
// //! server missed a `@required` value, then you could create a `clientValidator` that encodes
// //! that mode/behavior.
// //!
// //! #### Basic user experience
// //!
// //! Basically, I expect shape construction to look like:
// //! ```rust, ignore
// //! let shape = Shape::builder()
// //!     .deserialize(deserializer)? // this will raise any deserialization errors
// //!     .build(validator // optional)? // this with raise any build/validation errors
// //! ```
// //!
// //! This allows us to neatly separate validation and serialization exceptions so we can
// //! clearly understand _where_ in the request/response pipeline an error occured.
// //!
// //! Ok... more examples. As with Smithy-java we want to be able to
// //! ```rust,ignore
// //! use std::fmt::Error;
// //! use smithy4rs_core::errors;
// //! use smithy4rs_core::schema::SchemaRef;
// //! use smithy4rs_core::serde::validation::*;
// //!
// //!  /// Example type
// //! struct Test {
// //!     a: Option<String>,
// //!     nest: Nested
// //! }
// //!
// //! struct TestBuilder {
// //!     a: Option<String>,
// //!     b: Option<Nested>
// //! }
// //!
// //! struct Nested {
// //!     c: String
// //! }
// //!
// //! struct NestedBuilder {
// //!     c: Option<String>
// //! }
// //! static MEMBER_A: &SchemaRef = todo!();
// //! static MEMBER_B: &SchemaRef = todo!();
// //! /// NOTE: Consumes self
// //! impl TestBuilder {
// //! fn build_impl(self, validator: &mut impl Validator) -> Result<Self, ValidationErrors> {
// //!     let errors = ValidationErrors::new();
// //!     let value = Self {
// //!         a: validator.validate_optional_and_extend(&MEMBER_A, self.a, errors).map_err(errors::extend),
// //!         b: validator.validate_required_and_extend(&MEMBER_B, self.b).map_err(errors::extend)
// //!     };
// //!     if errors.is_empty() {
// //!         Ok(value)
// //!     } else {
// //!         Err(errors)
// //!     }
// //!  }
// //! }
// //! ```
//
// use std::cmp::min;
// use crate::schema::{Document, SchemaRef, ShapeId, ShapeType};
// use bigdecimal::BigDecimal;
// use bytebuffer::ByteBuffer;
// use indexmap::IndexMap;
// use num_bigint::BigInt;
// use std::error::Error;
// use std::fmt::Display;
// use std::marker::PhantomData;
// use std::time::Instant;
// use thiserror::Error;
// use crate::prelude::{LengthTrait, LengthTraitBuilder};
// use crate::serde::se::ListSerializer;
// use crate::serde::shapes::{Builder, ShapeBuilder};
//
// macro_rules! check_type {
//     ($schema:ident, $expected:path) => {
//         if $schema.shape_type() != &$expected {
//             return Err(ValidationErrors::from(ValidationError::invalid_type(
//                 $schema, $expected,
//             )));
//         }
//     };
// }
//
// /// Validates that a shape conforms to the constraints of a Schema
// ///
// /// *Note*: The `IN_PLACE` type parameter is used to indicate whether validation
// /// happens in place (no modification) or creates a new value (as a builder does).
// /// Users should, in general, use the default value.
// pub trait Validate<const IN_PLACE: bool = true> {
//     type Value;
//
//     /// Validate a shape given its schema and a validator.
//     ///
//     /// NOTE: For builders this will result in them being built
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V,
//     ) -> Result<Self::Value, ValidationErrors>;
// }
//
// // TODO: could this be a serializer?
// // TODO: How to handle max allowed errors and max depth? Dont want to allow huge reccursion
// /// NOTE: validate_struct is not needed here as that is handled by the builders.
// pub trait Validator: Sized {
//     type ItemValidator: ItemValidator;
//
//     /// Validates a required field, returning a non-optional value.
//     ///
//     /// Default implementation errors on missing values.
//     fn validate_required<V: Validate>(
//         self,
//         schema: &SchemaRef,
//         value: Option<V>,
//     ) -> Result<V::Value, ValidationErrors> {
//         let Some(value) = value else {
//             return Err(ValidationError::required(schema).into());
//         };
//         value.validate(schema, self)
//     }
//
//     fn validate_optional<V: Validate>(
//         self,
//         schema: &SchemaRef,
//         value: Option<V>,
//     ) -> Result<Option<V::Value>, ValidationErrors> {
//         if let Some(value) = value {
//             value.validate(schema, self).map(|v| Some(v))
//         } else {
//             Ok(None)
//         }
//     }
//
//     fn validate_blob(
//         self,
//         schema: &SchemaRef,
//         blob: ByteBuffer,
//     ) -> Result<ByteBuffer, ValidationErrors> {
//         check_type!(schema, ShapeType::Blob);
//         Ok(blob)
//     }
//
//     fn validate_boolean(
//         self,
//         schema: &SchemaRef,
//         boolean: bool
//     ) -> Result<bool, ValidationErrors> {
//         check_type!(schema, ShapeType::Boolean);
//         Ok(boolean)
//     }
//
//     fn validate_string(
//         self,
//         schema: &SchemaRef,
//         string: String,
//     ) -> Result<String, ValidationErrors> {
//         check_type!(schema, ShapeType::String);
//         // TODO: By default should check length and pattern
//         Ok(string)
//     }
//
//     fn validate_timestamp(
//         self,
//         schema: &SchemaRef,
//         timestamp: Instant,
//     ) -> Result<Instant, ValidationErrors> {
//         check_type!(schema, ShapeType::Timestamp);
//         Ok(timestamp)
//     }
//
//     /// TODO: Could these delegate to the largest int validation?
//     /// TODO: All the number impls should check range.
//     fn validate_byte(
//         self,
//         schema: &SchemaRef,
//         byte: i8
//     ) -> Result<i8, ValidationErrors> {
//         check_type!(schema, ShapeType::Byte);
//         Ok(byte)
//     }
//
//     fn validate_short(
//         self,
//         schema: &SchemaRef,
//         short: i16
//     ) -> Result<i16, ValidationErrors> {
//         check_type!(schema, ShapeType::Short);
//         Ok(short)
//     }
//
//     fn validate_integer(
//         self,
//         schema: &SchemaRef,
//         integer: i32
//     ) -> Result<i32, ValidationErrors> {
//         check_type!(schema, ShapeType::Integer);
//         Ok(integer)
//     }
//
//     fn validate_long(
//         self,
//         schema: &SchemaRef,
//         long: i64
//     ) -> Result<i64, ValidationErrors> {
//         check_type!(schema, ShapeType::Long);
//         Ok(long)
//     }
//
//     fn validate_float(
//         self,
//         schema: &SchemaRef,
//         float: f32
//     ) -> Result<f32, ValidationErrors> {
//         check_type!(schema, ShapeType::Float);
//         Ok(float)
//     }
//
//     fn validate_double(
//         self,
//         schema: &SchemaRef,
//         double: f64
//     ) -> Result<f64, ValidationErrors> {
//         check_type!(schema, ShapeType::Double);
//         Ok(double)
//     }
//
//     fn validate_big_integer(
//         self,
//         schema: &SchemaRef,
//         big_int: BigInt,
//     ) -> Result<BigInt, ValidationErrors> {
//         check_type!(schema, ShapeType::BigInteger);
//         Ok(big_int)
//     }
//
//     fn validate_big_decimal(
//         self,
//         schema: &SchemaRef,
//         big_decimal: BigDecimal,
//     ) -> Result<BigDecimal, ValidationErrors> {
//         check_type!(schema, ShapeType::BigDecimal);
//         Ok(big_decimal)
//     }
//
//     fn validate_document(
//         self,
//         schema: &SchemaRef,
//         document: Document,
//     ) -> Result<Document, ValidationErrors> {
//         // TODO: How should this be handled?
//         check_type!(schema, ShapeType::Document);
//         Ok(document)
//     }
//
//     // TODO: Should the enums check if the value is out of expected for validation? Could just check if
//     //       the value is ::__Unknown?
//     /// TODO: Should these check string validation?
//     fn validate_enum<E>(
//         self,
//         schema: &SchemaRef,
//         value: E
//     ) -> Result<E, ValidationErrors> {
//         check_type!(schema, ShapeType::Enum);
//         Ok(value)
//     }
//
//     fn validate_int_enum<E>(
//         self,
//         schema: &SchemaRef,
//         value: E
//     ) -> Result<E, ValidationErrors> {
//         check_type!(schema, ShapeType::IntEnum);
//         Ok(value)
//     }
//
//     fn validate_list(
//         self,
//         schema: &SchemaRef,
//         size: usize,
//     ) -> Result<Self::ItemValidator, ValidationErrors>;
//
//     fn validate_map(
//         self,
//         schema: &SchemaRef,
//         size: usize,
//     ) -> Result<Self::ItemValidator, ValidationErrors>;
//
//     fn validate_struct<V: Validate>(
//         &mut self,
//         _schema: &SchemaRef,
//         _builder: V,
//     ) -> Result<V::Value, ValidationErrors> {
//         // NOTE: This validates a _built_ structure, not a stucture builder!
//         // TODO: Check type?
//         // TODO: are there any structure-level constraints that might need validation?
//         todo!()
//     }
// }
//
// pub trait ItemValidator {
//     fn validate_item<V: Validate>(
//         &mut self,
//         item_schema: &SchemaRef,
//         value: V,
//     ) -> Result<V::Value, ValidationErrors>;
// }
//
// /////////////////////////////////////////////////////////////////////////////////
// // `Validate` Implementations
// /////////////////////////////////////////////////////////////////////////////////
//
// // Blanket implementation for Shape builders.
// // Validating a shape builder will build and validate it's contained values.
// // TODO: Get blanket impl working. May require marker traits.
// // impl <B: ShapeBuilder<S>, S> Validate for B {
// //     // Return the built shape
// //     type Value = S;
// //
// //     fn validate<V: Validator>(self, _: &SchemaRef, validator: &V) -> Result<Self::Value, ValidationErrors> {
// //         self.build_with_validator(validator)
// //     }
// // }
//
// impl Validate for ByteBuffer {
//     type Value = Self;
//
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V,
//     ) -> Result<Self, ValidationErrors> {
//         validator.validate_blob(schema, self)
//     }
// }
//
// impl Validate for bool {
//     type Value = Self;
//
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V,
//     ) -> Result<Self, ValidationErrors> {
//         validator.validate_boolean(schema, self)
//     }
// }
//
// impl Validate for String {
//     type Value = Self;
//
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V
//     ) -> Result<Self::Value, ValidationErrors> {
//         validator.validate_string(schema, self)
//     }
// }
//
// impl Validate for Instant {
//     type Value = Self;
//
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V
//     ) -> Result<Self::Value, ValidationErrors> {
//         validator.validate_timestamp(schema, self)
//     }
// }
//
// impl Validate for i8 {
//     type Value = Self;
//
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V,
//     ) -> Result<Self, ValidationErrors> {
//         validator.validate_byte(schema, self)
//     }
// }
//
// impl Validate for i16 {
//     type Value = Self;
//
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V,
//     ) -> Result<Self, ValidationErrors> {
//         validator.validate_short(schema, self)
//     }
// }
//
// impl Validate for i32 {
//     type Value = Self;
//
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V,
//     ) -> Result<Self, ValidationErrors> {
//         validator.validate_integer(schema, self)
//     }
// }
//
// impl Validate for i64 {
//     type Value = Self;
//
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V,
//     ) -> Result<Self, ValidationErrors> {
//         validator.validate_long(schema, self)
//     }
// }
//
//
// impl Validate for f32 {
//     type Value = Self;
//
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V,
//     ) -> Result<Self, ValidationErrors> {
//         validator.validate_float(schema, self)
//     }
// }
//
// impl Validate for f64 {
//     type Value = Self;
//
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V,
//     ) -> Result<Self, ValidationErrors> {
//         validator.validate_double(schema, self)
//     }
// }
//
// impl Validate for BigDecimal {
//     type Value = Self;
//
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V,
//     ) -> Result<Self, ValidationErrors> {
//         validator.validate_big_decimal(schema, self)
//     }
// }
//
// impl Validate for BigInt {
//     type Value = Self;
//
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V,
//     ) -> Result<Self, ValidationErrors> {
//         validator.validate_big_integer(schema, self)
//     }
// }
//
// // In place validation of a list value (i.e. does not create new list)
// impl<I> Validate<true> for Vec<I>
// where
//     I: Validate<true>,
//     I::Value: Validate<true>
// {
//     type Value = Vec<I::Value>;
//
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V,
//     ) -> Result<Self::Value, ValidationErrors> {
//         let mut item_validator = validator.validate_list(schema, self.len())?;
//         let Some(item_schema) = schema.get_member("member") else {
//             return Err(ValidationError::expected_member(schema, "member").into())
//         };
//         let mut errors = ValidationErrors::new();
//         for item in &self {
//             match item_validator.validate_item(item_schema, item) {
//                 Ok(_) => () /* do nothing */,
//                 Err(e) => errors.extend(e)
//             };
//         }
//         // TODO: Limit validation depth
//         if errors.is_empty() {
//             Ok(self)
//         } else {
//             Err(errors)
//         }
//     }
// }
//
// // Validation of a value that is moved (i.e. requires a new list to be created)
// // Note that moving value should produce an in-place validatable value.
// impl<I> Validate<false> for Vec<I>
// where
//     I: Validate<true>,
//     I::Value: Validate<false>
// {
//     type Value = Vec<I::Value>;
//
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V,
//     ) -> Result<Self::Value, ValidationErrors> {
//         let mut item_validator = validator.validate_list(schema, self.len())?;
//         let Some(item_schema) = schema.get_member("member") else {
//             return Err(ValidationError::expected_member(schema, "member").into())
//         };
//         let mut errors = ValidationErrors::new();
//         // TODO: limit depth of validation
//         let res = self.into_iter()
//             .map(|item| item_validator.validate_item(item_schema, item))
//             .filter_map(|res| res.map_err(|e| errors.extend(e)).ok())
//             .collect();
//         if errors.is_empty() {
//             Ok(res)
//         } else {
//             Err(errors)
//         }
//     }
// }
//
// impl<V: Validate> Validate for IndexMap<String, V>
// {
//     type Value = IndexMap<String, V::Value>;
//
//     fn validate<T: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: T,
//     ) -> Result<Self::Value, ValidationErrors> {
//         let mut item_validator = validator.validate_map(schema, self.len())?;
//         let Some(value_schema) = schema.get_member("value") else {
//             return Err(ValidationError::expected_member(schema, "value").into())
//         };
//         let Some(key_schema) = schema.get_member("key") else {
//             return Err(ValidationError::expected_member(schema, "key").into())
//         };
//         check_type!(key_schema, ShapeType::String);
//
//         let mut errors = ValidationErrors::new();
//         // TODO: limit depth of validation
//         let res = self.into_iter()
//             .map(|(key, value)| match item_validator.validate_item(value_schema, value) {
//                 Ok(val) => Ok((key, val)),
//                 Err(e) => Err(errors.extend(e)),
//             })
//             .filter_map(|res| res.ok())
//             .collect();
//         if errors.is_empty() {
//             Ok(res)
//         } else {
//             Err(errors)
//         }
//     }
// }
//
// // TODO: Is it possible to make a blanket impl for builders?
//
// // TODO: VALIDATE FOR DOCUMENT
//
// //////////////////////////////////////////////////////////////////////////////
// // Pre-built Validators
// //////////////////////////////////////////////////////////////////////////////
//
// pub struct DefaultValidator;
// impl DefaultValidator {
//     pub(crate) const fn new() -> Self {
//         DefaultValidator {}
//     }
// }
//
// impl <'a> Validator for &'a mut DefaultValidator {
//     type ItemValidator = DefaultItemValidator<'a>;
//
//     fn validate_list(
//         self,
//         schema: &SchemaRef,
//         size: usize
//     ) -> Result<Self::ItemValidator, ValidationErrors> {
//         check_type!(schema, ShapeType::List);
//         check_length(schema, size)?;
//         Ok(DefaultItemValidator { validator: self })
//     }
//
//     fn validate_map(
//         self,
//         schema: &SchemaRef,
//         size: usize
//     ) -> Result<Self::ItemValidator, ValidationErrors> {
//         check_type!(schema, ShapeType::Map);
//         check_length(schema, size)?;
//         Ok(DefaultItemValidator { validator: self })
//     }
// }
//
// fn check_length(schema: &SchemaRef, size: usize) -> Result<(), ValidationErrors> {
//     if let Some(length) = schema.get_trait_as::<LengthTrait>() {
//         let errors = ValidationErrors::new();
//         let min = length.min.unwrap_or(0);
//         let max = length.max.unwrap_or(usize::MAX);
//         if size > max || size < min {
//             return Err(ValidationError::length(schema, size, min, max).into());
//         }
//     }
//     Ok(())
// }
//
// // TODO: This could handle the validation of uniqueness and sparseness!
// pub struct DefaultItemValidator<'a> {
//     validator: &'a mut DefaultValidator
// }
// impl ItemValidator for DefaultItemValidator<'_> {
//     fn validate_item<V: Validate>(
//         &mut self,
//         item_schema: &SchemaRef,
//         value: V,
//     ) -> Result<V::Value, ValidationErrors> {
//         value.validate(item_schema, &mut *self.validator)
//     }
// }
//
// // TODO: Add an empty, pass-through validator.
//
// // TODO: Also add a tracking validator that is infallible, but collects all errors.
//
// // TODO: How to validate built shape?
//
// //////////////////////////////////////////////////////////////////////////////
// // ERRORS
// //////////////////////////////////////////////////////////////////////////////
//
// // Aggregated list of all validation errors encountered while building a shape.
// #[derive(Error, Debug)]
// pub struct ValidationErrors {
//     errors: Vec<ValidationError>,
// }
//
// impl Display for ValidationErrors {
//     fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         todo!();
//     }
// }
//
// impl ValidationErrors {
//     pub const fn new() -> Self {
//         Self { errors: Vec::new() }
//     }
//
//     pub fn extend(&mut self, other: ValidationErrors) {
//         self.errors.extend(other.errors);
//     }
//
//     pub fn add(&mut self, error: ValidationError) {
//         self.errors.push(error);
//     }
//
//     pub fn is_empty(&self) -> bool {
//         self.errors.is_empty()
//     }
// }
//
// #[derive(Debug)]
// pub struct ValidationError {
//     path: ShapeId,
//     code: Box<dyn ValidationErrorCode>,
// }
// impl Display for ValidationError {
//     fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }
//
// impl ValidationError {
//     fn custom(schema: &SchemaRef, code: Box<dyn ValidationErrorCode>) -> Self {
//         Self {
//             path: schema.id().clone(),
//             code,
//         }
//     }
//
//     fn invalid_type(schema: &SchemaRef, expected: ShapeType) -> Self {
//         Self {
//             path: schema.id().clone(),
//             code: Box::new(SmithyConstraints::InvalidType(
//                 *schema.shape_type(),
//                 expected,
//             )),
//         }
//     }
//
//     fn expected_member(schema: &SchemaRef, expected: &str) -> Self {
//         Self {
//             path: schema.id().clone(),
//             code: Box::new(SmithyConstraints::ExpectedMember(expected.to_owned())),
//         }
//     }
//
//     fn required(schema: &SchemaRef) -> Self {
//         Self {
//             path: schema.id().clone(),
//             code: Box::new(SmithyConstraints::Required),
//         }
//     }
//
//     fn unsupported(schema: &SchemaRef) -> Self {
//         Self {
//             path: schema.id().clone(),
//             code: Box::new(SmithyConstraints::Required),
//         }
//     }
//
//     fn length(schema: &SchemaRef, size: usize, min: usize, max: usize) -> Self {
//         Self {
//             path: schema.id().clone(),
//             code: Box::new(SmithyConstraints::Length(size, min, max)),
//         }
//     }
// }
//
// impl From<ValidationError> for ValidationErrors {
//     fn from(field: ValidationError) -> Self {
//         let mut vec: Vec<ValidationError> = Vec::with_capacity(1);
//         vec.push(field);
//         ValidationErrors { errors: vec }
//     }
// }
//
// /// Marker trait for validation errors.
// pub trait ValidationErrorCode: Error {}
//
// #[derive(Error, Debug)]
// pub enum SmithyConstraints {
//     /// This error should only occur for manually constructed schemas.
//     /// It should be unreachable in generated shapes.
//     #[error("Expected member: {0}")]
//     ExpectedMember(String),
//     /// This error should only ever occur for manual schema interactions,
//     /// not for automatically generated Shapes.
//     #[error("Invalid Shape type. Expected {0:?}, recieved {1:?}.")]
//     InvalidType(ShapeType, ShapeType),
//     #[error("Field is Required.")]
//     Required,
//     // TODO: Better error messages when min/max None
//     #[error("Size: {0} does not conform to @length constraint. Expected between {1} and {2}.")]
//     Length(usize, usize, usize),
//     #[error("Type did not conform to expected pattern {0}")]
//     Pattern(String),
//     // TODO: Better error messages when min/max None
//     #[error("Size: {0} does not conform to @range constraint. Expected between {1} and {2}.")]
//     Range(BigDecimal, BigDecimal, BigDecimal),
//     // Could this be security risk if non-unique are returned?
//     #[error("Items in collection should be unique.")]
//     UniqueItems,
//     // TODO: SHould this be in a different enum?
//     #[error("Unsupported validation operation.")]
//     Unsupported,
// }
// impl ValidationErrorCode for SmithyConstraints {}
//
// #[cfg(test)]
// mod tests {
//     use std::ops::Deref;
//     use super::*;
//     use crate::Ref;
//     use crate::prelude::STRING;
//     use crate::schema::{SchemaShape, LIST_DOCUMENT_SCHEMA};
//     use crate::schema::{Schema, ShapeId};
//     use crate::{lazy_schema, traits};
//     use indexmap::IndexMap;
//     use std::sync::LazyLock;
//     use std::time::SystemTime;
//
//     lazy_schema!(
//         MAP_SCHEMA,
//         Schema::map_builder(ShapeId::from("com.example#Map"), traits![]),
//         ("key", STRING, traits![]),
//         ("value", STRING, traits![])
//     );
//     lazy_schema!(
//         LIST_SCHEMA,
//         Schema::list_builder(ShapeId::from("com.example#List"), traits![
//             LengthTrait::builder().min(1).max(2).build()
//         ]),
//         ("member", STRING, traits![])
//     );
//
//     #[test]
//     fn checks_list_too_long() {
//         let too_long = vec!["a".to_string(), "b".to_string(), "c".to_string()];
//         let mut validator = DefaultValidator::new();
//         let res = too_long.validate(&LIST_DOCUMENT_SCHEMA, &mut validator);
//         let Err(ValidationErrors { errors }) = res else {
//             panic!("Expected an error")
//         };
//         assert_eq!(errors.len(), 1);
//         let expected = "Size: 3 does not conform to @length constraint. Expected between 1 and 2.".to_string();
//         assert_eq!(expected, format!("{}", errors.get(0).unwrap().code));
//     }
//
//     struct A {
//         field: String,
//     }
//     struct ABuilder {
//         field: Option<String>
//     }
//     impl ABuilder {
//         fn build(self) -> A {
//             let errors = ValidationErrors::new();
//             let field = DefaultValidator.validate_required(&STRING, self.field).unwrap();
//
//             if errors.is_empty() {
//                 A { field }
//             }
//
//         }
//     }
//
//     #[doc(hidden)]
//     impl Validate for A {
//         type Value = ();
//
//         #[doc(hidden)]
//         fn validate<V: Validator>(self, schema: &SchemaRef, validator: V) -> Result<Self::Value, ValidationErrors> {
//             todo!()
//         }
//     }
//
//     fn x() {
//         let a = A;
//         a.validate()
//
//
//     }
// }
