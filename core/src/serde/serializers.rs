#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::missing_errors_doc)]

use crate::schema::SchemaShape;
use crate::schema::{Document, Schema};
use crate::{BigDecimal, BigInt, ByteBuffer, Instant};
use indexmap::IndexMap;
use std::error::Error as StdError;
use std::fmt::Display;

/// Serialize a shape with its pre-defined schema.
///
/// This trait provides an automatic, blanket implementation for all shapes
/// with both a [`SchemaShape`], and [`SerializeWithSchema`] implementation.
pub trait Serialize: SchemaShape + SerializeWithSchema {
    /// Serialize a shape with its pre-defined schema
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok<'_>, S::Error>;
}

// Blanket implementation of serialization for all Implement
impl<T: SchemaShape + SerializeWithSchema> Serialize for T {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok<'_>, S::Error> {
        self.serialize_with_schema(self.schema(), serializer)
    }
}

/// Schema-Guided serialization implementation.
pub trait SerializeWithSchema {
    /// Serialize a Shape using a schema to guide the process
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok<'_>, S::Error>;
}

/// List Serializer that can be called in a loop to serialize list values
pub trait ListSerializer {
    /// Must match the `Error` type of our `Serializer`.
    type Error: Error;

    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Serialize a sequence element.
    fn serialize_element<T>(
        &mut self,
        element_schema: &Schema,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema;

    /// Finish serializing a sequence.
    fn end(self, schema: &Schema) -> Result<Self::Ok, Self::Error>;
}

/// Map Serializer that can be called in a loop to serialize map values
pub trait MapSerializer {
    /// Must match the `Error` type of our [`Serializer`].
    type Error: Error;

    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Serialize a single map entry
    fn serialize_entry<K, V>(
        &mut self,
        key_schema: &Schema,
        value_schema: &Schema,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: ?Sized + SerializeWithSchema,
        V: ?Sized + SerializeWithSchema;

    /// Finish serializing a map.
    fn end(self, schema: &Schema) -> Result<Self::Ok, Self::Error>;
}

// TODO: Docs
pub trait StructSerializer {
    /// Must match the `Error` type of our [`Serializer`].
    type Error: Error;

    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Serialize a member on the struct
    fn serialize_member<T>(
        &mut self,
        member_schema: &Schema,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema;

    /// Serializes an optional member.
    ///
    /// This method will call [`StructSerializer::skip`] any optional members
    /// that are `None`, otherwise the `Some` value is unwrapped and serialized as normal.
    fn serialize_optional_member<T: SerializeWithSchema>(
        &mut self,
        member_schema: &Schema,
        value: &Option<T>,
    ) -> Result<(), Self::Error> {
        if let Some(value) = value {
            self.serialize_member(member_schema, value)
        } else {
            self.skip_member(member_schema)
        }
    }

    /// Skips a member in a structure.
    fn skip_member(&mut self, schema: &Schema) -> Result<(), Self::Error> {
        /* Do nothing on skip by default */
        Ok(())
    }

    /// Finish serializing a structure.
    fn end(self, schema: &Schema) -> Result<Self::Ok, Self::Error>;
}

/// Basically just a clone of the serde::Error trait.
/// We use our own to ensure we don't enforce a `serde` dependency on consumers.
pub trait Error: Sized + StdError {
    fn custom<T: Display>(msg: T) -> Self;
}

// TODO: datastream?
// TODO: event stream?
// TODO: Docs
pub trait Serializer: Sized {
    /// Error type emitted on failed serialization.
    ///
    /// **Note**: Serializers need to be able to catch and convert dyn Errors from their code.
    type Error: Error;

    /// Ok return type. Should almost always be ()
    type Ok<'ok>;

    /// Type returned from [`write_list`] for serializing the contents of a
    /// list.
    ///
    /// [`write_list`]: #tymethod.write_list
    type SerializeList<'sl>: ListSerializer<Ok = Self::Ok<'sl>, Error = Self::Error>;

    /// Type returned from [`write_map`] for serializing the contents of a
    /// map.
    ///
    /// [`write_map`]: #tymethod.write_map
    type SerializeMap: MapSerializer<Ok = Self::Ok, Error = Self::Error>;

    /// Type returned from [`write_struct`] for serializing the contents of a
    /// struct or union.
    ///
    /// [`write_struct`]: #tymethod.write_struct
    type SerializeStruct: StructSerializer<Ok = Self::Ok, Error = Self::Error>;

    /// Begin to serialize a variably sized structure or union. This call must be
    /// followed by zero or more calls to `serialize_member`, then a call to
    /// `end`.
    fn write_struct(
        self,
        schema: &Schema,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error>;

    /// Begin to serialize a variably sized map. This call must be
    /// followed by zero or more calls to `serialize_entry`, then a call to
    /// `end`.
    fn write_map(self, schema: &Schema, len: usize) -> Result<Self::SerializeMap, Self::Error>;

    /// Begin to serialize a variably sized list. This call must be
    /// followed by zero or more calls to `serialize_element`, then a call to
    /// `end`.
    fn write_list<'a>(self, schema: &'a Schema, len: usize) -> Result<Self::SerializeList<'a>, Self::Error>;

