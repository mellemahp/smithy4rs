#![allow(dead_code)]
#![allow(unused_variables)]

use crate::schema::documents::{Document};
use crate::schema::{Schema};
use crate::BigDecimal;
use crate::BigInt;
use crate::ByteBuffer;
use std::error::Error;
use std::time::Instant;
use indexmap::IndexMap;

/// Schema-Guided serialization
/// TODO: Docs
pub trait Serialize {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error>;
}

// TODO: Docs
pub trait ListSerializer {
    /// Must match the `Error` type of our `Serializer and be able to handle unknown errors.
    type Error: Error + From<Box<dyn Error>>;

    /// Must match the `OK` type of our `Serializer`.
    type Ok;

    /// Serialize a sequence element.
    fn serialize_element<T>(&mut self, element_schema: &Schema, value: &T) -> Result<(), Self::Error>
    where T: ?Sized + Serialize;

    /// Finish serializing a sequence.
    fn end(self, schema: &Schema) -> Result<Self::Ok, Self::Error>;
}

// TODO: Docs
pub trait MapSerializer {
    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Must match the `Error` type of our `Serializer`.
    type Error: Error + From<Box<dyn Error>>;

    fn serialize_entry<K, V>(&mut self, key_schema: &Schema, value_schema: &Schema, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize;

    /// Finish serializing a map.
    fn end(self, schema: &Schema) -> Result<Self::Ok, Self::Error>;
}

// TODO: Docs
pub trait StructSerializer {
    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Must match the `Error` type of our `Serializer`.
    type Error: Error + From<Box<dyn Error>>;

    /// Serialize a member on the struct
    fn serialize_member<T>(&mut self, member_schema: &Schema, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize;

    fn serialize_optional_member<T: Serialize>(&mut self, member_schema: &Schema, value: &Option<T>) -> Result<(), Self::Error>
    {
        if let Some(value) = value {
            self.serialize_member(member_schema, value)
        } else {
            self.skip_member(member_schema)
        }
    }

    fn skip_member(&mut self, schema: &Schema) -> Result<(), Self::Error> {
        /* Do nothing on skip by default */
        Ok(())
    }

    fn end(self, schema: &Schema) -> Result<Self::Ok, Self::Error>;
}

// TODO: datastream?
// TODO: event stream?
// TODO: Docs
pub trait Serializer: Sized {
    type Error: Error + From<Box<dyn Error>>;
    type Ok;

    type SerializeList<'l>: ListSerializer<Ok=Self::Ok, Error=Self::Error>
        where Self: 'l;
    type SerializeMap<'m>: MapSerializer<Ok=Self::Ok, Error=Self::Error>
        where Self: 'm;
    type SerializeStruct<'s>: StructSerializer<Ok=Self::Ok, Error=Self::Error>
        where Self: 's;

    fn write_struct(&mut self, schema: &Schema, len: usize) -> Result<Self::SerializeStruct<'_>, Self::Error>;
    fn write_map(&mut self, schema: &Schema, len: usize) -> Result<Self::SerializeMap<'_>, Self::Error>;
    fn write_list(&mut self, schema: &Schema, len: usize) -> Result<Self::SerializeList<'_>, Self::Error>;
    fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<Self::Ok, Self::Error>;
    fn write_byte(&mut self, schema: &Schema, value: i8) -> Result<Self::Ok, Self::Error>;
    fn write_short(&mut self, schema: &Schema, value: i16) -> Result<Self::Ok, Self::Error>;
    fn write_integer(&mut self, schema: &Schema, value: i32) -> Result<Self::Ok, Self::Error>;
    fn write_long(&mut self, schema: &Schema, value: i64) -> Result<Self::Ok, Self::Error>;
    fn write_float(&mut self, schema: &Schema, value: f32) -> Result<Self::Ok, Self::Error>;
    fn write_double(&mut self, schema: &Schema, value: f64) -> Result<Self::Ok, Self::Error>;
    fn write_big_integer(&mut self, schema: &Schema, value: &BigInt) -> Result<Self::Ok, Self::Error>;
    fn write_big_decimal(&mut self, schema: &Schema, value: &BigDecimal) -> Result<Self::Ok, Self::Error>;
    fn write_string(&mut self, schema: &Schema, value: &String) -> Result<Self::Ok, Self::Error>;
    fn write_blob(&mut self, schema: &Schema, value: &ByteBuffer) -> Result<Self::Ok, Self::Error>;
    fn write_timestamp(&mut self, schema: &Schema, value: &Instant) -> Result<Self::Ok, Self::Error>;
    fn write_document(&mut self, schema: &Schema, value: &Document) -> Result<Self::Ok, Self::Error>;
    fn write_null(&mut self, schema: &Schema) -> Result<Self::Ok, Self::Error>;
    fn skip(&mut self, schema: &Schema) -> Result<Self::Ok, Self::Error>;

    // TODO: Is this necessary?
    fn flush(&mut self) -> Result<Self::Ok, Self::Error> {
        todo!();
    }
}

// === Default implementations ===
impl<T: Serialize> Serialize for Vec<T> {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error>
    {
        let mut list = serializer.write_list(schema, self.len())?;
        let value_schema = schema.expect_member("member");
        for element in self {
            list.serialize_element(value_schema.as_ref(), element)?;
        }
        list.end(schema)
    }
}

impl <K,V> Serialize for IndexMap<K,V>
where
    K: Serialize,
    V: Serialize
{
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error>
    {
        let mut map = serializer.write_map(schema, self.len())?;
        // TODO: is there a more efficient way to store/get these schemas?
        let key_schema = schema.get_member("key").expect("Should have key schema");
        let value_schema = schema.get_member("value").expect("Should have value schema");
        for (k, v) in self {
            map.serialize_entry(key_schema.as_ref(), value_schema.as_ref(), k, v)?;
        }
        map.end(schema)
    }
}

impl Serialize for bool {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error> {
        serializer.write_boolean(schema, *self)
    }
}

impl Serialize for i8 {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error> {
        serializer.write_byte(schema, *self)
    }
}

impl Serialize for i16 {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error> {
        serializer.write_short(schema, *self)
    }
}

impl Serialize for i32 {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error> {
        serializer.write_integer(schema, *self)
    }
}

impl Serialize for i64 {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error> {
        serializer.write_long(schema, *self)
    }
}

impl Serialize for f32 {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error> {
        serializer.write_float(schema, *self)
    }
}

impl Serialize for f64 {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error> {
        serializer.write_double(schema, *self)
    }
}

impl Serialize for BigInt {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error> {
        serializer.write_big_integer(schema, self)
    }
}

impl Serialize for BigDecimal {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error> {
        serializer.write_big_decimal(schema, self)
    }
}

impl Serialize for ByteBuffer {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error> {
        serializer.write_blob(schema, self)
    }
}

impl Serialize for String {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error>
    {
        serializer.write_string(schema, self)
    }
}

impl <T: Serialize> Serialize for Option<T> {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error> {
        if let Some(value) = self.as_ref() {
            value.serialize(schema, serializer)
        } else {
            serializer.skip(schema)
        }
    }
}
