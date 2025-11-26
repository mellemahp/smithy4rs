use std::collections::BTreeMap;
use std::convert::Into;
use std::error::Error;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use bigdecimal::{BigDecimal, Zero};
use indexmap::IndexMap;
use num_bigint::BigInt;
use rustc_hash::FxHasher;
use thiserror::Error;
use crate::schema::{Document, DocumentValue, SchemaRef, ShapeType};
use crate::Instant;
use crate::prelude::{LengthTrait, PatternTrait, UniqueItemsTrait};

//////////////////////////////////////////////////////////////////////////////
// Traits
//////////////////////////////////////////////////////////////////////////////

// macro_rules! check_type {
//     ($self:ident, $schema:ident, $expected:expr) => {
//         if *$schema.shape_type() != $expected {
//             $self.emitError($schema, ValidationFailure::InvalidType($schema.shape_type().clone(), $expected))?;
//         }
//     };
// }

// macro_rules! check_range {
//     ($self:ident, $schema:ident, $value:ident) => {
//         if let Some(range) = &$schema.get_trait_as::<RangeTrait>() {
//             if ($value < range.min() || $value > range.max()) {
//                 $self.emit_error($schema, SmithyConstraints::Range($value.into(), range.min(), range.max()))?;
//             }
//         }
//     };
// }


// TODO: Update docs to be a bit less clunky
/// NOTE: Smithy error correction is not implemented directly.
pub trait Validator {
    /// Validates list items
    type ItemValidator: ListValidator;

    /// Validates map entries
    type EntryValidator: MapValidator;

    /// Validates structure builders
    type StructureValidator: StructureValidator;

    /// Emit an error for accumulation.
    ///
    /// This method should only emit an error when the maximum number
    /// of errors is hit. At that point it should simply raise all the
    /// existing validation errors list plus an extra appended error
    /// to indicate the error limit was reached.
    fn emit_error<E: ValidationError + 'static>(&mut self, path: &SchemaRef, err: E) -> Result<(), ValidationErrors>;

    /// Return all collected validation errors
    ///
    /// This returns a `Result` type to allow `?` raising.
    fn results(self) -> Result<(), ValidationErrors>;

    /// Checks top-level list constraints and returns a validator used validate list items
    fn validate_list(
        self,
        schema: &SchemaRef,
        size: usize,
    ) -> Result<Self::ItemValidator, ValidationErrors>;

    /// Checks top-level map constraints and returns a validator used validate map entries
    fn validate_map(
        self,
        schema: &SchemaRef,
        size: usize,
    ) -> Result<Self::EntryValidator, ValidationErrors>;

    /// Checks top-level structure constraints and returns a validator used to validate structure members
    fn validate_struct(
        self,
        schema: &SchemaRef,
    ) -> Result<Self::StructureValidator, ValidationErrors>;

    // /// Validate a `boolean` in place
    // fn validate_boolean(&mut self, schema: &SchemaRef, _bool: &bool) -> Result<(), ValidationErrors> {
    //     //check_type!(self, schema, ShapeType::Boolean);
    //     Ok(())
    // }

    /// Validate a `String` in place
    fn validate_string(
        self,
        schema: &SchemaRef,
        value: &String,
    ) -> Result<(), ValidationErrors>;

    /// Validate a byte (`i8`) in place
    // fn validate_byte(&mut self, schema: &SchemaRef, byte: &i8) -> Result<(), ValidationErrors> {
    //     // check_type!(self, schema, ShapeType::Byte);
    //     //check_range!(self, schema, byte);
    //     Ok(())
    // }
    //
    // /// Validate a short (`i16`) in place
    // fn validate_short(&mut self, schema: &SchemaRef, short: &i16) -> Result<(), ValidationErrors> {
    //     check_type!(self, schema, ShapeType::Short);
    //     //check_range!(self, schema, short);
    //     Ok(())
    // }

    /// Validate an integer (`i32`) in place
    fn validate_integer(
        &mut self,
        schema: &SchemaRef,
        value: &i32
    ) -> Result<(), ValidationErrors>;

    ///// Validate a long (`i64`) in place
    // fn validate_long(&mut self, schema: &SchemaRef, long: &i64) -> Result<(), ValidationErrors> {
    //     check_type!(self, schema, ShapeType::Long);
    //     //check_range!(self, schema, long);
    //     Ok(())
    // }
    //
    // /// Validate a float (`f32`) in place
    // fn validate_float(&mut self, schema: &SchemaRef, float: &f32) -> Result<(), ValidationErrors> {
    //     check_type!(self, schema, ShapeType::Float);
    //     //check_range!(self, schema, float);
    //     Ok(())
    // }