    /// Serialize a `boolean`
    fn write_boolean(self, schema: &Schema, value: bool) -> Result<Self::Ok<'_>, Self::Error>;

    /// Serialize a byte (`i8`)
    fn write_byte(self, schema: &Schema, value: i8) -> Result<Self::Ok<'_>, Self::Error>;

    /// Serialize a short (`i16`)
    fn write_short(self, schema: &Schema, value: i16) -> Result<Self::Ok<'_>, Self::Error>;

    /// Serialize an integer (`i32`)
    fn write_integer(self, schema: &Schema, value: i32) -> Result<Self::Ok<'_>, Self::Error>;

    /// Serialize a long (`i64`)
    fn write_long(self, schema: &Schema, value: i64) -> Result<Self::Ok<'_>, Self::Error>;

    /// Serialize a float (`f32`)
    fn write_float(self, schema: &Schema, value: f32) -> Result<Self::Ok<'_>, Self::Error>;

    /// Serialize a double (`f64`)
    fn write_double(self, schema: &Schema, value: f64) -> Result<Self::Ok<'_>, Self::Error>;

    /// Serialize a [`BigInt`]
    fn write_big_integer<'a, 'b>(self, schema: &'a Schema, value: &'b BigInt)
    -> Result<Self::Ok<'a>, Self::Error>;

    /// Serialize a [`BigDecimal`]
    fn write_big_decimal<'a, 'b>(
        self,
        schema: &'a Schema,
        value: &'b BigDecimal,
    ) -> Result<Self::Ok<'a>, Self::Error>;

    /// Serialize a string (`&str`)
    fn write_string<'a>(self, schema: &'a Schema, value: &'a str) -> Result<Self::Ok<'a>, Self::Error>;

    /// Serialize a blob (i.e. a buffer)
    fn write_blob<'a>(self, schema: &'a Schema, value: &'a ByteBuffer) -> Result<Self::Ok<'a>, Self::Error>;

    /// Serialize a timestamp
    fn write_timestamp<'a>(self, schema: &'a Schema, value: &'a Instant) -> Result<Self::Ok<'a>, Self::Error>;

    /// Serialize an untyped [`Document`]
    fn write_document<'a>(self, schema: &'a Schema, value: &'a Document) -> Result<Self::Ok<'a>, Self::Error>;

    /// Serialize a `null` value
    fn write_null(self, schema: &Schema) -> Result<Self::Ok<'_>, Self::Error>;

    /// Skip the serialization of a value.
    fn skip(self, _schema: &Schema) -> Result<Self::Ok<'_>, Self::Error>;

    // TODO: Is this necessary?
    /// Flush all remaining data.
    fn flush(self) -> Result<Self::Ok<'static>, Self::Error> {
        todo!();
    }
}

// === Default implementations ===
impl<T: SerializeWithSchema> SerializeWithSchema for Vec<T> {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok<'_>, S::Error> {
        let mut list = serializer.write_list(schema, self.len())?;
        let value_schema = schema.expect_member("member");
        for element in self {
            list.serialize_element(value_schema, element)?;
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
    ) -> Result<S::Ok<'_>, S::Error> {
        let mut map = serializer.write_map(schema, self.len())?;
        // TODO: is there a more efficient way to store/get these schemas?
        let key_schema = schema.get_member("key").expect("Should have key schema");
        let value_schema = schema
            .get_member("value")
            .expect("Should have value schema");
        for (k, v) in self {
            map.serialize_entry(key_schema, value_schema, k, v)?;
        }
        map.end(schema)
    }
}

impl SerializeWithSchema for bool {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok<'_>, S::Error> {
        serializer.write_boolean(schema, *self)
    }
}

impl SerializeWithSchema for i8 {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok<'_>, S::Error> {
        serializer.write_byte(schema, *self)
    }
}

impl SerializeWithSchema for i16 {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok<'_>, S::Error> {
        serializer.write_short(schema, *self)
    }
}

impl SerializeWithSchema for i32 {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok<'_>, S::Error> {
        serializer.write_integer(schema, *self)
    }
}

impl SerializeWithSchema for i64 {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok<'_>, S::Error> {
        serializer.write_long(schema, *self)
    }
}

impl SerializeWithSchema for f32 {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok<'_>, S::Error> {
        serializer.write_float(schema, *self)
    }
}

impl SerializeWithSchema for f64 {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok<'_>, S::Error> {
        serializer.write_double(schema, *self)
    }
}

impl SerializeWithSchema for BigInt {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok<'_>, S::Error> {
        serializer.write_big_integer(schema, self)
    }
}

impl SerializeWithSchema for BigDecimal {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok<'_>, S::Error> {
        serializer.write_big_decimal(schema, self)
    }
}

impl SerializeWithSchema for ByteBuffer {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok<'_>, S::Error> {
        serializer.write_blob(schema, self)
    }
}

impl SerializeWithSchema for String {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok<'_>, S::Error> {
        serializer.write_string(schema, self)
    }
}

impl<T: SerializeWithSchema> SerializeWithSchema for Option<T> {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok<'_>, S::Error> {
        if let Some(value) = self.as_ref() {
            value.serialize_with_schema(schema, serializer)
        } else {
            serializer.skip(schema)
        }
    }
}
