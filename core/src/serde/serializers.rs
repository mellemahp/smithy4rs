#![allow(dead_code)]

use std::time::{Instant};
use crate::schema::Schema;
use crate::BigDecimal;
use crate::BigInt;
use crate::ByteBuffer;
use crate::documents::Document;

pub trait Serializable {
    /// Serialize the state of the shape into the given serializer.
    fn serialize<S: Serializer>(&self, serializer: &mut S);
}

pub trait SerializableStruct: Serializable {
    fn schema() -> &'static Schema<'static>;
    fn serialize_members<S: Serializer>(&self, serializer: &mut S);
    // TODO: get member value
}

// TODO: docs
// TODO: Add result returns
// Could these try to write to an `Extends` in a way that allows
// coercion into multiple outputs? (string, byte, etc)
pub trait Serializer:  {
    fn write_struct<T: SerializableStruct>(&mut self, schema: &Schema, structure: &T);
    fn write_map<T, M: MapSerializer>(&mut self, schema: &Schema, map_state:T, size: usize, consumer: MapConsumer<T, M>);
    fn write_list<T, L: Serializer>(&mut self, schema: &Schema, list_state: T, size: usize, consumer: ListConsumer<T, L>);
    fn write_boolean(&mut self, schema: &Schema, value: bool);
    fn write_byte(&mut self, schema: &Schema, value: u8);
    fn write_short(&mut self, schema: &Schema, value: i16);
    fn write_integer(&mut self, schema: &Schema, value: i32);
    fn write_long(&mut self,schema: &Schema, value: i64);
    fn write_float(&mut self, schema: &Schema, value: f32);
    fn write_double(&mut self, schema: &Schema, value: f64);
    fn write_big_integer(&mut self, schema: &Schema, value: BigInt);
    fn write_big_decimal(&mut self, schema: &Schema, value: BigDecimal);
    fn write_string(&mut self, schema: &Schema, value: &str);
    fn write_blob(&mut self, schema: &Schema, value: ByteBuffer);
    // TODO: datastream?
    // TODO: event stream?
    fn write_timestamp(&mut self, schema: &Schema, value: Instant);
    fn write_document(&mut self, schema: &Schema, value: Document);
    fn write_null(&mut self, schema: &Schema);
    // TODO: Is flush really needed?
    fn flush(&self);
}

pub type MapConsumer<T, M: MapSerializer> = fn(T, M);
pub type ListConsumer<T, L: Serializer> = fn(T, L);

pub trait MapSerializer {
    fn write_entry<T, S: Serializer>(key_schema: &Schema, key: &str, state: T, value_serializer: fn(T, S));
}

#[allow(unused_variables)]
pub trait Interceptor<S: Serializer> {
    fn before(&mut self, schema: &Schema, sink: &mut S) {
        // Do nothing by default.
    }
    fn after(&mut self, schema: &Schema, sink: &mut S) {
        // Do nothing by default.
    }
}

struct InterceptingSerializer<'a, S: Serializer, I: Interceptor<S>> {
    delegate: &'a mut S,
    decorator: I
}

impl <'a, S: Serializer, I: Interceptor<S>> InterceptingSerializer<'a, S, I> {
    pub fn new(delegate: &'a mut S, decorator: I) -> Self {
        InterceptingSerializer { delegate, decorator }
    }
}

impl <S: Serializer, I: Interceptor<S>> Serializer for InterceptingSerializer<'_, S, I> {
    fn write_struct<T: SerializableStruct>(&mut self, schema: &Schema, structure: &T) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_struct(schema, structure);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_map<T, M: MapSerializer>(&mut self, schema: &Schema, map_state: T, size: usize, consumer: fn(T, M)) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_map(schema, map_state, size, consumer);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_list<T, L: Serializer>(&mut self, schema: &Schema, list_state: T, size: usize, consumer: fn(T, L)) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_list(schema, list_state, size, consumer);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_boolean(&mut self, schema: &Schema, value: bool) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_boolean(schema, value);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_byte(&mut self, schema: &Schema, value: u8) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_byte(schema, value);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_short(&mut self, schema: &Schema, value: i16) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_short(schema, value);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_integer(&mut self, schema: &Schema, value: i32) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_integer(schema, value);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_long(&mut self, schema: &Schema, value: i64) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_long(schema, value);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_float(&mut self, schema: &Schema, value: f32) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_float(schema, value);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_double(&mut self, schema: &Schema, value: f64) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_double(schema, value);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_big_integer(&mut self, schema: &Schema, value: BigInt) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_big_integer(schema, value);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_big_decimal(&mut self, schema: &Schema, value: BigDecimal) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_big_decimal(schema, value);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_string(&mut self, schema: &Schema, value: &str) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_string(schema, value);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_blob(&mut self, schema: &Schema, value: ByteBuffer) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_blob(schema, value);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_timestamp(&mut self, schema: &Schema, value: Instant) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_timestamp(schema, value);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_document(&mut self, schema: &Schema, value: Document) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_document(schema, value);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn write_null(&mut self, schema: &Schema) {
        self.decorator.before(schema, &mut self.delegate);
        self.delegate.write_null(schema);
        self.decorator.after(schema, &mut self.delegate);
    }

    fn flush(&self) {
        self.delegate.flush();
    }
}

