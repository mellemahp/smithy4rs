//! Schema-guided deserialization of data into a Smithy Shape
//!
//! This module provides traits for deserializing data formats (JSON, CBOR, etc.)
//! into Smithy shapes, guided by schemas.
//!
//! # Architecture
//!
//! The deserialization system uses a reader-based pattern for compound types:
//!
//! - [`Deserializer`]: Entry point that reads primitives and creates readers
//! - [`StructReader`]: Iterates struct members with `read_member()` / `read_value()`
//! - [`ListReader`]: Iterates list elements with `read_element()`
//! - [`MapReader`]: Iterates map entries with `read_key()` / `read_value()`
//!
//! This design (inspired by `serde`) separates iteration from value reading,
//! allowing callers to control the deserialization flow.

use std::{error::Error as StdError, fmt::Display};

use crate::{
    BigDecimal, BigInt, ByteBuffer, IndexMap, Instant,
    schema::{Document, Schema, SchemaShape, StaticSchemaShape},
};

// ============================================================================
// Error Trait
// ============================================================================

/// Error trait for deserialization errors.
pub trait Error: Sized + StdError {
    /// Create a custom error message
    fn custom<T: Display>(msg: T) -> Self;
}

// ============================================================================
// Reader Traits
// ============================================================================

/// Reader for struct members.
///
/// Iterates through known struct members using a two-step pattern:
/// 1. Call `read_member()` to get the next known member's schema
/// 2. Call `read_value()` to deserialize the value, or `skip_value()` to skip it
///
/// Unknown fields are skipped internally by the reader.
///
/// # Example
///
/// ```ignore
/// let mut reader = deserializer.read_struct(schema)?;
///
/// let mut name: Option<String> = None;
/// let mut age: Option<i32> = None;
///
/// while let Some(member_schema) = reader.read_member()? {
///     if &member_schema == &*MEMBER_NAME {
///         name = Some(reader.read_value(&member_schema)?);
///         continue;
///     }
///     if &member_schema == &*MEMBER_AGE {
///         age = Some(reader.read_value(&member_schema)?);
///         continue;
///     }
///     reader.skip_value()?;
/// }
/// ```
pub trait StructReader<'de> {
    /// The error type returned by reader operations.
    type Error: Error;

    /// Read the next known member, returning its schema.
    ///
    /// Unknown fields are skipped internally by the reader.
    /// Returns `None` when all fields have been read.
    ///
    /// After this returns `Some`, you must call either `read_value()` or
    /// `skip_value()` before calling `read_member()` again.
    fn read_member<'a>(&mut self, schema: &'a Schema) -> Result<Option<&'a Schema>, Self::Error>;

    /// Read the current member's value.
    ///
    /// Must be called after `read_member()` returns `Some`.
    fn read_value<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<T, Self::Error>;

    /// Skip the current member's value.
    ///
    /// Use this for unknown fields or fields you don't need.
    fn skip_value(&mut self) -> Result<(), Self::Error>;

    /// Hint about the number of remaining members, if known.
    fn size_hint(&self) -> Option<usize> {
        None
    }
}

/// Reader for list elements.
///
/// Iterates through list elements, deserializing each one.
///
/// # Example
///
/// ```ignore
/// let element_schema = schema.get_member("member").unwrap();
/// let mut reader = deserializer.read_list()?;
///
/// let mut result = Vec::with_capacity(reader.size_hint().unwrap_or(0));
/// while let Some(element) = reader.read_element(element_schema)? {
///     result.push(element);
/// }
/// ```
pub trait ListReader<'de> {
    /// The error type returned by reader operations.
    type Error: Error;

    /// Read the next element, or `None` if the list is exhausted.
    fn read_element<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<Option<T>, Self::Error>;

    /// Hint about the number of remaining elements, if known.
    fn size_hint(&self) -> Option<usize> {
        None
    }
}

