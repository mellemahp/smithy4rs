use std::error::Error;
use std::fmt::Display;
use bigdecimal::{BigDecimal, Zero};
use indexmap::IndexMap;
use num_bigint::BigInt;
use thiserror::Error;
use crate::schema::{Document, DocumentValue, SchemaRef, ShapeType};
use crate::Instant;
use crate::prelude::{LengthTrait, PatternTrait};

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
pub fn validate_required<S: Validator, V: Validate>(
    mut validator: S,
    schema: &SchemaRef,
    value: Option<V>,
) -> Result<V::Value, ValidationErrors> {
    if let Some(v) = value {
        v.validate(schema, validator)
    } else {
        validator.emit_error(schema, SmithyConstraints::Required)?;
        Ok(V::Value::default(schema))
    }
}

/// Validate an optional Field
///
/// Validation errors are _only_ raised from this method if the validator hits
/// a max depth or maximum number of errors. This allows the validator to short-circuit
/// in those cases. Otherwise, users should use the [`results()?`] method to raise any
/// validation errors _after_ all fields have been validated.
pub fn validate_optional<S: Validator, V: Validate>(
    validator: S,
    schema: &SchemaRef,
    value: Option<V>,
) -> Result<Option<V::Value>, ValidationErrors> {
    if let Some(v) = value {
        let result = v.validate(schema, validator)?;
        Ok(Some(result))
    } else {
        Ok(None)
    }
}

// TODO: Update docs to be a bit less clunky
/// NOTE: Smithy error correction is not implemented directly.
pub trait Validator {
    /// Validates list items
    // type ItemValidator: ListValidator;
    //
    // /// Validates map entries
    // type EntryValidator: MapValidator;

    /// Validates structure builders
    // type BuilderValidator: Validator;

    // fn validate_list(
    //     self,
    //     schema: &SchemaRef,
    //     size: usize,
    // ) -> Result<Self::ItemValidator, ValidationErrors>;
    //
    // fn validate_map(
    //     self,
    //     schema: &SchemaRef,
    //     size: usize,
    // ) -> Result<Self::EntryValidator, ValidationErrors>;
    //
    // fn validate_struct<V: Validate>(
    //     &mut self,
    //     schema: &SchemaRef,
    // ) -> Result<Self::BuilderValidator, ValidationErrors>;

    // /// Validate a `boolean` in place
    // fn validate_boolean(&mut self, schema: &SchemaRef, _bool: &bool) -> Result<(), ValidationErrors> {
    //     //check_type!(self, schema, ShapeType::Boolean);
    //     Ok(())
    // }

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


// /// List Serializer that can be called in a loop to serialize list values
// pub trait ListValidator {
//     type Value;
//
//     /// Validate a sequence element in place.
//     fn validate_in_place<T>(
//         &mut self,
//         element_schema: &SchemaRef,
//         value: &T,
//     ) -> Result<(), ValidationErrors>
//     where
//         T: ?Sized + Validate;
//
//     /// Validates and moves an element
//     /// NOTE: This is primarily intended to support builder conversions
//     fn validate_and_move<T>(
//         &mut self,
//         element_schema: &SchemaRef,
//         value: T
//     ) -> Result<T::Value, ValidationErrors>
//     where
//         T: ?Sized + Validate;
// }

// trait MapValidator {
//     /// Validate a single map entry
//     fn validate_entry_in_place<V>(
//         &mut self,
//         key_schema: &SchemaRef,
//         value_schema: &SchemaRef,
//         key: &String,
//         value: &V,
//     ) -> Result<(), ValidationErrors>
//     where
//         V: ?Sized + SerializeWithSchema;
//
//     /// Validates an entry and returns a new owned value for it.
//     fn validate_entry_and_move<V>(
//         &mut self,
//         key_schema: &SchemaRef,
//         value_schema: &SchemaRef,
//         key: String,
//         value: V,
//     ) -> Result<(String, V::Value), ValidationErrors>
//     where
//         V: ?Sized + Validate;
// }


/// Indicates that a type can be validated by a [`Validator`] implementation.
///
/// All validate-able types must be able to provide a sane
/// default value through error correction.
pub trait Validate {
    /// Output type
    type Value: ErrorCorrection;

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


/// Default validator implementation
///
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
impl Validator for &mut DefaultValidator {
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

