//! Schema-guided serialization of data from a Smithy Shape
//!
//! This module provides traits for serializing shapes into
//! a data format (JSON, CBOR, etc.), guided by schemas.
//!
//! # Architecture
//!
//! The serialization system uses a writer-based pattern for compound types:
//!
//! - [`Serializer`]: Entry point that writes primitives and creates writes
//! - [`StructWriter`]: Consumes structure members with `write_member(...)` methods
//! - [`ListWriter`]: Consumes list elements with `write_element(...)`
//! - [`MapWriter`]: Consumes map entries with `write_entry(...)`
//!
//! This design (inspired by `serde`) separates iteration from value reading,
//! allowing callers to control the serialization flow.
use std::{error::Error as StdError, fmt::Display};

use crate::{
    BigDecimal, BigInt, ByteBuffer, IndexMap, Instant,
    schema::{Document, Schema, SchemaShape, ShapeId},
};

// ============================================================================
// Shape Traits
// ============================================================================

/// Serialize a shape with its pre-defined schema.
///
/// ## Blanket Implementations
/// This trait provides blanket implementations for all shapes
/// with both a [`SchemaShape`], and [`SerializeWithSchema`] implementation.
pub trait SerializableShape: SchemaShape + SerializeWithSchema {
    /// Serialize a shape with its pre-defined schema
    ///
    /// # Errors
    /// Returns an [`Error`] if the shape could not be serialized.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

// Blanket implementation of serialization for all Implement
impl<T: SchemaShape + SerializeWithSchema> SerializableShape for T {
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.serialize_with_schema(self.schema(), serializer)
    }
}

/// Schema-guided serialization implementation.
pub trait SerializeWithSchema {
    /// Serialize a Shape using a schema to guide the process
    ///
    /// # Errors
    /// Returns an [`Error`] if the shape could not be serialized.
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error>;
}

// ============================================================================
// Error Trait
// ============================================================================

/// Error trait for serialization errors.
///
/// <div class="note">
/// **NOTE**: This is essentially a clone of the `serde::Error` trait, but
/// we create a new type to ensure we don't enforce a `serde` dependency
/// on consumers.
/// </div>
///
pub trait Error: Sized + StdError {
    /// Create an error with a custom message
    fn custom<T: Display>(msg: T) -> Self;
}

// ============================================================================
// Core Serialize Traits
// ============================================================================

/// List Serializer that can be called in a loop to serialize a sequence
///
/// # Example
///
/// ```ignore
/// // List to serialize
/// let list = vec![1,2,3,4];
///
/// // Create writer
/// let mut writer = serializer.write_list(&schema, list.len())?;
///
/// // Get schema to use for each element of list
/// let element_schema = schema.get_member("member").unwrap();
///
/// // loop over list items
/// for i in list {
///     writer.write_element(&element_schema, i).unwrap();
/// }
/// // close writer
/// writer.end().unwrap();
/// ```
pub trait ListWriter {
    /// Must match the `Error` type of our `Serializer`.
    type Error: Error;

    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Serialize a sequence element.
    ///
    /// # Errors
    /// Returns an [`Error`] matching the parent serializer if
    /// the element could not be serialized.
    fn write_element<T>(&mut self, element_schema: &Schema, value: &T) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema;

    /// Finish serializing a sequence.
    ///
    /// # Errors
    /// [`Error`] if the sequence could not be closed
    fn end(self, schema: &Schema) -> Result<Self::Ok, Self::Error>;
}