/// Reader for map entries.
///
/// Iterates through map entries using a two-step pattern:
/// 1. Call `read_key()` to get the next key
/// 2. Call `read_value()` to deserialize the value, or `skip_value()` to skip it
///
/// # Example
///
/// ```ignore
/// let value_schema = schema.get_member("value").unwrap();
/// let mut reader = deserializer.read_map()?;
///
/// let mut result = HashMap::with_capacity(reader.size_hint().unwrap_or(0));
/// while let Some(key) = reader.read_key()? {
///     let value = reader.read_value(value_schema)?;
///     result.insert(key, value);
/// }
/// ```
pub trait MapReader<'de> {
    /// The error type returned by reader operations.
    type Error: Error;

    /// Read the next key, or `None` if no more entries.
    ///
    /// After this returns `Some`, you must call either `read_value()` or
    /// `skip_value()` before calling `read_key()` again.
    // TODO(optimization): Do we return Cow<'de, str> for flexibility and reduce allocation?
    fn read_key(&mut self) -> Result<Option<String>, Self::Error>;

    /// Read the current entry's value.
    ///
    /// Must be called after `read_key()` returns `Some`.
    fn read_value<V: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<V, Self::Error>;

    /// Skip the current entry's value.
    fn skip_value(&mut self) -> Result<(), Self::Error>;

    /// Hint about the number of remaining entries, if known.
    fn size_hint(&self) -> Option<usize> {
        None
    }
}

// ============================================================================
// Core Deserializer Trait
// ============================================================================