    // /// Validate a double (`f64`) in place
    // fn validate_double(&mut self, schema: &SchemaRef, double: &f64) -> Result<(), ValidationErrors> {
    //     check_type!(self, schema, ShapeType::Float);
    //     //check_range!(self, schema, double);
    //     Ok(())
    // }

    // /// Validate a Big Integer (`BigInt`) in place
    // fn validate_big_integer(
    //     &mut self,
    //     schema: &SchemaRef,
    //     _big_int: &BigInt,
    // ) -> Result<(), ValidationErrors> {
    //     check_type!(self, schema, ShapeType::BigInteger);
    //     // TODO: Check range
    //     Ok(())
    // }
    //
    // /// Validate a `BigDecimal` in place
    // fn validate_big_decimal(
    //     self,
    //     schema: &SchemaRef,
    //     _big_decimal: BigDecimal,
    // ) -> Result<(), ValidationErrors> {
    //     check_type!(self, schema, ShapeType::BigDecimal);
    //     // TODO: Check range
    //     Ok(())
    // }

    // /// Validate a document in place
    // fn validate_document(
    //     self,
    //     schema: &SchemaRef,
    //     _document: &Document,
    // ) -> Result<(), ValidationErrors> {
    //     // check_type!(self, schema, ShapeType::Document);
    //     todo!("Should this check nested types against schema?");
    // }
    //
    // // TODO: Should the enums check if the value is out of expected for validation? Could just check if
    // //       the value is ::__Unknown?
    // /// TODO: Should these check string validation?
    // fn validate_enum<E>(&mut self, schema: &SchemaRef, value: E) {
    //     check_type!(self, schema, ShapeType::Enum);
    //     todo!()
    // }
    //
    // fn validate_int_enum<E>(&mut self, schema: &SchemaRef, value: E) {
    //     check_type!(self, schema, ShapeType::IntEnum);
    //     todo!()
    // }



}

/// List Validator that can be called in a loop to validate list items
pub trait ListValidator {
    /// Validate a sequence element in place.
    ///
    /// Values must be hashable so that unique items can be tracked.
    fn validate_in_place<I>(
        &mut self,
        element_schema: &SchemaRef,
        value: &I,
    ) -> Result<(), ValidationErrors>
    where
        for<'a> &'a I: Validate,
        I: Hash;

    /// Validates an element and
    /// NOTE: This is primarily intended to support builder conversions
    fn validate_and_move<I: Validate>(
        &mut self,
        element_schema: &SchemaRef,
        value: I
    ) -> Result<I::Value, ValidationErrors>
    where
        I::Value: Hash;

    /// Checks if an item was already seen by this validator.
    ///
    /// This is used to support `@uniqueItems` constraint.
    /// **Impl Note**: the value tracker used by implementations is
    /// expected to only store hashes, not the actual value of items.
    fn check_uniqueness<T: Hash>(&mut self, element_schema: &SchemaRef, value: T) -> Result<(), ValidationErrors>;
}

/// Map Validator that can be called in a loop to validate map entries
pub trait MapValidator {
    /// Validate a single map entry in place
    fn validate_entry_in_place<K, V>(
        &mut self,
        key_schema: &SchemaRef,
        value_schema: &SchemaRef,
        key: &K,
        value: &V,
    ) -> Result<(), ValidationErrors>
    where
        for<'a> &'a K: Validate,
        for<'a> &'a V: Validate;

    /// Validates an entry and returns a new, owned value for it.
    fn validate_entry_and_move<K: Validate, V: Validate>(
        &mut self,
        key_schema: &SchemaRef,
        value_schema: &SchemaRef,
        key: K,
        value: V,
    ) -> Result<(K::Value, V::Value), ValidationErrors>;
}

pub trait StructureValidator {

    /// Validates a required field, returning a non-optional value.
    ///
    /// By default, on a missing value a [`SmithyConstraints::Required`] validation
    /// error is emitted.
    ///
    /// Validation errors are _only_ raised from this method if the validator hits
    /// a max depth or maximum number of errors. This allows the validator to short-circuit
    /// in those cases. Otherwise, users should use the [`results()?`] method to raise any
    /// validation errors _after_ all fields have been validated.
    ///
    /// IMPL NOTE: Implementations must still return a default value even if the
    /// required value is missing. Otherwise, we would be unable to accumulate values
    /// from nested shapes with required fields.
    fn validate_required<V: Validate>
    (
        &mut self,
        schema: &SchemaRef,
        value: Option<V>,
    ) -> Result<V::Value, ValidationErrors>
    where V::Value: ErrorCorrection;