    fn validate_string(mut self, schema: &SchemaRef, value: &String) -> Result<(), ValidationErrors> {
        if *schema.shape_type() != ShapeType::String {
            self.emit_error(schema, ValidationFailure::InvalidType(schema.shape_type().clone(), ShapeType::String))?;
        }

        // TODO: Move into a "ValidationRule"
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
    fn from(value: String) -> Self {
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

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
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
        write!(f, "{:#?}:{:#?}", self.path.id(), self.error)
    }
}

/// Marker trait for validation errors.
pub trait ValidationError: Error {}

// Implement conversion for any Error enums implementing Validation error
impl <T: ValidationError + 'static> From<T> for Box<dyn ValidationError> {
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
    #[error("Unsupported validation operation.")]
    Unsupported,
}
impl ValidationError for ValidationFailure {}

#[derive(Error, Debug)]
pub enum SmithyConstraints {
    /// [@required](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-required-trait)
    #[error("Field is Required.")]
    Required,
    /// [@length](https://smithy.io/2.0/spec/constraint-traits.html#length-trait)
    #[error("Size: {0} does not conform to @length constraint. Expected between {1} and {2}.")]
    Length(usize, usize, usize),
    /// [@pattern](https://smithy.io/2.0/spec/constraint-traits.html#pattern-trait)
    #[error("Value {0} did not conform to expected pattern {1}")]
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
    use crate::prelude::{INTEGER, STRING};
    use crate::schema::{Schema, ShapeId};
    use crate::serde::de::Deserializer;
    use crate::serde::deserializers::DeserializeWithSchema;
    use crate::serde::ShapeBuilder;
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

    static VALIDATION_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("test#ValidationStruct"), Vec::new())
            .put_member("field_a", &STRING, Vec::new())
            .put_member("field_b", &INTEGER, Vec::new())
            .build()
    });
    static FIELD_A: LazyLock<&SchemaRef> = LazyLock::new(|| VALIDATION_SCHEMA.expect_member("field_a"));
    static FIELD_B: LazyLock<&SchemaRef> = LazyLock::new(|| VALIDATION_SCHEMA.expect_member("field_b"));

    pub struct SimpleStruct {
        field_a: String,
        field_b: Option<i32>
    }

    pub struct SimpleStructBuilder {
        field_a: Option<String>,
        field_b: Option<i32>,
    }
    impl SimpleStructBuilder {
        pub fn new() -> Self {
            Self {
                field_a: None,
                field_b: None,
            }
        }

        pub fn field_a(mut self, value: String) -> Self {
            self.field_a = Some(value);
            self
        }

        pub fn field_b(mut self, value: i32) -> Self {
            self.field_b = Some(value);
            self
        }
    }

    impl <'de> ShapeBuilder<'de, SimpleStruct> for SimpleStructBuilder {
        fn new() -> Self {
            Self {
                field_a: None,
                field_b: None,
            }
        }

        fn build_with_validator<V>(self, mut validator: V) -> Result<SimpleStruct, ValidationErrors>
            where for<'a> &'a mut V: Validator
        {
            let result = SimpleStruct {
                field_a: validate_required(&mut validator, &FIELD_A, self.field_a)?,
                field_b: validate_optional(&mut validator, &FIELD_B, self.field_b)?
            };
            validator.results()?;
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

    #[test]
    fn required_string_field_is_validated() {
        let builder = SimpleStructBuilder::new();
        // EXPECTED TO IMPLODE!
        let output = builder.build().unwrap();
    }
}