/// Map Serializer that can be called in a loop to serialize map values
///
/// # Example
///
/// ```ignore
/// // Map to serialize
/// let mut map: IndexMap<String, String> = IndexMap::new();
/// map.insert("a".to_string(), "b".to_string());
/// map.insert("c".to_string(), "d".to_string());
///
/// let mut writer = serializer.write_map(schema, map.len())?;
/// let (key_schema, value_schema) = schema.get_key_value().unwrap();
///
/// // Iterate over map entries
/// for (k, v) in map {
///     writer.write_entry(key_schema, value_schema, k, v).unwrap();
/// }
/// map.end().unwrap();
/// ```
pub trait MapWriter {
    /// Must match the `Error` type of our [`Serializer`].
    type Error: Error;

    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Serialize a single map entry
    ///
    /// # Errors
    /// Returns an [`Error`] matching the parent serializer if
    /// the entry could not be serialized.
    fn write_entry<K, V>(
        &mut self,
        key_schema: &Schema,
        value_schema: &Schema,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: SerializeWithSchema,
        V: SerializeWithSchema;

    /// Finish serializing a map.
    ///
    /// # Errors
    /// Returns an [`Error`] matching the parent serializer if
    /// the map could not be closed.
    fn end(self, schema: &Schema) -> Result<Self::Ok, Self::Error>;
}

/// Struct Serializer that can be called to serialize struct member values
///
/// # Example
///
/// ```ignore
/// struct MyStruct {
///     pub a: String,
///     pub b: i32
/// }
///
/// fn serialize_struct(value: MyStruct) {
///     let mut ser = serializer.write_struct(schema, 2usize).unwrap();
///     ser.serialize_member_named(&schema_member_a, &value.a).unwrap();
///     ser.serialize_member(&schema_member_b, &value.b).unwrap();
///     // close serializer
///     ser.end(schema).unwrap()
/// }
/// ```
pub trait StructWriter {
    /// Must match the `Error` type of our [`Serializer`].
    type Error: Error;

    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Optionally serializes the discriminator of a shape.
    ///
    /// In general this is only done for document types to allow for
    /// over-the-wire polymorphism, and by default this method does nothing.
    ///
    /// # Errors
    /// Returns an [`Error`] matching the parent serializer if
    /// the discriminator could not be serialized.
    #[inline]
    fn write_discriminator(&mut self, _discriminator: &ShapeId) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Serialize a member on the struct
    ///
    /// # Errors
    /// Returns an [`Error`] matching the parent serializer if
    /// the member could not be serialized.
    fn write_member<T>(&mut self, member_schema: &Schema, value: &T) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema;

    /// Serialize a member on the struct with a pre-known field name.
    /// This is an optimization to avoid extracting the name from the schema.
    ///
    /// # Errors
    /// Returns an [`Error`] matching the parent serializer if
    /// the member could not be serialized.
    #[inline]
    fn write_member_named<T>(
        &mut self,
        _member_name: &str,
        member_schema: &Schema,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        // Default implementation falls back to regular serialize_member
        self.write_member(member_schema, value)
    }

    /// Serializes an optional member.
    ///
    /// This method will call [`StructWriter::skip_member`] on any optional members
    /// that are `None`, otherwise the `Some` value is unwrapped and serialized as normal.
    ///
    /// # Errors
    /// Returns an [`Error`] matching the parent serializer if
    /// the member could not be serialized.
    fn write_optional_member<T: SerializeWithSchema>(
        &mut self,
        member_schema: &Schema,
        value: &Option<T>,
    ) -> Result<(), Self::Error> {
        if let Some(value) = value {
            self.write_member(member_schema, value)
        } else {
            self.skip_member(member_schema)
        }
    }

    /// Serializes an optional member with a pre-known field name.
    /// This is an optimization to avoid extracting the name from the schema.
    ///
    /// # Errors
    /// Returns an [`Error`] matching the parent serializer if
    /// the member could not be serialized.
    #[inline]
    fn write_optional_member_named<T: SerializeWithSchema>(
        &mut self,
        member_name: &str,
        member_schema: &Schema,
        value: &Option<T>,
    ) -> Result<(), Self::Error> {
        if let Some(value) = value {
            self.write_member_named(member_name, member_schema, value)
        } else {
            self.skip_member(member_schema)
        }
    }

    /// Skips a member in a structure.
    ///
    /// # Errors
    /// Returns an [`Error`] matching the parent serializer if
    /// the member could not be skipped.
    #[inline]
    fn skip_member(&mut self, _schema: &Schema) -> Result<(), Self::Error> {
        /* Do nothing on skip by default */
        Ok(())
    }

    /// Handle unknown values.
    ///
    /// # Errors
    /// Returns an [`Error`] matching the parent serializer if
    /// the unknown member could not be serialized.
    #[cold]
    fn write_unknown(&mut self, _schema: &Schema, name: &String) -> Result<(), Self::Error> {
        // Error out on unknown by default
        Err(Self::Error::custom(format!(
            "Attempted to serialize unknown value: {name:?}"
        )))
    }

    /// Finish serializing a structure
    ///
    /// # Errors
    /// Returns an [`Error`] matching the parent serializer if
    /// the structure could not be closed.
    fn end(self, schema: &Schema) -> Result<Self::Ok, Self::Error>;
}

// TODO(streams): How should we handle data stream serialization?
// TODO(events): Do we need any custom handling for event streams?
/// A `Serialize` writes data from an output sink, guided by Smithy schemas.
///
/// This trait mirrors the [`Serializer`](serde::Serializer) trait, providing
/// schema-guided serialization for all Smithy data types.
///
/// The serializer is stateful and methods take `self`. Implementations should,
/// consider implement `Serializer` for `&mut` variants.
pub trait Serializer: Sized {
    /// Error type emitted on failed serialization.
    ///
    /// <div class ="note">
    /// **NOTE**: Serializers need to be able to catch and convert dyn Errors from their code.
    /// </div>
    type Error: Error;

