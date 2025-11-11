#![allow(dead_code)]
#![allow(unused_variables)]

use std::{error::Error as StdError, fmt::Display};

use indexmap::IndexMap;

use crate::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    schema::{Document, SchemaRef},
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

/// A **Deserializer** reads data from an input source, guided by Smithy schemas.
///
/// This trait mirrors the `Serializer` trait, providing schema-guided deserialization
/// for all Smithy data types. It uses a consumer pattern for compound types (structs,
/// lists, maps) where the deserializer iterates and "pushes" values to consumer functions.
///
/// The deserializer is stateful and methods take `&mut self` to advance through the input.
pub trait Deserializer<'de>: Sized {
    /// The error type that can be returned if deserialization fails.
    type Error: Error;

    // === Primitive deserialization ===

    /// Read a boolean value
    fn read_bool(&mut self, schema: &SchemaRef) -> Result<bool, Self::Error>;

    /// Read a byte (i8)
    fn read_byte(&mut self, schema: &SchemaRef) -> Result<i8, Self::Error>;

    /// Read a short (i16)
    fn read_short(&mut self, schema: &SchemaRef) -> Result<i16, Self::Error>;

    /// Read an integer (i32)
    fn read_integer(&mut self, schema: &SchemaRef) -> Result<i32, Self::Error>;

    /// Read a long (i64)
    fn read_long(&mut self, schema: &SchemaRef) -> Result<i64, Self::Error>;

    /// Read a float (f32)
    fn read_float(&mut self, schema: &SchemaRef) -> Result<f32, Self::Error>;

    /// Read a double (f64)
    fn read_double(&mut self, schema: &SchemaRef) -> Result<f64, Self::Error>;

    /// Read a big integer
    fn read_big_integer(&mut self, schema: &SchemaRef) -> Result<BigInt, Self::Error>;

    /// Read a big decimal
    fn read_big_decimal(&mut self, schema: &SchemaRef) -> Result<BigDecimal, Self::Error>;

    /// Read a string
    fn read_string(&mut self, schema: &SchemaRef) -> Result<String, Self::Error>;

    /// Read a blob
    fn read_blob(&mut self, schema: &SchemaRef) -> Result<ByteBuffer, Self::Error>;

    /// Read a timestamp
    fn read_timestamp(&mut self, schema: &SchemaRef) -> Result<Instant, Self::Error>;

    /// Read a document
    fn read_document(&mut self, schema: &SchemaRef) -> Result<Document, Self::Error>;

    // === Compound types (consumer pattern) ===

    /// Read a struct by calling a consumer function for each member.
    ///
    /// The deserializer iterates through struct members and calls the consumer
    /// for each one. The consumer receives the builder, member schema, and a
    /// mutable reference to the deserializer to read the member value.
    ///
    /// # Example (generated code)
    ///
    /// ```ignore
    /// impl<'de> Deserialize<'de> for CitySummary {
    ///     fn deserialize<D: Deserializer<'de>>(
    ///         schema: &SchemaRef,
    ///         deserializer: &mut D
    ///     ) -> Result<Self, D::Error> {
    ///         let mut builder = Builder::new();
    ///         deserializer.read_struct(schema, &mut builder, |builder, member, de| {
    ///             match member.member_index() {
    ///                 0 => builder.city_id(de.read_string(member)?),
    ///                 1 => builder.name(de.read_string(member)?),
    ///                 _ => {}, // Unknown field
    ///             }
    ///             Ok(())
    ///         })?;
    ///         builder.build()
    ///     }
    /// }
    /// ```
    fn read_struct<B, F>(
        &mut self,
        schema: &SchemaRef,
        builder: &mut B,
        consumer: F,
    ) -> Result<(), Self::Error>
    where
        F: FnMut(&mut B, &SchemaRef, &mut Self) -> Result<(), Self::Error>;

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
    fn is_null(&self) -> bool;

    /// Read a null value.
    fn read_null(&mut self) -> Result<(), Self::Error>;
}

// ============================================================================
// Deserialize Trait
// ============================================================================

/// A data structure that can be deserialized from any data format supported
/// by smithy4rs, guided by a schema.
///
/// This trait mirrors `SerializeWithSchema` on the serialization side.
pub trait Deserialize<'de>: Sized {
    /// Deserialize this value from the given deserializer using the provided schema.
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

// ============================================================================
// Implementations for standard types
// ============================================================================

// === Primitives ===

impl<'de> Deserialize<'de> for bool {
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_bool(schema)
    }
}

impl<'de> Deserialize<'de> for i8 {
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_byte(schema)
    }
}

impl<'de> Deserialize<'de> for i16 {
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_short(schema)
    }
}

impl<'de> Deserialize<'de> for i32 {
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_integer(schema)
    }
}

impl<'de> Deserialize<'de> for i64 {
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_long(schema)
    }
}

impl<'de> Deserialize<'de> for f32 {
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_float(schema)
    }
}

impl<'de> Deserialize<'de> for f64 {
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_double(schema)
    }
}

impl<'de> Deserialize<'de> for BigInt {
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_big_integer(schema)
    }
}

impl<'de> Deserialize<'de> for BigDecimal {
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_big_decimal(schema)
    }
}

impl<'de> Deserialize<'de> for String {
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_string(schema)
    }
}

impl<'de> Deserialize<'de> for ByteBuffer {
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_blob(schema)
    }
}

impl<'de> Deserialize<'de> for Instant {
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_timestamp(schema)
    }
}

// === Vec<T> (list) ===

impl<'de, T> Deserialize<'de> for Vec<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut vec = Vec::new();

        // Get member schema for list elements
        let member_schema = schema
            .get_member("member")
            .ok_or_else(|| Error::custom("list schema missing member"))?;

        deserializer.read_list(schema, &mut vec, |vec, element_schema, de| {
            let elem = T::deserialize(element_schema, de)?;
            vec.push(elem);
            Ok(())
        })?;

        Ok(vec)
    }
}

// === IndexMap<K, V> (map) ===

impl<'de, V> Deserialize<'de> for IndexMap<String, V>
where
    V: Deserialize<'de>,
{
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = IndexMap::new();

        // Get value schema
        let value_schema = schema
            .get_member("value")
            .ok_or_else(|| Error::custom("map schema missing value"))?;

        deserializer.read_map(schema, &mut map, |map, key, de| {
            let value = V::deserialize(value_schema, de)?;
            map.insert(key, value);
            Ok(())
        })?;

        Ok(map)
    }
}

// === Option<T> ===

impl<'de, T> Deserialize<'de> for Option<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_null() {
            deserializer.read_null()?;
            Ok(None)
        } else {
            T::deserialize(schema, deserializer).map(Some)
        }
    }
}

impl<'de, T> Deserialize<'de> for Box<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(schema: &SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(schema, deserializer).map(Box::new)
    }
}
