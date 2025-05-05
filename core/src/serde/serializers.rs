#![allow(dead_code)]
#![allow(unused_variables)]

use crate::BigDecimal;
use crate::BigInt;
use crate::ByteBuffer;
use crate::schema::Schema;
use crate::schema::documents::{Document, DocumentError};
use crate::schema::traits::SensitiveTrait;
use std::error::Error;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::Instant;
use thiserror::Error;

pub trait Serializable {
    /// Serialize the state of the shape into the given serializer.
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error>;
}

pub trait SerializableStruct: Serializable + Sized {
    fn schema(&self) -> &Schema;

    fn serialize_members<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error>;

    #[allow(unused_variables)]
    fn get_member_value<T>(&self, member_schema: &Schema) -> T {
        todo!();
    }
}

// TODO: datastream?
// TODO: event stream?
pub trait Serializer: Sized {
    type Error: Error + Default + From<DocumentError>;

    fn write_struct<T: SerializableStruct>(
        &mut self,
        schema: &Schema,
        structure: &T,
    ) -> Result<(), Self::Error>;

    // TODO: Should this be write string map?
    fn write_map<K, V, C: MapEntryConsumer<K, V>>(
        &mut self,
        schema: &Schema,
        map_state: impl Iterator<Item = (K, V)> + ExactSizeIterator,
        consumer: C,
    ) -> Result<(), Self::Error>;

    fn write_map_entry<K, V, C: MapEntryConsumer<K, V>>(
        &mut self,
        schema: &Schema,
        key: K,
        value: V,
        consumer: &C
    ) -> Result<(), Self::Error>;

    fn write_list<I, C: ListItemConsumer<I>>(
        &mut self,
        schema: &Schema,
        list_state: impl Iterator<Item = I> + ExactSizeIterator,
        consumer: C,
    ) -> Result<(), Self::Error>;

    fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<(), Self::Error>;
    fn write_byte(&mut self, schema: &Schema, value: i8) -> Result<(), Self::Error>;
    fn write_short(&mut self, schema: &Schema, value: i16) -> Result<(), Self::Error>;
    fn write_integer(&mut self, schema: &Schema, value: i32) -> Result<(), Self::Error>;
    fn write_long(&mut self, schema: &Schema, value: i64) -> Result<(), Self::Error>;
    fn write_float(&mut self, schema: &Schema, value: f32) -> Result<(), Self::Error>;
    fn write_double(&mut self, schema: &Schema, value: f64) -> Result<(), Self::Error>;
    fn write_big_integer(&mut self, schema: &Schema, value: &BigInt) -> Result<(), Self::Error>;
    fn write_big_decimal(&mut self, schema: &Schema, value: &BigDecimal)
    -> Result<(), Self::Error>;
    fn write_string(&mut self, schema: &Schema, value: &String) -> Result<(), Self::Error>;
    fn write_blob(&mut self, schema: &Schema, value: &ByteBuffer) -> Result<(), Self::Error>;
    fn write_timestamp(&mut self, schema: &Schema, value: &Instant) -> Result<(), Self::Error>;
    fn write_document(&mut self, schema: &Schema, value: &Document) -> Result<(), Self::Error>;
    fn write_null(&mut self, schema: &Schema) -> Result<(), Self::Error>;

    // TODO: Is this necessary?
    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!();
    }
}

/// Allow's the compiler to monomorphize the specific executions of the `accept` consumer
/// *Impl Note*: Implementers should typically be zero-sized types.
pub trait ListItemConsumer<I> {
    fn write_item<S: Serializer>(item: I, serializer: &mut S) -> Result<(), S::Error>;
}

pub trait MapSerializer<S: Serializer> {
    fn write_entry<K, V, C: MapEntryConsumer<K, V>>(schema: &Schema, key: K, value: V, consumer: C, serializer: &mut S) -> Result<(), S::Error>;
}

pub trait MapEntryConsumer<K, V> {
    fn write_key<S: Serializer>(key: K, serializer: &mut S) -> Result<(), S::Error>;
    fn write_value<S: Serializer>(value: V, serializer: &mut S) -> Result<(), S::Error>;
}

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

pub struct InterceptingSerializer<'a, S: Serializer, I: Interceptor<S>> {
    delegate: &'a mut S,
    decorator: I,
}

impl<'a, S: Serializer, I: Interceptor<S>> InterceptingSerializer<'a, S, I> {
    pub fn new(delegate: &'a mut S, decorator: I) -> Self {
        InterceptingSerializer {
            delegate,
            decorator,
        }
    }
}

