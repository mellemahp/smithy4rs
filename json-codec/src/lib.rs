extern crate smithy4rs_core;

use std::time::Instant;
use json::JsonValue;
use smithy4rs_core::{BigDecimal, BigInt, ByteBuffer};
use smithy4rs_core::schema::Schema;
use smithy4rs_core::serde::{MapSerializer, SerializableStruct, Serializer};

pub struct JsonSerializer {
    pub string: String,
}

impl JsonSerializer {
    pub const fn new() -> Self {
        JsonSerializer { string: String::new() }
    }
}

impl Serializer for JsonSerializer {
    fn write_struct<T: SerializableStruct>(&mut self, schema: &Schema, structure: &T) {
        let mut data = JsonValue::new_object();
        structure.serialize_members(&mut StructSerializer::new(&mut data));
        self.string.push_str(data.dump().as_str())
    }

    fn write_map<T, M: MapSerializer>(&mut self, schema: &Schema, map_state: T, size: usize, consumer: fn(T, M)) {
        todo!()
    }

    fn write_list<T, L: Serializer>(&mut self, schema: &Schema, list_state: T, size: usize, consumer: fn(T, L)) {
        todo!()
    }

    fn write_boolean(&mut self, schema: &Schema, value: bool) {
        self.string.push_str(json::stringify(value).as_str())
    }

    fn write_byte(&mut self, schema: &Schema, value: u8) {
        self.string.push_str(json::stringify(value).as_str())
    }

    fn write_short(&mut self, schema: &Schema, value: i16) {
        self.string.push_str(json::stringify(value).as_str())
    }

    fn write_integer(&mut self, schema: &Schema, value: i32) {
        self.string.push_str(json::stringify(value).as_str())
    }

    fn write_long(&mut self, schema: &Schema, value: i64) {
        self.string.push_str(json::stringify(value).as_str())
    }

    fn write_float(&mut self, schema: &Schema, value: f32) {
        self.string.push_str(json::stringify(value).as_str())
    }

    fn write_double(&mut self, schema: &Schema, value: f64) {
        self.string.push_str(json::stringify(value).as_str())
    }

    fn write_big_integer(&mut self, schema: &Schema, value: BigInt) {
        todo!()
    }

    fn write_big_decimal(&mut self, schema: &Schema, value: BigDecimal) {
        todo!()
    }

    fn write_string(&mut self, schema: &Schema, value: &str) {
        self.string.push_str(json::stringify(value).as_str())
    }

    fn write_blob(&mut self, schema: &Schema, value: ByteBuffer) {
        todo!()
    }

    fn write_timestamp(&mut self, schema: &Schema, value: Instant) {
        todo!()
    }

    fn write_document(&mut self, schema: &Schema, value: smithy4rs_core::documents::Document) {
        todo!()
    }

    fn write_null(&mut self, schema: &Schema) {
        self.string.push_str(JsonValue::Null.dump().as_str())
    }

    fn flush(&self) {
        // Do nothing
    }
}

struct StructSerializer<'s> {
    parent: &'s mut JsonValue,
}

impl <'a> StructSerializer<'a> {
    const fn new(parent: &'a mut JsonValue) -> Self {
        StructSerializer { parent }
    }
}

impl Serializer for StructSerializer<'_> {
    fn write_struct<T: SerializableStruct>(&mut self, schema: &Schema, structure: &T) {
        let mut data = JsonValue::new_object();
        structure.serialize_members(&mut StructSerializer::new(&mut data));
        self.parent[get_member_name(schema)] = data;
    }

    fn write_map<T, M: MapSerializer>(&mut self, schema: &Schema, map_state: T, size: usize, consumer: fn(T, M)) {
        todo!()
    }

    fn write_list<T, L: Serializer>(&mut self, schema: &Schema, list_state: T, size: usize, consumer: fn(T, L)) {
        todo!()
    }

    fn write_boolean(&mut self, schema: &Schema, value: bool) {
        self.parent[get_member_name(schema)] = json::from(value);
    }

    fn write_byte(&mut self, schema: &Schema, value: u8) {
        todo!()
    }

    fn write_short(&mut self, schema: &Schema, value: i16) {
        self.parent[get_member_name(schema)] = json::from(value);
    }

    fn write_integer(&mut self, schema: &Schema, value: i32) {
        self.parent[get_member_name(schema)] = json::from(value);
    }

    fn write_long(&mut self, schema: &Schema, value: i64) {
        self.parent[get_member_name(schema)] = json::from(value);
    }

    fn write_float(&mut self, schema: &Schema, value: f32) {
        self.parent[get_member_name(schema)] = json::from(value);
    }

    fn write_double(&mut self, schema: &Schema, value: f64) {
        self.parent[get_member_name(schema)] = json::from(value);
    }

    fn write_big_integer(&mut self, schema: &Schema, value: BigInt) {
        todo!()
    }

    fn write_big_decimal(&mut self, schema: &Schema, value: BigDecimal) {
        todo!()
    }

    fn write_string(&mut self, schema: &Schema, value: &str) {
        self.parent[get_member_name(schema)] = json::from(value);
    }

    fn write_blob(&mut self, schema: &Schema, value: ByteBuffer) {
        todo!()
    }

    fn write_timestamp(&mut self, schema: &Schema, value: Instant) {
        todo!()
    }

    fn write_document(&mut self, schema: &Schema, value: smithy4rs_core::documents::Document) {
        todo!()
    }

    fn write_null(&mut self, schema: &Schema) {
        self.parent[get_member_name(schema)] = JsonValue::Null;
    }

    fn flush(&self) {
        // Do nothing on flush
    }
}

fn get_member_name<'s>(schema: &'s Schema) -> &'s str {
    schema.member_name.as_ref().expect("Should have a member name")
}