/// A `Deserializer` reads data from an input source, guided by Smithy schemas.
///
/// This trait mirrors the [`Serializer`](crate::serde::se::Serializer) trait, providing
/// schema-guided deserialization for all Smithy data types. It uses a reader pattern
/// for compound types (structs, lists, maps) where the deserializer returns reader
/// objects that allow iterating through members/elements/entries.
pub trait Deserializer<'de>: Sized {
    /// The error type that can be returned if deserialization fails.
    type Error: Error;

    /// The reader type for structs, parameterized by the borrow lifetime.
    type StructReader: StructReader<'de, Error = Self::Error>;

    /// The reader type for lists, parameterized by the borrow lifetime.
    type ListReader: ListReader<'de, Error = Self::Error>;

    /// The reader type for maps, parameterized by the borrow lifetime.
    type MapReader: MapReader<'de, Error = Self::Error>;

    // === Primitive deserialization ===

    /// Read a boolean value
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a bool.
    fn read_bool(self, _schema: &Schema) -> Result<bool, Self::Error> {
        Err(Error::custom(
            "read_bool is not supported by this deserializer",
        ))
    }

    /// Read a byte (`i8`)
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `byte`.
    fn read_byte(self, _schema: &Schema) -> Result<i8, Self::Error> {
        Err(Error::custom(
            "read_byte is not supported by this deserializer",
        ))
    }

    /// Read a short (`i16`)
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `short`.
    fn read_short(self, _schema: &Schema) -> Result<i16, Self::Error> {
        Err(Error::custom(
            "read_short is not supported by this deserializer",
        ))
    }

    /// Read an integer (`i32`)
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as an `integer`.
    fn read_integer(self, _schema: &Schema) -> Result<i32, Self::Error> {
        Err(Error::custom(
            "read_integer is not supported by this deserializer",
        ))
    }

    /// Read a long (i64)
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `long`.
    fn read_long(self, _schema: &Schema) -> Result<i64, Self::Error> {
        Err(Error::custom(
            "read_long is not supported by this deserializer",
        ))
    }

    /// Read a float (`f32`)
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `float`.
    fn read_float(self, _schema: &Schema) -> Result<f32, Self::Error> {
        Err(Error::custom(
            "read_float is not supported by this deserializer",
        ))
    }

    /// Read a double (`f64`)
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `double`.
    fn read_double(self, _schema: &Schema) -> Result<f64, Self::Error> {
        Err(Error::custom(
            "read_double is not supported by this deserializer",
        ))
    }

    /// Read a big integer
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `bigInteger`.
    fn read_big_integer(self, _schema: &Schema) -> Result<BigInt, Self::Error> {
        Err(Error::custom(
            "read_big_integer is not supported by this deserializer",
        ))
    }

    /// Read a big decimal
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `bigDecimal`.
    fn read_big_decimal(self, _schema: &Schema) -> Result<BigDecimal, Self::Error> {
        Err(Error::custom(
            "read_big_decimal is not supported by this deserializer",
        ))
    }

    /// Read a string
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `string`.
    // TODO(performance): If we had associated type `type Str: AsRef<str>`, deserializers could
    // return borrowed strings (`&'de str`), avoiding exrta allocations.
    // Callers would use `.as_ref()` for borrowed access or `.to_owned()` where needed.
    fn read_string(self, _schema: &Schema) -> Result<String, Self::Error> {
        Err(Error::custom(
            "read_string is not supported by this deserializer",
        ))
    }

    /// Read a blob
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `blob`.
    fn read_blob(self, _schema: &Schema) -> Result<ByteBuffer, Self::Error> {
        Err(Error::custom(
            "read_blob is not supported by this deserializer",
        ))
    }

    /// Read a timestamp
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `timestamp`.
    fn read_timestamp(self, _schema: &Schema) -> Result<Instant, Self::Error> {
        Err(Error::custom(
            "read_timestamp is not supported by this deserializer",
        ))
    }

    /// Read data as untyped [`Document`]
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `document`.
    fn read_document(self, _schema: &Schema) -> Result<Box<dyn Document>, Self::Error> {
        Err(Error::custom(
            "read_document is not supported by this deserializer",
        ))
    }

    // === Compound types ===

    // TODO(unknown members): Union unknown types are not well supported by
    //                        the current impl.
    /// Begin reading a struct, returning a reader for its members.
    ///
    /// The reader borrows from the deserializer and allows iterating through struct
    /// members. The schema is passed so the reader can resolve wire names/tags to
    /// member schemas internally. Call `read_member()` to get the next known member's
    /// schema, then `read_value()` to deserialize the value or `skip_value()` to skip it.
    ///
    /// # Example (generated code)
    ///
    /// ```rust,ignore
    /// impl<'de> DeserializeWithSchema<'de> for CitySummary {
    ///     fn deserialize<D: Deserializer<'de>>(
    ///         schema: &Schema,
    ///         deserializer: &mut D
    ///     ) -> Result<Self, D::Error> {
    ///         let mut reader = deserializer.read_struct(schema)?;
    ///
    ///         let mut city_id: Option<String> = None;
    ///         let mut name: Option<String> = None;
    ///
    ///         while let Some(member_schema) = reader.read_member()? {
    ///             if &member_schema == &*MEMBER_CITY_ID {
    ///                 city_id = Some(reader.read_value(&member_schema)?);
    ///                 continue;
    ///             }
    ///             if &member_schema == &*MEMBER_NAME {
    ///                 name = Some(reader.read_value(&member_schema)?);
    ///                 continue;
    ///             }
    ///             reader.skip_value()?;
    ///         }
    ///
    ///         Ok(CitySummary {
    ///             city_id: city_id.ok_or_else(|| Error::missing_field("cityId"))?,
    ///             name: name.ok_or_else(|| Error::missing_field("name"))?,
    ///         })
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    /// Returns [`Error`] if the struct could not be started (e.g., expected `{`).
    fn read_struct(self, _schema: &Schema) -> Result<Self::StructReader, Self::Error> {
        Err(Error::custom(
            "read_struct is not supported by this deserializer",
        ))
    }

    /// Begin reading a list, returning a reader for its elements.
    ///
    /// The reader borrows from the deserializer and allows iterating through list
    /// elements by calling `read_element()` which returns `None` when exhausted.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let element_schema = schema.get_member("member").unwrap();
    /// let mut reader = deserializer.read_list()?;
    ///
    /// let mut vec = Vec::new();
    /// while let Some(elem) = reader.read_element(element_schema)? {
    ///     vec.push(elem);
    /// }
    /// ```
    ///
    /// # Errors
    /// Returns [`Error`] if the list could not be started (e.g., expected `[`).
    fn read_list(self, _schema: &Schema) -> Result<Self::ListReader, Self::Error> {
        Err(Error::custom(
            "read_list is not supported by this deserializer",
        ))
    }

    /// Begin reading a map, returning a reader for its entries.
    ///
    /// The reader borrows from the deserializer and allows iterating through map
    /// entries using a two-step pattern: call `read_key()` to get the next key,
    /// then `read_value()` to deserialize the value or `skip_value()` to skip it.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let value_schema = schema.get_member("value").unwrap();
    /// let mut reader = deserializer.read_map()?;
    ///
    /// let mut map = IndexMap::new();
    /// while let Some(key) = reader.read_key()? {
    ///     let value = reader.read_value(value_schema)?;
    ///     map.insert(key, value);
    /// }
    /// ```
    ///
    /// # Errors
    /// Returns [`Error`] if the map could not be started (e.g., expected `{`).
    fn read_map(self, _schema: &Schema) -> Result<Self::MapReader, Self::Error> {
        Err(Error::custom(
            "read_map is not supported by this deserializer",
        ))
    }

    // === Null handling ===

    /// Check if the next value is null without consuming it.
    #[allow(clippy::wrong_self_convention)]
    fn is_null(&mut self) -> bool {
        false
    }

    /// Read a null value.
    ///
    /// # Errors
    /// Returns [`Error`] if an element could not be read as `null`/empty value.
    fn read_null(self) -> Result<(), Self::Error> {
        Err(Error::custom(
            "read_null is not supported by this deserializer",
        ))
    }
}

