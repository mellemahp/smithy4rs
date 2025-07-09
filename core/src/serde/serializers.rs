#![allow(dead_code)]
#![allow(unused_variables)]

use crate::schema::{Schema, SchemaRef, Document};
use crate::{BigDecimal, BigInt, ByteBuffer, Instant};
use indexmap::IndexMap;
use std::error::Error;

/// Schema-Guided serialization
pub trait Serialize {
    /// Serialize a Shape using a
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error>;
}

#[derive(serde::Serialize)]
pub struct Test {
    name: String,
    other: i32
}
#[automatically_derived]
impl serde::Serialize for Test {
    fn serialize<__S>(&self, __serializer: __S) -> serde::__private::Result<__S::Ok, __S::Error>
    where
        __S: serde::Serializer,
    {
        let mut _serde_state = serde::Serializer::serialize_struct(__serializer, "Test", false as usize + 1 + 1)?;
        serde::ser::SerializeStruct::serialize_field(&mut _serde_state, "name", &self.name)?;
        serde::ser::SerializeStruct::serialize_field(&mut _serde_state, "other", &self.other)?;
        serde::ser::SerializeStruct::end(_serde_state)
    }
}

/// Represents the empty return of a serializer call that could fail.
pub type SerializerResult<E> = Result<(), E>;

/// List Serializer that can be called in a loop to serialize list values
pub trait ListSerializer {
    /// Must match the `Error` type of our `Serializer and be able to handle unknown errors.
    type Error: Error + From<Box<dyn Error>>;

    /// Serialize a sequence element.
    fn serialize_element<T>(
        &mut self,
        element_schema: &SchemaRef,
        value: &T,
    ) -> SerializerResult<Self::Error>
    where
        T: ?Sized + Serialize;

    /// Finish serializing a sequence.
    fn end(self, schema: &SchemaRef) -> SerializerResult<Self::Error>;
}

// TODO: Docs
pub trait MapSerializer {
    /// Must match the `Error` type of our [`Serializer`].
    type Error: Error + From<Box<dyn Error>>;

    /// Serialize a single map entry
    fn serialize_entry<K, V>(
        &mut self,
        key_schema: &SchemaRef,
        value_schema: &SchemaRef,
        key: &K,
        value: &V,
    ) -> SerializerResult<Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize;

    /// Finish serializing a map.
    fn end(self, schema: &SchemaRef) -> SerializerResult<Self::Error>;
}

// TODO: Docs
pub trait StructSerializer {
    /// Must match the `Error` type of our [`Serializer`].
    type Error: Error + From<Box<dyn Error>>;

    /// Serialize a member on the struct
    fn serialize_member<T>(
        &mut self,
        member_schema: &SchemaRef,
        value: &T,
    ) -> SerializerResult<Self::Error>
    where
        T: ?Sized + Serialize;

    /// Serializes an optional member.
    ///
    /// This method will call [`StructSerializer::skip`] any optional members
    /// that are `None`, otherwise the `Some` value is unwrapped and serialized as normal.
    fn serialize_optional_member<T: Serialize>(
        &mut self,
        member_schema: &SchemaRef,
        value: &Option<T>,
    ) -> SerializerResult<Self::Error> {
        if let Some(value) = value {
            self.serialize_member(member_schema, value)
        } else {
            self.skip_member(member_schema)
        }
    }

    /// Skips a member in a structure.
    fn skip_member(&mut self, schema: &Schema) -> SerializerResult<Self::Error> {
        /* Do nothing on skip by default */
        Ok(())
    }

    /// Finish serializing a structure.
    fn end(self, schema: &SchemaRef) -> SerializerResult<Self::Error>;
}

// TODO: datastream?
// TODO: event stream?
// TODO: Docs
pub trait Serializer: Sized {
    /// Error type emitted on failed serialization.
    ///
    /// **Note**: Serializers need to be able to catch and convert dyn Errors from their code.
    type Error: Error + From<Box<dyn Error>>;

    type SerializeList<'l>: ListSerializer<Error = Self::Error>
    where
        Self: 'l;
    type SerializeMap<'m>: MapSerializer<Error = Self::Error>
    where
        Self: 'm;
    type SerializeStruct<'s>: StructSerializer<Error = Self::Error>
    where
        Self: 's;

