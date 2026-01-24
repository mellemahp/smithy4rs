//! # Serde Util
//!
//! Utilities for (de)serialization.

use std::marker::PhantomData;

use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;
use temporal_rs::Instant;

use crate::{
    schema::{Document, Schema},
    serde::se::{
        Error, ListSerializer, MapSerializer, SerializeWithSchema, Serializer, StructSerializer,
    },
};

// ============================================================================
// Key Converter
// ============================================================================

/// Converts a key value to a String if possible.
///
/// Used to support validation paths and string map document key conversions
pub(crate) struct KeySerializer<E: Error>(PhantomData<E>);
impl<E: Error> KeySerializer<E> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}
impl<E: Error> Serializer for &mut KeySerializer<E> {
    type Error = E;
    type Ok = String;
    type SerializeList = NoOpSerializer<E>;
    type SerializeMap = NoOpSerializer<E>;
    type SerializeStruct = NoOpSerializer<E>;

    #[cold]
    fn write_struct(
        self,
        schema: &Schema,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(invalid_key_error(schema))
    }

    #[cold]
    fn write_map(self, schema: &Schema, _len: usize) -> Result<Self::SerializeMap, Self::Error> {
        Err(invalid_key_error(schema))
    }

    #[cold]
    fn write_list(
        self,
        schema: &Schema,
        _len: usize,
    ) -> Result<Self::SerializeList, Self::Error> {
        Err(invalid_key_error(schema))
    }

    #[cold]
    fn write_boolean(self, schema: &Schema, _value: bool) -> Result<Self::Ok, Self::Error> {
        Err(invalid_key_error(schema))
    }

    #[inline]
    fn write_byte(self, _schema: &Schema, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    #[inline]
    fn write_short(self, _schema: &Schema, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    #[inline]
    fn write_integer(self, _schema: &Schema, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    #[inline]
    fn write_long(self, _schema: &Schema, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    #[cold]
    fn write_float(self, schema: &Schema, _value: f32) -> Result<Self::Ok, Self::Error> {
        Err(invalid_key_error(schema))
    }

    #[cold]
    fn write_double(self, schema: &Schema, _value: f64) -> Result<Self::Ok, Self::Error> {
        Err(invalid_key_error(schema))
    }

    #[cold]
    fn write_big_integer(
        self,
        schema: &Schema,
        _value: &BigInt,
    ) -> Result<Self::Ok, Self::Error> {
        Err(invalid_key_error(schema))
    }

    #[cold]
    fn write_big_decimal(
        self,
        schema: &Schema,
        _value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        Err(invalid_key_error(schema))
    }

    #[inline]
    fn write_string(self, _schema: &Schema, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    #[cold]
    fn write_blob(self, schema: &Schema, _value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        Err(invalid_key_error(schema))
    }

    #[cold]
    fn write_timestamp(
        self,
        schema: &Schema,
        _value: &Instant,
    ) -> Result<Self::Ok, Self::Error> {
        Err(invalid_key_error(schema))
    }

    #[cold]
    fn write_document(
        self,
        schema: &Schema,
        _value: &Box<dyn Document>,
    ) -> Result<Self::Ok, Self::Error> {
        Err(invalid_key_error(schema))
    }

    #[cold]
    fn write_null(self, schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Err(invalid_key_error(schema))
    }

    #[cold]
    fn skip(self, schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Err(invalid_key_error(schema))
    }
}

fn invalid_key_error<E: Error>(schema: &Schema) -> E {
    E::custom(format!("Invalid key type: {}", schema.shape_type()))
}

// Structures, maps, and lists cannot be used as map keys so these implementations will never actually be called.
pub(crate) struct NoOpSerializer<E: Error>(PhantomData<E>);
impl<E: Error> ListSerializer for NoOpSerializer<E> {
    type Error = E;
    type Ok = String;

    #[cold]
    fn serialize_element<T>(
        &mut self,
        _element_schema: &Schema,
        _value: &T,
    ) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        unreachable!()
    }

    #[cold]
    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}
impl<E: Error> MapSerializer for NoOpSerializer<E> {
    type Error = E;
    type Ok = String;

    #[cold]
    fn serialize_entry<K, V>(
        &mut self,
        _key_schema: &Schema,
        _value_schema: &Schema,
        _key: &K,
        _value: &V,
    ) -> Result<(), Self::Error>
    where
        K: SerializeWithSchema,
        V: SerializeWithSchema,
    {
        unreachable!()
    }

    #[cold]
    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}
impl<E: Error> StructSerializer for NoOpSerializer<E> {
    type Error = E;
    type Ok = String;

    fn serialize_member<T>(
        &mut self,
        _member_schema: &Schema,
        _value: &T,
    ) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        unreachable!()
    }

    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}
