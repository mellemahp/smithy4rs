//! # Validation
//!
//! Validation compare a shape against a set of constraints defined in the Smithy Model.
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
//! By default, users should _not_ be able to manually construct shapes that violate the basic Smithy contraints.
//! In `smithy4rs`,  [`ShapeBuilder`] implementations are validated with the [`DefaultValidator`] on [`ShapeBuilder::build()`].
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
//! the [`ShapeBuilder::build_with_validator`] method with your custom implementation.
//!
//! ## Validating Protocol-specific constraints
//! Some protocols may have additional constraints that they need to check in addition to the basic
//! Smithy constraints.
//!
//! To support protocol-specific validation, [`Protocol`] implementations provide a [`Validator`]
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

use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;
use rustc_hash::FxHasher;
use thiserror::Error;

use crate::{
    Instant,
    prelude::{LengthTrait, PatternTrait, UniqueItemsTrait},
    schema::{Document, SchemaRef, ShapeType},
    serde::{
        se::{SerializeWithSchema, Serializer},
        serializers,
        serializers::{ListSerializer, MapSerializer, StructSerializer},
    },
};

//////////////////////////////////////////////////////////////////////////////
// Validation Traits
//////////////////////////////////////////////////////////////////////////////

/// Validator that ensures shapes conform to constraint traits.
///
/// Under the hood a validator is [`Serializer`] that walks a serializable shape,
/// comparing each shape/member against the provided schema.
///
/// **NOTE**: Implementations should only return `Err` results from the serialization
/// methods if they wish to short-circuit validation (for example if a maximum number of
/// errors is reached). Otherwise, they should aggregate all errors internally and return
/// them as in the aggregate [`ValidationErrors`] result of the [`Validator::validate`] method.
///
/// For the default Validator implementation that supports built-in Smithy constraints
/// see [`DefaultValidator`].
pub trait Validator: Serializer<Ok = (), Error = ValidationErrors> {
    /// Validates a type against a schema.
    ///
    /// **IMPL NOTE**: If any validation errors are found, this method SHOULD return
    /// an `Err` result containing an aggregate of all the validation errors encountered.
    fn validate<V: SerializeWithSchema>(
        self,
        schema: &SchemaRef,
        value: &V,
    ) -> Result<(), ValidationErrors>;
}

//////////////////////////////////////////////////////////////////////////////
// Default Implementation
//////////////////////////////////////////////////////////////////////////////

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
/// [`ShapeBuilder::build()`] implementation). It can also be used standalone on any serializable shapes.
/// ```rust, ignore
/// let result = DefaultValidator::new().validate(MySerializableShape);
/// ```
///
/// To customize the `Validator` used when building a shape, pass a custom [`Validator`] implementation
/// into the [`ShapeBuilder::build_with_validator()`] method on the builder.
///
/// * `D` - Maximum validation depth (Default: 10).
/// * `E` - Maximum number of errors the validator can track (Default: 20).
pub struct DefaultValidator<const D: usize = 10, const E: usize = 20> {
    errors: Option<ValidationErrors>,
    path_stack: Vec<PathElement>,
}