    /// Validate an optional Field
    ///
    /// Validation errors are _only_ raised from this method if the validator hits
    /// a max depth or maximum number of errors. This allows the validator to short-circuit
    /// in those cases. Otherwise, users should use the [`results()?`] method to raise any
    /// validation errors _after_ all fields have been validated.
    fn validate_optional<V: Validate>(
        &mut self,
        schema: &SchemaRef,
        value: Option<V>,
    ) -> Result<Option<V::Value>, ValidationErrors>;
}



/// Indicates that a type can be validated by a [`Validator`] implementation.
///
/// All validate-able types must be able to provide a sane
/// default value through error correction.
pub trait Validate {
    /// Output type
    type Value;

    /// Validate a shape given its schema and a validator.
    ///
    /// NOTE: For builders this will result in them being built
    fn validate<V: Validator>(
        self,
        schema: &SchemaRef,
        validator: V,
    ) -> Result<Self::Value, ValidationErrors>;
}

//////////////////////////////////////////////////////////////////////////////
// Validate Implementations
//////////////////////////////////////////////////////////////////////////////

//
// impl Validate for IndexMap<String, String> {
//     type Value = Self;
//
//     fn validate<V: Validator>(self, schema: &SchemaRef, validator: V) -> Result<Self::Value, ValidationErrors> {
//         let mut map = validator.validate_map(schema, self.len())?;
//         let key_schema = schema.expect_member("key");
//         let value_schema = schema.expect_member("value");
//         for (k, v) in &self {
//             map.validate_entry_in_place(key_schema, value_schema, k, v)?;
//         }
//         Ok(self)
//     }
// }
// impl <'de, S, B: ShapeBuilder<'de, S>> Validate for Vec<B> {
//     type Value = Vec<S>;
//
//     fn validate<V: Validator>(self, schema: &SchemaRef, validator: V) -> Result<Self::Value, ValidationErrors> {
//         let mut list = validator.validate_list(schema, self.len())?;
//         let member_schema = schema.expect_member("member");
//         let mut result_list = Vec::with_capacity(self.len());
//         for item in self {
//             result_list.push(list.validate_and_move(member_schema, item)?);
//         }
//         Ok(result_list)
//     }
// }

// impl Validate for Vec<String> {
//     type Value = Vec<String>;
//     fn validate<V: Validator>(
//         self,
//         schema: &SchemaRef,
//         validator: V
//     ) -> Result<Self::Value, ValidationErrors> {
//         let mut list = validator.validate_list(schema, self.len())?;
//         let member_schema = schema.expect_member("member");
//         for item in &self {
//             list.validate_in_place(member_schema, item)?;
//         }
//         Ok(self)
//     }
// }

impl Validate for String {
    type Value = Self;

    fn validate<V: Validator>(
        self,
        schema: &SchemaRef,
        validator: V
    ) -> Result<Self::Value, ValidationErrors> {
        validator.validate_string(schema, &self)?;
        Ok(self)
    }
}

impl Validate for &String {
    type Value = Self;

    fn validate<V: Validator>(
        self,
        schema: &SchemaRef,
        validator: V
    ) -> Result<Self::Value, ValidationErrors> {
        validator.validate_string(schema, self)?;
        Ok(self)
    }
}

impl Validate for IndexMap<String, String> {
    type Value = Self;

    fn validate<V: Validator>(self, schema: &SchemaRef, validator: V) -> Result<Self::Value, ValidationErrors> {
        let mut map = validator.validate_map(schema, self.len())?;
        // TODO(errors): Should this short circuit to a validation error?
        let key_schema = schema.expect_member("key");
        let value_schema = schema.expect_member("value");
        for (k, v) in &self {
            map.validate_entry_in_place(key_schema, value_schema, k, v)?;
        }
        Ok(self)
    }
}

impl Validate for Vec<String> {
    type Value = Self;

    fn validate<V: Validator>(
        self,
        schema: &SchemaRef,
        validator: V
    ) -> Result<Self::Value, ValidationErrors> {
        let mut list = validator.validate_list(schema, self.len())?;
        // TODO(errors): Should this short circuit to a validation error?
        let member_schema = schema.expect_member("member");
        for item in &self {
            list.validate_in_place(member_schema, item)?;
        }
        Ok(self)
    }
}

impl Validate for i32 {
    type Value = Self;

    fn validate<V: Validator>(
        self,
        schema: &SchemaRef,
        mut validator: V,
    ) -> Result<Self::Value, ValidationErrors> {
        validator.validate_integer(schema, &self)?;
        Ok(self)
    }
}

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
    ///
    /// Schema reference must be passed to support document types.
    fn default(schema: &SchemaRef) -> Self;
}

