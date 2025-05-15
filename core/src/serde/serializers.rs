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
    fn serialize<'a, S: Serializer<'a>>(&self, schema: &'a Schema, serializer: &mut S) -> SerializerResult<S::Error>;
}

/// Represents the empty return of a serializer call that could fail.
pub type SerializerResult<E> = Result<(), E>;

// TODO: Docs
pub trait ListSerializer<'l> {
    /// Must match the `Error` type of our `Serializer and be able to handle unknown errors.
    type Error: Error + From<Box<dyn Error>>;

    /// Serialize a sequence element.
    fn serialize_element<'a, T>(&mut self, element_schema: &'a Schema, value: &T) -> SerializerResult<Self::Error>
    where
        T: ?Sized + Serialize;

    /// Finish serializing a sequence.
    fn end(&mut self, schema: &Schema) -> SerializerResult<Self::Error> {
        /* Does nothing by default */
        Ok(())
    }
}

// TODO: Docs
pub trait MapSerializer<'m> {
    /// Must match the `Error` type of our [`Serializer`].
    type Error: Error + From<Box<dyn Error>>;

    /// Serialize a single map entry
    fn serialize_entry<'a, K, V>(&mut self, key_schema: &'a Schema, value_schema: &'a Schema, key: &K, value: &V) -> SerializerResult<Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize;

    /// Finish serializing a map.
    fn end(&mut self, schema: &Schema) -> SerializerResult<Self::Error> {
        /* Does nothing by default */
        Ok(())
    }
}

// TODO: Docs
pub trait StructSerializer<'s> {
    /// Must match the `Error` type of our [`Serializer`].
    type Error: Error + From<Box<dyn Error>>;

    /// Serialize a member on the struct
    fn serialize_member<'a, T>(&mut self, member_schema: &'a Schema, value: &T) -> SerializerResult<Self::Error>
    where
        T: ?Sized + Serialize,
        'a: 's;

    /// Serializes an optional member.
    ///
    /// This method will call [`StructSerializer::skip`] any optional members
    /// that are `None`, otherwise the `Some` value is unwrapped and serialized as normal.
    fn serialize_optional_member<T: Serialize>(&mut self, member_schema: &Schema, value: &Option<T>) -> SerializerResult<Self::Error> {
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
    fn end<'a>(&mut self, schema: &'a Schema<'a>) -> SerializerResult<Self::Error> {
        /* Do nothing by default */
        Ok(())
    }
}

