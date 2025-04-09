#![allow(dead_code)]

use std::time::Instant;
use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;
use crate::documents::Document;
use crate::schema::Schema;
use crate::serde::Serializable;

pub trait ShapeBuilder<T: Serializable>: Sized {
    fn schema() -> &'static Schema<'static>;

    fn build(self) -> T;

    fn deserialize<D: Deserializer>(&mut self, decoder: &mut D) -> &mut Self;

    fn deserialize_member<D: Deserializer> (&mut self, schema: &Schema, decoder: &mut D) -> &mut Self {
        self.deserialize(decoder)
    }

    fn ignore(&mut self) {
        /* ignore result returned by builder */
    }

    // TODO: Set member value by schema
    // TODO: Error correction
}

pub trait Deserializer {
    fn read_struct<T, C: StructMemberConsumer<T, Self>>(&mut self, schema: &Schema, state: &mut  T, consumer: C);
    fn read_list<T>(&mut self, schema: &Schema, state: T, consumer: ListMemberConsumer<T, Self>);
    fn read_string_map<T>(schema: &Schema, state: T, consumer: MapMemberConsumer<String, T, Self>);
    fn read_boolean(&mut self, schema: &Schema) -> bool;
    fn read_blob(&mut self, schema: &Schema) -> ByteBuffer;
    // TODO: datastream?
    // TODO: event stream?
    fn read_byte(&mut self, schema: &Schema) -> u8;
    fn read_short(&mut self, schema: &Schema) -> i16;
    fn read_integer(&mut self, schema: &Schema) -> i32;
    fn read_long(&mut self, schema: &Schema) -> i64;
    fn read_float(&mut self, schema: &Schema) -> f32;
    fn read_double(&mut self, schema: &Schema) -> f64;
    fn read_big_integer(&mut self, schema: &Schema) -> BigInt;
    fn read_big_decimal(&mut self, schema: &Schema) -> BigDecimal;
    fn read_string(&mut self, schema: &Schema) -> &str;
    fn read_timestamp(&mut self, schema: &Schema) -> Instant;
    fn read_document(&mut self, schema: &Schema) -> Document;
    fn is_null() -> bool;
    //  Read (skip) the null value. Only makes sense after is_null().
    fn read_null<T>();
}

// TODO: Should this use `FnMut`?
// TODO: Should T, have bounds?
pub trait StructMemberConsumer<T, D: Deserializer + ?Sized> {
    fn accept(&self, state: &mut T, member_schema: &Schema , member_deserializer: &mut D);

    fn unknown_member(state: T, member_name: String) {
        /* Do nothing by default */
    }
}

// TODO: Should these allow closures or only function pointers?
pub type ListMemberConsumer<T, S: Deserializer> = fn(state: T, member_deserializer: S);
pub type MapMemberConsumer<K, T, S: Deserializer> = fn(key: K, state: T, member_deserializer: S);