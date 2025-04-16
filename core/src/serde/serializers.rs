#![allow(dead_code)]
#![allow(unused_variables)]

use std::error::Error;
use std::ffi::IntoStringError;
use std::time::{Instant};
use crate::schema::Schema;
use crate::BigDecimal;
use crate::BigInt;
use crate::ByteBuffer;
use crate::documents::Document;

pub trait Serializable {
    /// Serialize the state of the shape into the given serializer.
    /// NOTE: This consumes the shapes.
    fn serialize<S: Serializer>(self, serializer: &mut S) -> Result<(), S::Error>;
}

pub trait SerializableStruct: Serializable + Sized {
    fn schema() -> &'static Schema<'static>;

    fn serialize<S: Serializer>(self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.write_struct(Self::schema(), self)
    }

    fn serialize_members<S: Serializer>(self, serializer: &mut S) -> Result<(), S::Error>;

    #[allow(unused_variables)]
    fn get_member_value<T>(&self, member_schema: &Schema) -> T {
        todo!();
    }
}

// TODO: datastream?
// TODO: event stream?
pub trait Serializer: Sized {
    type Error: Error;

    fn write_struct<T: SerializableStruct>(&mut self, schema: &Schema, structure: T) -> Result<(), Self::Error>;
    // TODO: Should this be write string map?
    fn write_map<T, C: MapConsumer<T>>(&mut self, schema: &Schema, size: usize, map_state:T, consumer: C) -> Result<(), Self::Error>;
    fn write_list<T, C: ListConsumer<T>>(&mut self, schema: &Schema, size: usize, list_state: T, consumer: C) -> Result<(), Self::Error>;
    fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<(), Self::Error>;
    fn write_byte(&mut self, schema: &Schema, value: u8) -> Result<(), Self::Error>;
    fn write_short(&mut self, schema: &Schema, value: i16) -> Result<(), Self::Error>;
    fn write_integer(&mut self, schema: &Schema, value: i32) -> Result<(), Self::Error>;
    fn write_long(&mut self,schema: &Schema, value: i64) -> Result<(), Self::Error>;
    fn write_float(&mut self, schema: &Schema, value: f32) -> Result<(), Self::Error>;
    fn write_double(&mut self, schema: &Schema, value: f64) -> Result<(), Self::Error>;
    fn write_big_integer(&mut self, schema: &Schema, value: BigInt) -> Result<(), Self::Error>;
    fn write_big_decimal(&mut self, schema: &Schema, value: BigDecimal) -> Result<(), Self::Error>;
    fn write_string(&mut self, schema: &Schema, value: String) -> Result<(), Self::Error>;
    fn write_blob(&mut self, schema: &Schema, value: ByteBuffer) -> Result<(), Self::Error>;
    fn write_timestamp(&mut self, schema: &Schema, value: Instant) -> Result<(), Self::Error>;
    fn write_document(&mut self, schema: &Schema, value: Document) -> Result<(), Self::Error>;
    fn write_null(&mut self, schema: &Schema) -> Result<(), Self::Error>;

    fn flush(&self) -> Result<(), Self::Error> {
        todo!();
    }
}


// TODO: Should <T> be required to be iterable? Then the actual thing would be dependent on the item.
pub trait ListConsumer<T> {
    fn accept<S: Serializer>(self, state: T, serializer: &mut S) -> Result<(), S::Error>;
}

pub trait MapConsumer<T> {
    fn accept<S: Serializer>(key_schema: &Schema, key: &str, state: T, value_serializer: MapValueSerializer<T, S>) -> Result<(), S::Error>;
}

pub type MapValueSerializer<T, S> = fn(state: T, serializer:  &mut S) -> Result<(), <S as Serializer>::Error>;

#[allow(unused_variables)]
pub trait Interceptor<S: Serializer> {
    fn before(&mut self, schema: &Schema, sink: &mut S) -> Result<(), S::Error> {
        /* Do nothing by default */
        Ok(())
    }