// TODO: datastream?
// TODO: event stream?
// TODO: Docs
pub trait Serializer<'serializer>: Sized {
    /// Error type emitted on failed serialization.
    ///
    /// **Note**: Serializers need to be able to catch and convert dyn Errors from their code.
    type Error: Error + From<Box<dyn Error>>;

    type SerializeList<'l>: ListSerializer<'l, Error=Self::Error>
    where Self: 'l;
    type SerializeMap<'m>: MapSerializer<'m, Error=Self::Error>
    where Self: 'm;
    type SerializeStruct<'s>: StructSerializer<'s, Error=Self::Error>
    where Self: 's;

    fn write_struct(&mut self, schema: &Schema, len: usize) -> Result<Self::SerializeStruct<'_>, Self::Error>;
    fn write_map(&mut self, schema: &Schema, len: usize) -> Result<Self::SerializeMap<'_>, Self::Error>;
    fn write_list(&mut self, schema: &Schema, len: usize) -> Result<Self::SerializeList<'_>, Self::Error>;
    fn write_boolean<'a>(&mut self, schema: &'a Schema, value: bool) -> SerializerResult<Self::Error>
    where 'a: 'serializer;
    fn write_byte<'a>(&mut self, schema: &'a Schema, value: i8) -> SerializerResult<Self::Error>
    where 'a: 'serializer;
    fn write_short<'a>(&mut self, schema: &'a Schema, value: i16) -> SerializerResult<Self::Error>
    where 'a: 'serializer;
    fn write_integer<'a>(&mut self, schema: &'a Schema, value: i32) -> SerializerResult<Self::Error>
    where 'a: 'serializer;
    fn write_long<'a>(&mut self, schema: &'a Schema, value: i64) -> SerializerResult<Self::Error>
    where 'a: 'serializer;
    fn write_float<'a>(&mut self, schema: &'a Schema, value: f32) -> SerializerResult<Self::Error>
    where 'a: 'serializer;
    fn write_double<'a>(&mut self, schema: &'a Schema, value: f64) -> SerializerResult<Self::Error>
    where 'a: 'serializer;
    fn write_big_integer<'a>(&mut self, schema: &'a Schema, value: &BigInt) -> SerializerResult<Self::Error>
    where 'a: 'serializer;
    fn write_big_decimal<'a>(&mut self, schema: &'a Schema, value: &BigDecimal) -> SerializerResult<Self::Error>
    where 'a: 'serializer;
    fn write_string<'a>(&mut self, schema: &'a Schema, value: &String) -> SerializerResult<Self::Error>
    where 'a: 'serializer;
    fn write_blob<'a>(&mut self, schema: &'a Schema, value: &ByteBuffer) -> SerializerResult<Self::Error>
    where 'a: 'serializer;
    fn write_timestamp<'a>(&mut self, schema: &'a Schema, value: &Instant) -> SerializerResult<Self::Error>
    where 'a: 'serializer;
    fn write_document<'a>(&mut self, schema: &'a Schema, value: &Document) -> SerializerResult<Self::Error>
    where 'a: 'serializer;
    fn write_null(&mut self, schema: &Schema) -> SerializerResult<Self::Error>;
    fn skip(&mut self, schema: &Schema) -> SerializerResult<Self::Error>;

    // TODO: Is this necessary?
    fn flush(&mut self) -> SerializerResult<Self::Error> {
        todo!();
    }
}

// === Default implementations ===
impl<T: Serialize> Serialize for Vec<T> {
    fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> SerializerResult<S::Error>
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
    fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> SerializerResult<S::Error>
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
   fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> SerializerResult<S::Error> {
        serializer.write_boolean(schema, *self)
    }
}

impl Serialize for i8 {
   fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> SerializerResult<S::Error> {
        serializer.write_byte(schema, *self)
    }
}

impl Serialize for i16 {
    fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> SerializerResult<S::Error> {
        serializer.write_short(schema, *self)
    }
}

impl Serialize for i32 {
   fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> SerializerResult<S::Error> {
        serializer.write_integer(schema, *self)
    }
}

impl Serialize for i64 {
   fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> SerializerResult<S::Error> {
        serializer.write_long(schema, *self)
    }
}

impl Serialize for f32 {
   fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> SerializerResult<S::Error> {
        serializer.write_float(schema, *self)
    }
}

impl Serialize for f64 {
   fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> SerializerResult<S::Error> {
        serializer.write_double(schema, *self)
    }
}

impl Serialize for BigInt {
   fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> SerializerResult<S::Error> {
        serializer.write_big_integer(schema, self)
    }
}

impl Serialize for BigDecimal {
   fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> SerializerResult<S::Error> {
        serializer.write_big_decimal(schema, self)
    }
}

impl Serialize for ByteBuffer {
   fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> SerializerResult<S::Error> {
        serializer.write_blob(schema, self)
    }
}

impl Serialize for String {
    fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> SerializerResult<S::Error>
    {
        serializer.write_string(schema, self)
    }
}

impl <T: Serialize> Serialize for Option<T> {
   fn serialize<'a, S: Serializer<'a>>(&self, schema: &Schema, serializer: &mut S) -> SerializerResult<S::Error> {
        if let Some(value) = self.as_ref() {
            value.serialize(schema, serializer)
        } else {
            serializer.skip(schema)
        }
    }
}
