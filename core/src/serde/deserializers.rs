//! Schema-guided deserialization of data into a Smithy Shape
//!
//! TODO(docs): Implementation docs
use std::{error::Error as StdError, fmt::Display};

use crate::{
    BigDecimal, BigInt, ByteBuffer, IndexMap, Instant,
    schema::{Document, SchemaRef, SchemaShape, StaticSchemaShape},
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
// Core Deserializer Trait
// ============================================================================

/// A `Deserializer` reads data from an input source, guided by Smithy schemas.
///
/// This trait mirrors the [`Serializer`](crate::serde::se::Serializer) trait, providing
/// schema-guided deserialization for all Smithy data types. It uses a consumer pattern
/// for compound types (structs, lists, maps) where the deserializer iterates and "pushes"
/// values to consumer functions.
///
/// The deserializer is stateful and methods take `&mut self` to advance through the input.
pub trait Deserializer<'de>: Sized {
    /// The error type that can be returned if deserialization fails.
    type Error: Error;

    // === Primitive deserialization ===

    /// Read a boolean value
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a bool.
    fn read_bool(&mut self, schema: &SchemaRef) -> Result<bool, Self::Error>;

    /// Read a byte (`i8`)
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `byte`.
    fn read_byte(&mut self, schema: &SchemaRef) -> Result<i8, Self::Error>;

    /// Read a short (`i16`)
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `short`.
    fn read_short(&mut self, schema: &SchemaRef) -> Result<i16, Self::Error>;

    /// Read an integer (`i32`)
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as an `integer`.
    fn read_integer(&mut self, schema: &SchemaRef) -> Result<i32, Self::Error>;

    /// Read a long (i64)
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `long`.
    fn read_long(&mut self, schema: &SchemaRef) -> Result<i64, Self::Error>;

    /// Read a float (`f32`)
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `float`.
    fn read_float(&mut self, schema: &SchemaRef) -> Result<f32, Self::Error>;

    /// Read a double (`f64`)
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `double`.
    fn read_double(&mut self, schema: &SchemaRef) -> Result<f64, Self::Error>;

    /// Read a big integer
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `bigInteger`.
    fn read_big_integer(&mut self, schema: &SchemaRef) -> Result<BigInt, Self::Error>;

    /// Read a big decimal
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `bigDecimal`.
    fn read_big_decimal(&mut self, schema: &SchemaRef) -> Result<BigDecimal, Self::Error>;

    /// Read a string
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `string`.
    fn read_string(&mut self, schema: &SchemaRef) -> Result<String, Self::Error>;

    /// Read a blob
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `blob`.
    fn read_blob(&mut self, schema: &SchemaRef) -> Result<ByteBuffer, Self::Error>;

    /// Read a timestamp
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `timestamp`.
    fn read_timestamp(&mut self, schema: &SchemaRef) -> Result<Instant, Self::Error>;

    /// Read data as untyped [`Document`]
    ///
    /// # Errors
    /// Returns [`Error`] if the data could not be read as a `document`.
    fn read_document(&mut self, schema: &SchemaRef) -> Result<Box<dyn Document>, Self::Error>;

    // === Compound types (consumer pattern) ===

    // TODO(unknown members): Union unknown types are not well supported by
    //                        the current impl.
    /// Read a struct by calling a consumer function for each member.
    ///
    /// The deserializer iterates through struct members and calls the consumer
    /// for each one. The consumer receives the builder, member schema, and a
    /// mutable reference to the deserializer to read the member value.
    ///
    /// # Example (generated code)
    ///
    /// ```rust,ignore
    /// impl<'de> DeserializeWithSchema<'de> for CitySummaryBuilder {
    ///     fn deserialize<D: Deserializer<'de>>(
    ///         schema: &SchemaRef,
    ///         deserializer: &mut D
    ///     ) -> Result<Self, D::Error> {
    ///         let builder = Builder::new();
    ///         deserializer.read_struct(schema, builder, |builder, member, de| {
    ///             if member.member_index() == 0 {
    ///                 return Ok(builder.city_id(de.read_string(member)?));
    ///             }
    ///             if member.member_index() == 1 {
    ///                 return Ok(builder.name(de.read_string(member)?));
    ///             }
    ///             Ok(builder) // Unknown field
    ///         })
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    /// Returns [`Error`] if a builder member could not be read correctly. Some
    /// shapes may also return an error on an unexpected member value.
    fn read_struct<B, F>(
        &mut self,
        schema: &SchemaRef,
        builder: B,
        consumer: F,
    ) -> Result<B, Self::Error>
    where
        F: FnMut(B, &SchemaRef, &mut Self) -> Result<B, Self::Error>;

    /// Read a list by calling a consumer function for each element.
    ///
    /// The deserializer iterates through list elements and calls the consumer
    /// for each one. The consumer receives the element schema and a mutable
    /// reference to the deserializer to read the element value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut vec = Vec::new();
    /// deserializer.read_list(schema, &mut vec, |vec, element_schema, de| {
    ///     let elem = T::deserialize(element_schema, de)?;
    ///     vec.push(elem);
    ///     Ok(())
    /// })?;
    /// ```
    ///
    /// # Errors
    /// Returns [`Error`] if a list element could not be read correctly.
    fn read_list<T, F>(
        &mut self,
        schema: &SchemaRef,
        state: &mut T,
        consumer: F,
    ) -> Result<(), Self::Error>
    where
        F: FnMut(&mut T, &SchemaRef, &mut Self) -> Result<(), Self::Error>;

    /// Read a map by calling a consumer function for each entry.
    ///
    /// The deserializer iterates through map entries and calls the consumer
    /// for each one. The consumer receives the key (as a String) and a
    /// mutable reference to the deserializer positioned at the value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut map = IndexMap::new();
    /// deserializer.read_map(schema, &mut map, |map, key, de| {
    ///     let value = V::deserialize(value_schema, de)?;
    ///     map.insert(key, value);
    ///     Ok(())
    /// })?;
    /// ```
    ///
    /// # Errors
    /// Returns [`Error`] if a map entry could not be read correctly.
    fn read_map<T, F>(
        &mut self,
        schema: &SchemaRef,
        state: &mut T,
        consumer: F,
    ) -> Result<(), Self::Error>
    where
        F: FnMut(&mut T, String, &mut Self) -> Result<(), Self::Error>;

    // === Null handling ===

    /// Check if the next value is null without consuming it.
    fn is_null(&mut self) -> bool;

    /// Read a null value.
    ///
    /// # Errors
    /// Returns [`Error`] if an element could not be read as `null`/empty value.
    fn read_null(&mut self) -> Result<(), Self::Error>;
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
    fn deserialize<D: Deserializer<'de>>(deserializer: &mut D) -> Result<Self, D::Error>;
}