macro_rules! correction_impl {
    ($t:ty, $v:expr) => {
        impl ErrorCorrection for $t {
            #[inline(always)]
            fn default(_schema: &SchemaRef) -> $t {
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
    fn default(schema: &SchemaRef) -> Self {
        Document {
            schema: schema.clone(),
            value: DocumentValue::Null,
            discriminator: None,
        }
    }
}

impl <E: ErrorCorrection> ErrorCorrection for Vec<E> {
    fn default(_schema: &SchemaRef) -> Self {
        Vec::new()
    }
}

impl <E: ErrorCorrection> ErrorCorrection for IndexMap<String, E> {
    fn default(_schema: &SchemaRef) -> Self {
        IndexMap::new()
    }
}

// TODO: ENUM AND INT ENUM IMPLS + Byte buffer impls


//////////////////////////////////////////////////////////////////////////////
// Default Validator Implementation
//////////////////////////////////////////////////////////////////////////////

/// Default validator that ensures shapes conform to base Smithy constraints.
///
/// For more info on built-in Smithy constraints see: [Smithy Documentation](https://smithy.io/2.0/spec/constraint-traits.html)
///
/// TODO: How to support custom validations?
/// TODO: Maybe use const generics for sizing?
pub struct DefaultValidator {
    errors: Option<ValidationErrors>,
    max_depth: usize,
    max_errors: usize,
}
impl DefaultValidator {
    pub const fn new() -> Self {
        DefaultValidator {
            errors: None,
            max_depth: 50,
            max_errors: 20,
        }
    }
}
impl <'a> Validator for &'a mut DefaultValidator {
    type ItemValidator = DefaultListValidator<'a>;
    type EntryValidator = DefaultMapValidator<'a>;
    type StructureValidator = DefaultStructValidator<'a>;

    fn emit_error<E: ValidationError + 'static>(&mut self, path: &SchemaRef, err: E) -> Result<(), ValidationErrors> {
        let errors = self.errors.get_or_insert(ValidationErrors::new());
        errors.add(path, err);
        if errors.len() >= self.max_errors {
            errors.add(path, ValidationFailure::MaxErrorsReached(self.max_errors));
            // SAFETY: Safe to unwrap as errors will alway be set to `SOME` above
            // TODO(code quality): maybe use a lazy initializer struct.
            return Err(self.errors.take().unwrap());
        }
        Ok(())
    }

    fn results(self) -> Result<(), ValidationErrors> {
        if let Some(errors) = self.errors.take() {
            return Err(errors);
        }
        Ok(())
    }

    fn validate_list(
        mut self,
        schema: &SchemaRef,
        size: usize
    ) -> Result<Self::ItemValidator, ValidationErrors> {
        // Short circuit if the list is larger than the allowed depth.
        // TODO(extensibility): Make this separately configurable property?
        if size > self.max_depth {
            self.emit_error(schema, ValidationFailure::ListTooLarge(self.max_depth))?;
            return Err(self.errors.take().unwrap());
        }

        // Check that list does not exceed length constraint
        if let Some(length) = schema.get_trait_as::<LengthTrait>() {
            if size < length.min() || size > length.max() {
                self.emit_error(schema, SmithyConstraints::Length(size, length.min(), length.max()))?;
            }
        }
        Ok(DefaultListValidator {
            root: self,
            unique: schema.contains_type::<UniqueItemsTrait>(),
            lookup: UniquenessTracker::new()
        })
    }

    fn validate_map(mut self, schema: &SchemaRef, size: usize) -> Result<Self::EntryValidator, ValidationErrors> {
        if size > self.max_depth {
            self.emit_error(schema, ValidationFailure::MapTooLarge(self.max_depth))?;
            return Err(self.errors.take().unwrap());
        }

        // Check length
        if let Some(length) = schema.get_trait_as::<LengthTrait>() {
            if size < length.min() || size > length.max() {
                self.emit_error(schema, SmithyConstraints::Length(size, length.min(), length.max()))?
            }
        }

        Ok(DefaultMapValidator { root: self })
    }

    fn validate_struct(mut self, schema: &SchemaRef) -> Result<Self::StructureValidator, ValidationErrors> {
        // TODO(completeness): check that schema is struct.
        // TODO(completeness): ADD DEPTH CHECKS
        Ok(DefaultStructValidator { root: self })
    }

    fn validate_string(mut self, schema: &SchemaRef, value: &String) -> Result<(), ValidationErrors> {
        if *schema.shape_type() != ShapeType::String {
            self.emit_error(schema, ValidationFailure::InvalidType(schema.shape_type().clone(), ShapeType::String))?;
        }

        // TODO(extensibility): Move into a "ValidationRule"?
        // Check pattern
        if let Some(pattern) = schema.get_trait_as::<PatternTrait>() {
            match pattern.pattern().find(value) {
                Some(_) => Ok(()),
                _ => self.emit_error(schema, SmithyConstraints::Pattern(value.clone(), pattern.pattern().to_string()))
            }?;
        }

        // Check length
        if let Some(length) = schema.get_trait_as::<LengthTrait>() {
            if value.len() < length.min() || value.len() > length.max() {
                self.emit_error(schema, SmithyConstraints::Length(value.len(), length.min(), length.max()))?
            }
        }
        Ok(())
    }

    fn validate_integer(&mut self, _schema: &SchemaRef, _value: &i32) -> Result<(), ValidationErrors> {
        todo!()
    }
}

#[doc(hidden)]
pub struct DefaultListValidator<'a> {
    root: &'a mut DefaultValidator,
    unique: bool,
    lookup: UniquenessTracker
}

impl ListValidator for DefaultListValidator<'_> {
    fn validate_in_place<T>(&mut self, element_schema: &SchemaRef, value: &T) -> Result<(), ValidationErrors>
    where
        for<'a> &'a T: Validate,
        T: Hash
    {
        let _ = value.validate(element_schema, &mut *self.root)?;
        self.check_uniqueness(element_schema, &value)
    }

    fn validate_and_move<T: Validate>(&mut self, element_schema: &SchemaRef, value: T) -> Result<T::Value, ValidationErrors>
    where
        T::Value: Hash
    {
        let output= value.validate(element_schema, &mut *self.root)?;
        self.check_uniqueness(element_schema, &output)?;
        Ok(output)
    }

    fn check_uniqueness<T: Hash>(&mut self, element_schema: &SchemaRef, value: T) -> Result<(), ValidationErrors> {
        if self.unique && self.lookup.add(value) {
            self.root.emit_error(element_schema, SmithyConstraints::UniqueItems)?;
        }
        Ok(())
    }
}