impl<S: Serializer, I: Interceptor<S>> Serializer for InterceptingSerializer<'_, S, I> {
    type Error = S::Error;

    fn write_struct<T: SerializableStruct>(
        &mut self,
        schema: &Schema,
        structure: &T,
    ) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_struct(schema, structure)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_map<K, V, C: MapEntryConsumer<K, V>>(
        &mut self,
        schema: &Schema,
        map_state: impl Iterator<Item = (K, V)> + ExactSizeIterator,
        consumer: C,
    ) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_map(schema, map_state, consumer)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_map_entry<K, V, C: MapEntryConsumer<K, V>>(
        &mut self, schema:
        &Schema,
        key: K,
        value: V,
        consumer: &C
    ) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_map_entry(schema, key, value, consumer)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_list<It, C: ListItemConsumer<It>>(
        &mut self,
        schema: &Schema,
        list_state: impl Iterator<Item = It> + ExactSizeIterator,
        consumer: C,
    ) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_list(schema, list_state, consumer)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_boolean(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_byte(&mut self, schema: &Schema, value: i8) -> Result<(), Self::Error> {
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

    fn write_big_integer(&mut self, schema: &Schema, value: &BigInt) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_big_integer(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_big_decimal(
        &mut self,
        schema: &Schema,
        value: &BigDecimal,
    ) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_big_decimal(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_string(&mut self, schema: &Schema, value: &String) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_string(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_blob(&mut self, schema: &Schema, value: &ByteBuffer) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_blob(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_timestamp(&mut self, schema: &Schema, value: &Instant) -> Result<(), Self::Error> {
        self.decorator.before(schema, &mut self.delegate)?;
        self.delegate.write_timestamp(schema, value)?;
        self.decorator.after(schema, &mut self.delegate)?;
        Ok(())
    }

    fn write_document(&mut self, schema: &Schema, value: &Document) -> Result<(), Self::Error> {
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

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.delegate.flush()
    }
}

/// Implements fmt method for shapes, taking the sensitive trait into account.
// TODO: Implement sensitive redaction
#[derive(Default)]
pub struct FmtSerializer {
    pub string: String,
}

impl FmtSerializer {
    pub const fn new() -> Self {
        FmtSerializer {
            string: String::new(),
        }
    }

    pub fn serialize<T: Serializable>(shape: T) -> String {
        let mut serializer = Self::new();
        shape
            .serialize(&mut serializer)
            .expect("serialization failed");
        serializer.string
    }
}


#[derive(Error, Debug, Default)]
pub enum FmtError {
    #[error("Failed to serialize string")]
    #[default]
    Generic,
    #[error("data store disconnected")]
    DocumentConversion(#[from] DocumentError),
}
const REDACTED_STRING: &str = "**REDACTED**";
macro_rules! redact {
    ($self:ident, $schema:ident, $expr:expr) => {
        if $schema.contains_trait_type::<SensitiveTrait>() {
            $self.string.push_str(REDACTED_STRING);
        } else {
            $expr
        }
    };
}
// TODO: Could this be made infallible?
impl Serializer for FmtSerializer {
    type Error = FmtError;

    fn write_struct<T: SerializableStruct>(
        &mut self,
        schema: &Schema,
        structure: &T,
    ) -> Result<(), Self::Error> {
        let name = schema
            .member_target
            .map(|t| &t.id.name)
            .unwrap_or(&schema.id.name);
        self.string.push_str(name);
        self.string.push('[');
        redact!(
            self,
            schema,
            structure
                .serialize_members(&mut StructWriter::wrap(self))?
        );
        self.string.push(']');
        Ok(())
    }

    fn write_map<K, V, C: MapEntryConsumer<K, V>>(
        &mut self,
        schema: &Schema,
        map_state: impl Iterator<Item = (K, V)> + ExactSizeIterator,
        consumer: C,
    ) -> Result<(), Self::Error> {
        self.string.push('{');
        if schema.contains_trait_type::<SensitiveTrait>() {
            self.string.push_str(REDACTED_STRING);
        } else {
            let mut map_serializer = CommaWriter::wrap(self);
            for (key, value) in map_state {
                map_serializer.write_map_entry(schema, key, value, &consumer)?
            }
        }
        self.string.push('}');
        Ok(())
    }

    fn write_map_entry<K, V, C: MapEntryConsumer<K, V>>(&mut self, schema: &Schema, key: K, value: V, consumer: &C) -> Result<(), Self::Error> {
        C::write_key(key, self)?;
        self.string.push(':');
        C::write_value(value, self)
    }

    fn write_list<I, C: ListItemConsumer<I>>(
        &mut self,
        schema: &Schema,
        list_state: impl Iterator<Item = I> + ExactSizeIterator,
        consumer: C,
    ) -> Result<(), Self::Error> {
        self.string.push('[');
        if schema.contains_trait_type::<SensitiveTrait>() {
            self.string.push_str(REDACTED_STRING);
        } else {
            let mut inner_serializer = CommaWriter::wrap(self);
            for item in list_state {
                C::write_item(item, &mut inner_serializer)?;
            }
        }
        self.string.push(']');
        Ok(())
    }

    fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<(), Self::Error> {
        redact!(self, schema, self.string.push_str(&value.to_string()));
        Ok(())
    }

    fn write_byte(&mut self, schema: &Schema, value: i8) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.string.push_str(value.to_string().as_str())
        );
        Ok(())
    }

    fn write_short(&mut self, schema: &Schema, value: i16) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.string.push_str(value.to_string().as_str())
        );
        Ok(())
    }

    fn write_integer(&mut self, schema: &Schema, value: i32) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.string.push_str(value.to_string().as_str())
        );
        Ok(())
    }

    fn write_long(&mut self, schema: &Schema, value: i64) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.string.push_str(value.to_string().as_str())
        );
        Ok(())
    }

    fn write_float(&mut self, schema: &Schema, value: f32) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.string.push_str(value.to_string().as_str())
        );
        Ok(())
    }

    fn write_double(&mut self, schema: &Schema, value: f64) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.string.push_str(value.to_string().as_str())
        );
        Ok(())
    }

    fn write_big_integer(&mut self, schema: &Schema, value: &BigInt) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.string.push_str(value.to_string().as_str())
        );
        Ok(())
    }

    fn write_big_decimal(
        &mut self,
        schema: &Schema,
        value: &BigDecimal,
    ) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.string.push_str(value.to_string().as_str())
        );
        Ok(())
    }

    fn write_string(&mut self, schema: &Schema, value: &String) -> Result<(), Self::Error> {
        redact!(self, schema, self.string.push_str(value.as_str()));
        Ok(())
    }

    fn write_blob(&mut self, _: &Schema, value: &ByteBuffer) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_timestamp(&mut self, schema: &Schema, value: &Instant) -> Result<(), Self::Error> {
        // TODO: This is incorrect and needs to be fixed. Just to get all branches running
        redact!(
            self,
            schema,
            self.string
                .push_str(value.elapsed().as_secs().to_string().as_str())
        );
        Ok(())
    }

    fn write_document(&mut self, _: &Schema, value: &Document) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_null(&mut self, _: &Schema) -> Result<(), Self::Error> {
        self.string.push_str("null");
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // Does nothing for string serializer
        Ok(())
    }
}