    fn write_struct(
        &mut self,
        schema: &SchemaRef,
        len: usize,
    ) -> Result<Self::SerializeStruct<'_>, Self::Error>;
    fn write_map(
        &mut self,
        schema: &SchemaRef,
        len: usize,
    ) -> Result<Self::SerializeMap<'_>, Self::Error>;
    fn write_list(
        &mut self,
        schema: &SchemaRef,
        len: usize,
    ) -> Result<Self::SerializeList<'_>, Self::Error>;
    fn write_boolean(&mut self, schema: &SchemaRef, value: bool) -> SerializerResult<Self::Error>;
    fn write_byte(&mut self, schema: &SchemaRef, value: i8) -> SerializerResult<Self::Error>;
    fn write_short(&mut self, schema: &SchemaRef, value: i16) -> SerializerResult<Self::Error>;
    fn write_integer(&mut self, schema: &SchemaRef, value: i32) -> SerializerResult<Self::Error>;
    fn write_long(&mut self, schema: &SchemaRef, value: i64) -> SerializerResult<Self::Error>;
    fn write_float(&mut self, schema: &SchemaRef, value: f32) -> SerializerResult<Self::Error>;
    fn write_double(&mut self, schema: &SchemaRef, value: f64) -> SerializerResult<Self::Error>;
    fn write_big_integer(
        &mut self,
        schema: &SchemaRef,
        value: &BigInt,
    ) -> SerializerResult<Self::Error>;
    fn write_big_decimal(
        &mut self,
        schema: &SchemaRef,
        value: &BigDecimal,
    ) -> SerializerResult<Self::Error>;
    fn write_string(&mut self, schema: &SchemaRef, value: &String)
    -> SerializerResult<Self::Error>;
    fn write_blob(
        &mut self,
        schema: &SchemaRef,
        value: &ByteBuffer,
    ) -> SerializerResult<Self::Error>;
    fn write_timestamp(
        &mut self,
        schema: &SchemaRef,
        value: &Instant,
    ) -> SerializerResult<Self::Error>;
    fn write_document(
        &mut self,
        schema: &SchemaRef,
        value: &Document,
    ) -> SerializerResult<Self::Error>;
    fn write_null(&mut self, schema: &SchemaRef) -> SerializerResult<Self::Error>;
    fn skip(&mut self, schema: &SchemaRef) -> SerializerResult<Self::Error>;

    // TODO: Is this necessary?
    fn flush(&mut self) -> SerializerResult<Self::Error> {
        todo!();
    }
}

// === Default implementations ===
impl<T: Serialize> Serialize for Vec<T> {
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
        let mut list = serializer.write_list(schema, self.len())?;
        let value_schema = schema.expect_member("member");
        for element in self {
            list.serialize_element(&value_schema, element)?;
        }
        list.end(schema)
    }
}

impl<K, V> Serialize for IndexMap<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
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

impl Serialize for bool {
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
        serializer.write_boolean(schema, *self)
    }
}

impl Serialize for i8 {
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
        serializer.write_byte(schema, *self)
    }
}

impl Serialize for i16 {
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
        serializer.write_short(schema, *self)
    }
}

impl Serialize for i32 {
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
        serializer.write_integer(schema, *self)
    }
}

impl Serialize for i64 {
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
        serializer.write_long(schema, *self)
    }
}

impl Serialize for f32 {
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
        serializer.write_float(schema, *self)
    }
}

impl Serialize for f64 {
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
        serializer.write_double(schema, *self)
    }
}

impl Serialize for BigInt {
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
        serializer.write_big_integer(schema, self)
    }
}

impl Serialize for BigDecimal {
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
        serializer.write_big_decimal(schema, self)
    }
}

impl Serialize for ByteBuffer {
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
        serializer.write_blob(schema, self)
    }
}

impl Serialize for String {
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
        serializer.write_string(schema, self)
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
        if let Some(value) = self.as_ref() {
            value.serialize(schema, serializer)
        } else {
            serializer.skip(schema)
        }
    }
}