/// Tracker for unique items using a hash lookup directly
struct UniquenessTracker {
    // A b-tree is used here as it should be faster for
    // search for a relatively small number of numeric
    // values than a hashmap
    lookup: BTreeMap<u64, ()>,
}
impl UniquenessTracker {
    fn new() -> Self {
        UniquenessTracker {
            lookup: BTreeMap::new()
        }
    }

    fn add<T: Hash>(&mut self, value: T) -> bool {
        let mut hasher = FxHasher::default();
        value.hash(&mut hasher);
        self.lookup.insert(hasher.finish(), ()).is_some()
    }
}

#[doc(hidden)]
pub struct DefaultMapValidator<'a> {
    root: &'a mut DefaultValidator,
}
impl MapValidator for DefaultMapValidator<'_> {
    fn validate_entry_in_place<K, V>(&mut self, key_schema: &SchemaRef, value_schema: &SchemaRef, key: &K, value: &V) -> Result<(), ValidationErrors>
    where
            for<'a> &'a K: Validate,
            for<'a> &'a V: Validate
    {
        let _key = key.validate(key_schema, &mut *self.root)?;
        let _value = value.validate(value_schema, &mut *self.root)?;
        Ok(())
    }

    fn validate_entry_and_move<K:Validate, V: Validate>(
        &mut self,
        key_schema: &SchemaRef,
        value_schema: &SchemaRef,
        key: K,
        value: V
    ) -> Result<(K::Value, V::Value), ValidationErrors> {
        let key = key.validate(key_schema, &mut *self.root)?;
        let value = value.validate(value_schema, &mut *self.root)?;
        Ok((key, value))
    }
}

