//! Validation of shapes against a schema
//!
//! Validation compares a shape against a set of constraints defined in the Smithy Model.
//!
//! ## Some basic requirements
//! Validation in `smithy4rs` has a few fundamental requirements:
//! 1. **Validation MUST occur AFTER deserialization** -- Validation should occur only
//!    once all data have been unmarshalled into a Builder. Multiple different
//!    sources could be deserialized into a single shape definition, so ALL deserialization
//!    must be completed before validation can occur. This also avoids validating multiple times
//!    if multiple sources are used to construct a built shape.
//! 2. **Validation must aggregate all errors from all nested types** -- Users should get a single
//!    error for _ALL_ of their validation errors so they can fix them all at once.
//!    **NOTE**: Validation errors are the ONLY errors that we will aggregate in this way.
//! 3. **Validation must have a depth limit** -- If we allowed validation to walk arbitrarily deep into
//!    a shape tree then it would be relatively easy to implement a DOS attack against any Document, Map,
//!    List or recursive types. To prevent such attacks, [`Validator`] Implementations MUST have limits
//!    on the maximum walk depth and maximum number of errors.
//!
//! ## Build and Validation
//!
//! ### Default Validation
//! By default, users should _not_ be able to manually construct shapes that violate the basic Smithy constraints.
//! In `smithy4rs`,  [`ShapeBuilder`](crate::serde::builders::ShapeBuilder) implementations are validated
//! with the [`DefaultValidator`] on [`ShapeBuilder::build()`](crate::serde::builders::ShapeBuilder::build).
//!
//! This [`DefaultValidator`] (and therefore the `build()` method) will check the following built-in Smithy constraints:
//! - [`@length`](<https://smithy.io/2.0/spec/constraint-traits.html#length-trait>)
//! - [`@range`](<https://smithy.io/2.0/spec/constraint-traits.html#range-trait>)
//! - [`@pattern`](<https://smithy.io/2.0/spec/constraint-traits.html#pattern-trait>)
//! - [`@uniqueItems`](<https://smithy.io/2.0/spec/constraint-traits.html#uniqueitems-trait>)
//! - [`@required`](<https://smithy.io/2.0/spec/type-refinement-traits.html#required-trait>)
//!
//! In addition to checking these constraint traits, the default validator also checks that the type in
//! the schema is compatible with the data type present in the shape.
//!
//! For more info on built-in Smithy constraints see: [Smithy Documentation](<https://smithy.io/2.0/spec/constraint-traits.html>)
//!
//! ### Custom Constraints
//! Users may have a different definition of "Invalid". Custom definitions of validity should be encoded into
//! a [`Validator`] implementation.
//!
//! For example, if you don't care if a response from a server missed a `@required` value,
//! then you could create a `ClientValidator` that ignores missing `required` values.
//!
//! To use validate a builder using a custom validation implementation, use
//! the [`ShapeBuilder::build_with_validator`](crate::serde::builders::ShapeBuilder::build_with_validator)
//! method with your custom implementation.
//!
//! ## Validating Protocol-specific constraints
//! Some protocols may have additional constraints that they need to check in addition to the basic
//! Smithy constraints.
//!
//! To support protocol-specific validation, Protocol implementations provide a [`Validator`]
//! implementation (defaulting to the [`DefaultValidator`]) that is used to validate all shapes
//! deserialized with that protocol.
//!
use std::{
    collections::BTreeSet,
    convert::Into,
    error::Error,
    fmt::Display,
    hash::{Hash, Hasher},
};

use bigdecimal::{FromPrimitive, ToPrimitive};
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;
use rustc_hash::FxHasher;
use stack_array::{Array, ArrayBuf};
use thiserror::Error;

use crate::{
    BigDecimal, FxIndexSet, Instant,
    schema::{
        Document, Schema, ShapeType,
        prelude::{LengthTrait, PatternTrait, RangeTrait, UniqueItemsTrait},
    },
    serde::{
        se::{SerializeWithSchema, Serializer},
        serializers,
        serializers::{ListSerializer, MapSerializer, StructSerializer},
        utils::KeySerializer,
    },
};
// ============================================================================
// Validator Trait
// ============================================================================

/// Validator that ensures shapes conform to constraint traits.
///
/// Under the hood a validator is [`Serializer`] that walks a serializable shape,
/// comparing each shape/member against the provided schema.
///
/// <div class="note">
/// **NOTE**: Implementations should only return `Err` results from the serialization
/// methods if they wish to short-circuit validation (for example if a maximum number of
/// errors is reached). Otherwise, they should aggregate all errors internally and return
/// them as in the aggregate [`ValidationErrors`] result of the [`Validator::validate`] method.
/// </div>
///
/// ## Default Implementation
/// Generated shape builders will use the [`DefaultValidator`] by default when calling the
/// `build()` method.
///
/// The [`DefaultValidator`] supports all built-in Smithy constraints (i.e. `@range`, `@length`, etc.).
///
/// ## Using a custom validator
/// To use a custom [`Validator`] when building a shape, use the
/// [`ShapeBuilder::build_with_validator`](crate::serde::builders::ShapeBuilder::build_with_validator) method.
///
/// For example:
/// ```rust,ignore
/// let my_builder = MyShape::builder().a("string".to_string());
/// let built = my_builder::build_with_validator(MyValidator)
///     .expect("shape invalid");
/// ```
///
pub trait Validator: Serializer<Ok = (), Error = ValidationErrors> {
    /// Validates a type against a schema.
    ///
    /// # Errors
    /// Aggregation of all the validation issues encountered.
    fn validate<V: SerializeWithSchema>(
        self,
        schema: &Schema,
        value: &V,
    ) -> Result<(), ValidationErrors>;
}

// ============================================================================
// Default Implementation
// ============================================================================
/// Validator that ensures shapes conform to built-in Smithy constraints
///
/// This validator will check that shapes conform to the provided schema types
/// and any built-in Smithy constraint traits found in that schema, including:
/// - [`@length`](<https://smithy.io/2.0/spec/constraint-traits.html#length-trait>)
/// - [`@range`](<https://smithy.io/2.0/spec/constraint-traits.html#range-trait>)
/// - [`@pattern`](<https://smithy.io/2.0/spec/constraint-traits.html#pattern-trait>)
/// - [`@uniqueItems`](<https://smithy.io/2.0/spec/constraint-traits.html#uniqueitems-trait>)
/// - [`@required`](<https://smithy.io/2.0/spec/type-refinement-traits.html#required-trait>)
///
/// For more info on built-in Smithy constraints see: [Smithy Documentation](<https://smithy.io/2.0/spec/constraint-traits.html>)
///
/// This is the default [`Validator`] implementation used in built shapes (i.e. when calling the default
/// [`ShapeBuilder::build()`](crate::serde::builders::ShapeBuilder::build) implementation).
/// It can also be used standalone on any serializable shapes.
/// ```rust, ignore
/// let result = DefaultValidator::new().validate(MySerializableShape);
/// ```
///
/// To customize the `Validator` used when building a shape, pass a custom [`Validator`] implementation
/// into the [`ShapeBuilder::build_with_validator()`](crate::serde::builders::ShapeBuilder::build_with_validator)
/// method on the builder.
///
/// * `D` - Maximum validation depth (Default: 10).
/// * `E` - Maximum number of errors the validator can track (Default: 20).
pub struct DefaultValidator<const D: usize = 10, const E: usize = 20> {
    errors: Option<ValidationErrors>,
    path_stack: ArrayBuf<PathElement, D>,
}