struct StructWriter {
    is_first: bool,
}

impl StructWriter {
    const fn new() -> Self {
        StructWriter { is_first: true }
    }

    pub fn wrap(ser: &mut FmtSerializer) -> InterceptingSerializer<FmtSerializer, Self> {
        InterceptingSerializer::new(ser, Self::new())
    }
}
impl<'a> Interceptor<FmtSerializer> for StructWriter {
    fn before(&mut self, schema: &Schema<'_>, sink: &mut FmtSerializer) -> Result<(), FmtError> {
        if !self.is_first {
            sink.string.push_str(", ");
        } else {
            self.is_first = false;
        }
        sink.string
            .push_str(schema.member_name.as_ref().expect("missing member name"));
        sink.string.push('=');
        Ok(())
    }
}

struct CommaWriter {
    is_first: bool,
}
impl CommaWriter {
    fn new() -> Self {
        CommaWriter { is_first: true }
    }

    pub fn wrap(ser: &mut FmtSerializer) -> InterceptingSerializer<FmtSerializer, Self> {
        InterceptingSerializer::new(ser, Self::new())
    }
}
impl<'a> Interceptor<FmtSerializer> for CommaWriter {
    fn before(&mut self, _: &Schema<'_>, sink: &mut FmtSerializer) -> Result<(), FmtError> {
        if !self.is_first {
            sink.string.push_str(", ");
        } else {
            self.is_first = false;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::ShapeId;
    use crate::schema::prelude;
    use crate::{lazy_member_schema, traits};
    use std::sync::Arc;
    use std::sync::LazyLock;

    static SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("com.example#Shape"))
            .put_member("a", &prelude::STRING, traits![])
            .put_member("b", &prelude::STRING, traits![SensitiveTrait::new()])
            .build()
    });
    lazy_member_schema!(MEMBER_A, SCHEMA, "a");
    lazy_member_schema!(MEMBER_B, SCHEMA, "b");

    //#[derive(SerializableStruct)]
    //#[schema(SCHEMA)]
    pub(crate) struct SerializeMe {
        pub member_a: String,
        pub member_b: String,
    }

    impl Serializable for SerializeMe {
        fn serialize<S: Serializer>(self, serializer: &mut S) -> Result<(), S::Error> {
            serializer.write_struct(&SCHEMA, &self)
        }
    }

    impl SerializableStruct for SerializeMe {
        fn schema(&self) -> &'static Schema<'static> {
            &SCHEMA
        }

        fn serialize_members<S: Serializer>(self, serializer: &mut S) -> Result<(), S::Error> {
            serializer.write_string(&MEMBER_A, &self.member_a)?;
            serializer.write_string(&MEMBER_B, &self.member_b)?;
            Ok(())
        }
    }

    #[test]
    fn fmt_serializer_simple() {
        let mut fmter = FmtSerializer::new();
        let struct_to_write = SerializeMe {
            member_a: "a".to_string(),
            member_b: "b".to_string(),
        };
        struct_to_write
            .serialize(&mut fmter)
            .expect("serialization failed");
        assert_eq!(fmter.string, "Shape[a=a, b=**REDACTED**]");
    }
}