#[doc(hidden)]
pub struct DefaultStructValidator<'a> {
    root: &'a mut DefaultValidator,
}
impl StructureValidator for DefaultStructValidator<'_> {
    fn validate_required<V: Validate>(&mut self, schema: &SchemaRef, value: Option<V>) -> Result<V::Value, ValidationErrors>
    where
        V::Value: ErrorCorrection
    {
        if let Some(v) = value {
            v.validate(schema, &mut *self.root)
        } else {
            self.root.emit_error(schema, SmithyConstraints::Required)?;
            Ok(V::Value::default(schema))
        }
    }

    fn validate_optional<V: Validate>(&mut self, schema: &SchemaRef, value: Option<V>) -> Result<Option<V::Value>, ValidationErrors> {
        if let Some(v) = value {
            let result = v.validate(schema, &mut *self.root)?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}


//////////////////////////////////////////////////////////////////////////////
// ERRORS
//////////////////////////////////////////////////////////////////////////////

/// Aggregated list of all validation errors encountered while building a shape.
///
/// When executing validation of a Builder, more than one field could be invalid.
/// All of these [`ValidationError`]'s are aggregated together into a list on this
/// aggregate error type.
#[derive(Error, Debug)]
pub struct ValidationErrors {
    errors: Vec<ValidationErrorWrapper>
}

// TODO: REMOVE AT SOME POINT
impl From<String> for ValidationErrors {
    fn from(_value: String) -> Self {
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
    pub fn add(&mut self, path: &SchemaRef, error: impl Into<Box<dyn ValidationError>>) {
        self.errors.push(ValidationErrorWrapper::new(path.clone(), error.into()));
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

/// Wrapper that groups a validation error with the schema location at which it occured.
#[derive(Error, Debug)]
pub struct ValidationErrorWrapper {
    path: SchemaRef,
    error: Box<dyn ValidationError>
}
impl ValidationErrorWrapper {
    pub fn new(path: SchemaRef, error: Box<dyn ValidationError>) -> Self {
        Self { path, error }
    }
}
impl Display for ValidationErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}:{:?}", self.path.id().name(), self.error)
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
    #[error("Expected member: {0}")]
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
    use std::collections::HashMap;
    use std::sync::LazyLock;
    use crate::traits;
    use crate::prelude::{INTEGER, STRING};
    use crate::schema::{Schema, ShapeId, StaticSchemaShape};
    use crate::serde::de::Deserializer;
    use crate::serde::deserializers::DeserializeWithSchema;
    use crate::serde::ShapeBuilder;
    use crate::serde::shapes::{StructOrBuilder, VecOfStructsOrBuilders};
    use super::*;

    #[test]
    fn test_validation_errors_aggregate() {
        let mut errors = ValidationErrors::new();
        errors.add(&STRING, SmithyConstraints::Required);
        errors.add(&STRING, SmithyConstraints::Length(1,2,3));
        errors.add(&STRING, SmithyConstraints::Required);
        assert_eq!(errors.errors.len(), 3);
        assert_eq!(&errors.errors[0].error.to_string(), "Field is Required.");
        assert_eq!(&errors.errors[2].error.to_string(), "Field is Required.");
    }

    static LIST_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(ShapeId::from("com.example#List"), traits![LengthTrait::builder().max(3).build(), UniqueItemsTrait])
            .put_member("member", &STRING, traits![LengthTrait::builder().max(4).build()])
            .build()
    });
    static LIST_OF_NESTED_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(ShapeId::from("com.example#ListOfNested"), traits![LengthTrait::builder().max(3).build(), UniqueItemsTrait])
            .put_member("member", &NESTED_SCHEMA, traits![])
            .build()
    });

    static MAP_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::map_builder(ShapeId::from("com.example#Map"), traits![LengthTrait::builder().max(2).build(), UniqueItemsTrait])
            .put_member("key", &STRING, traits![PatternTrait::new("^[a-zA-Z]*$")])
            .put_member("value", &STRING, traits![LengthTrait::builder().max(4).build()])
            .build()
    });

    static VALIDATION_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("test#ValidationStruct"), Vec::new())
            .put_member("field_a", &STRING, traits![PatternTrait::new("^[a-zA-Z]*$")])
            .put_member("field_b", &INTEGER, traits![])
            .put_member("field_list", &LIST_SCHEMA, traits![])
            .put_member("field_map", &MAP_SCHEMA, traits![])
            .put_member("field_nested", &NESTED_SCHEMA, traits![])
            .put_member("field_nested_list", &LIST_OF_NESTED_SCHEMA, traits![])
            .build()
    });
    static FIELD_A: LazyLock<&SchemaRef> = LazyLock::new(|| VALIDATION_SCHEMA.expect_member("field_a"));
    static FIELD_B: LazyLock<&SchemaRef> = LazyLock::new(|| VALIDATION_SCHEMA.expect_member("field_b"));
    static FIELD_LIST: LazyLock<&SchemaRef> = LazyLock::new(|| VALIDATION_SCHEMA.expect_member("field_list"));
    static FIELD_MAP: LazyLock<&SchemaRef> = LazyLock::new(|| VALIDATION_SCHEMA.expect_member("field_map"));
    static FIELD_NESTED: LazyLock<&SchemaRef> = LazyLock::new(|| VALIDATION_SCHEMA.expect_member("field_nested"));
    static FIELD_NESTED_LIST: LazyLock<&SchemaRef> = LazyLock::new(|| VALIDATION_SCHEMA.expect_member("field_nested_list"));
    static NESTED_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("test#ValidationStruct"), Vec::new())
            .put_member("field_c", &STRING, traits![PatternTrait::new("^[a-z]*$")])
            .build()
    });
    static FIELD_C: LazyLock<&SchemaRef> = LazyLock::new(|| { NESTED_SCHEMA.expect_member("field_c") });

    pub struct SimpleStruct {
        field_a: String,
        field_b: Option<i32>,
        field_list: Option<Vec<String>>,
        field_map: Option<IndexMap<String, String>>,
        field_nested: Option<NestedStruct>,
        field_nested_list: Option<Vec<NestedStruct>>
    }
    impl StaticSchemaShape for SimpleStruct {
        fn schema() -> &'static SchemaRef {
            &VALIDATION_SCHEMA
        }
    }

    pub struct SimpleStructBuilder {
        field_a: Option<String>,
        field_b: Option<i32>,
        field_list: Option<Vec<String>>,
        field_map: Option<IndexMap<String, String>>,
        field_nested: Option<StructOrBuilder<NestedStruct, NestedStructBuilder>>,
        field_nested_list: Option<VecOfStructsOrBuilders<NestedStruct, NestedStructBuilder>>
    }
    impl SimpleStructBuilder {
        pub fn field_a(mut self, value: String) -> Self {
            self.field_a = Some(value);
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

        pub fn field_nested(mut self, value: NestedStruct) -> Self {
            self.field_nested = Some(StructOrBuilder::Struct(value));
            self
        }

        pub fn field_nested_builder(mut self, value: NestedStructBuilder) -> Self {
            self.field_nested = Some(StructOrBuilder::Builder(value));
            self
        }

        pub fn field_nested_list(mut self, values: Vec<NestedStruct>) -> Self {
            self.field_nested_list = Some(VecOfStructsOrBuilders::Structs(values));
            self
        }

        pub fn field_nested_list_builder(mut self, values: Vec<NestedStructBuilder>) -> Self {
            self.field_nested_list = Some(VecOfStructsOrBuilders::Builders(values));
            self
        }
    }
    impl <'de> ShapeBuilder<'de, SimpleStruct> for SimpleStructBuilder {
        fn new() -> Self {
            Self {
                field_a: None,
                field_b: None,
                field_list: None,
                field_map: None,
                field_nested: None,
                field_nested_list: None
            }
        }
    }
    impl Validate for SimpleStructBuilder {
        type Value = SimpleStruct;

        fn validate<V: Validator>(self, schema: &SchemaRef, mut validator: V) -> Result<Self::Value, ValidationErrors> {
            let mut struct_validator = validator.validate_struct(schema)?;
            let result = SimpleStruct {
                field_a: struct_validator.validate_required(&FIELD_A, self.field_a)?,
                field_b: struct_validator.validate_optional(&FIELD_B, self.field_b)?,
                field_list: struct_validator.validate_optional(&FIELD_LIST, self.field_list)?,
                field_map: struct_validator.validate_optional(&FIELD_MAP, self.field_map)?,
                field_nested: struct_validator.validate_optional(&FIELD_NESTED, self.field_nested)?,
                field_nested_list: struct_validator.validate_optional(&FIELD_NESTED_LIST, self.field_nested_list)?,
            };
            Ok(result)
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

    #[derive(Hash)]
    pub struct NestedStruct {
        field_c: String,
    }
    impl StaticSchemaShape for NestedStruct {
        fn schema() -> &'static SchemaRef {
            &NESTED_SCHEMA
        }
    }

    pub struct NestedStructBuilder {
        field_c: Option<String>,
    }
    impl NestedStructBuilder {
        pub fn field_c(mut self, value: String) -> Self {
            self.field_c = Some(value);
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

    impl Validate for NestedStructBuilder {
        type Value = NestedStruct;

        fn validate<V: Validator>(self, schema: &SchemaRef, validator: V) -> Result<Self::Value, ValidationErrors> {
            let mut struct_validator = validator.validate_struct(schema)?;
            let result = NestedStruct {
                field_c: struct_validator.validate_required(&FIELD_C, self.field_c)?,
            };
            Ok(result)
        }
    }

    impl <'de> ShapeBuilder<'de, NestedStruct> for NestedStructBuilder {
        fn new() -> Self {
            Self {
                field_c: None,
            }
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
    fn required_field_is_validated() {
        let builder = SimpleStructBuilder::new();
        let Err(err) = builder.build() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);
        let error_wrapper = err.errors.iter().next().unwrap();
        assert_eq!(error_wrapper.path, **FIELD_A);
        assert_eq!(error_wrapper.error.to_string(), "Field is Required.".to_string());
    }

    #[test]
    fn basic_string_validations_are_performed() {
        let builder = SimpleStructBuilder::new();
        let inner_vec = vec!["too long of a string".to_string()];
        let Some(err) = builder.field_list(inner_vec).field_a("field-a".to_string()).build().err() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 2);
        let error_pattern = err.errors.get(0).unwrap();
        let error_length = err.errors.get(1).unwrap();
        assert_eq!(error_pattern.path, **FIELD_A);
        assert_eq!(error_pattern.error.to_string(), "Value `field-a` did not conform to expected pattern `^[a-zA-Z]*$`".to_string());

        assert_eq!(error_length.path, *LIST_SCHEMA.expect_member("member"));
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

        assert_eq!(error_required.path, **FIELD_A);
        assert_eq!(error_required.error.to_string(), "Field is Required.".to_string());

        assert_eq!(error_length.path, *LIST_SCHEMA.expect_member("member"));
        assert_eq!(error_length.error.to_string(), "Size: 20 does not conform to @length constraint. Expected between 0 and 4.".to_string());
    }

    #[test]
    fn list_constraints_checked() {
        let builder = SimpleStructBuilder::new();
        let inner_vec = vec!["a".to_string(), "b".to_string(), "c".to_string(), "a".to_string(), "d".to_string()];
        let Some(err) = builder.field_list(inner_vec).field_a("fieldA".to_string()).build().err() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 2);
        let error_length = err.errors.get(0).unwrap();
        let error_unique = err.errors.get(1).unwrap();

        assert_eq!(error_length.path, **FIELD_LIST);
        assert_eq!(error_length.error.to_string(), "Size: 5 does not conform to @length constraint. Expected between 0 and 3.".to_string());

        assert_eq!(error_unique.path, *LIST_SCHEMA.expect_member("member"));
        assert_eq!(error_unique.error.to_string(), "Items in collection should be unique.".to_string());
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
        let error_key = err.errors.get(1).unwrap();
        let error_value = err.errors.get(2).unwrap();

        assert_eq!(error_length.path, **FIELD_MAP);
        assert_eq!(error_length.error.to_string(), "Size: 3 does not conform to @length constraint. Expected between 0 and 2.".to_string());
        assert_eq!(error_key.path, *MAP_SCHEMA.expect_member("key"));
        assert_eq!(error_key.error.to_string(), "Value `bad-key` did not conform to expected pattern `^[a-zA-Z]*$`".to_string());
        assert_eq!(error_value.path, *MAP_SCHEMA.expect_member("value"));
        assert_eq!(error_value.error.to_string(), "Size: 18 does not conform to @length constraint. Expected between 0 and 4.".to_string());
    }

    #[test]
    fn nested_struct_fields_build_if_no_errors() {
        let builder_nested = NestedStructBuilder::new().field_c("field".to_string());
        let builder = SimpleStructBuilder::new();
        let value = builder.field_a("fieldA".to_string()).field_nested_builder(builder_nested).build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_fields_build_with_pre_built_shape() {
        let builder_nested = NestedStructBuilder::new().field_c("field".to_string()).build().expect("Failed to build NestedStruct");
        let builder = SimpleStructBuilder::new();
        let value = builder.field_a("fieldA".to_string()).field_nested(builder_nested).build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_fields_checked() {
        let builder_nested = NestedStructBuilder::new().field_c("dataWithCaps".to_string());
        let builder = SimpleStructBuilder::new();
        let Some(err) = builder.field_a("fieldA".to_string()).field_nested_builder(builder_nested).build().err() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);
        let error_pattern = err.errors.get(0).unwrap();
        assert_eq!(error_pattern.path, **FIELD_C);
        assert_eq!(error_pattern.error.to_string(), "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string());
    }

    #[test]
    fn nested_struct_list_build_if_no_errors() {
        let nested_list = vec![NestedStructBuilder::new().field_c("data".to_string())];
        let builder = SimpleStructBuilder::new();
        builder.field_a("fieldA".to_string()).field_nested_list_builder(nested_list).build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_list_fields_build_with_pre_built_shapes() {
        let nested_list = vec![
            NestedStructBuilder::new().field_c("a".to_string()).build().expect("Failed to build NestedStruct"),
            NestedStructBuilder::new().field_c("b".to_string()).build().expect("Failed to build NestedStruct")
        ];
        let builder = SimpleStructBuilder::new();
        let value = builder.field_a("fieldA".to_string())
            .field_nested_list(nested_list).build()
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
        let builder = SimpleStructBuilder::new();
        let Some(err) = builder.field_a("fieldA".to_string()).field_nested_list_builder(nested_list).build().err() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 3);

        let error_length = err.errors.get(0).unwrap();
        let error_pattern = err.errors.get(1).unwrap();
        let error_unique = err.errors.get(2).unwrap();

        assert_eq!(error_length.path, **FIELD_NESTED_LIST);
        assert_eq!(error_length.error.to_string(), "Size: 4 does not conform to @length constraint. Expected between 0 and 3.".to_string());

        assert_eq!(error_pattern.path, **FIELD_C);
        assert_eq!(error_pattern.error.to_string(), "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string());

        assert_eq!(error_unique.path, *LIST_OF_NESTED_SCHEMA.expect_member("member"));
        assert_eq!(error_unique.error.to_string(), "Items in collection should be unique.".to_string());
    }
}