impl<const D: usize, const ERR: usize> DefaultValidator<D, ERR> {
    /// Create a new [`Validator`] instance.
    #[must_use]
    pub const fn new() -> Self {
        DefaultValidator {
            errors: None,
            path_stack: ArrayBuf::new(),
        }
    }

    /// Emit an error for accumulation
    ///
    /// This method _only_ returns an error response when the maximum number
    /// of errors is hit. At that point it returns a list of all previously encountered
    /// validation errors plus an extra appended error to indicate the error limit was reached.
    fn emit_error<E: ValidationError + 'static>(&mut self, err: E) -> Result<(), ValidationErrors> {
        let errors = self.errors.get_or_insert(ValidationErrors::new());

        // Short circuit if the maximum number of
        if errors.len() >= ERR {
            errors.add(&self.path_stack, ValidationFailure::MaxErrorsReached(ERR));
            return Err(self.errors.take().unwrap());
        }
        errors.add(&self.path_stack, err);
        Ok(())
    }

    /// Return all collected validation errors
    ///
    /// This returns a `Result` type to allow `?` raising.
    ///
    /// # Errors
    /// All validation errors found.
    pub fn results(&mut self) -> Result<(), ValidationErrors> {
        if let Some(errors) = self.errors.take() {
            return Err(errors);
        }
        Ok(())
    }

    /// Add a path segment to the path stack
    fn push_path(&mut self, path: impl Into<PathElement>) -> Result<(), ValidationErrors> {
        if self.path_stack.len() + 1 > D {
            return self.short_circuit(ValidationFailure::MaximumDepthExceeded(D));
        }
        self.path_stack.push(path.into());
        Ok(())
    }

    /// Short circuit validation, returning this error and any others collected up to this point
    fn short_circuit<E: ValidationError + 'static>(
        &mut self,
        error: E,
    ) -> Result<(), ValidationErrors> {
        self.emit_error(error)?;
        // SAFETY: Safe to unwrap as errors will always be set to `SOME` above
        Err(self.errors.take().unwrap())
    }

    /// Pop a path segment from the path stack
    ///
    /// Returns an error if the call would pop from an empty stack.
    fn pop_path(&mut self) -> Result<(), ValidationErrors> {
        if self.path_stack.pop().is_none() {
            return self.short_circuit(ValidationFailure::PopFromEmptyValidator);
        }
        Ok(())
    }
}

impl Validator for &mut DefaultValidator {
    #[inline]
    fn validate<V: SerializeWithSchema>(
        self,
        schema: &Schema,
        value: &V,
    ) -> Result<(), ValidationErrors> {
        value.serialize_with_schema(schema, &mut *self)?;
        self.results()
    }
}
impl<const D: usize, const ERR: usize> Default for DefaultValidator<D, ERR> {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! shape_type {
    ($self:ident, $schema:ident, $ty:expr) => {
        if !$schema.shape_type().eq(&$ty) {
            $self.emit_error(SmithyConstraints::ShapeType(*$schema.shape_type()))?;
        }
    };
}
macro_rules! length {
    ($self:ident, $schema:ident, $len:ident) => {
        if let Some(length) = $schema.get_trait_as::<LengthTrait>() {
            if $len < length.min() || $len > length.max() {
                $self.emit_error(SmithyConstraints::Length($len, length.min(), length.max()))?;
            }
        }
    };
}

// TODO(warnings): Should this emit a warning or error on unrepresentable value??
macro_rules! range {
    ($self:ident, $schema:ident, $value:ident, $converter:ident) => {
        if let Some(range) = $schema.get_trait_as::<RangeTrait>()
            && let (Some(min), Some(max)) = (range.min().$converter(), range.max().$converter())
            && ($value < min || $value > max)
        {
            $self.emit_error(SmithyConstraints::Range(
                BigDecimal::from($value),
                range.min().clone(),
                range.max().clone(),
            ))?;
        };
    };
}

impl<'a> Serializer for &'a mut DefaultValidator {
    type Error = ValidationErrors;
    type Ok = ();
    type SerializeList = DefaultListValidator<'a>;
    type SerializeMap = DefaultMapValidator<'a>;
    type SerializeStruct = DefaultStructValidator<'a>;

    fn write_struct(
        self,
        schema: &Schema,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        if !schema.shape_type().eq(&ShapeType::Structure)
            && !schema.shape_type().eq(&ShapeType::Union)
        {
            self.emit_error(SmithyConstraints::ShapeType(*schema.shape_type()))?;
        }
        Ok(DefaultStructValidator { root: self })
    }

    fn write_map(self, schema: &Schema, len: usize) -> Result<Self::SerializeMap, Self::Error> {
        shape_type!(self, schema, ShapeType::Map);
        length!(self, schema, len);
        Ok(DefaultMapValidator { root: self })
    }

    fn write_list(self, schema: &Schema, len: usize) -> Result<Self::SerializeList, Self::Error> {
        shape_type!(self, schema, ShapeType::List);
        length!(self, schema, len);
        Ok(DefaultListValidator {
            root: self,
            unique: schema.contains_type::<UniqueItemsTrait>(),
            lookup: UniquenessTracker::new(),
            index: 0,
        })
    }

