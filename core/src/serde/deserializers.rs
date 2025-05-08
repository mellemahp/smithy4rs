#![allow(dead_code)]

use crate::schema::documents::Document;
use crate::schema::Schema;
use crate::serde::serializers::Serialize;
use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;
use std::error::Error;
use std::time::Instant;

pub trait Deserializable: Sized {
    fn schema() -> &'static Schema<'static>;

    fn deserialize<D: Deserializer>(mut self, decoder: &mut D) -> Result<Self, D::Error> {
        decoder.read_struct(Self::schema(), &mut self, Self::deserialize_member)?;
        Ok(self)
    }

    fn deserialize_member<D: Deserializer>(
        &mut self,
        member_schema: &Schema,
        member_deserializer: &mut D,
    ) -> Result<(), D::Error>;

    fn error_correction(&mut self) {
        todo!()
    }
}

pub trait ShapeBuilder<T: Serialize>: Deserializable {
    fn build(self) -> T;

    #[allow(unused_variables)]
    fn set_member<V>(&mut self, member_schema: &Schema, value: V) {
        todo!()
    }
}

// TODO: datastream?
// TODO: event stream?
pub trait Deserializer: Sized {
    type Error: Error;

    fn read_struct<T>(
        &mut self,
        schema: &Schema,
        state: &mut T,
        consumer: StructConsumer<T, Self>,
    ) -> Result<(), Self::Error>;

    fn read_list<T>(
        &mut self,
        schema: &Schema,
        state: T,
        consumer: ListConsumer<T, Self>,
    ) -> Result<(), Self::Error>;

    fn read_string_map<T>(
        schema: &Schema,
        state: T,
        consumer: StringMapConsumer<T, Self>,
    ) -> Result<(), Self::Error>;

    fn read_boolean(&mut self, schema: &Schema) -> Result<bool, Self::Error>;

    fn read_blob(&mut self, schema: &Schema) -> Result<ByteBuffer, Self::Error>;

    fn read_byte(&mut self, schema: &Schema) -> Result<u8, Self::Error>;

    fn read_short(&mut self, schema: &Schema) -> Result<i16, Self::Error>;

    fn read_integer(&mut self, schema: &Schema) -> Result<i32, Self::Error>;

    fn read_long(&mut self, schema: &Schema) -> Result<i64, Self::Error>;

    fn read_float(&mut self, schema: &Schema) -> Result<f32, Self::Error>;

    fn read_double(&mut self, schema: &Schema) -> Result<f64, Self::Error>;

    fn read_big_integer(&mut self, schema: &Schema) -> Result<BigInt, Self::Error>;

    fn read_big_decimal(&mut self, schema: &Schema) -> Result<BigDecimal, Self::Error>;

    fn read_string(&mut self, schema: &Schema) -> Result<&str, Self::Error>;

    fn read_timestamp(&mut self, schema: &Schema) -> Result<Instant, Self::Error>;

    fn read_document(&mut self, schema: &Schema) -> Result<Document, Self::Error>;

    // Peek at next value to determine if it is null without consuming
    fn is_null(&mut self) -> bool;

    //  Read (skip) the null value. Only makes sense after is_null().
    fn read_null<T>(&mut self) -> Result<(), Self::Error>;

    // Finish reading all remaining data
    fn finish(&mut self) -> Result<(), Self::Error>;
}

pub type StructConsumer<T, D> =
    fn(state: &mut T, schema: &Schema, decoder: &mut D) -> Result<(), <D as Deserializer>::Error>;
pub type ListConsumer<T, D> =
    fn(state: &mut T, schema: &Schema, decoder: &mut D) -> Result<(), <D as Deserializer>::Error>;
pub type StringMapConsumer<T, D> = fn(
    state: &mut T,
    schema: &Schema,
    key: &str,
    decoder: &mut D,
) -> Result<(), <D as Deserializer>::Error>;

// INTERCEPTING DESERIALIZER?
