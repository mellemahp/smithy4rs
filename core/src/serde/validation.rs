use std::collections::{BTreeMap, BTreeSet};
use std::convert::Into;
use std::error::Error;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use bigdecimal::{BigDecimal, Zero};
use bigdecimal::num_traits::Float;
use bytebuffer::ByteBuffer;
use indexmap::IndexMap;
use num_bigint::BigInt;
use rustc_hash::FxHasher;
use smallvec::SmallVec;
use thiserror::Error;
use crate::schema::{Document, DocumentValue, Schema, SchemaRef, ShapeType};
use crate::Instant;
use crate::prelude::{LengthTrait, PatternTrait, UniqueItemsTrait, DOCUMENT};
use crate::serde::se::{SerializableShape, SerializeWithSchema, Serializer};
use crate::serde::serializers;
use crate::serde::serializers::{ListSerializer, MapSerializer, StructSerializer};


//////////////////////////////////////////////////////////////////////////////
// Validator
//////////////////////////////////////////////////////////////////////////////

/// Validator that ensures shapes conform to constraints.
///
/// By default, this validator will check that shapes conform to all built-in Smithy
/// constraint traits found in a schema, including:
/// - `@length`
/// - `@range`
/// - `@uniqueItems`
/// - `@required`
/// - ???
///
/// For more info on built-in Smithy constraints see: [Smithy Documentation](https://smithy.io/2.0/spec/constraint-traits.html)
// TODO: Support enum validation?
// TODO: Add example
pub struct Validator {
    errors: Option<ValidationErrors>,
    // TODO(optimization): Should this use smallvec?
    path_stack: Vec<SchemaRef>,
    max_depth: usize,
    max_errors: usize,
}
impl Validator {
    /// Create a new [`Validator`] instance.
    pub const fn new() -> Self {
        Validator {
            errors: None,
            path_stack: Vec::new(),
            max_depth: 10,
            max_errors: 10,
        }
    }

    /// Validates a type against a schema.
    ///
    /// If any validation errors are found, this method will return an `Err` result containing
    /// an aggregate of all the validation errors encountered.
    pub fn validate<V: Validate>(
        &mut self,
        schema: &SchemaRef,
        value: &V
    ) -> Result<(), ValidationErrors> {
        value.validate(schema, self)?;
        self.results()
    }

    /// Emit an error for accumulation.
    ///
    /// This method should _only_ returns an error response when the maximum number
    /// of errors is hit. At that point it returns a list of all previously encountered
    /// validation errors plus an extra appended error to indicate the error limit was reached.
    pub fn emit_error<E: ValidationError + 'static>(&mut self, path: &SchemaRef, err: E) -> Result<(), ValidationErrors> {
        let errors = self.errors.get_or_insert(ValidationErrors::new());
        if errors.len() >= self.max_errors {
            errors.add(&self.path_stack, ValidationFailure::MaxErrorsReached(self.max_errors));
            // SAFETY: Safe to unwrap as errors will alway be set to `SOME` above
            // TODO(code quality): maybe use a lazy initializer struct.
            return Err(self.errors.take().unwrap());
        }
        errors.add_with_path(&self.path_stack, path, err);
        Ok(())
    }

    /// Return all collected validation errors
    ///
    /// This returns a `Result` type to allow `?` raising.
    fn results(&mut self) -> Result<(), ValidationErrors> {
        if let Some(errors) = self.errors.take() {
            return Err(errors);
        }
        Ok(())
    }

    fn push_path(&mut self, path: &SchemaRef) -> Result<(), ValidationErrors> {
        if self.path_stack.len() + 1 > self.max_depth {
            self.emit_error(path, ValidationFailure::MaximumDepthExceeded(self.max_depth))?;
            // SAFETY: Safe to unwrap as errors will alway be set to `SOME` above
            // TODO(code quality): maybe use a lazy initializer struct.
            return Err(self.errors.take().unwrap());
        }
        self.path_stack.push(path.clone());
        Ok(())
    }

    fn pop_path(&mut self, schema: &SchemaRef) -> Result<(), ValidationErrors> {
        if let None = self.path_stack.pop() {
            // If we have reached the base path something went wrong and there
            // was an unbalanced validator call somewhere.
            self.emit_error(schema, ValidationFailure::PopFromEmptyValidator)?;
            // SAFETY: Safe to unwrap as errors will alway be set to `SOME` above
            // TODO(code quality): maybe use a lazy initializer struct.
            return Err(self.errors.take().unwrap());
        }
        Ok(())
    }
}

impl <'a> Serializer for &'a mut Validator {
    type Error = ValidationErrors;

    /// Serializable types can be validated in-place
    type Ok = ();
    type SerializeList = ListValidator<'a>;
    type SerializeMap = MapValidator<'a>;
    type SerializeStruct = StructValidator<'a>;

    fn write_struct(self, schema: &SchemaRef, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        // TODO(completeness): check that schema is struct.
        // TODO(completeness): ADD DEPTH CHECKS
        // Push to stack
        self.push_path(schema)?;
        Ok(StructValidator { root: self })
    }

    fn write_map(self, schema: &SchemaRef, len: usize) -> Result<Self::SerializeMap, Self::Error> {
        // Check that list does not exceed length constraint
        if let Some(length) = schema.get_trait_as::<LengthTrait>() {
            if len < length.min() || len > length.max() {
                self.emit_error(schema, SmithyConstraints::Length(len, length.min(), length.max()))?;
            }
        }
        self.push_path(schema)?;
        Ok(MapValidator { root: self })
    }

    fn write_list(self, schema: &SchemaRef, len: usize) -> Result<Self::SerializeList, Self::Error> {
        // Short circuit if the list is larger than the allowed depth.
        // TODO(extensibility): Make this separately configurable property?
        if len > self.max_depth {
            self.emit_error(schema, ValidationFailure::ListTooLarge(self.max_depth))?;
            return Err(self.errors.take().unwrap());
        }

        // Check that list does not exceed length constraint
        if let Some(length) = schema.get_trait_as::<LengthTrait>() {
            if len < length.min() || len > length.max() {
                self.emit_error(schema, SmithyConstraints::Length(len, length.min(), length.max()))?;
            }
        }
        self.push_path(schema)?;
        Ok(ListValidator {
            root: self,
            unique: schema.contains_type::<UniqueItemsTrait>(),
            lookup: UniquenessTracker::new()
        })
    }