    fn write_boolean(self, schema: &Schema, _value: bool) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Boolean);
        Ok(())
    }

    fn write_byte(self, schema: &Schema, value: i8) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Byte);
        range!(self, schema, value, to_i8);
        Ok(())
    }

    fn write_short(self, schema: &Schema, value: i16) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Short);
        range!(self, schema, value, to_i16);
        Ok(())
    }

    fn write_integer(self, schema: &Schema, value: i32) -> Result<Self::Ok, Self::Error> {
        // IntEnums are treated as Integers
        if schema.shape_type().eq(&ShapeType::Integer) {
            range!(self, schema, value, to_i32);
        } else if schema.shape_type().eq(&ShapeType::IntEnum) {
            let Some(enum_schema) = schema.as_int_enum() else {
                unreachable!("Only intEnum schemas can be constructed with an enum type");
            };
            if !enum_schema.values().contains(&value) {
                self.emit_error(SmithyConstraints::IntEnumValue(
                    value,
                    enum_schema.values().clone(),
                ))?;
            }
        } else {
            self.emit_error(SmithyConstraints::ShapeType(*schema.shape_type()))?;
        }
        Ok(())
    }

    fn write_long(self, schema: &Schema, value: i64) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Long);
        range!(self, schema, value, to_i64);
        Ok(())
    }

    fn write_float(self, schema: &Schema, value: f32) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Float);
        if let Some(range) = schema.get_trait_as::<RangeTrait>()
            && let (Some(min), Some(max)) = (range.min().to_f32(), range.max().to_f32())
            && (value < min || value > max)
        {
            self.emit_error(SmithyConstraints::Range(
                BigDecimal::from_f32(value).unwrap_or_default(),
                range.min().clone(),
                range.max().clone(),
            ))?;
        }
        Ok(())
    }

    fn write_double(self, schema: &Schema, value: f64) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Double);
        if let Some(range) = schema.get_trait_as::<RangeTrait>()
            && let (Some(min), Some(max)) = (range.min().to_f64(), range.max().to_f64())
            && (value < min || value > max)
        {
            self.emit_error(SmithyConstraints::Range(
                BigDecimal::from_f64(value).unwrap_or_default(),
                range.min().clone(),
                range.max().clone(),
            ))?;
        }
        Ok(())
    }

    fn write_big_integer(self, schema: &Schema, value: &BigInt) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::BigInteger);
        if let Some(range) = schema.get_trait_as::<RangeTrait>() {
            let big_value = BigDecimal::from_bigint(value.clone(), 0);
            if &big_value < range.min() || &big_value > range.max() {
                self.emit_error(SmithyConstraints::Range(
                    big_value,
                    range.min().clone(),
                    range.max().clone(),
                ))?;
            }
        }
        Ok(())
    }

    fn write_big_decimal(
        self,
        schema: &Schema,
        value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::BigDecimal);
        if let Some(range) = schema.get_trait_as::<RangeTrait>()
            && (value < range.min() || value > range.max())
        {
            self.emit_error(SmithyConstraints::Range(
                value.clone(),
                range.min().clone(),
                range.max().clone(),
            ))?;
        }
        Ok(())
    }

    fn write_string(self, schema: &Schema, value: &str) -> Result<Self::Ok, Self::Error> {
        // Enums are treated as strings for the purpose of validation
        if schema.shape_type().eq(&ShapeType::String) {
            let len = value.len();
            length!(self, schema, len);

            // Check @pattern trait matches provided.
            if let Some(pattern) = schema.get_trait_as::<PatternTrait>()
                && pattern.pattern().find(value).is_none()
            {
                self.emit_error(SmithyConstraints::Pattern(
                    value.to_string(),
                    pattern.pattern().to_string(),
                ))?;
            }
        } else if schema.shape_type().eq(&ShapeType::Enum) {
            let Some(enum_schema) = schema.as_enum() else {
                unreachable!("Only enum schemas can be constructed with an enum type");
            };
            if !enum_schema.values().contains(value) {
                self.emit_error(SmithyConstraints::EnumValue(
                    value.to_owned(),
                    enum_schema.values().clone(),
                ))?;
            }
        } else {
            self.emit_error(SmithyConstraints::ShapeType(*schema.shape_type()))?;
        }
        Ok(())
    }

    fn write_blob(self, schema: &Schema, _value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Blob);
        Ok(())
    }

    fn write_timestamp(self, schema: &Schema, _value: &Instant) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Timestamp);
        Ok(())
    }

    fn write_document(
        self,
        schema: &Schema,
        _value: &Box<dyn Document>,
    ) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Document);
        Ok(())
    }

    fn write_null(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        /* Skip null value validation */
        // TODO(sparse lists): Decide how sparseness be handled in validator
        Ok(())
    }

    #[inline]
    fn write_missing(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        self.emit_error(SmithyConstraints::Required)
    }

    #[inline]
    fn skip(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        /* Do nothing on skip */
        Ok(())
    }
}

#[doc(hidden)]
pub struct DefaultListValidator<'a> {
    root: &'a mut DefaultValidator,
    unique: bool,
    lookup: UniquenessTracker,
    index: usize,
}

impl ListSerializer for DefaultListValidator<'_> {
    type Error = ValidationErrors;
    type Ok = ();

    fn serialize_element<T>(
        &mut self,
        element_schema: &Schema,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        self.root.push_path(PathElement::Index(self.index))?;
        if self.unique {
            match self.lookup.add(element_schema, value) {
                Ok(true) => self.root.emit_error(SmithyConstraints::UniqueItems),
                // Return early on this error. Something is wrong with the schema.
                Err(err) => return self.root.short_circuit(err),
                _ => Ok(()),
            }?;
        }
        value.serialize_with_schema(element_schema, &mut *self.root)?;
        self.root.pop_path()?;
        self.index += 1;
        Ok(())
    }

    #[inline]
    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        //self.root.pop_path()
        Ok(())
    }
}

// ============================================================================
// @Unique Support
// ============================================================================

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
            lookup: BTreeSet::new(),
        }
    }

    /// Add an item to the set.
    ///
    /// Returns true if the item was already in the set.
    fn add<T: SerializeWithSchema>(
        &mut self,
        schema: &Schema,
        value: &T,
    ) -> Result<bool, ValidationFailure> {
        let mut serializer = HashingSerializer::new();
        value.serialize_with_schema(schema, &mut serializer)?;
        Ok(!self.lookup.insert(serializer.result()))
    }
}

/// This type generates a unique hash for all items if possible.
///
/// Errors are raised if unhashable types (i.e. `f32` and `f64`) are
/// checked for uniqueness. Such a check is considered invalid in the
/// Smithy data model.
struct HashingSerializer {
    hasher: FxHasher,
}
impl HashingSerializer {
    /// Create a new [`HashingSerializer`]
    fn new() -> Self {
        HashingSerializer {
            hasher: FxHasher::default(),
        }
    }

    /// Utility function to compute `Hash` for types when possible.
    #[inline]
    fn hash<T: Hash>(&mut self, value: T) {
        value.hash(&mut self.hasher);
    }

    /// Get the final hash result
    fn result(self) -> u64 {
        self.hasher.finish()
    }
}
macro_rules! hash_impl {
    ($self:ident, $value:ident) => {
        $self.hash($value);
        return Ok(());
    };
}
impl<'a> Serializer for &'a mut HashingSerializer {
    type Error = ValidationFailure;
    type Ok = ();
    type SerializeList = InnerHasher<'a>;
    type SerializeMap = InnerHasher<'a>;
    type SerializeStruct = InnerHasher<'a>;

    #[inline]
    fn write_struct(
        self,
        _schema: &Schema,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(InnerHasher { root: self })
    }

    #[inline]
    fn write_map(self, _schema: &Schema, _len: usize) -> Result<Self::SerializeMap, Self::Error> {
        Ok(InnerHasher { root: self })
    }

    #[inline]
    fn write_list(self, _schema: &Schema, _len: usize) -> Result<Self::SerializeList, Self::Error> {
        Ok(InnerHasher { root: self })
    }

    #[inline]
    fn write_boolean(self, _schema: &Schema, value: bool) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_byte(self, _schema: &Schema, value: i8) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_short(self, _schema: &Schema, value: i16) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_integer(self, _schema: &Schema, value: i32) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_long(self, _schema: &Schema, value: i64) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[cold]
    fn write_float(self, _schema: &Schema, _value: f32) -> Result<Self::Ok, Self::Error> {
        Err(ValidationFailure::UniqueItemOnFloat)
    }

    #[cold]
    fn write_double(self, _schema: &Schema, _value: f64) -> Result<Self::Ok, Self::Error> {
        Err(ValidationFailure::UniqueItemOnFloat)
    }