    /// Ok return type. Should usually be `()`
    type Ok;

    /// Type returned from [`write_list`] for serializing the contents of a
    /// list.
    ///
    /// [`write_list`]: #tymethod.write_list
    type ListWriter: ListWriter<Ok = Self::Ok, Error = Self::Error>;

    /// Type returned from [`write_map`] for serializing the contents of a
    /// map.
    ///
    /// [`write_map`]: #tymethod.write_map
    type MapWriter: MapWriter<Ok = Self::Ok, Error = Self::Error>;

    /// Type returned from [`write_struct`] for serializing the contents of a
    /// struct or union.
    ///
    /// [`write_struct`]: #tymethod.write_struct
    // TODO: Might want to do `StructWriter<'s>` so write_struct can return a
    // struct writer that borrows from `schema`. This would eliminate the Arc
    // clone needed by HTTP binding's struct serialization (for payload/body).
    type StructWriter: StructWriter<Ok = Self::Ok, Error = Self::Error>;

    /// Begin to serialize a variably sized structure or union. This call must be
    /// followed by zero or more calls to `write_member`, then a call to `end`.
    ///
    /// # Errors
    /// `Self::Error` if the structure could not be opened.
    fn write_struct(
        self,
        _schema: &Schema,
        _len: usize,
    ) -> Result<Self::StructWriter, Self::Error> {
        Err(Error::custom(
            "write_struct is not supported by this serializer",
        ))
    }

    /// Begin to serialize a variably sized map. This call must be
    /// followed by zero or more calls to `write_entry`, then a call to `end`.
    ///
    /// # Errors
    /// `Self::Error` if the map could not be opened.
    fn write_map(self, _schema: &Schema, _len: usize) -> Result<Self::MapWriter, Self::Error> {
        Err(Error::custom(
            "write_map is not supported by this serializer",
        ))
    }

    /// Begin to serialize a variably sized list. This call must be
    /// followed by zero or more calls to `write_element`, then a call to `end`.
    ///
    /// # Errors
    /// `Self::Error` if the list could not be opened.
    fn write_list(self, _schema: &Schema, _len: usize) -> Result<Self::ListWriter, Self::Error> {
        Err(Error::custom(
            "write_list is not supported by this serializer",
        ))
    }

    /// Serialize a `boolean`
    ///
    /// # Errors
    /// `Self::Error` if the value could not be serialized as a boolean.
    fn write_boolean(self, _schema: &Schema, _value: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "write_boolean is not supported by this serializer",
        ))
    }

    /// Serialize a byte (`i8`)
    ///
    /// # Errors
    /// `Self::Error` if the value could not be serialized as a `byte`.
    fn write_byte(self, _schema: &Schema, _value: i8) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "write_byte is not supported by this serializer",
        ))
    }

    /// Serialize a short (`i16`)
    ///
    /// # Errors
    /// `Self::Error` if the value could not be serialized as a `short`.
    fn write_short(self, _schema: &Schema, _value: i16) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "write_short is not supported by this serializer",
        ))
    }

    /// Serialize an integer (`i32`)
    ///
    /// # Errors
    /// `Self::Error` if the value could not be serialized as an integer.
    fn write_integer(self, _schema: &Schema, _value: i32) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "write_integer is not supported by this serializer",
        ))
    }

    /// Serialize a long (`i64`)
    ///
    /// # Errors
    /// `Self::Error` if the value could not be serialized as a `long`.
    fn write_long(self, _schema: &Schema, _value: i64) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "write_long is not supported by this serializer",
        ))
    }

    /// Serialize a float (`f32`)
    ///
    /// # Errors
    /// `Self::Error` if the value could not be serialized as a `float`.
    fn write_float(self, _schema: &Schema, _value: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "write_float is not supported by this serializer",
        ))
    }

    /// Serialize a double (`f64`)
    ///
    /// # Errors
    /// `Self::Error` if the value could not be serialized as a `double`.
    fn write_double(self, _schema: &Schema, _value: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "write_double is not supported by this serializer",
        ))
    }

    /// Serialize a [`BigInt`]
    ///
    /// # Errors
    /// `Self::Error` if the value could not be serialized as a `bigInteger`.
    fn write_big_integer(self, _schema: &Schema, _value: &BigInt) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "write_big_integer is not supported by this serializer",
        ))
    }

    /// Serialize a [`BigDecimal`]
    ///
    /// # Errors
    /// `Self::Error` if the value could not be serialized as a `bigDecimal`.
    fn write_big_decimal(
        self,
        _schema: &Schema,
        _value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "write_big_decimal is not supported by this serializer",
        ))
    }

    /// Serialize a string (`&str`)
    ///
    /// # Errors
    /// `Self::Error` if the value could not be serialized as a `string`.
    fn write_string(self, _schema: &Schema, _value: &str) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "write_string is not supported by this serializer",
        ))
    }

    /// Serialize a blob (i.e. a buffer)
    ///
    /// # Errors
    /// `Self::Error` if the value could not be serialized as a `blob`.
    fn write_blob(self, _schema: &Schema, _value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "write_blob is not supported by this serializer",
        ))
    }

    /// Serialize a timestamp
    ///
    /// # Errors
    /// `Self::Error` if the value could not be serialized as a `timestamp`.
    fn write_timestamp(self, _schema: &Schema, _value: &Instant) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "write_timestamp is not supported by this serializer",
        ))
    }

    /// Serialize an untyped [`Document`]
    ///
    /// # Errors
    /// `Self::Error` if the value could not be serialized as a `document`.
    #[allow(clippy::borrowed_box)]
    fn write_document(
        self,
        _schema: &Schema,
        _value: &Box<dyn Document>,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "write_document is not supported by this serializer",
        ))
    }

    /// Serialize a `null` value
    ///
    /// # Errors
    /// `Self::Error` if the value could not be serialized as an empty (`null`) value.
    fn write_null(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "write_null is not supported by this serializer",
        ))
    }

    /// Write a missing expected value.
    ///
    /// Default implementation simply `skip()`s the missing value.
    ///
    /// # Errors
    /// `Self::Error` if the missing value could not be serialized.
    fn write_missing(self, schema: &Schema) -> Result<Self::Ok, Self::Error> {
        self.skip(schema)
    }

    /// Skip the serialization of a value.
    ///
    /// # Errors
    /// `Self::Error` if the value could not be skipped.
    fn skip(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom("skip is not supported by this serializer"))
    }

    /// Flush all remaining data.
    ///
    /// # Errors
    /// `Self::Error` if the underlying data source was not flushed successfully.
    fn flush(self) -> Result<Self::Ok, Self::Error> {
        todo!();
    }
}