    fn after(&mut self, schema: &Schema, sink: &mut S) -> Result<(), S::Error> {
        /* Do nothing by default */
        Ok(())
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
    type Error = S::Error;

    fn write_struct<T: SerializableStruct>(&mut self, schema: &Schema, structure: T) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_struct(schema, structure)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_map<T, C: MapConsumer<T>>(&mut self, schema: &Schema, size: usize, map_state: T, consumer: C) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_map(schema,  size, map_state, consumer)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_list<T, C: ListConsumer<T>>(&mut self, schema: &Schema, size: usize, list_state: T, consumer: C) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_list(schema, size, list_state, consumer)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_boolean(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_byte(&mut self, schema: &Schema, value: u8) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_byte(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_short(&mut self, schema: &Schema, value: i16) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_short(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_integer(&mut self, schema: &Schema, value: i32) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_integer(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_long(&mut self, schema: &Schema, value: i64) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_long(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_float(&mut self, schema: &Schema, value: f32) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_float(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_double(&mut self, schema: &Schema, value: f64) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_double(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_big_integer(&mut self, schema: &Schema, value: BigInt) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_big_integer(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_big_decimal(&mut self, schema: &Schema, value: BigDecimal) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_big_decimal(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_string(&mut self, schema: &Schema, value: String) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_string(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_blob(&mut self, schema: &Schema, value: ByteBuffer) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_blob(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_timestamp(&mut self, schema: &Schema, value: Instant) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_timestamp(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_document(&mut self, schema: &Schema, value: Document) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_document(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_null(&mut self, schema: &Schema) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_null(schema)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn flush(&self) -> Result<(), Self::Error> {
        self.delegate.flush()
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
        shape.serialize(&mut serializer).expect("serialization failed");
        serializer.string
    }
}

// TODO: Could this be made infallible?
impl Serializer for FmtSerializer {
    type Error = IntoStringError;

    fn write_struct<T: SerializableStruct>(&mut self, schema: &Schema, structure: T) -> Result<(), Self::Error> {
        let name = schema.member_target.map(|t| &t.id.name ).unwrap_or(&schema.id.name);
        self.string.push_str(name);
        self.string.push('[');
        structure.serialize_members(&mut InterceptingSerializer::new(self, StructWriter::new()))?;
        self.string.push(']');
        Ok(())
    }

    #[allow(unused_variables)]
    fn write_map<T, C: MapConsumer<T>>(&mut self, schema: &Schema, size: usize, map_state: T, consumer: C) -> Result<(), Self::Error> {
        todo!()
    }

    #[allow(unused_variables)]
    fn write_list<T, C: ListConsumer<T>>(&mut self, schema: &Schema, size: usize, list_state: T, consumer: C) -> Result<(), Self::Error> {
        if size == 0 {
            return Ok(())
        }
        self.string.push('[');
        consumer.accept(list_state, &mut InterceptingSerializer::new(self, CommaWriter::new()))?;
        self.string.push(']');
        Ok(())
    }

    fn write_boolean(&mut self, _: &Schema, value: bool) -> Result<(), Self::Error> {
        self.string.push_str(&value.to_string());
        Ok(())
    }

    fn write_byte(&mut self, _: &Schema, value: u8) -> Result<(), Self::Error> {
        self.string.push_str(value.to_string().as_str());
        Ok(())
    }

    fn write_short(&mut self, _: &Schema, value: i16) -> Result<(), Self::Error> {
        self.string.push_str(value.to_string().as_str());
        Ok(())
    }

    fn write_integer(&mut self, _: &Schema, value: i32) -> Result<(), Self::Error> {
        self.string.push_str(value.to_string().as_str());
        Ok(())
    }

    fn write_long(&mut self, _: &Schema, value: i64) -> Result<(), Self::Error> {
        self.string.push_str(value.to_string().as_str());
        Ok(())
    }

    fn write_float(&mut self, _: &Schema, value: f32) -> Result<(), Self::Error> {
        self.string.push_str(value.to_string().as_str());
        Ok(())
    }

    fn write_double(&mut self, _: &Schema, value: f64) -> Result<(), Self::Error> {
        self.string.push_str(value.to_string().as_str());
        Ok(())
    }

    fn write_big_integer(&mut self, _: &Schema, value: BigInt) -> Result<(), Self::Error> {
        self.string.push_str(value.to_string().as_str());
        Ok(())
    }

    fn write_big_decimal(&mut self, _: &Schema, value: BigDecimal) -> Result<(), Self::Error> {
        self.string.push_str(value.to_string().as_str());
        Ok(())
    }

    fn write_string(&mut self, _: &Schema, value: String) -> Result<(), Self::Error> {
        self.string.push_str(value.as_str());
        Ok(())
    }

    fn write_blob(&mut self, _: &Schema, value: ByteBuffer) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_timestamp(&mut self, _: &Schema, value: Instant) -> Result<(), Self::Error> {
        // TODO: This is incorrect and needs to be fixed. Just to get all branches running
        self.string.push_str(value.elapsed().as_secs().to_string().as_str());
        Ok(())
    }

    fn write_document(&mut self, _: &Schema, value: Document) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_null(&mut self, _: &Schema) -> Result<(), Self::Error> {
        self.string.push_str("null");
        Ok(())
    }

    fn flush(&self) -> Result<(), Self::Error> {
        // Does nothing for string serializer
        Ok(())
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
    fn before(&mut self, schema: &Schema<'_>, sink: &mut FmtSerializer) -> Result<(), IntoStringError> {
        if !self.is_first {
            sink.string.push_str(", ");
        } else {
            self.is_first = false;
        }
        sink.string.push_str(schema.member_name.as_ref().expect("missing member name"));
        sink.string.push('=');
        Ok(())
    }
}

struct CommaWriter {
    is_first: bool
}
impl CommaWriter {
    const fn new() -> Self {
        CommaWriter { is_first: true }
    }
}
impl <'a> Interceptor<FmtSerializer> for CommaWriter {
    fn before(&mut self, _: &Schema<'_>, sink: &mut FmtSerializer) -> Result<(), IntoStringError> {
        if !self.is_first {
            sink.string.push_str(", ");
        } else {
            self.is_first = false;
        }
        Ok(())
    }
}