    fn write_boolean(self, schema: &SchemaRef, value: bool) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn write_byte(self, schema: &SchemaRef, value: i8) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn write_short(self, schema: &SchemaRef, value: i16) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn write_integer(self, schema: &SchemaRef, value: i32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn write_long(self, schema: &SchemaRef, value: i64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn write_float(self, schema: &SchemaRef, value: f32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn write_double(self, schema: &SchemaRef, value: f64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn write_big_integer(self, schema: &SchemaRef, value: &BigInt) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn write_big_decimal(self, schema: &SchemaRef, value: &BigDecimal) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn write_string(self, schema: &SchemaRef, value: &str) -> Result<Self::Ok, Self::Error> {
        if *schema.shape_type() != ShapeType::String {
            self.emit_error(schema, ValidationFailure::InvalidType(schema.shape_type().clone(), ShapeType::String))?;
        }

        // TODO(extensibility): Move into a "ValidationRule"?
        // Check pattern
        if let Some(pattern) = schema.get_trait_as::<PatternTrait>() {
            if let None = pattern.pattern().find(value) {
                self.emit_error(schema, SmithyConstraints::Pattern(value.to_string(), pattern.pattern().to_string()))?;
            }
        }

        // Check length
        if let Some(length) = schema.get_trait_as::<LengthTrait>() {
            if value.len() < length.min() || value.len() > length.max() {
                self.emit_error(schema, SmithyConstraints::Length(value.len(), length.min(), length.max()))?
            }
        }
        Ok(())
    }

    fn write_blob(self, schema: &SchemaRef, value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn write_timestamp(self, schema: &SchemaRef, value: &Instant) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn write_document(self, schema: &SchemaRef, value: &Document) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn write_null(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn write_missing(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.emit_error(schema, SmithyConstraints::Required)
    }

    fn skip(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        /* Do nothing on skip */
        Ok(())
    }
}

#[doc(hidden)]
pub struct ListValidator<'a> {
    root: &'a mut Validator,
    unique: bool,
    lookup: UniquenessTracker
}

impl ListSerializer for ListValidator<'_> {
    type Error = ValidationErrors;
    type Ok = ();

    fn serialize_element<T>(&mut self, element_schema: &SchemaRef, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema
    {
        value.serialize_with_schema(element_schema, &mut *self.root)
        //     self.check_uniqueness(element_schema, &value)
    }

    fn end(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.root.pop_path(schema)
    }
}

/// Tracker for unique items using a hash lookup directly
struct UniquenessTracker {
    // A b-tree is used here as it should be faster for
    // search for a relatively small number of numeric
    // values than a hashmap
    lookup: BTreeSet<u64>,
}
impl UniquenessTracker {
    fn new() -> Self {
        UniquenessTracker {
            lookup: BTreeSet::new()
        }
    }

    /// Add an item to the set.
    fn add<T: Hash>(&mut self, value: T) -> bool {
        let mut hasher = FxHasher::default();
        value.hash(&mut hasher);
        !self.lookup.insert(hasher.finish())
    }
}

#[doc(hidden)]
pub struct MapValidator<'a> {
    root: &'a mut Validator,
}
impl MapSerializer for MapValidator<'_> {
    type Error = ValidationErrors;
    type Ok = ();

    fn serialize_entry<K, V>(&mut self, key_schema: &SchemaRef, value_schema: &SchemaRef, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + SerializeWithSchema,
        V: ?Sized + SerializeWithSchema
    {
        let _key = key.serialize_with_schema(key_schema, &mut *self.root)?;
        let _value = value.serialize_with_schema(value_schema, &mut *self.root)?;
        Ok(())
    }

    fn end(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.root.pop_path(schema)
    }
}

#[doc(hidden)]
pub struct StructValidator<'a> {
    root: &'a mut Validator,
}
impl StructSerializer for StructValidator<'_> {
    type Error = ValidationErrors;
    type Ok = ();

    fn serialize_member<T>(&mut self, member_schema: &SchemaRef, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema
    {
        value.serialize_with_schema(member_schema, &mut *self.root)
    }

    fn end(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.root.pop_path(schema)
    }
}

//////////////////////////////////////////////////////////////////////////////
// Validate Implementations
//////////////////////////////////////////////////////////////////////////////

/// Indicates that a type can be validated by a [`Validator`] instance.
///
/// All validate-able types must be able to provide a sane
/// default value through error correction.
pub trait Validate {
    /// Validate a shape given its schema and a validator.
    ///
    /// NOTE: For builders this will result in them being built
    fn validate(
        &self,
        schema: &SchemaRef,
        validator: &mut Validator,
    ) -> Result<(), ValidationErrors>;
}

// Any shape that is serializable can just be validated in place.
impl <S: SerializeWithSchema> Validate for S {
    fn validate(
        &self,
        schema: &SchemaRef,
        validator: &mut Validator
    ) -> Result<(), ValidationErrors> {
        self.serialize_with_schema(schema, validator)
    }
}

//
// /// This thin wrapper simply allows us to implement [`Hash`] for
// /// so lists of timestamp values can support `@uniqueItem` constraint.
// #[repr(transparent)]
// struct TimeStampHashWrapper<'a>(&'a Instant);
// impl <'a> Validate for &'a TimeStampHashWrapper<'_> {
//     type Value = ();
//
//     #[inline]
//     fn validate<V: Validator>(self, schema: &SchemaRef, validator: V) -> Result<Self::Value, ValidationErrors> {
//          self.0.validate(schema, validator)
//     }
// }
// impl <'a> Hash for TimeStampHashWrapper<'a> {
//     #[inline]
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.0.epoch_nanoseconds().0.hash(state);
//     }
// }

// ==== Nested Collections ====
// TODO

//////////////////////////////////////////////////////////////////////////////
// ERROR CORRECTION
//////////////////////////////////////////////////////////////////////////////

/// Implements Smithy [Error Correction](https://smithy.io/2.0/spec/aggregate-types.html#client-error-correction) for a type.
///
/// Error correction fills missing required values to allow invalid shapes to still be correctly
/// constructed. This is primarily useful for validation and to avoid deserialization issues in
/// some clients.
pub trait ErrorCorrection {
    /// Returns a default value for the type in case of errors
    fn default() -> Self;
}

macro_rules! correction_impl {
    ($t:ty, $v:expr) => {
        impl ErrorCorrection for $t {
            #[inline(always)]
            fn default() -> $t {
                $v
            }
        }
    };
}
correction_impl!(bool, true);
correction_impl!(i8, 0i8);
correction_impl!(i16, 0i16);
correction_impl!(i32, 0i32);
correction_impl!(i64, 0i64);
correction_impl!(f32, 0f32);
correction_impl!(f64, 0f64);
// SAFETY: Unwrap will never fail
correction_impl!(Instant, Instant::from_epoch_milliseconds(0).unwrap());
correction_impl!(String, String::new());
correction_impl!(BigDecimal, BigDecimal::zero());
correction_impl!(BigInt, BigInt::zero());

impl ErrorCorrection for Document {
    fn default() -> Self {
        Document {
            schema: DOCUMENT.clone(),
            value: DocumentValue::Null,
            discriminator: None,
        }
    }
}

impl <E: ErrorCorrection> ErrorCorrection for Vec<E> {
    fn default() -> Self {
        Vec::new()
    }
}

impl <E: ErrorCorrection> ErrorCorrection for IndexMap<String, E> {
    fn default() -> Self {
        IndexMap::new()
    }
}

// TODO: ENUM AND INT ENUM IMPLS + Byte buffer impls

//////////////////////////////////////////////////////////////////////////////
// ERRORS
//////////////////////////////////////////////////////////////////////////////

/// Aggregated list of all validation errors encountered while building a shape.
///
/// When executing validation of a Builder, more than one field could be invalid.
/// All of these [`ValidationError`]'s are aggregated together into a list on this
/// aggregate error type.
// TODO: Could this be a trie? Would that actually be faster?
#[derive(Error, Debug)]
pub struct ValidationErrors {
    errors: Vec<ValidationErrorWrapper>
}

impl serializers::Error for ValidationErrors {
    fn custom<T: Display>(msg: T) -> Self {
        todo!()
    }
}

impl Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:#?}", self.errors)
    }
}

impl ValidationErrors {
    /// Create a new [`ValidationErrors`] error.
    ///
    /// **NOTE**: This method instantiates the error type with an
    /// empty list of errors. Actual validation errors must be added
    /// using the [`ValidationErrors::extend`] or [`ValidationErrors::add`]
    /// methods.
    pub const fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Extends an aggregate validation error with the contents of
    /// another aggregate validation error.
    pub fn extend(&mut self, other: ValidationErrors) {
        self.errors.extend(other.errors);
    }