/// Deserialize a shape with its pre-defined schema.
///
/// This trait provides an automatic, blanket implementation for all shapes
/// with both a [`SchemaShape`], and [`DeserializeWithSchema`] implementation.
///
pub trait DeserializableShape<'de>: SchemaShape + DeserializeWithSchema<'de> {
    /// Deserialize a shape with its pre-defined schema
    ///
    ///
    /// # Errors
    /// Returns [`Error`] if data from the `Deserializer` could not be read into
    /// this shape type.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>;
}

impl<'de, T: StaticSchemaShape + DeserializeWithSchema<'de>> DeserializableShape<'de> for T {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::deserialize_with_schema(Self::schema(), deserializer)
    }
}

// ============================================================================
// DeserializeWithSchema Trait
// ============================================================================

/// A data structure that can be deserialized from any data format supported
/// by `smithy4rs`, guided by a schema.
///
/// This trait mirrors [`SerializeWithSchema`](crate::serde::se::SerializeWithSchema)
/// on the serialization side.
pub trait DeserializeWithSchema<'de>: Sized {
    /// Deserialize this value from the given deserializer using the provided schema.
    ///
    /// The deserializer is taken by mutable reference. Readers created by compound
    /// methods borrow from the deserializer.
    ///
    /// # Errors
    /// Returns [`Error`] if data from the `Deserializer` could not be read into
    /// this shape type. This could be due to either a schema or data mismatch.
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

// ============================================================================
// Implementations for standard types
// ============================================================================

// === Primitives ===

impl<'de> DeserializeWithSchema<'de> for bool {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_bool(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for i8 {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_byte(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for i16 {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_short(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for i32 {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_integer(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for i64 {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_long(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for f32 {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_float(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for f64 {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_double(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for BigInt {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_big_integer(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for BigDecimal {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_big_decimal(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for String {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_string(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for ByteBuffer {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_blob(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for Instant {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_timestamp(schema)
    }
}

// === Vec<T> (list) ===

impl<'de, T> DeserializeWithSchema<'de> for Vec<T>
where
    T: DeserializeWithSchema<'de>,
{
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let element_schema = schema
            .get_member("member")
            .ok_or_else(|| Error::custom("list schema missing member"))?;

        let mut reader = deserializer.read_list(schema)?;

        let mut vec = reader
            .size_hint()
            .map_or_else(Vec::new, |size| Vec::with_capacity(size));

        while let Some(elem) = reader.read_element(element_schema)? {
            vec.push(elem);
        }

        Ok(vec)
    }
}

// === IndexMap<K, V> (map) ===

// TODO(maps): Support non-string keys
impl<'de, V> DeserializeWithSchema<'de> for IndexMap<String, V>
where
    V: DeserializeWithSchema<'de>,
{
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value_schema = schema
            .get_member("value")
            .ok_or_else(|| Error::custom("map schema missing value"))?;

        let mut reader = deserializer.read_map(schema)?;

        let mut map = reader
            .size_hint()
            .map_or_else(IndexMap::new, |size| IndexMap::with_capacity(size));

        while let Some(key) = reader.read_key()? {
            let value = reader.read_value(value_schema)?;
            map.insert(key, value);
        }

        Ok(map)
    }
}

// === Option<T> ===

impl<'de, T> DeserializeWithSchema<'de> for Option<T>
where
    T: DeserializeWithSchema<'de>,
{
    fn deserialize_with_schema<D>(schema: &Schema, mut deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_null() {
            deserializer.read_null()?;
            Ok(None)
        } else {
            T::deserialize_with_schema(schema, deserializer).map(Some)
        }
    }
}

impl<'de, T> DeserializeWithSchema<'de> for Box<T>
where
    T: DeserializeWithSchema<'de>,
{
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize_with_schema(schema, deserializer).map(Box::new)
    }
}