    #[inline]
    fn write_big_integer(self, _schema: &Schema, value: &BigInt) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_big_decimal(
        self,
        _schema: &Schema,
        value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_string(self, _schema: &Schema, value: &str) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_blob(self, _schema: &Schema, value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_timestamp(self, _schema: &Schema, value: &Instant) -> Result<Self::Ok, Self::Error> {
        self.hash(value.epoch_nanoseconds().0);
        Ok(())
    }

    fn write_document(
        self,
        _schema: &Schema,
        _value: &Box<dyn Document>,
    ) -> Result<Self::Ok, Self::Error> {
        // TODO(document validation): How to hash document types?
        todo!()
    }

    #[inline]
    fn write_null(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        self.hash("null");
        Ok(())
    }

    #[inline]
    fn skip(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        /* Do not execute hash on skip */
        Ok(())
    }
}

// ============================================================================
// Hashing Support
// ============================================================================

struct InnerHasher<'a> {
    root: &'a mut HashingSerializer,
}
impl ListSerializer for InnerHasher<'_> {
    type Error = ValidationFailure;
    type Ok = ();

    #[inline]
    fn serialize_element<T>(
        &mut self,
        element_schema: &Schema,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        value.serialize_with_schema(element_schema, &mut *self.root)?;
        Ok(())
    }

    #[inline]
    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
impl MapSerializer for InnerHasher<'_> {
    type Error = ValidationFailure;
    type Ok = ();

    #[inline]
    fn serialize_entry<K, V>(
        &mut self,
        key_schema: &Schema,
        value_schema: &Schema,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: SerializeWithSchema,
        V: SerializeWithSchema,
    {
        key.serialize_with_schema(key_schema, &mut *self.root)?;
        value.serialize_with_schema(value_schema, &mut *self.root)?;
        Ok(())
    }

    #[inline]
    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
impl StructSerializer for InnerHasher<'_> {
    type Error = ValidationFailure;
    type Ok = ();

    fn serialize_member<T>(&mut self, member_schema: &Schema, value: &T) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        let Some(member_name) = member_schema.id().member() else {
            return Err(ValidationFailure::ExpectedMember(
                member_schema.id().name().into(),
            ));
        };
        self.serialize_member_named(member_name, member_schema, value)?;
        Ok(())
    }

    #[inline]
    fn serialize_member_named<T>(
        &mut self,
        member_name: &str,
        member_schema: &Schema,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        self.root.hash(member_name);
        value.serialize_with_schema(member_schema, &mut *self.root)?;
        Ok(())
    }

    #[inline]
    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[doc(hidden)]
pub struct DefaultMapValidator<'a> {
    root: &'a mut DefaultValidator,
}
impl MapSerializer for DefaultMapValidator<'_> {
    type Error = ValidationErrors;
    type Ok = ();

    fn serialize_entry<K, V>(
        &mut self,
        key_schema: &Schema,
        value_schema: &Schema,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: SerializeWithSchema,
        V: SerializeWithSchema,
    {
        match key.serialize_with_schema(key_schema, &mut KeySerializer::<ValidationFailure>::new())
        {
            Ok(val) => self.root.push_path(PathElement::Key(val))?,
            // Return early on this error. Something is wrong with the schema.
            Err(err) => return self.root.short_circuit(err),
        }
        key.serialize_with_schema(key_schema, &mut *self.root)?;
        value.serialize_with_schema(value_schema, &mut *self.root)?;
        self.root.pop_path()
    }

    #[inline]
    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[doc(hidden)]
pub struct DefaultStructValidator<'a> {
    root: &'a mut DefaultValidator,
}
impl StructSerializer for DefaultStructValidator<'_> {
    type Error = ValidationErrors;
    type Ok = ();

    #[inline]
    fn serialize_member<T>(&mut self, member_schema: &Schema, value: &T) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        self.root.push_path(member_schema)?;
        value.serialize_with_schema(member_schema, &mut *self.root)?;
        self.root.pop_path()
    }

    #[inline]
    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// ============================================================================
// Validation Errors
// ============================================================================

/// Aggregated list of all validation errors encountered while building a shape.
///
/// When executing validation of a Builder, more than one field could be invalid.
/// All of these [`ValidationErrorField`]'s are aggregated together into a list on this
/// aggregate error type.
#[derive(Error, Debug)]
pub struct ValidationErrors {
    errors: Vec<ValidationErrorField>,
}

impl ValidationErrors {
    /// Create a new [`ValidationErrors`] error.
    ///
    /// **NOTE**: This method instantiates the error type with an
    /// empty list of errors. Actual validation errors must be added
    /// using the [`ValidationErrors::extend`] or `ValidationErrors::add`
    /// methods.
    #[must_use]
    pub const fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Extends an aggregate validation error with the contents of
    /// another aggregate validation error.
    pub fn extend(&mut self, other: ValidationErrors) {
        self.errors.extend(other.errors);
    }

    /// Add a new validation error to the list of errors.
    pub(super) fn add(&mut self, path: &[PathElement], error: impl Into<Box<dyn ValidationError>>) {
        self.errors.push(ValidationErrorField::new(path, error));
    }

    /// Get the number of child-errors contained in this error.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Returns true if this error has no children
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}
impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}
impl serializers::Error for ValidationErrors {
    fn custom<T: Display>(msg: T) -> Self {
        let err = ValidationErrorField::new(&[], ValidationFailure::Custom(msg.to_string()));
        Self { errors: vec![err] }
    }
}

impl Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:#?}", self.errors)
    }
}

/// Describes one specific validation failure and it's location.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ValidationErrorField {
    paths: Vec<PathElement>,
    error: Box<dyn ValidationError>,
}
impl ValidationErrorField {
    /// Create a new validation error field from a validation error and a path
    pub fn new(paths: &[PathElement], error: impl Into<Box<dyn ValidationError>>) -> Self {
        Self {
            paths: Vec::from(paths),
            error: error.into(),
        }
    }
}

/// Represents a `JsonPointer` path element.
///
/// - **See** - [JsonPointer specification](https://datatracker.ietf.org/doc/html/rfc6901)
///
/// ## Example
/// The `JsonPointer` `/field_a/1/field_b` would be represented as:
///
/// ```rust,ignore
/// use smithy4rs_core::serde::validate::{Path, PathItem};
/// let paths = vec![
///     Path::Schema(FIELD_A.clone()),
///     Path::Index(1),
///     Path::Schema(FIELD_B.clone())
/// ];
/// ```
///
#[derive(Debug, Clone, PartialEq)]
pub enum PathElement {
    /// A Schema path element such as a member identifier
    Schema(Schema),
    /// An index path element (for list elements)
    Index(usize),
    /// A key path element (for map keys)
    Key(String),
}
impl From<&Schema> for PathElement {
    fn from(schema_ref: &Schema) -> Self {
        PathElement::Schema(schema_ref.clone())
    }
}

/// Marker trait for validation errors.
pub trait ValidationError: Error {}

// Implement conversion for any Error enums implementing Validation error
impl<T: ValidationError + 'static> From<T> for Box<dyn ValidationError> {
    #[inline]
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

// ============================================================================
// Validator core errors
// ============================================================================