    /// Add a new validation error to the list of errors.
    pub(super) fn add(&mut self, path: &Vec<SchemaRef>, error: impl Into<Box<dyn ValidationError>>) {
        self.errors.push(ValidationErrorWrapper::new(path.clone(), error.into()));
    }

    pub(super) fn add_with_path(
        &mut self,
        path_stack: &Vec<SchemaRef>,
        path: &SchemaRef,
        error: impl Into<Box<dyn ValidationError>>
    ) {
        let mut new: Vec<SchemaRef> = Vec::with_capacity(path_stack.len() + 1);
        new.extend(path_stack.iter().cloned());
        new.push(path.clone());
        self.errors.push(ValidationErrorWrapper::new(new, error.into()));
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

/// Wrapper that groups a validation error with the schema location at which it occured.
#[derive(Error, Debug)]
pub struct ValidationErrorWrapper {
    paths: Vec<SchemaRef>,
    error: Box<dyn ValidationError>
}
impl ValidationErrorWrapper {
    pub fn new(paths: Vec<SchemaRef>, error: Box<dyn ValidationError>) -> Self {
        Self { paths, error }
    }
}
impl Display for ValidationErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}:{:?}", self.paths, self.error)
    }
}

/// Marker trait for validation errors.
pub trait ValidationError: Error {}

// Implement conversion for any Error enums implementing Validation error
impl <T: ValidationError + 'static> From<T> for Box<dyn ValidationError> {
    #[inline]
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

/// Captures validation failures that could happen for any validator.
///
/// These errors should only occur for manually constructed schemas.
/// If you encounter one of these in a generated shape using the default
/// validator then this is a bug.
#[derive(Error, Debug)]
pub enum ValidationFailure {
    #[error("Expected schema to contain member: `{0}`")]
    ExpectedMember(String),
    /// This error should only ever occur for manual schema interactions,
    /// not for automatically generated Shapes.
    #[error("Invalid Shape type. Expected {0:?}, recieved {1:?}.")]
    InvalidType(ShapeType, ShapeType),
    #[error("Maximum Validation depth: {0} exceeded")]
    MaximumDepthExceeded(usize),
    #[error("Maximum Number of errors ({0}) reached")]
    MaxErrorsReached(usize),
    #[error("List exceeds maximum validation size ({0})")]
    ListTooLarge(usize),
    #[error("Map exceeds maximum validation size ({0})")]
    MapTooLarge(usize),
    #[error("Tried to pop from an empty path stack. This is a bug.")]
    PopFromEmptyValidator,
    #[error("Unsupported validation operation.")]
    Unsupported,
}
impl ValidationError for ValidationFailure {}