/// Implements fmt method for shapes, taking the sensitive trait into account.
// TODO: Implement sensitive redaction
#[derive(Default)]
pub struct FmtSerializer {
    pub string: String
}

impl FmtSerializer {
    pub const fn new() -> Self {
        FmtSerializer { string: String::new() }
    }

    pub fn serialize<T: Serializable>(shape: T) -> String {
        let mut serializer = Self::new();
        shape.serialize(&mut serializer);
        serializer.string
    }
}

impl Serializer for FmtSerializer {
    fn write_struct<T: SerializableStruct>(&mut self, schema: &Schema, structure: &T) {
        let name = schema.member_target.map(|t| &t.id.name ).unwrap_or(&schema.id.name);
        self.string.push_str(name);
        self.string.push_str("[");
        structure.serialize_members(&mut InterceptingSerializer::new(self, StructWriter::new()));
        self.string.push_str("]");
    }

    fn write_map<T, M: MapSerializer>(&mut self, schema: &Schema, map_state: T, size: usize, consumer: fn(T, M)) {
        todo!()
    }

    fn write_list<T, L: Serializer>(&mut self, schema: &Schema, list_state: T, size: usize, consumer: fn(T, L)) {
        todo!()
    }

    fn write_boolean(&mut self, _: &Schema, value: bool) {
        self.string.push_str(&value.to_string());
    }

    fn write_byte(&mut self, _: &Schema, value: u8) {
        self.string.push_str(value.to_string().as_str());
    }

    fn write_short(&mut self, _: &Schema, value: i16) {
        self.string.push_str(value.to_string().as_str());
    }

    fn write_integer(&mut self, _: &Schema, value: i32) {
        self.string.push_str(value.to_string().as_str());
    }

    fn write_long(&mut self, _: &Schema, value: i64) {
        self.string.push_str(value.to_string().as_str());
    }

    fn write_float(&mut self, _: &Schema, value: f32) {
        self.string.push_str(value.to_string().as_str());
    }

    fn write_double(&mut self, _: &Schema, value: f64) {
        self.string.push_str(value.to_string().as_str());
    }

    fn write_big_integer(&mut self, _: &Schema, value: BigInt) {
        self.string.push_str(value.to_string().as_str());
    }

    fn write_big_decimal(&mut self, _: &Schema, value: BigDecimal) {
        self.string.push_str(value.to_string().as_str());
    }

    fn write_string(&mut self, _: &Schema, value: &str) {
        self.string.push_str(value);
    }

    fn write_blob(&mut self, _: &Schema, value: ByteBuffer) {
        todo!()
    }

    fn write_timestamp(&mut self, _: &Schema, value: Instant) {
        // TODO: This is incorrect and needs to be fixed. Just to get all branches running
        self.string.push_str(value.elapsed().as_secs().to_string().as_str());
    }

    fn write_document(&mut self, _: &Schema, value: Document) {
        todo!()
    }

    fn write_null(&mut self, _: &Schema) {
        self.string.push_str("null");
    }

    fn flush(&self) {
        todo!()
    }
}

struct StructWriter {
    is_first: bool
}

impl StructWriter {
    const fn new() -> Self {
        StructWriter { is_first: true }
    }
}
impl <'a> Interceptor<FmtSerializer> for StructWriter {
    fn before(&mut self, schema: &Schema<'_>, sink: &mut FmtSerializer) {
        if !self.is_first {
            sink.string.push_str(", ");
        } else {
            self.is_first = false;
        }
        sink.string.push_str(schema.member_name.as_ref().expect("missing member name"));
        sink.string.push('=');
    }
}