// ============================================================================
// Default Implementations
// ============================================================================

// === Collection implementations ===
impl<T: SerializeWithSchema> SerializeWithSchema for Vec<T> {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut list = serializer.write_list(schema, self.len())?;
        let value_schema = schema
            .get_list_member()
            .ok_or_else(|| S::Error::custom("Expected a list schema"))?;
        for element in self {
            list.write_element(value_schema, element)?;
        }
        list.end(schema)
    }
}

impl<K, V> SerializeWithSchema for IndexMap<K, V>
where
    K: SerializeWithSchema,
    V: SerializeWithSchema,
{
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut map = serializer.write_map(schema, self.len())?;
        let (key_schema, value_schema) = schema
            .get_key_value()
            .ok_or_else(|| S::Error::custom("Expected a map schema"))?;
        for (k, v) in self {
            map.write_entry(key_schema, value_schema, k, v)?;
        }
        map.end(schema)
    }
}

// === Scalar type implementations ===

impl SerializeWithSchema for bool {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.write_boolean(schema, *self)
    }
}

impl SerializeWithSchema for i8 {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.write_byte(schema, *self)
    }
}

impl SerializeWithSchema for i16 {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.write_short(schema, *self)
    }
}

impl SerializeWithSchema for i32 {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.write_integer(schema, *self)
    }
}

impl SerializeWithSchema for i64 {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.write_long(schema, *self)
    }
}

impl SerializeWithSchema for f32 {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.write_float(schema, *self)
    }
}

impl SerializeWithSchema for f64 {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.write_double(schema, *self)
    }
}

impl SerializeWithSchema for BigInt {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.write_big_integer(schema, self)
    }
}

impl SerializeWithSchema for BigDecimal {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.write_big_decimal(schema, self)
    }
}

impl SerializeWithSchema for ByteBuffer {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.write_blob(schema, self)
    }
}

impl SerializeWithSchema for Instant {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.write_timestamp(schema, self)
    }
}

impl SerializeWithSchema for String {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.write_string(schema, self)
    }
}

// === Wrapper-type implementations ===

impl<T: SerializeWithSchema> SerializeWithSchema for Option<T> {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        if let Some(value) = self.as_ref() {
            value.serialize_with_schema(schema, serializer)
        } else {
            serializer.skip(schema)
        }
    }
}

impl<T: SerializeWithSchema> SerializeWithSchema for Box<T> {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        (**self).serialize_with_schema(schema, serializer)
    }
}