impl<const D: usize, const ERR: usize> DefaultValidator<D, ERR> {
    /// Create a new [`Validator`] instance.
    pub const fn new() -> Self {
        DefaultValidator {
            errors: None,
            path_stack: Vec::new(),
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
        // SAFETY: Safe to unwrap as errors will alway be set to `SOME` above
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
        schema: &SchemaRef,
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
impl<'a> Serializer for &'a mut DefaultValidator {
    type Error = ValidationErrors;
    type Ok = ();
    type SerializeList = DefaultListValidator<'a>;
    type SerializeMap = DefaultMapValidator<'a>;
    type SerializeStruct = DefaultStructValidator<'a>;

    fn write_struct(
        self,
        schema: &SchemaRef,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        if !schema.shape_type().eq(&ShapeType::Structure)
            && !schema.shape_type().eq(&ShapeType::Union)
        {
            self.emit_error(SmithyConstraints::ShapeType(*schema.shape_type()))?;
        }
        Ok(DefaultStructValidator { root: self })
    }

    fn write_map(self, schema: &SchemaRef, len: usize) -> Result<Self::SerializeMap, Self::Error> {
        shape_type!(self, schema, ShapeType::Map);
        length!(self, schema, len);
        Ok(DefaultMapValidator { root: self })
    }

    fn write_list(
        self,
        schema: &SchemaRef,
        len: usize,
    ) -> Result<Self::SerializeList, Self::Error> {
        shape_type!(self, schema, ShapeType::List);
        length!(self, schema, len);
        Ok(DefaultListValidator {
            root: self,
            unique: schema.contains_type::<UniqueItemsTrait>(),
            lookup: UniquenessTracker::new(),
            index: 0,
        })
    }

    fn write_boolean(self, schema: &SchemaRef, _value: bool) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Boolean);
        Ok(())
    }

    fn write_byte(self, schema: &SchemaRef, _value: i8) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Byte);
        // TODO: Range constraint
        Ok(())
    }

    fn write_short(self, schema: &SchemaRef, _value: i16) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Short);
        // TODO: Range constraint
        Ok(())
    }

    fn write_integer(self, schema: &SchemaRef, _value: i32) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Integer);
        // TODO: Range constraint
        Ok(())
    }

    fn write_long(self, schema: &SchemaRef, _value: i64) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Long);
        // TODO: Range constraint
        Ok(())
    }

    fn write_float(self, schema: &SchemaRef, _value: f32) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Float);
        // TODO: Range constraint
        Ok(())
    }

    fn write_double(self, schema: &SchemaRef, _value: f64) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Double);
        // TODO: Range constraint
        Ok(())
    }

    fn write_big_integer(
        self,
        schema: &SchemaRef,
        _value: &BigInt,
    ) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::BigInteger);
        // TODO: Range constraint
        Ok(())
    }

    fn write_big_decimal(
        self,
        schema: &SchemaRef,
        _value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::BigDecimal);
        // TODO: Range constraint
        Ok(())
    }

    fn write_string(self, schema: &SchemaRef, value: &str) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::String);
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

        Ok(())
    }

    fn write_blob(self, schema: &SchemaRef, _value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Blob);
        Ok(())
    }

    fn write_timestamp(
        self,
        schema: &SchemaRef,
        _value: &Instant,
    ) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Timestamp);
        Ok(())
    }

    fn write_document(
        self,
        schema: &SchemaRef,
        _value: &Document,
    ) -> Result<Self::Ok, Self::Error> {
        shape_type!(self, schema, ShapeType::Document);
        Ok(())
    }

    fn write_null(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        /* Skip null value validation */
        // TODO: How should sparseness be handled?
        Ok(())
    }

    #[inline]
    fn write_missing(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.emit_error(SmithyConstraints::Required)
    }

    #[inline]
    fn skip(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
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
        element_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
    {
        self.root.push_path(PathElement::Index(self.index))?;
        if self.unique {
            match self.lookup.add(element_schema, value) {
                Ok(true) => self.root.emit_error(SmithyConstraints::UniqueItems),
                // Return early on this error. Something is wrong with the schema.
                Err(err) => return self.root.short_circuit(err),
                _ => Ok(()),
            }?
        }
        value.serialize_with_schema(element_schema, &mut *self.root)?;
        self.root.pop_path()?;
        self.index += 1;
        Ok(())
    }

    #[inline]
    fn end(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        //self.root.pop_path()
        Ok(())
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
            lookup: BTreeSet::new(),
        }
    }

    /// Add an item to the set.
    ///
    /// Returns true if the item was already in the set.
    fn add<T: ?Sized + SerializeWithSchema>(
        &mut self,
        schema: &SchemaRef,
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
        _schema: &SchemaRef,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(InnerHasher { root: self })
    }

    #[inline]
    fn write_map(
        self,
        _schema: &SchemaRef,
        _len: usize,
    ) -> Result<Self::SerializeMap, Self::Error> {
        Ok(InnerHasher { root: self })
    }

    #[inline]
    fn write_list(
        self,
        _schema: &SchemaRef,
        _len: usize,
    ) -> Result<Self::SerializeList, Self::Error> {
        Ok(InnerHasher { root: self })
    }

    #[inline]
    fn write_boolean(self, _schema: &SchemaRef, value: bool) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_byte(self, _schema: &SchemaRef, value: i8) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_short(self, _schema: &SchemaRef, value: i16) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_integer(self, _schema: &SchemaRef, value: i32) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_long(self, _schema: &SchemaRef, value: i64) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_float(self, _schema: &SchemaRef, _value: f32) -> Result<Self::Ok, Self::Error> {
        Err(ValidationFailure::UniqueItemOnFloat)
    }

    #[inline]
    fn write_double(self, _schema: &SchemaRef, _value: f64) -> Result<Self::Ok, Self::Error> {
        Err(ValidationFailure::UniqueItemOnFloat)
    }

    #[inline]
    fn write_big_integer(
        self,
        _schema: &SchemaRef,
        value: &BigInt,
    ) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_big_decimal(
        self,
        _schema: &SchemaRef,
        value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_string(self, _schema: &SchemaRef, value: &str) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_blob(self, _schema: &SchemaRef, value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        hash_impl!(self, value);
    }

    #[inline]
    fn write_timestamp(
        self,
        _schema: &SchemaRef,
        value: &Instant,
    ) -> Result<Self::Ok, Self::Error> {
        self.hash(value.epoch_nanoseconds().0);
        Ok(())
    }

    fn write_document(
        self,
        _schema: &SchemaRef,
        _value: &Document,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    #[inline]
    fn write_null(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.hash("null");
        Ok(())
    }

    #[inline]
    fn skip(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        /* Do not execute hash on skip */
        Ok(())
    }
}

struct InnerHasher<'a> {
    root: &'a mut HashingSerializer,
}
impl ListSerializer for InnerHasher<'_> {
    type Error = ValidationFailure;
    type Ok = ();

    #[inline]
    fn serialize_element<T>(
        &mut self,
        element_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
    {
        value.serialize_with_schema(element_schema, &mut *self.root)?;
        Ok(())
    }

    #[inline]
    fn end(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
impl MapSerializer for InnerHasher<'_> {
    type Error = ValidationFailure;
    type Ok = ();

    #[inline]
    fn serialize_entry<K, V>(
        &mut self,
        key_schema: &SchemaRef,
        value_schema: &SchemaRef,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: ?Sized + SerializeWithSchema,
        V: ?Sized + SerializeWithSchema,
    {
        key.serialize_with_schema(key_schema, &mut *self.root)?;
        value.serialize_with_schema(value_schema, &mut *self.root)?;
        Ok(())
    }

    #[inline]
    fn end(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
impl StructSerializer for InnerHasher<'_> {
    type Error = ValidationFailure;
    type Ok = ();

    fn serialize_member<T>(
        &mut self,
        member_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
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
        member_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
    {
        self.root.hash(member_name);
        value.serialize_with_schema(member_schema, &mut *self.root)?;
        Ok(())
    }

    #[inline]
    fn end(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
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
        key_schema: &SchemaRef,
        value_schema: &SchemaRef,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: ?Sized + SerializeWithSchema,
        V: ?Sized + SerializeWithSchema,
    {
        match key.serialize_with_schema(key_schema, &mut KeySerializer) {
            Ok(val) => self.root.push_path(PathElement::Key(val))?,
            // Return early on this error. Something is wrong with the schema.
            Err(err) => return self.root.short_circuit(err),
        };
        key.serialize_with_schema(key_schema, &mut *self.root)?;
        value.serialize_with_schema(value_schema, &mut *self.root)?;
        self.root.pop_path()
    }

    #[inline]
    fn end(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// Converts a key value to a String so Keys can be represented as a path element.
struct KeySerializer;
impl Serializer for &mut KeySerializer {
    type Error = ValidationFailure;
    type Ok = String;
    type SerializeList = NoOpSerializer;
    type SerializeMap = NoOpSerializer;
    type SerializeStruct = NoOpSerializer;

    #[inline]
    fn write_struct(
        self,
        schema: &SchemaRef,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(ValidationFailure::InvalidKeyType(*schema.shape_type()))
    }

    #[inline]
    fn write_map(self, schema: &SchemaRef, _len: usize) -> Result<Self::SerializeMap, Self::Error> {
        Err(ValidationFailure::InvalidKeyType(*schema.shape_type()))
    }

    #[inline]
    fn write_list(
        self,
        schema: &SchemaRef,
        _len: usize,
    ) -> Result<Self::SerializeList, Self::Error> {
        Err(ValidationFailure::InvalidKeyType(*schema.shape_type()))
    }

    #[inline]
    fn write_boolean(self, schema: &SchemaRef, _value: bool) -> Result<Self::Ok, Self::Error> {
        Err(ValidationFailure::InvalidKeyType(*schema.shape_type()))
    }

    #[inline]
    fn write_byte(self, _schema: &SchemaRef, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    #[inline]
    fn write_short(self, _schema: &SchemaRef, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    #[inline]
    fn write_integer(self, _schema: &SchemaRef, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    #[inline]
    fn write_long(self, _schema: &SchemaRef, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    #[inline]
    fn write_float(self, schema: &SchemaRef, _value: f32) -> Result<Self::Ok, Self::Error> {
        Err(ValidationFailure::InvalidKeyType(*schema.shape_type()))
    }

    #[inline]
    fn write_double(self, schema: &SchemaRef, _value: f64) -> Result<Self::Ok, Self::Error> {
        Err(ValidationFailure::InvalidKeyType(*schema.shape_type()))
    }

    #[inline]
    fn write_big_integer(
        self,
        schema: &SchemaRef,
        _value: &BigInt,
    ) -> Result<Self::Ok, Self::Error> {
        Err(ValidationFailure::InvalidKeyType(*schema.shape_type()))
    }

    #[inline]
    fn write_big_decimal(
        self,
        schema: &SchemaRef,
        _value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        Err(ValidationFailure::InvalidKeyType(*schema.shape_type()))
    }

    #[inline]
    fn write_string(self, _schema: &SchemaRef, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    #[inline]
    fn write_blob(self, schema: &SchemaRef, _value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        Err(ValidationFailure::InvalidKeyType(*schema.shape_type()))
    }

    #[inline]
    fn write_timestamp(
        self,
        schema: &SchemaRef,
        _value: &Instant,
    ) -> Result<Self::Ok, Self::Error> {
        Err(ValidationFailure::InvalidKeyType(*schema.shape_type()))
    }

    #[inline]
    fn write_document(
        self,
        schema: &SchemaRef,
        _value: &Document,
    ) -> Result<Self::Ok, Self::Error> {
        Err(ValidationFailure::InvalidKeyType(*schema.shape_type()))
    }

    #[inline]
    fn write_null(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Err(ValidationFailure::InvalidKeyType(*schema.shape_type()))
    }

    #[inline]
    fn skip(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Err(ValidationFailure::InvalidKeyType(*schema.shape_type()))
    }
}

// Structures, maps, and lists cannot be used as map keys so these implementations will never actually be called.
struct NoOpSerializer;
impl ListSerializer for NoOpSerializer {
    type Error = ValidationFailure;
    type Ok = String;

    fn serialize_element<T>(
        &mut self,
        _element_schema: &SchemaRef,
        _value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
    {
        unreachable!()
    }

    fn end(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}
impl MapSerializer for NoOpSerializer {
    type Error = ValidationFailure;
    type Ok = String;

    fn serialize_entry<K, V>(
        &mut self,
        _key_schema: &SchemaRef,
        _value_schema: &SchemaRef,
        _key: &K,
        _value: &V,
    ) -> Result<(), Self::Error>
    where
        K: ?Sized + SerializeWithSchema,
        V: ?Sized + SerializeWithSchema,
    {
        unreachable!()
    }

    fn end(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}
impl StructSerializer for NoOpSerializer {
    type Error = ValidationFailure;
    type Ok = String;

    fn serialize_member<T>(
        &mut self,
        _member_schema: &SchemaRef,
        _value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
    {
        unreachable!()
    }

    fn end(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        unreachable!()
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
    fn serialize_member<T>(
        &mut self,
        member_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
    {
        self.root.push_path(member_schema)?;
        value.serialize_with_schema(member_schema, &mut *self.root)?;
        self.root.pop_path()
    }

    #[inline]
    fn end(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

//////////////////////////////////////////////////////////////////////////////
// ERRORS
//////////////////////////////////////////////////////////////////////////////

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
    pub(super) fn add(&mut self, path: &[PathElement], error: impl Into<Box<dyn ValidationError>>) {
        self.errors.push(ValidationErrorField::new(path, error));
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

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
    fn custom<T: Display>(_msg: T) -> Self {
        todo!()
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
    pub fn new(paths: &[PathElement], error: impl Into<Box<dyn ValidationError>>) -> Self {
        Self {
            paths: Vec::from(paths),
            error: error.into(),
        }
    }
}

/// Represents a [JsonPointer](https://datatracker.ietf.org/doc/html/rfc6901) path element.
///
/// For example, the `JsonPointer` `/field_a/1/field_b` would be represented as:
/// ```rust,ignore
/// use smithy4rs_core::serde::validate::{Path, PathItem};
/// let paths = vec![
///     Path::Schema(FIELD_A.clone()),
///     Path::Index(1),
///     Path::Schema(FIELD_B.clone())
/// ];
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum PathElement {
    Schema(SchemaRef),
    Index(usize),
    Key(String),
}
impl From<&SchemaRef> for PathElement {
    fn from(schema_ref: &SchemaRef) -> Self {
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
    #[error("Attempted to perform `@uniqueItem` check on float. This is invalid")]
    UniqueItemOnFloat,
    #[error("{0}")]
    Custom(String),
    #[error("Type: {0} is not a valid map key")]
    InvalidKeyType(ShapeType),
    #[error("Unsupported validation operation.")]
    Unsupported,
}
impl serializers::Error for ValidationFailure {
    fn custom<T: Display>(msg: T) -> Self {
        ValidationFailure::Custom(msg.to_string())
    }
}
impl ValidationError for ValidationFailure {}

#[derive(Error, Debug, PartialEq)]
pub enum SmithyConstraints {
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
}
impl ValidationError for SmithyConstraints {}

#[cfg(test)]
#[allow(clippy::type_complexity)]
mod tests {
    use std::sync::LazyLock;

    use indexmap::IndexMap;

    use super::*;
    use crate::{
        prelude::{INTEGER, STRING},
        schema::{Schema, ShapeId, StaticSchemaShape},
        serde::{
            ShapeBuilder,
            builders::{MaybeBuilt, Required},
            correction::{ErrorCorrection, ErrorCorrectionDefault},
            de::Deserializer,
            deserializers::DeserializeWithSchema,
        },
        traits,
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

    /// ==== Basic Shape Validations ====
    static LIST_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(
            ShapeId::from("com.example#List"),
            traits![LengthTrait::builder().max(3).build(), UniqueItemsTrait],
        )
        .put_member(
            "member",
            &STRING,
            traits![LengthTrait::builder().max(4).build()],
        )
        .build()
    });
    static MAP_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::map_builder(
            ShapeId::from("com.example#Map"),
            traits![LengthTrait::builder().max(2).build(), UniqueItemsTrait],
        )
        .put_member("key", &STRING, traits![PatternTrait::new("^[a-zA-Z]*$")])
        .put_member(
            "value",
            &STRING,
            traits![LengthTrait::builder().max(4).build()],
        )
        .build()
    });
    static BASIC_VALIDATION_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("test#ValidationStruct"), Vec::new())
            .put_member(
                "field_a",
                &STRING,
                traits![PatternTrait::new("^[a-zA-Z]*$")],
            )
            .put_member("field_b", &INTEGER, traits![])
            .put_member("field_list", &LIST_SCHEMA, traits![])
            .put_member("field_map", &MAP_SCHEMA, traits![])
            .build()
    });
    static FIELD_A: LazyLock<&SchemaRef> =
        LazyLock::new(|| BASIC_VALIDATION_SCHEMA.expect_member("field_a"));
    static FIELD_B: LazyLock<&SchemaRef> =
        LazyLock::new(|| BASIC_VALIDATION_SCHEMA.expect_member("field_b"));
    static FIELD_LIST: LazyLock<&SchemaRef> =
        LazyLock::new(|| BASIC_VALIDATION_SCHEMA.expect_member("field_list"));
    static FIELD_MAP: LazyLock<&SchemaRef> =
        LazyLock::new(|| BASIC_VALIDATION_SCHEMA.expect_member("field_map"));

    #[allow(dead_code)]
    pub struct SimpleStruct {
        field_a: String,
        field_b: Option<i32>,
        field_list: Option<Vec<String>>,
        field_map: Option<IndexMap<String, String>>,
    }
    pub struct SimpleStructBuilder {
        field_a: Required<String>,
        field_b: Option<i32>,
        field_list: Option<Vec<String>>,
        field_map: Option<IndexMap<String, String>>,
    }
    impl SimpleStructBuilder {
        pub fn field_a(mut self, value: String) -> Self {
            self.field_a = Required::Set(value);
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
    impl<'de> ShapeBuilder<'de, SimpleStruct> for SimpleStructBuilder {
        fn new() -> Self {
            Self {
                field_a: Required::Unset,
                field_b: None,
                field_list: None,
                field_map: None,
            }
        }
    }
    impl ErrorCorrection for SimpleStructBuilder {
        type Value = SimpleStruct;

        fn correct(self) -> Self::Value {
            SimpleStruct {
                field_a: self.field_a.get(),
                field_b: self.field_b,
                field_list: self.field_list,
                field_map: self.field_map,
            }
        }
    }
    impl SerializeWithSchema for SimpleStructBuilder {
        fn serialize_with_schema<S: Serializer>(
            &self,
            schema: &SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 8usize)?;
            ser.serialize_member_named("field_a", &FIELD_A, &self.field_a)?;
            ser.serialize_optional_member_named("field_b", &FIELD_B, &self.field_b)?;
            ser.serialize_optional_member_named("field_list", &FIELD_LIST, &self.field_list)?;
            ser.serialize_optional_member_named("field_list", &FIELD_MAP, &self.field_map)?;
            ser.end(schema)
        }
    }
    impl<'de> DeserializeWithSchema<'de> for SimpleStructBuilder {
        fn deserialize_with_schema<D>(
            _schema: &SchemaRef,
            _deserializer: &mut D,
        ) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
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

        let error_field_a = err.errors.first().unwrap();
        assert_eq!(
            error_field_a.paths,
            vec![PathElement::Schema(FIELD_A.clone())]
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
            vec![PathElement::Schema(FIELD_A.clone())]
        );
        assert_eq!(
            error_pattern.error.to_string(),
            "Value `field-a` did not conform to expected pattern `^[a-zA-Z]*$`".to_string()
        );

        let error_length = err.errors.get(1).unwrap();
        assert_eq!(
            error_length.paths,
            vec![
                PathElement::Schema(FIELD_LIST.clone()),
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
            vec![PathElement::Schema(FIELD_A.clone())]
        );
        assert_eq!(
            error_required.error.to_string(),
            "Field is Required.".to_string()
        );

        assert_eq!(
            error_length.paths,
            vec![
                PathElement::Schema(FIELD_LIST.clone()),
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
            vec![PathElement::Schema(FIELD_LIST.clone())]
        );
        assert_eq!(
            error_length.error.to_string(),
            "Size: 5 does not conform to @length constraint. Expected between 0 and 3.".to_string()
        );

        let error_unique = err.errors.get(1).unwrap();
        assert_eq!(
            error_unique.paths,
            vec![
                PathElement::Schema(FIELD_LIST.clone()),
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
            vec![PathElement::Schema(FIELD_MAP.clone())]
        );
        assert_eq!(
            error_length.error.to_string(),
            "Size: 3 does not conform to @length constraint. Expected between 0 and 2.".to_string()
        );

        let error_key = err.errors.get(1).unwrap();
        assert_eq!(
            error_key.paths,
            vec![
                PathElement::Schema(FIELD_MAP.clone()),
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
                PathElement::Schema(FIELD_MAP.clone()),
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
    static NESTED_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("test#ValidationStruct"), Vec::new())
            .put_member("field_c", &STRING, traits![PatternTrait::new("^[a-z]*$")])
            .build()
    });
    static FIELD_C: LazyLock<&SchemaRef> = LazyLock::new(|| NESTED_SCHEMA.expect_member("field_c"));

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
            serializer: S,
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
    impl<'de> DeserializeWithSchema<'de> for NestedStructBuilder {
        fn deserialize_with_schema<D>(
            _schema: &SchemaRef,
            _deserializer: &mut D,
        ) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            unimplemented!("We dont need to deserialize to test.")
        }
    }
    impl ErrorCorrectionDefault for NestedStruct {
        fn default() -> Self {
            NestedStructBuilder::new().correct()
        }
    }
    impl SerializeWithSchema for NestedStructBuilder {
        fn serialize_with_schema<S: Serializer>(
            &self,
            schema: &SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 1usize)?;
            ser.serialize_member(&FIELD_C, &self.field_c)?;
            ser.end(schema)
        }
    }
    impl<'de> ShapeBuilder<'de, NestedStruct> for NestedStructBuilder {
        fn new() -> Self {
            Self {
                field_c: Required::Unset,
            }
        }
    }
    impl ErrorCorrection for NestedStructBuilder {
        type Value = NestedStruct;

        fn correct(self) -> Self::Value {
            NestedStruct {
                field_c: self.field_c.get(),
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
    static FIELD_NESTED: LazyLock<&SchemaRef> =
        LazyLock::new(|| STRUCT_WITH_NESTED_SCHEMA.expect_member("field_nested"));
    static FIELD_NESTED_REQUIRED: LazyLock<&SchemaRef> =
        LazyLock::new(|| STRUCT_WITH_NESTED_SCHEMA.expect_member("field_nested_required"));

    #[allow(dead_code)]
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
    impl<'de> DeserializeWithSchema<'de> for StructWithNestedBuilder {
        fn deserialize_with_schema<D>(
            _schema: &SchemaRef,
            _deserializer: &mut D,
        ) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            unimplemented!("We dont need to deserialize to test.")
        }
    }
    impl ErrorCorrection for StructWithNestedBuilder {
        type Value = StructWithNested;

        fn correct(self) -> Self::Value {
            StructWithNested {
                field_nested: self.field_nested.correct(),
                field_required_nested: self.field_required_nested.get().correct(),
            }
        }
    }
    impl SerializeWithSchema for StructWithNestedBuilder {
        fn serialize_with_schema<S: Serializer>(
            &self,
            schema: &SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 2usize)?;
            ser.serialize_optional_member(&FIELD_NESTED, &self.field_nested)?;
            ser.serialize_member(&FIELD_NESTED_REQUIRED, &self.field_required_nested)?;
            ser.end(schema)
        }
    }
    impl<'de> ShapeBuilder<'de, StructWithNested> for StructWithNestedBuilder {
        fn new() -> Self {
            StructWithNestedBuilder {
                field_nested: None,
                field_required_nested: Required::Unset,
            }
        }
    }
    impl StructWithNestedBuilder {
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
        let _value = builder
            .field_nested_required_builder(builder_nested)
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
            .field_nested_required(built_nested)
            .build()
            .expect("Failed to build SimpleStruct");
    }

    #[test]
    fn nested_struct_fields_checked() {
        let builder_nested = NestedStructBuilder::new().field_c("dataWithCaps".to_string());
        let builder = StructWithNestedBuilder::new();
        let Some(err) = builder
            .field_nested_required_builder(builder_nested)
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
                PathElement::Schema(FIELD_NESTED_REQUIRED.clone()),
                PathElement::Schema(FIELD_C.clone())
            ]
        );
        assert_eq!(
            error_pattern.error.to_string(),
            "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string()
        );
    }

    // ==== Nested List Validations ====
    static LIST_OF_NESTED_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(
            ShapeId::from("com.example#ListOfNested"),
            traits![LengthTrait::builder().max(3).build()],
        )
        .put_member("member", &NESTED_SCHEMA, traits![])
        .build()
    });
    static LIST_OF_LIST_OF_NESTED: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(
            ShapeId::from("com.example#ListOfList"),
            traits![LengthTrait::builder().max(2).build()],
        )
        .put_member("member", &LIST_OF_NESTED_SCHEMA, traits![])
        .build()
    });
    static LIST_OF_LIST_OF_LIST_OF_NESTED: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(
            ShapeId::from("com.example#ListOfListOfList"),
            traits![LengthTrait::builder().max(2).build()],
        )
        .put_member("member", &LIST_OF_LIST_OF_NESTED, traits![])
        .build()
    });
    static STRUCT_WITH_NESTED_LIST_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("test#StructWithNestedList"), Vec::new())
            .put_member("field_nested_list", &LIST_OF_NESTED_SCHEMA, traits![])
            .put_member(
                "field_nested_list_required",
                &LIST_OF_NESTED_SCHEMA,
                traits![],
            )
            .put_member(
                "field_deeply_nested_list",
                &LIST_OF_LIST_OF_LIST_OF_NESTED,
                traits![],
            )
            .build()
    });
    static FIELD_NESTED_LIST: LazyLock<&SchemaRef> =
        LazyLock::new(|| STRUCT_WITH_NESTED_LIST_SCHEMA.expect_member("field_nested_list"));
    static FIELD_NESTED_LIST_REQUIRED: LazyLock<&SchemaRef> = LazyLock::new(|| {
        STRUCT_WITH_NESTED_LIST_SCHEMA.expect_member("field_nested_list_required")
    });
    static FIELD_DEEPLY_NESTED_LIST: LazyLock<&SchemaRef> =
        LazyLock::new(|| STRUCT_WITH_NESTED_LIST_SCHEMA.expect_member("field_deeply_nested_list"));

    #[allow(dead_code)]
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
        field_required_nested_list:
            Required<MaybeBuilt<Vec<NestedStruct>, Vec<NestedStructBuilder>>>,
        field_deeply_nested_list:
            Option<MaybeBuilt<Vec<Vec<Vec<NestedStruct>>>, Vec<Vec<Vec<NestedStructBuilder>>>>>,
    }
    impl<'de> DeserializeWithSchema<'de> for StructWithNestedListsBuilder {
        fn deserialize_with_schema<D>(
            _schema: &SchemaRef,
            _deserializer: &mut D,
        ) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            unimplemented!("We dont need to deserialize for testing.")
        }
    }
    impl SerializeWithSchema for StructWithNestedListsBuilder {
        fn serialize_with_schema<S: Serializer>(
            &self,
            schema: &SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 3usize)?;
            ser.serialize_optional_member(&FIELD_NESTED_LIST, &self.field_nested_list)?;
            ser.serialize_member(
                &FIELD_NESTED_LIST_REQUIRED,
                &self.field_required_nested_list,
            )?;
            ser.serialize_optional_member(
                &FIELD_DEEPLY_NESTED_LIST,
                &self.field_deeply_nested_list,
            )?;
            ser.end(schema)
        }
    }
    impl ErrorCorrection for StructWithNestedListsBuilder {
        type Value = StructWithNestedLists;

        fn correct(self) -> Self::Value {
            StructWithNestedLists {
                field_nested_list: self.field_nested_list.correct(),
                field_required_nested_list: self.field_required_nested_list.get().correct(),
                field_deeply_nested_list: self.field_deeply_nested_list.correct(),
            }
        }
    }
    impl<'de> ShapeBuilder<'de, StructWithNestedLists> for StructWithNestedListsBuilder {
        fn new() -> Self {
            StructWithNestedListsBuilder {
                field_nested_list: None,
                field_required_nested_list: Required::Unset,
                field_deeply_nested_list: None,
            }
        }
    }
    impl StructWithNestedListsBuilder {
        pub fn field_require_nested_list(mut self, values: Vec<NestedStruct>) -> Self {
            self.field_required_nested_list = Required::Set(MaybeBuilt::Struct(values));
            self
        }

        pub fn field_required_nested_list_builder(
            mut self,
            values: Vec<NestedStructBuilder>,
        ) -> Self {
            self.field_required_nested_list = Required::Set(MaybeBuilt::Builder(values));
            self
        }

        pub fn field_deeply_nested_list_builder(
            mut self,
            values: Vec<Vec<Vec<NestedStructBuilder>>>,
        ) -> Self {
            self.field_deeply_nested_list = Some(MaybeBuilt::Builder(values));
            self
        }
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
            .field_require_nested_list(nested_list)
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
            vec![PathElement::Schema(FIELD_NESTED_LIST_REQUIRED.clone())]
        );
        assert_eq!(
            error_length.error.to_string(),
            "Size: 4 does not conform to @length constraint. Expected between 0 and 3.".to_string()
        );

        let error_pattern = err.errors.get(1).unwrap();
        assert_eq!(
            error_pattern.paths,
            vec![
                PathElement::Schema(FIELD_NESTED_LIST_REQUIRED.clone()),
                PathElement::Index(2),
                PathElement::Schema(FIELD_C.clone())
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
                PathElement::Schema(FIELD_DEEPLY_NESTED_LIST.clone()),
                PathElement::Index(0),
                PathElement::Index(0),
                PathElement::Index(0),
                PathElement::Schema(FIELD_C.clone())
            ]
        );
        assert_eq!(
            error_pattern.error.to_string(),
            "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string()
        );
    }

    // ==== `@uniqueItem` Validations ====

    static SET_OF_STRUCT: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(
            ShapeId::from("com.example#SetOfStruct"),
            traits![UniqueItemsTrait],
        )
        .put_member("member", &NESTED_SCHEMA, traits![])
        .build()
    });
    static SET_OF_STRING: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(
            ShapeId::from("com.example#SetOfString"),
            traits![UniqueItemsTrait],
        )
        .put_member("member", &STRING, traits![])
        .build()
    });
    static LIST_OF_INT: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(ShapeId::from("com.example#ListOfInt"), traits![])
            .put_member("member", &INTEGER, traits![])
            .build()
    });
    static SET_OF_LIST: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(
            ShapeId::from("com.example#SetOfList"),
            traits![UniqueItemsTrait],
        )
        .put_member("member", &LIST_OF_INT, traits![])
        .build()
    });
    static MAP_OF_INT: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::map_builder(ShapeId::from("com.example#MapOfInt"), traits![])
            .put_member("key", &STRING, traits![])
            .put_member("value", &INTEGER, traits![])
            .build()
    });
    static SET_OF_MAP: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::list_builder(
            ShapeId::from("com.example#SetOfMap"),
            traits![UniqueItemsTrait],
        )
        .put_member("member", &MAP_OF_INT, traits![])
        .build()
    });
    static STRUCT_WITH_SETS: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("test#StructWithSets"), Vec::new())
            .put_member("set_of_struct", &SET_OF_STRUCT, traits![])
            .put_member("set_of_simple", &SET_OF_STRING, traits![])
            .put_member("set_of_list", &SET_OF_LIST, traits![])
            .put_member("set_of_map", &SET_OF_MAP, traits![])
            .build()
    });
    static FIELD_SET_OF_STRUCT: LazyLock<&SchemaRef> =
        LazyLock::new(|| STRUCT_WITH_SETS.expect_member("set_of_struct"));
    static FIELD_SET_OF_SIMPLE: LazyLock<&SchemaRef> =
        LazyLock::new(|| STRUCT_WITH_SETS.expect_member("set_of_simple"));
    static FIELD_SET_OF_LIST: LazyLock<&SchemaRef> =
        LazyLock::new(|| STRUCT_WITH_SETS.expect_member("set_of_list"));
    static FIELD_SET_OF_MAP: LazyLock<&SchemaRef> =
        LazyLock::new(|| STRUCT_WITH_SETS.expect_member("set_of_map"));

    #[allow(dead_code)]
    struct StructWithSets {
        set_of_struct: Option<Vec<NestedStruct>>,
        set_of_simple: Option<Vec<String>>,
        set_of_list: Option<Vec<Vec<i32>>>,
        set_of_map: Option<Vec<IndexMap<String, i32>>>,
    }
    impl StaticSchemaShape for StructWithSets {
        fn schema() -> &'static SchemaRef {
            &STRUCT_WITH_SETS
        }
    }

    struct StructWithSetsBuilder {
        set_of_struct: Option<MaybeBuilt<Vec<NestedStruct>, Vec<NestedStructBuilder>>>,
        set_of_simple: Option<Vec<String>>,
        set_of_list: Option<Vec<Vec<i32>>>,
        set_of_map: Option<Vec<IndexMap<String, i32>>>,
    }
    impl<'de> DeserializeWithSchema<'de> for StructWithSetsBuilder {
        fn deserialize_with_schema<D>(
            _schema: &SchemaRef,
            _deserializer: &mut D,
        ) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            unimplemented!("We dont need to deserialize for testing.")
        }
    }
    impl SerializeWithSchema for StructWithSetsBuilder {
        fn serialize_with_schema<S: Serializer>(
            &self,
            schema: &SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 4usize)?;
            ser.serialize_optional_member(&FIELD_SET_OF_STRUCT, &self.set_of_struct)?;
            ser.serialize_optional_member(&FIELD_SET_OF_SIMPLE, &self.set_of_simple)?;
            ser.serialize_optional_member(&FIELD_SET_OF_LIST, &self.set_of_list)?;
            ser.serialize_optional_member(&FIELD_SET_OF_MAP, &self.set_of_map)?;
            ser.end(schema)
        }
    }
    impl ErrorCorrection for StructWithSetsBuilder {
        type Value = StructWithSets;

        fn correct(self) -> Self::Value {
            StructWithSets {
                set_of_struct: self.set_of_struct.correct(),
                set_of_simple: self.set_of_simple,
                set_of_list: self.set_of_list,
                set_of_map: self.set_of_map,
            }
        }
    }
    impl<'de> ShapeBuilder<'de, StructWithSets> for StructWithSetsBuilder {
        fn new() -> Self {
            StructWithSetsBuilder {
                set_of_struct: None,
                set_of_simple: None,
                set_of_list: None,
                set_of_map: None,
            }
        }
    }
    impl StructWithSetsBuilder {
        pub fn set_of_struct_builder(mut self, values: Vec<NestedStructBuilder>) -> Self {
            self.set_of_struct = Some(MaybeBuilt::Builder(values));
            self
        }

        pub fn set_of_simple(mut self, values: Vec<String>) -> Self {
            self.set_of_simple = Some(values);
            self
        }

        pub fn set_of_list(mut self, values: Vec<Vec<i32>>) -> Self {
            self.set_of_list = Some(values);
            self
        }

        pub fn set_of_map(mut self, values: Vec<IndexMap<String, i32>>) -> Self {
            self.set_of_map = Some(values);
            self
        }
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
        // println!("{:#?}", error_unique_struct.paths.iter().map(|p| p.id()));

        let error_unique_struct = err.errors.first().unwrap();
        assert_eq!(
            error_unique_struct.paths,
            vec![
                PathElement::Schema(FIELD_SET_OF_STRUCT.clone()),
                PathElement::Index(2)
            ]
        );

        let error_unique_simple = err.errors.get(1).unwrap();
        assert_eq!(
            error_unique_simple.paths,
            vec![
                PathElement::Schema(FIELD_SET_OF_SIMPLE.clone()),
                PathElement::Index(2)
            ]
        );

        let error_unique_simple = err.errors.get(1).unwrap();
        assert_eq!(
            error_unique_simple.paths,
            vec![
                PathElement::Schema(FIELD_SET_OF_SIMPLE.clone()),
                PathElement::Index(2)
            ]
        );

        let error_unique_list = err.errors.get(2).unwrap();
        assert_eq!(
            error_unique_list.paths,
            vec![
                PathElement::Schema(FIELD_SET_OF_LIST.clone()),
                PathElement::Index(2)
            ]
        );

        let error_unique_map = err.errors.get(3).unwrap();
        assert_eq!(
            error_unique_map.paths,
            vec![
                PathElement::Schema(FIELD_SET_OF_MAP.clone()),
                PathElement::Index(2)
            ]
        );
    }

    // ==== Nested Map Validations ====
    static MAP_OF_NESTED_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::map_builder(
            ShapeId::from("com.example#MapOfNested"),
            traits![LengthTrait::builder().max(2).build()],
        )
        .put_member("key", &STRING, traits![])
        .put_member("value", &NESTED_SCHEMA, traits![])
        .build()
    });
    static MAP_OF_MAP_OF_NESTED: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::map_builder(
            ShapeId::from("com.example#MapOfMap"),
            traits![LengthTrait::builder().max(2).build()],
        )
        .put_member("key", &STRING, traits![])
        .put_member("value", &MAP_OF_NESTED_SCHEMA, traits![])
        .build()
    });
    static MAP_OF_MAP_OF_MAP_OF_NESTED: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::map_builder(
            ShapeId::from("com.example#MapOfMapOfMap"),
            traits![LengthTrait::builder().max(2).build()],
        )
        .put_member("key", &STRING, traits![])
        .put_member("value", &MAP_OF_MAP_OF_NESTED, traits![])
        .build()
    });
    static STRUCT_WITH_NESTED_MAP_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("test#StructWithNestedMap"), Vec::new())
            .put_member("field_nested_map", &MAP_OF_NESTED_SCHEMA, traits![])
            .put_member(
                "field_nested_map_required",
                &MAP_OF_NESTED_SCHEMA,
                traits![],
            )
            .put_member(
                "field_deeply_nested_map",
                &MAP_OF_MAP_OF_MAP_OF_NESTED,
                traits![],
            )
            .build()
    });
    static FIELD_NESTED_MAP: LazyLock<&SchemaRef> =
        LazyLock::new(|| STRUCT_WITH_NESTED_MAP_SCHEMA.expect_member("field_nested_map"));
    static FIELD_NESTED_MAP_REQUIRED: LazyLock<&SchemaRef> =
        LazyLock::new(|| STRUCT_WITH_NESTED_MAP_SCHEMA.expect_member("field_nested_map_required"));
    static FIELD_DEEPLY_NESTED_MAP: LazyLock<&SchemaRef> =
        LazyLock::new(|| STRUCT_WITH_NESTED_MAP_SCHEMA.expect_member("field_deeply_nested_map"));

    #[allow(dead_code)]
    struct StructWithNestedMaps {
        field_nested_map: Option<IndexMap<String, NestedStruct>>,
        field_nested_map_required: IndexMap<String, NestedStruct>,
        field_deeply_nested_map:
            Option<IndexMap<String, IndexMap<String, IndexMap<String, NestedStruct>>>>,
    }

    impl StaticSchemaShape for StructWithNestedMaps {
        fn schema() -> &'static SchemaRef {
            &STRUCT_WITH_NESTED_MAP_SCHEMA
        }
    }

    struct StructWithNestedMapsBuilder {
        field_nested_map: Option<
            MaybeBuilt<IndexMap<String, NestedStruct>, IndexMap<String, NestedStructBuilder>>,
        >,
        field_nested_map_required: Required<
            MaybeBuilt<IndexMap<String, NestedStruct>, IndexMap<String, NestedStructBuilder>>,
        >,
        field_deeply_nested_map: Option<
            MaybeBuilt<
                IndexMap<String, IndexMap<String, IndexMap<String, NestedStruct>>>,
                IndexMap<String, IndexMap<String, IndexMap<String, NestedStructBuilder>>>,
            >,
        >,
    }
    impl<'de> DeserializeWithSchema<'de> for StructWithNestedMapsBuilder {
        fn deserialize_with_schema<D>(
            _schema: &SchemaRef,
            _deserializer: &mut D,
        ) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            unimplemented!("We dont need to deserialize for testing.")
        }
    }
    impl SerializeWithSchema for StructWithNestedMapsBuilder {
        fn serialize_with_schema<S: Serializer>(
            &self,
            schema: &SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 3usize)?;
            ser.serialize_optional_member(&FIELD_NESTED_MAP, &self.field_nested_map)?;
            ser.serialize_member(&FIELD_NESTED_MAP_REQUIRED, &self.field_nested_map_required)?;
            ser.serialize_optional_member(&FIELD_DEEPLY_NESTED_MAP, &self.field_deeply_nested_map)?;
            ser.end(schema)
        }
    }
    impl ErrorCorrection for StructWithNestedMapsBuilder {
        type Value = StructWithNestedMaps;

        fn correct(self) -> Self::Value {
            StructWithNestedMaps {
                field_nested_map: self.field_nested_map.correct(),
                field_nested_map_required: self.field_nested_map_required.get().correct(),
                field_deeply_nested_map: self.field_deeply_nested_map.correct(),
            }
        }
    }
    impl<'de> ShapeBuilder<'de, StructWithNestedMaps> for StructWithNestedMapsBuilder {
        fn new() -> Self {
            StructWithNestedMapsBuilder {
                field_nested_map: None,
                field_nested_map_required: Required::Unset,
                field_deeply_nested_map: None,
            }
        }
    }
    impl StructWithNestedMapsBuilder {
        pub fn field_require_nested_map(mut self, values: IndexMap<String, NestedStruct>) -> Self {
            self.field_nested_map_required = Required::Set(MaybeBuilt::Struct(values));
            self
        }

        pub fn field_required_nested_map_builder(
            mut self,
            values: IndexMap<String, NestedStructBuilder>,
        ) -> Self {
            self.field_nested_map_required = Required::Set(MaybeBuilt::Builder(values));
            self
        }

        pub fn field_deeply_nested_map_builder(
            mut self,
            values: IndexMap<String, IndexMap<String, IndexMap<String, NestedStructBuilder>>>,
        ) -> Self {
            self.field_deeply_nested_map = Some(MaybeBuilt::Builder(values));
            self
        }
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
            .field_required_nested_map_builder(nested_map)
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
            .field_require_nested_map(nested_map)
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
        let Some(err) = builder
            .field_required_nested_map_builder(nested_map)
            .build()
            .err()
        else {
            panic!("Expected an error");
        };

        assert_eq!(err.errors.len(), 2);

        let error_length = err.errors.first().unwrap();
        assert_eq!(
            error_length.paths,
            vec![PathElement::Schema(FIELD_NESTED_MAP_REQUIRED.clone())]
        );
        assert_eq!(
            error_length.error.to_string(),
            "Size: 3 does not conform to @length constraint. Expected between 0 and 2.".to_string()
        );

        let error_pattern = err.errors.get(1).unwrap();
        assert_eq!(
            error_pattern.paths,
            vec![
                PathElement::Schema(FIELD_NESTED_MAP_REQUIRED.clone()),
                PathElement::Key("b".to_string()),
                PathElement::Schema(FIELD_C.clone())
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
            .field_required_nested_map_builder(nested_map)
            .field_deeply_nested_map_builder(deep_nesting)
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
                PathElement::Schema(FIELD_DEEPLY_NESTED_MAP.clone()),
                PathElement::Key("a".to_string()),
                PathElement::Key("a".to_string()),
                PathElement::Key("a".to_string()),
                PathElement::Schema(FIELD_C.clone())
            ]
        );
        assert_eq!(
            error_pattern.error.to_string(),
            "Value `dataWithCaps` did not conform to expected pattern `^[a-z]*$`".to_string()
        );
    }
}