#[derive(Error, Debug, PartialEq)]
pub enum SmithyConstraints {
    /// [@required](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-required-trait)
    #[error("Field is Required.")]
    Required,
    /// [@length](https://smithy.io/2.0/spec/constraint-traits.html#length-trait)
    #[error("Size: {0} does not conform to @length constraint. Expected between {1} and {2}.")]
    Length(usize, usize, usize),
    /// [@pattern](https://smithy.io/2.0/spec/constraint-traits.html#pattern-trait)
    #[error("Value `{0}` did not conform to expected pattern `{1}`")]
    Pattern(String, String),
    /// [@range](https://smithy.io/2.0/spec/constraint-traits.html#range-trait)
    #[error("Size: {0} does not conform to @range constraint. Expected between {1} and {2}.")]
    Range(BigDecimal, BigDecimal, BigDecimal),
    // TODO(question): Could this be security risk if non-unique are returned?
    /// [@uniqueItems](https://smithy.io/2.0/spec/constraint-traits.html#uniqueitems-trait]
    #[error("Items in collection should be unique.")]
    UniqueItems
}
impl ValidationError for SmithyConstraints {}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;
    use crate::traits;
    use crate::prelude::{INTEGER, STRING};
    use crate::schema::{Schema, ShapeId, StaticSchemaShape};
    use crate::serde::builders::{BuildWithCorrection, Required, MaybeBuilt};
    use crate::serde::de::Deserializer;
    use crate::serde::deserializers::DeserializeWithSchema;
    use crate::serde::ShapeBuilder;
    use super::*;

    #[test]
    fn test_validation_errors_aggregate() {
        let mut errors = ValidationErrors::new();
        errors.add(&vec![STRING.clone()], SmithyConstraints::Required);
        errors.add(&vec![STRING.clone()], SmithyConstraints::Length(1,2,3));
        errors.add(&vec![STRING.clone()], SmithyConstraints::Required);
        assert_eq!(errors.errors.len(), 3);
        assert_eq!(&errors.errors[0].error.to_string(), "Field is Required.");
        assert_eq!(&errors.errors[2].error.to_string(), "Field is Required.");
    }

    /// ==== Basic Shape Validations ====
    static LIST_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(ShapeId::from("com.example#List"), traits![LengthTrait::builder().max(3).build(), UniqueItemsTrait])
            .put_member("member", &STRING, traits![LengthTrait::builder().max(4).build()])
            .build()
    });
    static MAP_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::map_builder(ShapeId::from("com.example#Map"), traits![LengthTrait::builder().max(2).build(), UniqueItemsTrait])
            .put_member("key", &STRING, traits![PatternTrait::new("^[a-zA-Z]*$")])
            .put_member("value", &STRING, traits![LengthTrait::builder().max(4).build()])
            .build()
    });
    static BASIC_VALIDATION_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("test#ValidationStruct"), Vec::new())
            .put_member("field_a", &STRING, traits![PatternTrait::new("^[a-zA-Z]*$")])
            .put_member("field_b", &INTEGER, traits![])
            .put_member("field_list", &LIST_SCHEMA, traits![])
            .put_member("field_map", &MAP_SCHEMA, traits![])
            .build()
    });
    static FIELD_A: LazyLock<&SchemaRef> = LazyLock::new(|| BASIC_VALIDATION_SCHEMA.expect_member("field_a"));
    static FIELD_B: LazyLock<&SchemaRef> = LazyLock::new(|| BASIC_VALIDATION_SCHEMA.expect_member("field_b"));
    static FIELD_LIST: LazyLock<&SchemaRef> = LazyLock::new(|| BASIC_VALIDATION_SCHEMA.expect_member("field_list"));
    static FIELD_MAP: LazyLock<&SchemaRef> = LazyLock::new(|| BASIC_VALIDATION_SCHEMA.expect_member("field_map"));

    pub struct SimpleStruct {
        field_a: String,
        field_b: Option<i32>,
        field_list: Option<Vec<String>>,
        field_map: Option<IndexMap<String, String>>
    }
    pub struct SimpleStructBuilder {
        field_a: Required<String>,
        field_b: Option<i32>,
        field_list: Option<Vec<String>>,
        field_map: Option<IndexMap<String, String>>
    }
    impl SimpleStructBuilder {
        pub fn field_a(mut self, value: String) -> Self {
            self.field_a = Required::Set(value);
            self
        }

        pub fn field_b(mut self, value: i32) -> Self {
            self.field_b = Some(value);
            self
        }

        pub fn field_list(mut self, value: Vec<String>) -> Self {
            self.field_list = Some(value);
            self
        }

        pub fn field_map(mut self, value: IndexMap<String, String>) -> Self {
            self.field_map = Some(value);
            self
        }
    }
    impl StaticSchemaShape for SimpleStruct {
        fn schema() -> &'static SchemaRef {
            &BASIC_VALIDATION_SCHEMA
        }
    }
    impl <'de> ShapeBuilder<'de, SimpleStruct> for SimpleStructBuilder {
        fn new() -> Self {
            Self {
                field_a: Required::Unset,
                field_b: None,
                field_list: None,
                field_map: None
            }
        }
    }
    impl BuildWithCorrection<SimpleStruct> for SimpleStructBuilder {
        fn build_with_correction(self) -> SimpleStruct {
            SimpleStruct {
                field_a: self.field_a.get_or_resolve(),
                field_b: self.field_b,
                field_list: self.field_list,
                field_map: self.field_map
            }
        }
    }
    impl SerializeWithSchema for SimpleStructBuilder {
        fn serialize_with_schema<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 8usize)?;
            ser.serialize_member_named("field_a", &FIELD_A, &self.field_a)?;
            ser.serialize_optional_member_named("field_b", &FIELD_B, &self.field_b)?;
            ser.serialize_optional_member_named("field_list", &FIELD_LIST, &self.field_list)?;
            ser.serialize_optional_member_named("field_list", &FIELD_MAP, &self.field_map)?;
            ser.end(schema)
        }
    }
    impl <'de> DeserializeWithSchema<'de> for SimpleStructBuilder {
        fn deserialize_with_schema<D>(_schema: &SchemaRef, _deserializer: &mut D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>
        {
            unimplemented!("We dont need to deserialize to test.")
        }
    }

    #[test]
    fn builds_if_no_errors() {
        let output = SimpleStructBuilder::new()
            .field_a("fieldA".to_string())
            .build()
            .expect("Failed to build SimpleStruct");
        assert_eq!(output.field_a, "fieldA".to_string());
    }

    #[test]
    fn required_fields_are_validated() {
        let builder = SimpleStructBuilder::new();
        let Err(err) = builder.build() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);

        let error_field_a = err.errors.get(0).unwrap();
        assert_eq!(error_field_a.paths, vec![BASIC_VALIDATION_SCHEMA.clone(), FIELD_A.clone()]);
        assert_eq!(error_field_a.error.to_string(), "Field is Required.".to_string());
    }

    #[test]
    fn basic_string_validations_are_performed() {
        let builder = SimpleStructBuilder::new();
        let inner_vec = vec!["too long of a string".to_string()];
        let Some(err) = builder.field_list(inner_vec)
            .field_a("field-a".to_string())
            .build().err() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 2);
        let error_pattern = err.errors.get(0).unwrap();
        assert_eq!(error_pattern.paths, vec![BASIC_VALIDATION_SCHEMA.clone(), FIELD_A.clone()]);
        assert_eq!(error_pattern.error.to_string(), "Value `field-a` did not conform to expected pattern `^[a-zA-Z]*$`".to_string());

        let error_length = err.errors.get(1).unwrap();
        assert_eq!(error_length.paths, vec![BASIC_VALIDATION_SCHEMA.clone(), FIELD_LIST.clone(), LIST_SCHEMA.expect_member("member").clone()]);
        assert_eq!(error_length.error.to_string(), "Size: 20 does not conform to @length constraint. Expected between 0 and 4.".to_string());
    }

    #[test]
    fn required_field_does_not_short_circuit_validation() {
        let inner_vec = vec!["too long of a string".to_string()];
        let Err(err) = SimpleStructBuilder::new().field_list(inner_vec).build() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 2);
        let error_required = err.errors.get(0).unwrap();
        let error_length = err.errors.get(1).unwrap();

        assert_eq!(error_required.paths, vec![BASIC_VALIDATION_SCHEMA.clone(), FIELD_A.clone()]);
        assert_eq!(error_required.error.to_string(), "Field is Required.".to_string());

        assert_eq!(error_length.paths, vec![BASIC_VALIDATION_SCHEMA.clone(), FIELD_LIST.clone(), LIST_SCHEMA.expect_member("member").clone()]);
        assert_eq!(error_length.error.to_string(), "Size: 20 does not conform to @length constraint. Expected between 0 and 4.".to_string());
    }

    #[test]
    fn list_constraints_checked() {
        let builder = SimpleStructBuilder::new();
        let inner_vec = vec!["a".to_string(), "b".to_string(), "c".to_string(), "a".to_string(), "d".to_string()];
        let Some(err) = builder.field_list(inner_vec).field_a("fieldA".to_string()).build().err() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);
        let error_length = err.errors.get(0).unwrap();

        assert_eq!(error_length.paths, vec![BASIC_VALIDATION_SCHEMA.clone(), FIELD_LIST.clone()]);
        assert_eq!(error_length.error.to_string(), "Size: 5 does not conform to @length constraint. Expected between 0 and 3.".to_string());

        // TODO: Unique item support
        //let error_unique = err.errors.get(1).unwrap();
        // assert_eq!(error_unique.paths, vec![FIELD_LIST.clone(), LIST_SCHEMA.expect_member("member").clone()]);
        // assert_eq!(error_unique.error.to_string(), "Items in collection should be unique.".to_string());
    }

    #[test]
    fn map_constraints_checked() {
        let builder = SimpleStructBuilder::new();
        let mut inner_map = IndexMap::<String, String>::new();
        inner_map.insert("bad-key".to_string(), "a".to_string());
        inner_map.insert("a".to_string(), "value is too long!".to_string());
        inner_map.insert("b".to_string(), "a".to_string());
        let Some(err) = builder.field_map(inner_map).field_a("fieldA".to_string()).build().err() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 3);

        let error_length = err.errors.get(0).unwrap();
        assert_eq!(error_length.paths, vec![BASIC_VALIDATION_SCHEMA.clone(), FIELD_MAP.clone()]);
        assert_eq!(error_length.error.to_string(), "Size: 3 does not conform to @length constraint. Expected between 0 and 2.".to_string());

        let error_key = err.errors.get(1).unwrap();
        assert_eq!(error_key.paths, vec![BASIC_VALIDATION_SCHEMA.clone(), FIELD_MAP.clone(), MAP_SCHEMA.expect_member("key").clone()]);
        assert_eq!(error_key.error.to_string(), "Value `bad-key` did not conform to expected pattern `^[a-zA-Z]*$`".to_string());

        let error_value = err.errors.get(2).unwrap();
        assert_eq!(error_value.paths, vec![BASIC_VALIDATION_SCHEMA.clone(), FIELD_MAP.clone(), MAP_SCHEMA.expect_member("value").clone()]);
        assert_eq!(error_value.error.to_string(), "Size: 18 does not conform to @length constraint. Expected between 0 and 4.".to_string());
    }

    //// ====== NESTED SHAPE VALIDATION =====
    // Nested Shape
    static NESTED_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("test#ValidationStruct"), Vec::new())
            .put_member("field_c", &STRING, traits![PatternTrait::new("^[a-z]*$")])
            .build()
    });
    static FIELD_C: LazyLock<&SchemaRef> = LazyLock::new(|| { NESTED_SCHEMA.expect_member("field_c") });

    #[derive(Hash)]
    pub struct NestedStruct {
        field_c: String,
    }
    impl StaticSchemaShape for NestedStruct {
        fn schema() -> &'static SchemaRef {
            &NESTED_SCHEMA
        }
    }
    impl SerializeWithSchema for NestedStruct {
        fn serialize_with_schema<S: Serializer>(
            &self,
            schema: &SchemaRef,
            serializer: S
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 2usize)?;
            ser.serialize_member_named("field_c", &FIELD_C, &self.field_c)?;
            ser.end(schema)
        }
    }

    pub struct NestedStructBuilder {
        field_c: Required<String>,
    }
    impl NestedStructBuilder {
        pub fn field_c(mut self, value: String) -> Self {
            self.field_c = Required::Set(value);
            self
        }
    }
    impl <'de> DeserializeWithSchema<'de> for NestedStructBuilder {
        fn deserialize_with_schema<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>
        {
            unimplemented!("We dont need to deserialize to test.")
        }
    }
    impl ErrorCorrection for NestedStruct {
        fn default() -> Self {
            NestedStructBuilder::new().build_with_correction()
        }
    }
    impl SerializeWithSchema for NestedStructBuilder {
        fn serialize_with_schema<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 1usize)?;
            ser.serialize_member(&FIELD_C, &self.field_c)?;
            ser.end(schema)
        }
    }
    impl <'de> ShapeBuilder<'de, NestedStruct> for NestedStructBuilder {
        fn new() -> Self {
            Self {
                field_c: Required::Unset,
            }
        }
    }
    impl BuildWithCorrection<NestedStruct> for NestedStructBuilder {
        fn build_with_correction(self) -> NestedStruct {
            NestedStruct {
                field_c: self.field_c.get_or_resolve(),
            }
        }
    }

    // Shape with nested shape fields.
    static STRUCT_WITH_NESTED_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("test#StructWithNested"), Vec::new())
            .put_member("field_nested", &NESTED_SCHEMA, traits![])
            .put_member("field_nested_required", &NESTED_SCHEMA, traits![])
            .build()
    });
    static FIELD_NESTED: LazyLock<&SchemaRef> = LazyLock::new(|| STRUCT_WITH_NESTED_SCHEMA.expect_member("field_nested"));
    static FIELD_NESTED_REQUIRED: LazyLock<&SchemaRef> = LazyLock::new(|| STRUCT_WITH_NESTED_SCHEMA.expect_member("field_nested_required"));

    struct StructWithNested {
        field_nested: Option<NestedStruct>,
        field_required_nested: NestedStruct,
    }
    impl StaticSchemaShape for StructWithNested {
        fn schema() -> &'static SchemaRef {
            &STRUCT_WITH_NESTED_SCHEMA
        }
    }
    struct StructWithNestedBuilder {
        field_nested: Option<MaybeBuilt<NestedStruct, NestedStructBuilder>>,
        field_required_nested: Required<MaybeBuilt<NestedStruct, NestedStructBuilder>>,
    }
    impl <'de> DeserializeWithSchema<'de> for StructWithNestedBuilder {
        fn deserialize_with_schema<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>
        {
            unimplemented!("We dont need to deserialize to test.")
        }
    }
    impl BuildWithCorrection<StructWithNested> for StructWithNestedBuilder {
        fn build_with_correction(self) -> StructWithNested {
            StructWithNested {
                field_nested: self.field_nested.build_with_correction(),
                field_required_nested: self.field_required_nested.get_or_resolve().build_with_correction(),
            }
        }
    }
    impl SerializeWithSchema for StructWithNestedBuilder {
        fn serialize_with_schema<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 2usize)?;
            ser.serialize_optional_member(&FIELD_NESTED, &self.field_nested)?;
            ser.serialize_member(&FIELD_NESTED_REQUIRED, &self.field_required_nested)?;
            ser.end(schema)
        }
    }
    impl <'de> ShapeBuilder<'de, StructWithNested> for StructWithNestedBuilder {
        fn new() -> Self {
            StructWithNestedBuilder {
                field_nested: None,
                field_required_nested: Required::Unset,
            }
        }
    }
    impl StructWithNestedBuilder {
        pub fn field_nested(mut self, value: NestedStruct) -> Self {
            self.field_nested = Some(MaybeBuilt::Struct(value));
            self
        }

        #[doc(hidden)]
        pub fn field_nested_builder(mut self, value: NestedStructBuilder) -> Self {
            self.field_nested = Some(MaybeBuilt::Builder(value));
            self
        }

        pub fn field_nested_required(mut self, value: NestedStruct) -> Self {
            self.field_required_nested = Required::Set(MaybeBuilt::Struct(value));
            self
        }

        #[doc(hidden)]
        pub fn field_nested_required_builder(mut self, value: NestedStructBuilder) -> Self {
            self.field_required_nested = Required::Set(MaybeBuilt::Builder(value));
            self
        }
    }

    #[test]
    fn nested_struct_fields_build_if_no_errors() {
        let builder_nested = NestedStructBuilder::new().field_c("field".to_string());
        let builder = StructWithNestedBuilder::new();
        let value = builder.field_nested_required_builder(builder_nested)
            .build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_fields_build_with_pre_built_shape() {
        let built_nested = NestedStructBuilder::new().field_c("field".to_string())
            .build().expect("Failed to build NestedStruct");
        let builder = StructWithNestedBuilder::new();
        let value = builder.field_nested_required(built_nested).build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_fields_checked() {
        let builder_nested = NestedStructBuilder::new().field_c("dataWithCaps".to_string());
        let builder = StructWithNestedBuilder::new();
        let Some(err) = builder.field_nested_required_builder(builder_nested).build().err() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);
        let error_pattern = err.errors.get(0).unwrap();
        assert_eq!(error_pattern.paths, vec![STRUCT_WITH_NESTED_SCHEMA.clone(), FIELD_NESTED_REQUIRED.clone(), FIELD_C.clone()]);
        assert_eq!(error_pattern.error.to_string(), "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string());
    }

    // ==== Nested List Validations ====
    static LIST_OF_NESTED_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(ShapeId::from("com.example#ListOfNested"), traits![LengthTrait::builder().max(3).build(), UniqueItemsTrait])
            .put_member("member", &NESTED_SCHEMA, traits![])
            .build()
    });
    static LIST_OF_LIST_OF_NESTED: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(ShapeId::from("com.example#ListOfList"), traits![LengthTrait::builder().max(2).build()])
            .put_member("member", &LIST_OF_NESTED_SCHEMA, traits![])
            .build()
    });
    static LIST_OF_LIST_OF_LIST_OF_NESTED: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(ShapeId::from("com.example#ListOfListOfList"), traits![LengthTrait::builder().max(2).build()])
            .put_member("member", &LIST_OF_LIST_OF_NESTED, traits![])
            .build()
    });
    static STRUCT_WITH_NESTED_LIST_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("test#StructWithNestedList"), Vec::new())
            .put_member("field_nested_list", &LIST_OF_NESTED_SCHEMA, traits![])
            .put_member("field_nested_list_required", &LIST_OF_NESTED_SCHEMA, traits![])
            .put_member("field_deeply_nested_list", &LIST_OF_LIST_OF_LIST_OF_NESTED, traits![])
            .build()
    });
    static FIELD_NESTED_LIST: LazyLock<&SchemaRef> = LazyLock::new(|| STRUCT_WITH_NESTED_LIST_SCHEMA.expect_member("field_nested_list"));
    static FIELD_NESTED_LIST_REQUIRED: LazyLock<&SchemaRef> = LazyLock::new(|| STRUCT_WITH_NESTED_LIST_SCHEMA.expect_member("field_nested_list_required"));
    static FIELD_DEEPLY_NESTED_LIST: LazyLock<&SchemaRef> =  LazyLock::new(|| STRUCT_WITH_NESTED_LIST_SCHEMA.expect_member("field_deeply_nested_list"));


    struct StructWithNestedLists {
        field_nested_list: Option<Vec<NestedStruct>>,
        field_required_nested_list: Vec<NestedStruct>,
        field_deeply_nested_list: Option<Vec<Vec<Vec<NestedStruct>>>>,
    }
    impl StaticSchemaShape for StructWithNestedLists {
        fn schema() -> &'static SchemaRef {
            &STRUCT_WITH_NESTED_LIST_SCHEMA
        }
    }

    struct StructWithNestedListsBuilder {
        field_nested_list: Option<MaybeBuilt<Vec<NestedStruct>, Vec<NestedStructBuilder>>>,
        field_required_nested_list: Required<MaybeBuilt<Vec<NestedStruct>, Vec<NestedStructBuilder>>>,
        field_deeply_nested_list: Option<MaybeBuilt<Vec<Vec<Vec<NestedStruct>>>, Vec<Vec<Vec<NestedStructBuilder>>>>>
    }
    impl <'de> DeserializeWithSchema<'de> for StructWithNestedListsBuilder {
        fn deserialize_with_schema<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>
        {
            unimplemented!("We dont need to deserialize for testing.")
        }
    }
    impl SerializeWithSchema for StructWithNestedListsBuilder {
        fn serialize_with_schema<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 3usize)?;
            ser.serialize_optional_member(&FIELD_NESTED_LIST, &self.field_nested_list)?;
            ser.serialize_member(&FIELD_NESTED_LIST_REQUIRED, &self.field_required_nested_list)?;
            ser.serialize_optional_member(&FIELD_DEEPLY_NESTED_LIST, &self.field_deeply_nested_list)?;
            ser.end(schema)
        }
    }
    impl BuildWithCorrection<StructWithNestedLists> for StructWithNestedListsBuilder {
        fn build_with_correction(self) -> StructWithNestedLists {
            StructWithNestedLists {
                field_nested_list: self.field_nested_list.build_with_correction(),
                field_required_nested_list: self.field_required_nested_list.get_or_resolve().build_with_correction(),
                field_deeply_nested_list: self.field_deeply_nested_list.build_with_correction(),
            }
        }
    }
    impl <'de> ShapeBuilder<'de, StructWithNestedLists> for StructWithNestedListsBuilder {
        fn new() -> Self {
            StructWithNestedListsBuilder {
                field_nested_list: None,
                field_required_nested_list: Required::Unset,
                field_deeply_nested_list: None,
            }
        }
    }
    impl StructWithNestedListsBuilder {
        pub fn field_nested_list(mut self, values: Vec<NestedStruct>) -> Self {
            self.field_nested_list = Some(MaybeBuilt::Struct(values));
            self
        }

        #[doc(hidden)]
        pub fn field_nested_list_builder(mut self, values: Vec<NestedStructBuilder>) -> Self {
            self.field_nested_list = Some(MaybeBuilt::Builder(values));
            self
        }

        pub fn field_require_nested_list(mut self, values: Vec<NestedStruct>) -> Self {
            self.field_required_nested_list = Required::Set(MaybeBuilt::Struct(values));
            self
        }

        #[doc(hidden)]
        pub fn field_required_nested_list_builder(mut self, values: Vec<NestedStructBuilder>) -> Self {
            self.field_required_nested_list = Required::Set(MaybeBuilt::Builder(values));
            self
        }

        pub fn field_deeply_nested_list(mut self, values: Vec<Vec<Vec<NestedStruct>>>) -> Self {
            self.field_deeply_nested_list = Some(MaybeBuilt::Struct(values));
            self
        }

        #[doc(hidden)]
        pub fn field_deeply_nested_list_builder(mut self, values: Vec<Vec<Vec<NestedStructBuilder>>>) -> Self {
            self.field_deeply_nested_list = Some(MaybeBuilt::Builder(values));
            self
        }
    }

    #[test]
    fn nested_struct_list_build_if_no_errors() {
        let nested_list = vec![NestedStructBuilder::new().field_c("data".to_string())];
        let builder = StructWithNestedListsBuilder::new();
        builder.field_required_nested_list_builder(nested_list).build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_list_fields_build_with_pre_built_shapes() {
        let nested_list = vec![
            NestedStructBuilder::new().field_c("a".to_string()).build().expect("Failed to build NestedStruct"),
            NestedStructBuilder::new().field_c("b".to_string()).build().expect("Failed to build NestedStruct")
        ];
        let builder = StructWithNestedListsBuilder::new();
        let value = builder
            .field_require_nested_list(nested_list).build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_list_checked() {
        let nested_list = vec![
            NestedStructBuilder::new().field_c("a".to_string()),
            NestedStructBuilder::new().field_c("b".to_string()),
            NestedStructBuilder::new().field_c("dataWithCaps".to_string()),
            NestedStructBuilder::new().field_c("b".to_string())
        ];
        let builder = StructWithNestedListsBuilder::new();
        let Some(err) = builder.field_required_nested_list_builder(nested_list).build().err() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 2);

        let error_length = err.errors.get(0).unwrap();
        assert_eq!(error_length.paths, [STRUCT_WITH_NESTED_LIST_SCHEMA.clone(), FIELD_NESTED_LIST_REQUIRED.clone()]);
        assert_eq!(error_length.error.to_string(), "Size: 4 does not conform to @length constraint. Expected between 0 and 3.".to_string());

        let error_pattern = err.errors.get(1).unwrap();
        assert_eq!(error_pattern.paths, [STRUCT_WITH_NESTED_LIST_SCHEMA.clone(), FIELD_NESTED_LIST_REQUIRED.clone(), LIST_OF_NESTED_SCHEMA.expect_member("member").clone(), FIELD_C.clone()]);
        assert_eq!(error_pattern.error.to_string(), "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string());

        // TODO: ADD UNIQUENESS CHECKS
        // let error_unique = err.errors.get(2).unwrap();
        // assert_eq!(error_unique.path, *LIST_OF_NESTED_SCHEMA.expect_member("member"));
        // assert_eq!(error_unique.error.to_string(), "Items in collection should be unique.".to_string());
    }

    #[test]
    fn deeply_nested_struct_list_checks_validation_rules() {
        let nested_list = vec![NestedStructBuilder::new().field_c("data".to_string())];
        let deeply_nested_list = vec![vec![vec![NestedStructBuilder::new().field_c("dataWithCaps".to_string())]]];
        let builder = StructWithNestedListsBuilder::new();
        let Some(err) = builder
            .field_required_nested_list_builder(nested_list)
            .field_deeply_nested_list_builder(deeply_nested_list)
            .build().err() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);

        let error_pattern = err.errors.get(0).unwrap();
        assert_eq!(error_pattern.paths, [
            STRUCT_WITH_NESTED_LIST_SCHEMA.clone(),
            FIELD_DEEPLY_NESTED_LIST.clone(),
            LIST_OF_LIST_OF_LIST_OF_NESTED.expect_member("member").clone(),
            LIST_OF_LIST_OF_NESTED.expect_member("member").clone(),
            LIST_OF_NESTED_SCHEMA.expect_member("member").clone(),
            FIELD_C.clone()
        ]);
        assert_eq!(error_pattern.error.to_string(), "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string());
    }

    // ==== Nested Map Validations ====
    static MAP_OF_NESTED_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::map_builder(ShapeId::from("com.example#MapOfNested"), traits![LengthTrait::builder().max(2).build(), UniqueItemsTrait])
            .put_member("key", &STRING, traits![])
            .put_member("value", &NESTED_SCHEMA, traits![])
            .build()
    });
    static MAP_OF_MAP_OF_NESTED: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::map_builder(ShapeId::from("com.example#MapOfMap"), traits![LengthTrait::builder().max(2).build()])
            .put_member("key", &STRING, traits![])
            .put_member("value", &MAP_OF_NESTED_SCHEMA, traits![])
            .build()
    });
    static MAP_OF_MAP_OF_MAP_OF_NESTED: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::map_builder(ShapeId::from("com.example#MapOfMapOfMap"), traits![LengthTrait::builder().max(2).build()])
            .put_member("key", &STRING, traits![])
            .put_member("value", &MAP_OF_MAP_OF_NESTED, traits![])
            .build()
    });
    static STRUCT_WITH_NESTED_MAP_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("test#StructWithNestedMap"), Vec::new())
            .put_member("field_nested_map", &MAP_OF_NESTED_SCHEMA, traits![])
            .put_member("field_nested_map_required", &MAP_OF_NESTED_SCHEMA, traits![])
            .put_member("field_deeply_nested_map", &MAP_OF_MAP_OF_MAP_OF_NESTED, traits![])
            .build()
    });
    static FIELD_NESTED_MAP: LazyLock<&SchemaRef> = LazyLock::new(|| STRUCT_WITH_NESTED_MAP_SCHEMA.expect_member("field_nested_map"));
    static FIELD_NESTED_MAP_REQUIRED: LazyLock<&SchemaRef> = LazyLock::new(|| STRUCT_WITH_NESTED_MAP_SCHEMA.expect_member("field_nested_map_required"));
    static FIELD_DEEPLY_NESTED_MAP: LazyLock<&SchemaRef> =  LazyLock::new(|| STRUCT_WITH_NESTED_MAP_SCHEMA.expect_member("field_deeply_nested_map"));

    struct StructWithNestedMaps {
        field_nested_map: Option<IndexMap<String, NestedStruct>>,
        field_nested_map_required: IndexMap<String, NestedStruct>,
        field_deeply_nested_map: Option<IndexMap<String, IndexMap<String, IndexMap<String, NestedStruct>>>>,
    }

    impl StaticSchemaShape for StructWithNestedMaps {
        fn schema() -> &'static SchemaRef {
            &STRUCT_WITH_NESTED_MAP_SCHEMA
        }
    }

    struct StructWithNestedMapsBuilder {
        field_nested_map: Option<MaybeBuilt<IndexMap<String, NestedStruct>, IndexMap<String, NestedStructBuilder>>>,
        field_nested_map_required: Required<MaybeBuilt<IndexMap<String, NestedStruct>, IndexMap<String, NestedStructBuilder>>>,
        field_deeply_nested_map: Option<MaybeBuilt<
            IndexMap<String, IndexMap<String, IndexMap<String, NestedStruct>>>,
            IndexMap<String, IndexMap<String, IndexMap<String, NestedStructBuilder>>>
        >>,
    }
    impl <'de> DeserializeWithSchema<'de> for StructWithNestedMapsBuilder {
        fn deserialize_with_schema<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>
        {
            unimplemented!("We dont need to deserialize for testing.")
        }
    }
    impl SerializeWithSchema for StructWithNestedMapsBuilder {
        fn serialize_with_schema<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 3usize)?;
            ser.serialize_optional_member(&FIELD_NESTED_MAP, &self.field_nested_map)?;
            ser.serialize_member(&FIELD_NESTED_MAP_REQUIRED, &self.field_nested_map_required)?;
            ser.serialize_optional_member(&FIELD_DEEPLY_NESTED_MAP, &self.field_deeply_nested_map)?;
            ser.end(schema)
        }
    }
    impl BuildWithCorrection<StructWithNestedMaps> for StructWithNestedMapsBuilder {
        fn build_with_correction(self) -> StructWithNestedMaps {
            StructWithNestedMaps {
                field_nested_map: self.field_nested_map.build_with_correction(),
                field_nested_map_required: self.field_nested_map_required.get_or_resolve().build_with_correction(),
                field_deeply_nested_map: self.field_deeply_nested_map.build_with_correction(),
            }
        }
    }
    impl <'de> ShapeBuilder<'de, StructWithNestedMaps> for StructWithNestedMapsBuilder {
        fn new() -> Self {
            StructWithNestedMapsBuilder {
                field_nested_map: None,
                field_nested_map_required: Required::Unset,
                field_deeply_nested_map: None,
            }
        }
    }
    impl StructWithNestedMapsBuilder {
        pub fn field_nested_map(mut self, values: IndexMap<String, NestedStruct>) -> Self {
            self.field_nested_map = Some(MaybeBuilt::Struct(values));
            self
        }

        #[doc(hidden)]
        pub fn field_nested_map_builder(mut self, values: IndexMap<String, NestedStructBuilder>) -> Self {
            self.field_nested_map = Some(MaybeBuilt::Builder(values));
            self
        }

        pub fn field_require_nested_map(mut self, values: IndexMap<String, NestedStruct>) -> Self {
            self.field_nested_map_required = Required::Set(MaybeBuilt::Struct(values));
            self
        }

        #[doc(hidden)]
        pub fn field_required_nested_map_builder(mut self, values: IndexMap<String, NestedStructBuilder>) -> Self {
            self.field_nested_map_required = Required::Set(MaybeBuilt::Builder(values));
            self
        }

        pub fn field_deeply_nested_map(mut self, values: IndexMap<String, IndexMap<String, IndexMap<String, NestedStruct>>>) -> Self {
            self.field_deeply_nested_map = Some(MaybeBuilt::Struct(values));
            self
        }

        #[doc(hidden)]
        pub fn field_deeply_nested_map_builder(mut self, values: IndexMap<String, IndexMap<String, IndexMap<String, NestedStructBuilder>>>) -> Self {
            self.field_deeply_nested_map = Some(MaybeBuilt::Builder(values));
            self
        }
    }

    #[test]
    fn nested_struct_map_build_if_no_errors() {
        let mut nested_map = IndexMap::new();
        nested_map.insert("a".to_string(), NestedStructBuilder::new().field_c("data".to_string()));
        let builder = StructWithNestedMapsBuilder::new();
        builder.field_required_nested_map_builder(nested_map).build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_map_fields_build_with_pre_built_shapes() {
        let mut nested_map = IndexMap::new();
        nested_map.insert("a".to_string(), NestedStructBuilder::new().field_c("data".to_string()).build().expect("Failed to build nested"));
        let builder = StructWithNestedMapsBuilder::new();
        builder.field_require_nested_map(nested_map).build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_map_checked() {
        let mut nested_map = IndexMap::new();
        nested_map.insert("a".to_string(), NestedStructBuilder::new().field_c("a".to_string()));
        nested_map.insert("b".to_string(), NestedStructBuilder::new().field_c("dataWithCaps".to_string()));
        nested_map.insert("c".to_string(), NestedStructBuilder::new().field_c("c".to_string()));
        let builder = StructWithNestedMapsBuilder::new();
        let Some(err) = builder.field_required_nested_map_builder(nested_map).build().err() else {
            panic!("Expected an error");
        };

        assert_eq!(err.errors.len(), 2);

        let error_length = err.errors.get(0).unwrap();
        assert_eq!(error_length.paths, [STRUCT_WITH_NESTED_MAP_SCHEMA.clone(), FIELD_NESTED_MAP_REQUIRED.clone()]);
        assert_eq!(error_length.error.to_string(), "Size: 3 does not conform to @length constraint. Expected between 0 and 2.".to_string());

        let error_pattern = err.errors.get(1).unwrap();
        assert_eq!(error_pattern.paths, [STRUCT_WITH_NESTED_MAP_SCHEMA.clone(), FIELD_NESTED_MAP_REQUIRED.clone(), MAP_OF_NESTED_SCHEMA.expect_member("value").clone(), FIELD_C.clone()]);
        assert_eq!(error_pattern.error.to_string(), "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string());
    }

    #[test]
    fn deeply_nested_struct_map_build_if_no_errors() {
        let mut nested_map = IndexMap::new();
        nested_map.insert("a".to_string(), NestedStructBuilder::new().field_c("a".to_string()));

        let mut deep_nesting = IndexMap::new();
        let mut mid_nesting = IndexMap::new();
        let mut low_nesting = IndexMap::new();
        low_nesting.insert("a".to_string(), NestedStructBuilder::new().field_c("dataWithCaps".to_string()));
        mid_nesting.insert("a".to_string(), low_nesting);
        deep_nesting.insert("a".to_string(), mid_nesting);
        let builder = StructWithNestedMapsBuilder::new();
        let Some(err) = builder
            .field_required_nested_map_builder(nested_map)
            .field_deeply_nested_map_builder(deep_nesting)
            .build().err() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);

        let error_pattern = err.errors.get(0).unwrap();
        assert_eq!(error_pattern.paths, [
            STRUCT_WITH_NESTED_MAP_SCHEMA.clone(),
            FIELD_DEEPLY_NESTED_MAP.clone(),
            MAP_OF_MAP_OF_MAP_OF_NESTED.expect_member("value").clone(),
            MAP_OF_MAP_OF_NESTED.expect_member("value").clone(),
            MAP_OF_NESTED_SCHEMA.expect_member("value").clone(),
            FIELD_C.clone()
        ]);
        assert_eq!(error_pattern.error.to_string(), "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string());
    }
}