impl<'de, T: StaticSchemaShape + DeserializeWithSchema<'de>> DeserializableShape<'de> for T {
    fn deserialize<D: Deserializer<'de>>(deserializer: &mut D) -> Result<Self, D::Error> {
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
    /// # Errors
    /// Returns [`Error`] if data from the `Deserializer` could not be read into
    /// this shape type. This could be due to either a schema or data mismatch.
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

// ============================================================================
// Implementations for standard types
// ============================================================================

// === Primitives ===

impl<'de> DeserializeWithSchema<'de> for bool {
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_bool(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for i8 {
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_byte(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for i16 {
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_short(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for i32 {
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_integer(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for i64 {
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_long(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for f32 {
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_float(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for f64 {
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_double(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for BigInt {
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_big_integer(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for BigDecimal {
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_big_decimal(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for String {
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_string(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for ByteBuffer {
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_blob(schema)
    }
}

impl<'de> DeserializeWithSchema<'de> for Instant {
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
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
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut vec = Vec::new();
        deserializer.read_list(schema, &mut vec, |vec, element_schema, de| {
            let elem = T::deserialize_with_schema(element_schema, de)?;
            vec.push(elem);
            Ok(())
        })?;

        Ok(vec)
    }
}

// === IndexMap<K, V> (map) ===

impl<'de, V> DeserializeWithSchema<'de> for IndexMap<String, V>
where
    V: DeserializeWithSchema<'de>,
{
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = IndexMap::new();

        // Get value schema
        let value_schema = schema
            .get_member("value")
            .ok_or_else(|| Error::custom("map schema missing value"))?;

        deserializer.read_map(schema, &mut map, |map, key, de| {
            let value = V::deserialize_with_schema(value_schema, de)?;
            map.insert(key, value);
            Ok(())
        })?;

        Ok(map)
    }
}

// === Option<T> ===

impl<'de, T> DeserializeWithSchema<'de> for Option<T>
where
    T: DeserializeWithSchema<'de>,
{
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
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
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize_with_schema(schema, deserializer).map(Box::new)
    }
}