/// Captures validation failures that could happen for any validator.
///
/// These errors should only occur for manually constructed schemas.
/// If you encounter one of these in a generated shape using the default
/// validator then this is a bug.
#[derive(Error, Debug)]
enum ValidationFailure {
    #[error("Expected schema to contain member: `{0}`")]
    ExpectedMember(String),
    #[error("Maximum Validation depth: {0} exceeded")]
    MaximumDepthExceeded(usize),
    #[error("Maximum Number of errors ({0}) reached")]
    MaxErrorsReached(usize),
    #[error("Tried to pop from an empty path stack. This is a bug.")]
    PopFromEmptyValidator,
    #[error("Attempted to perform `@uniqueItem` check on float. This is invalid")]
    UniqueItemOnFloat,
    #[error("{0}")]
    Custom(String),
}
impl serializers::Error for ValidationFailure {
    fn custom<T: Display>(msg: T) -> Self {
        ValidationFailure::Custom(msg.to_string())
    }
}
impl ValidationError for ValidationFailure {}

// ============================================================================
// Base Smithy constraint errors
// ============================================================================

/// Validation errors from the built-in Smithy constraint traits.
#[derive(Error, Debug, PartialEq)]
enum SmithyConstraints {
    /// [@required](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-required-trait)
    #[error("Field is Required.")]
    Required,
    /// [@length](<https://smithy.io/2.0/spec/constraint-traits.html#length-trait>)
    #[error("Size: {0} does not conform to @length constraint. Expected between {1} and {2}.")]
    Length(usize, usize, usize),
    /// [@pattern](<https://smithy.io/2.0/spec/constraint-traits.html#pattern-trait>)
    #[error("Value `{0}` did not conform to expected pattern `{1}`")]
    Pattern(String, String),
    /// [@range](<https://smithy.io/2.0/spec/constraint-traits.html#range-trait>)
    #[error("Size: {0} does not conform to @range constraint. Expected between {1} and {2}.")]
    Range(BigDecimal, BigDecimal, BigDecimal),
    /// [@uniqueItems](<https://smithy.io/2.0/spec/constraint-traits.html#uniqueitems-trait>)
    #[error("Items in collection should be unique.")]
    UniqueItems,
    #[error("Shape type {0} does not match expected.")]
    ShapeType(ShapeType),
    #[error("Enum value `{0}` invalid. Expected one of: {1:?}.")]
    EnumValue(String, FxIndexSet<&'static str>),
    #[error("Enum value `{0}` invalid. Expected one of: {1:?}.")]
    IntEnumValue(i32, FxIndexSet<i32>),
}
impl ValidationError for SmithyConstraints {}

#[cfg(test)]
#[allow(clippy::type_complexity)]
mod tests {
    use super::*;
    use crate::{
        IndexMap,
        derive::SmithyShape,
        schema::prelude::{INTEGER, LengthTrait, PatternTrait, STRING, UniqueItemsTrait},
        serde::ShapeBuilder,
        smithy,
    };

    #[test]
    fn test_validation_errors_aggregate() {
        let mut errors = ValidationErrors::new();
        errors.add(
            &[PathElement::Schema(STRING.clone())],
            SmithyConstraints::Required,
        );
        errors.add(
            &[PathElement::Schema(STRING.clone())],
            SmithyConstraints::Length(1, 2, 3),
        );
        errors.add(
            &[PathElement::Schema(STRING.clone())],
            SmithyConstraints::Required,
        );
        assert_eq!(errors.errors.len(), 3);
        assert_eq!(&errors.errors[0].error.to_string(), "Field is Required.");
        assert_eq!(&errors.errors[2].error.to_string(), "Field is Required.");
    }

    // ==== Basic Shape Validations ====
    smithy!("com.test#ValidatedList": {
        @LengthTrait::builder().max(3).build();
        @UniqueItemsTrait;
        list LIST_SCHEMA {
            @LengthTrait::builder().max(4).build();
            member: STRING
        }
    });
    smithy!("com.test#ValidatedMap": {
        @LengthTrait::builder().max(2).build();
        map MAP_SCHEMA {
            @PatternTrait::new("^[a-zA-Z]*$");
            key: STRING
            @LengthTrait::builder().max(4).build();
            value: STRING
        }
    });
    smithy!("com.test#ValidationStruct": {
        structure BASIC_VALIDATION_SCHEMA {
            @PatternTrait::new("^[a-zA-Z]*$");
            A: STRING = "a"
            B: INTEGER = "b"
            LIST: LIST_SCHEMA = "list"
            MAP: MAP_SCHEMA = "map"
        }
    });
    #[derive(SmithyShape)]
    #[smithy_schema(BASIC_VALIDATION_SCHEMA)]
    pub struct SimpleStruct {
        #[smithy_schema(A)]
        field_a: String,
        #[smithy_schema(B)]
        field_b: Option<i32>,
        #[smithy_schema(LIST)]
        field_list: Option<Vec<String>>,
        #[smithy_schema(MAP)]
        field_map: Option<IndexMap<String, String>>,
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

        let error_field_a = err.errors.first().unwrap();
        assert_eq!(
            error_field_a.paths,
            vec![PathElement::Schema(
                _BASIC_VALIDATION_SCHEMA_MEMBER_A.clone()
            )]
        );
        assert_eq!(
            error_field_a.error.to_string(),
            "Field is Required.".to_string()
        );
    }

    #[test]
    fn basic_string_validations_are_performed() {
        let builder = SimpleStructBuilder::new();
        let inner_vec = vec!["too long of a string".to_string()];
        let Some(err) = builder
            .field_list(inner_vec)
            .field_a("field-a".to_string())
            .build()
            .err()
        else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 2);
        let error_pattern = err.errors.first().unwrap();
        assert_eq!(
            error_pattern.paths,
            vec![PathElement::Schema(
                _BASIC_VALIDATION_SCHEMA_MEMBER_A.clone()
            )]
        );
        assert_eq!(
            error_pattern.error.to_string(),
            "Value `field-a` did not conform to expected pattern `^[a-zA-Z]*$`".to_string()
        );

        let error_length = err.errors.get(1).unwrap();
        assert_eq!(
            error_length.paths,
            vec![
                PathElement::Schema(_BASIC_VALIDATION_SCHEMA_MEMBER_LIST.clone()),
                PathElement::Index(0)
            ]
        );
        assert_eq!(
            error_length.error.to_string(),
            "Size: 20 does not conform to @length constraint. Expected between 0 and 4."
                .to_string()
        );
    }

    #[test]
    fn required_field_does_not_short_circuit_validation() {
        let inner_vec = vec!["too long of a string".to_string()];
        let Err(err) = SimpleStructBuilder::new().field_list(inner_vec).build() else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 2);
        let error_required = err.errors.first().unwrap();
        let error_length = err.errors.get(1).unwrap();

        assert_eq!(
            error_required.paths,
            vec![PathElement::Schema(
                _BASIC_VALIDATION_SCHEMA_MEMBER_A.clone()
            )]
        );
        assert_eq!(
            error_required.error.to_string(),
            "Field is Required.".to_string()
        );

        assert_eq!(
            error_length.paths,
            vec![
                PathElement::Schema(_BASIC_VALIDATION_SCHEMA_MEMBER_LIST.clone()),
                PathElement::Index(0)
            ]
        );
        assert_eq!(
            error_length.error.to_string(),
            "Size: 20 does not conform to @length constraint. Expected between 0 and 4."
                .to_string()
        );
    }

    #[test]
    fn list_constraints_checked() {
        let builder = SimpleStructBuilder::new();
        let inner_vec = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "a".to_string(),
            "d".to_string(),
        ];
        let Some(err) = builder
            .field_list(inner_vec)
            .field_a("fieldA".to_string())
            .build()
            .err()
        else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 2);
        let error_length = err.errors.first().unwrap();

        assert_eq!(
            error_length.paths,
            vec![PathElement::Schema(
                _BASIC_VALIDATION_SCHEMA_MEMBER_LIST.clone()
            )]
        );
        assert_eq!(
            error_length.error.to_string(),
            "Size: 5 does not conform to @length constraint. Expected between 0 and 3.".to_string()
        );

        let error_unique = err.errors.get(1).unwrap();
        assert_eq!(
            error_unique.paths,
            vec![
                PathElement::Schema(_BASIC_VALIDATION_SCHEMA_MEMBER_LIST.clone()),
                PathElement::Index(3)
            ]
        );
        assert_eq!(
            error_unique.error.to_string(),
            "Items in collection should be unique.".to_string()
        );
    }

    #[test]
    fn map_constraints_checked() {
        let builder = SimpleStructBuilder::new();
        let mut inner_map = IndexMap::<String, String>::new();
        inner_map.insert("bad-key".to_string(), "a".to_string());
        inner_map.insert("a".to_string(), "value is too long!".to_string());
        inner_map.insert("b".to_string(), "a".to_string());
        let Some(err) = builder
            .field_map(inner_map)
            .field_a("fieldA".to_string())
            .build()
            .err()
        else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 3);

        let error_length = err.errors.first().unwrap();
        assert_eq!(
            error_length.paths,
            vec![PathElement::Schema(
                _BASIC_VALIDATION_SCHEMA_MEMBER_MAP.clone()
            )]
        );
        assert_eq!(
            error_length.error.to_string(),
            "Size: 3 does not conform to @length constraint. Expected between 0 and 2.".to_string()
        );

        let error_key = err.errors.get(1).unwrap();
        assert_eq!(
            error_key.paths,
            vec![
                PathElement::Schema(_BASIC_VALIDATION_SCHEMA_MEMBER_MAP.clone()),
                PathElement::Key("bad-key".to_string())
            ]
        );
        assert_eq!(
            error_key.error.to_string(),
            "Value `bad-key` did not conform to expected pattern `^[a-zA-Z]*$`".to_string()
        );

        let error_value = err.errors.get(2).unwrap();
        assert_eq!(
            error_value.paths,
            vec![
                PathElement::Schema(_BASIC_VALIDATION_SCHEMA_MEMBER_MAP.clone()),
                PathElement::Key("a".to_string())
            ]
        );
        assert_eq!(
            error_value.error.to_string(),
            "Size: 18 does not conform to @length constraint. Expected between 0 and 4."
                .to_string()
        );
    }

    // ====== NESTED SHAPE VALIDATION =====
    // Nested Shape
    smithy!("test#ValidationStruct": {
        structure NESTED_SCHEMA {
            @PatternTrait::new("^[a-z]*$");
            C: STRING = "c"
        }
    });

    #[derive(SmithyShape, Clone)]
    #[smithy_schema(NESTED_SCHEMA)]
    pub struct NestedStruct {
        #[smithy_schema(C)]
        field_c: String,
    }

    smithy!("test#StructWithNested": {
        structure STRUCT_WITH_NESTED_SCHEMA {
            NESTED: NESTED_SCHEMA = "nested"
            NESTED_REQUIRED: NESTED_SCHEMA = "required"
        }
    });
    #[derive(SmithyShape, Clone)]
    #[smithy_schema(STRUCT_WITH_NESTED_SCHEMA)]
    pub struct StructWithNested {
        #[smithy_schema(NESTED)]
        field_nested: Option<NestedStruct>,
        #[smithy_schema(NESTED_REQUIRED)]
        field_required_nested: NestedStruct,
    }

    #[test]
    fn nested_struct_fields_build_if_no_errors() {
        let builder_nested = NestedStructBuilder::new().field_c("field".to_string());
        let builder = StructWithNestedBuilder::new();
        let _value = builder
            .field_required_nested_builder(builder_nested)
            .build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_fields_build_with_pre_built_shape() {
        let built_nested = NestedStructBuilder::new()
            .field_c("field".to_string())
            .build()
            .expect("Failed to build NestedStruct");
        let builder = StructWithNestedBuilder::new();
        let _value = builder
            .field_required_nested(built_nested)
            .build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_fields_checked() {
        let builder_nested = NestedStructBuilder::new().field_c("dataWithCaps".to_string());
        let builder = StructWithNestedBuilder::new();
        let Some(err) = builder
            .field_required_nested_builder(builder_nested)
            .build()
            .err()
        else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);
        let error_pattern = err.errors.first().unwrap();
        assert_eq!(
            error_pattern.paths,
            vec![
                PathElement::Schema(_STRUCT_WITH_NESTED_SCHEMA_MEMBER_NESTED_REQUIRED.clone()),
                PathElement::Schema(_NESTED_SCHEMA_MEMBER_C.clone())
            ]
        );
        assert_eq!(
            error_pattern.error.to_string(),
            "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string()
        );
    }

    // ==== Nested List Validations ====
    smithy!("com.example#ListOfNested": {
        @LengthTrait::builder().max(3).build();
        list LIST_OF_NESTED_SCHEMA {
            member: NESTED_SCHEMA
        }
    });
    smithy!("com.example#ListOfList": {
        @LengthTrait::builder().max(2).build();
        list LIST_OF_LIST_OF_NESTED_SCHEMA {
            member: LIST_OF_NESTED_SCHEMA
        }
    });
    smithy!("com.example#ListOfListOfList": {
        @LengthTrait::builder().max(2).build();
        list LIST_OF_LIST_OF_LIST_OF_NESTED {
            member: LIST_OF_LIST_OF_NESTED_SCHEMA
        }
    });
    smithy!("test#StructWithNestedList": {
        structure STRUCT_WITH_NESTED_LIST_SCHEMA {
            LIST: LIST_OF_NESTED_SCHEMA = "list"
            LIST_REQUIRED: LIST_OF_NESTED_SCHEMA = "list_required"
            DEEPLY_NESTED: LIST_OF_LIST_OF_LIST_OF_NESTED = "deeply_nested"
        }
    });

    #[derive(SmithyShape, Clone)]
    #[smithy_schema(STRUCT_WITH_NESTED_LIST_SCHEMA)]
    pub struct StructWithNestedLists {
        #[smithy_schema(LIST)]
        field_nested_list: Option<Vec<NestedStruct>>,
        #[smithy_schema(LIST_REQUIRED)]
        field_required_nested_list: Vec<NestedStruct>,
        #[smithy_schema(DEEPLY_NESTED)]
        field_deeply_nested_list: Option<Vec<Vec<Vec<NestedStruct>>>>,
    }

    #[test]
    fn nested_struct_list_build_if_no_errors() {
        let nested_list = vec![NestedStructBuilder::new().field_c("data".to_string())];
        let builder = StructWithNestedListsBuilder::new();
        builder
            .field_required_nested_list_builder(nested_list)
            .build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_list_fields_build_with_pre_built_shapes() {
        let nested_list = vec![
            NestedStructBuilder::new()
                .field_c("a".to_string())
                .build()
                .expect("Failed to build NestedStruct"),
            NestedStructBuilder::new()
                .field_c("b".to_string())
                .build()
                .expect("Failed to build NestedStruct"),
        ];
        let builder = StructWithNestedListsBuilder::new();
        let _value = builder
            .field_required_nested_list(nested_list)
            .build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_list_checked() {
        let nested_list = vec![
            NestedStructBuilder::new().field_c("a".to_string()),
            NestedStructBuilder::new().field_c("b".to_string()),
            NestedStructBuilder::new().field_c("dataWithCaps".to_string()),
            NestedStructBuilder::new().field_c("b".to_string()),
        ];
        let builder = StructWithNestedListsBuilder::new();
        let Some(err) = builder
            .field_required_nested_list_builder(nested_list)
            .build()
            .err()
        else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 2);

        let error_length = err.errors.first().unwrap();
        assert_eq!(
            error_length.paths,
            vec![PathElement::Schema(
                _STRUCT_WITH_NESTED_LIST_SCHEMA_MEMBER_LIST_REQUIRED.clone()
            )]
        );
        assert_eq!(
            error_length.error.to_string(),
            "Size: 4 does not conform to @length constraint. Expected between 0 and 3.".to_string()
        );

        let error_pattern = err.errors.get(1).unwrap();
        assert_eq!(
            error_pattern.paths,
            vec![
                PathElement::Schema(_STRUCT_WITH_NESTED_LIST_SCHEMA_MEMBER_LIST_REQUIRED.clone()),
                PathElement::Index(2),
                PathElement::Schema(_NESTED_SCHEMA_MEMBER_C.clone())
            ]
        );
        assert_eq!(
            error_pattern.error.to_string(),
            "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string()
        );
    }

    #[test]
    fn deeply_nested_struct_list_checks_validation_rules() {
        let nested_list = vec![NestedStructBuilder::new().field_c("data".to_string())];
        let deeply_nested_list = vec![vec![vec![
            NestedStructBuilder::new().field_c("dataWithCaps".to_string()),
        ]]];
        let builder = StructWithNestedListsBuilder::new();
        let Some(err) = builder
            .field_required_nested_list_builder(nested_list)
            .field_deeply_nested_list_builder(deeply_nested_list)
            .build()
            .err()
        else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);

        let error_pattern = err.errors.first().unwrap();
        assert_eq!(
            error_pattern.paths,
            vec![
                PathElement::Schema(_STRUCT_WITH_NESTED_LIST_SCHEMA_MEMBER_DEEPLY_NESTED.clone()),
                PathElement::Index(0),
                PathElement::Index(0),
                PathElement::Index(0),
                PathElement::Schema(_NESTED_SCHEMA_MEMBER_C.clone())
            ]
        );
        assert_eq!(
            error_pattern.error.to_string(),
            "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string()
        );
    }

    // ==== `@uniqueItem` Validations ====
    smithy!("com.example#SetOfStruct": {
        @UniqueItemsTrait;
        list SET_OF_STRUCT {
            member: NESTED_SCHEMA
        }
    });
    smithy!("com.example#SetOfString": {
        @UniqueItemsTrait;
        list SET_OF_STRING {
            member: STRING
        }
    });
    smithy!("com.example#ListOfInt": {
        list LIST_OF_INT {
            member: INTEGER
        }
    });
    smithy!("com.example#SetOfList": {
        @UniqueItemsTrait;
        list SET_OF_LIST {
            member: LIST_OF_INT
        }
    });
    smithy!("com.example#MapOfInt": {
        map MAP_OF_INT {
            key: STRING
            value: INTEGER
        }
    });
    smithy!("com.example#SetOfMap": {
        @UniqueItemsTrait;
        list SET_OF_MAP {
            member: MAP_OF_INT
        }
    });
    smithy!("test#StructWithSets": {
        structure STRUCT_WITH_SETS {
            STRUCT: SET_OF_STRUCT = "set_of_struct"
            STRING: SET_OF_STRING = "set_of_simple"
            LIST: SET_OF_LIST = "set_of_list"
            MAP: SET_OF_MAP = "set_of_map"
        }
    });

    #[derive(SmithyShape, Clone)]
    #[smithy_schema(STRUCT_WITH_SETS)]
    pub struct StructWithSets {
        #[smithy_schema(STRUCT)]
        set_of_struct: Option<Vec<NestedStruct>>,
        #[smithy_schema(STRING)]
        set_of_simple: Option<Vec<String>>,
        #[smithy_schema(LIST)]
        set_of_list: Option<Vec<Vec<i32>>>,
        #[smithy_schema(MAP)]
        set_of_map: Option<Vec<IndexMap<String, i32>>>,
    }

    #[test]
    fn detects_duplicates_in_sets() {
        let structs = vec![
            NestedStructBuilder::new().field_c("a".to_string()),
            NestedStructBuilder::new().field_c("b".to_string()),
            NestedStructBuilder::new().field_c("b".to_string()),
        ];
        let simple = vec![
            "Stuff".to_string(),
            "Things".to_string(),
            "Stuff".to_string(),
        ];
        let list = vec![vec![1, 2, 3], vec![4, 5, 6], vec![1, 2, 3]];
        let mut map_a = IndexMap::new();
        map_a.insert("Stuff".to_string(), 1);
        map_a.insert("Things".to_string(), 2);
        let mut map_b = IndexMap::new();
        map_b.insert("Quux".to_string(), 1);
        map_b.insert("Other".to_string(), 2);
        let map = vec![map_a.clone(), map_b, map_a];
        let builder = StructWithSetsBuilder::new();
        let Some(err) = builder
            .set_of_struct_builder(structs)
            .set_of_simple(simple)
            .set_of_list(list)
            .set_of_map(map)
            .build()
            .err()
        else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 4);

        // Should _only_ be uniqueness errors
        for e in &err.errors {
            assert_eq!(
                e.error.to_string(),
                "Items in collection should be unique.".to_string()
            );
        }

        let error_unique_struct = err.errors.first().unwrap();
        assert_eq!(
            error_unique_struct.paths,
            vec![
                PathElement::Schema(_STRUCT_WITH_SETS_MEMBER_STRUCT.clone()),
                PathElement::Index(2)
            ]
        );

        let error_unique_simple = err.errors.get(1).unwrap();
        assert_eq!(
            error_unique_simple.paths,
            vec![
                PathElement::Schema(_STRUCT_WITH_SETS_MEMBER_STRING.clone()),
                PathElement::Index(2)
            ]
        );

        let error_unique_list = err.errors.get(2).unwrap();
        assert_eq!(
            error_unique_list.paths,
            vec![
                PathElement::Schema(_STRUCT_WITH_SETS_MEMBER_LIST.clone()),
                PathElement::Index(2)
            ]
        );

        let error_unique_map = err.errors.get(3).unwrap();
        assert_eq!(
            error_unique_map.paths,
            vec![
                PathElement::Schema(_STRUCT_WITH_SETS_MEMBER_MAP.clone()),
                PathElement::Index(2)
            ]
        );
    }

    // ==== Nested Map Validations ====
    smithy!("com.example#MapOfNested": {
        @LengthTrait::builder().max(2).build();
        map MAP_OF_NESTED_SCHEMA {
            key: STRING
            value: NESTED_SCHEMA
        }
    });
    smithy!("com.example#MapOfMap": {
        @LengthTrait::builder().max(2).build();
        map MAP_OF_MAP_OF_NESTED {
            key: STRING
            value: MAP_OF_NESTED_SCHEMA
        }
    });
    smithy!("com.example#MapOfMapOfMap": {
        @LengthTrait::builder().max(2).build();
        map MAP_OF_MAP_OF_MAP_OF_NESTED {
            key: STRING
            value: MAP_OF_MAP_OF_NESTED
        }
    });
    smithy!("test#StructWithNestedMap": {
        structure STRUCT_WITH_NESTED_MAP_SCHEMA {
            OPTIONAL: MAP_OF_NESTED_SCHEMA = "optional"
            REQUIRED: MAP_OF_NESTED_SCHEMA = "required"
            DEEPLY_NESTED: MAP_OF_MAP_OF_MAP_OF_NESTED = "deeply_nested"
        }
    });

    #[derive(SmithyShape, Clone)]
    #[smithy_schema(STRUCT_WITH_NESTED_MAP_SCHEMA)]
    pub struct StructWithNestedMaps {
        #[smithy_schema(OPTIONAL)]
        optional: Option<IndexMap<String, NestedStruct>>,
        #[smithy_schema(REQUIRED)]
        required: IndexMap<String, NestedStruct>,
        #[smithy_schema(DEEPLY_NESTED)]
        deeply_nested: Option<IndexMap<String, IndexMap<String, IndexMap<String, NestedStruct>>>>,
    }

    #[test]
    fn nested_struct_map_build_if_no_errors() {
        let mut nested_map = IndexMap::new();
        nested_map.insert(
            "a".to_string(),
            NestedStructBuilder::new().field_c("data".to_string()),
        );
        let builder = StructWithNestedMapsBuilder::new();
        builder
            .required_builder(nested_map)
            .build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_map_fields_build_with_pre_built_shapes() {
        let mut nested_map = IndexMap::new();
        nested_map.insert(
            "a".to_string(),
            NestedStructBuilder::new()
                .field_c("data".to_string())
                .build()
                .expect("Failed to build nested"),
        );
        let builder = StructWithNestedMapsBuilder::new();
        builder
            .required(nested_map)
            .build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_map_checked() {
        let mut nested_map = IndexMap::new();
        nested_map.insert(
            "a".to_string(),
            NestedStructBuilder::new().field_c("a".to_string()),
        );
        nested_map.insert(
            "b".to_string(),
            NestedStructBuilder::new().field_c("dataWithCaps".to_string()),
        );
        nested_map.insert(
            "c".to_string(),
            NestedStructBuilder::new().field_c("c".to_string()),
        );
        let builder = StructWithNestedMapsBuilder::new();
        let Some(err) = builder.required_builder(nested_map).build().err() else {
            panic!("Expected an error");
        };

        assert_eq!(err.errors.len(), 2);

        let error_length = err.errors.first().unwrap();
        assert_eq!(
            error_length.paths,
            vec![PathElement::Schema(
                _STRUCT_WITH_NESTED_MAP_SCHEMA_MEMBER_REQUIRED.clone()
            )]
        );
        assert_eq!(
            error_length.error.to_string(),
            "Size: 3 does not conform to @length constraint. Expected between 0 and 2.".to_string()
        );

        let error_pattern = err.errors.get(1).unwrap();
        assert_eq!(
            error_pattern.paths,
            vec![
                PathElement::Schema(_STRUCT_WITH_NESTED_MAP_SCHEMA_MEMBER_REQUIRED.clone()),
                PathElement::Key("b".to_string()),
                PathElement::Schema(_NESTED_SCHEMA_MEMBER_C.clone())
            ]
        );
        assert_eq!(
            error_pattern.error.to_string(),
            "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string()
        );
    }

    #[test]
    fn deeply_nested_struct_map_build_if_no_errors() {
        let mut nested_map = IndexMap::new();
        nested_map.insert(
            "a".to_string(),
            NestedStructBuilder::new().field_c("a".to_string()),
        );

        let mut deep_nesting = IndexMap::new();
        let mut mid_nesting = IndexMap::new();
        let mut low_nesting = IndexMap::new();
        low_nesting.insert(
            "a".to_string(),
            NestedStructBuilder::new().field_c("dataWithCaps".to_string()),
        );
        mid_nesting.insert("a".to_string(), low_nesting);
        deep_nesting.insert("a".to_string(), mid_nesting);
        let builder = StructWithNestedMapsBuilder::new();
        let Some(err) = builder
            .required_builder(nested_map)
            .deeply_nested_builder(deep_nesting)
            .build()
            .err()
        else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);

        let error_pattern = err.errors.first().unwrap();
        assert_eq!(
            error_pattern.paths,
            vec![
                PathElement::Schema(_STRUCT_WITH_NESTED_MAP_SCHEMA_MEMBER_DEEPLY_NESTED.clone()),
                PathElement::Key("a".to_string()),
                PathElement::Key("a".to_string()),
                PathElement::Key("a".to_string()),
                PathElement::Schema(_NESTED_SCHEMA_MEMBER_C.clone())
            ]
        );
        assert_eq!(
            error_pattern.error.to_string(),
            "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string()
        );
    }

    // ==== Enum Validations ====
    smithy!("test#Enum": {
        enum TEST_ENUM {
            A = "a"
            B = "b"
        }
    });

    #[test]
    fn checks_string_against_enum_value() {
        let mut validator = DefaultValidator::new();
        let Err(err) = validator.validate(&TEST_ENUM, &"lies!".to_string()) else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);
        let error_enum = err.errors.first().unwrap();
        assert_eq!(
            error_enum.error.to_string(),
            "Enum value `lies!` invalid. Expected one of: {\"a\", \"b\"}.".to_string()
        );
    }

    smithy!("test#intEnum": {
        intEnum TEST_INT_ENUM {
            A = 1
            B = 2
        }
    });

    #[test]
    fn checks_int_against_int_enum_value() {
        let mut validator = DefaultValidator::new();
        let Err(err) = validator.validate(&TEST_INT_ENUM, &4) else {
            panic!("Expected an error");
        };
        assert_eq!(err.errors.len(), 1);
        let error_enum = err.errors.first().unwrap();
        assert_eq!(
            error_enum.error.to_string(),
            "Enum value `4` invalid. Expected one of: {1, 2}.".to_string()
        );
    }
}
