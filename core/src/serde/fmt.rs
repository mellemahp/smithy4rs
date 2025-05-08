use std::fmt::{Error, Write};
use std::marker::PhantomData;
use std::time::Instant;
use bigdecimal::BigDecimal;
use bigdecimal::num_traits::real::Real;
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;
use thiserror::Error;
use crate::schema::documents::{Document, DocumentError};
use crate::schema::Schema;
use crate::schema::traits::SensitiveTrait;
use crate::serde::se::{ListSerializer, MapSerializer, Serialize, StructSerializer, Serializer};

macro_rules! comma {
    ($self:ident) => {
        if (!$self.is_first) {
            $self.parent.sink.write_str(", ")?;
        } else {
            $self.is_first = false;
        };
    };
}

const REDACTED_STRING: &str = "**REDACTED**";
macro_rules! redact {
    ($self:ident, $schema:ident, $expr:expr) => {
        if $schema.contains_trait_type::<SensitiveTrait>() {
            $self.sink.write_str(REDACTED_STRING)
        } else {
            $expr
        }
    };
}

/// Serializer used to format and print a shape.
///
/// U
pub struct FmtSerializer<W: Write> {
    pub(super) sink: W,
}

impl <'a, W: Write> FmtSerializer<W> {
    pub const fn new(sink: W) -> Self {
        FmtSerializer { sink }
    }

    pub fn flush(self) -> W {
        self.sink
    }
}

impl Default for FmtSerializer<String> {
    fn default() -> Self {
        FmtSerializer::new(String::new())
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

impl <W: Write> Serializer for FmtSerializer<W> {
    type Error = Error;
    type Ok = ();
    type SerializeList<'l> = FmtListSerialize<'l, W>
    where Self: 'l;
    type SerializeMap<'m> = FmtMapSerializer<'m, W>
    where Self: 'm;
    type SerializeStruct<'s> = FmtStructSerializer<'s, W>
    where Self: 's;

    fn write_struct(&mut self, schema: &Schema, size: usize) -> Result<Self::SerializeStruct<'_>, Self::Error> {
        self.sink.write_str(schema.id.name.as_str())?;
        self.sink.write_char('[')?;
        Ok(FmtStructSerializer::new(self, schema.contains_trait_type::<SensitiveTrait>()))
    }

    fn write_map(&mut self, schema: &Schema, _: usize) -> Result<Self::SerializeMap<'_>, Self::Error> {
        self.sink.write_str("{")?;
        Ok(FmtMapSerializer::new(self, schema.contains_trait_type::<SensitiveTrait>()))
    }

    fn write_list(&mut self, schema: &Schema, _: usize) -> Result<Self::SerializeList<'_>, Self::Error> {
        self.sink.write_str("[")?;
        Ok(FmtListSerialize::new(self, schema.contains_trait_type::<SensitiveTrait>()))
    }

    fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<(), Self::Error> {
        redact!(self, schema, self.sink.write_str(&value.to_string()))
    }

    fn write_byte(&mut self, schema: &Schema, value: i8) -> Result<(), Self::Error> {
        redact!(self, schema, self.sink.write_str(value.to_string().as_str()))
    }

    fn write_short(&mut self, schema: &Schema, value: i16) -> Result<(), Self::Error> {
        redact!(self, schema, self.sink.write_str(value.to_string().as_str()))
    }

    fn write_integer(&mut self, schema: &Schema, value: i32) -> Result<(), Self::Error> {
        redact!(self, schema, self.sink.write_str(value.to_string().as_str()))
    }

    fn write_long(&mut self, schema: &Schema, value: i64) -> Result<(), Self::Error> {
        redact!(self, schema, self.sink.write_str(value.to_string().as_str()))
    }

    fn write_float(&mut self, schema: &Schema, value: f32) -> Result<(), Self::Error> {
        redact!(self, schema, self.sink.write_str(value.to_string().as_str()))
    }

    fn write_double(&mut self, schema: &Schema, value: f64) -> Result<(), Self::Error> {
        redact!(self, schema, self.sink.write_str(value.to_string().as_str()))
    }

    fn write_big_integer(&mut self, schema: &Schema, value: &BigInt) -> Result<(), Self::Error> {
        redact!(self, schema, self.sink.write_str(value.to_string().as_str()))
    }

    fn write_big_decimal(
        &mut self,
        schema: &Schema,
        value: &BigDecimal,
    ) -> Result<(), Self::Error> {
        redact!(self, schema, self.sink.write_str(value.to_string().as_str()))
    }

    fn write_string(&mut self, schema: &Schema, value: &String) -> Result<(), Self::Error> {
        redact!(self, schema, self.sink.write_str(value.as_str()))
    }

    fn write_blob(&mut self, _: &Schema, value: &ByteBuffer) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_timestamp(&mut self, schema: &Schema, value: &Instant) -> Result<(), Self::Error> {
        // TODO: This is incorrect and needs to be fixed. Just to get all branches running
        redact!(self,schema,self.sink.write_str(value.elapsed().as_secs().to_string().as_str()))
    }

    fn write_document(&mut self, _: &Schema, value: &Document) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_null(&mut self, _: &Schema) -> Result<(), Self::Error> {
        self.sink.write_str("null")
    }

    fn skip(&mut self, _: &Schema) -> Result<(), Self::Error> {
        /* Do not write anything on non-present fields */
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // Does nothing for string serializer
        Ok(())
    }
}

pub struct FmtListSerialize<'se, W: Write> {
    parent: &'se mut FmtSerializer<W>,
    redacted: bool,
    is_first: bool,
}
impl <'a, W: Write> FmtListSerialize<'a, W> {
    fn new(parent: &'a mut FmtSerializer<W>, redacted: bool) -> Self {
        FmtListSerialize {  parent, redacted, is_first: true }
    }
}
impl <W: Write> ListSerializer for FmtListSerialize<'_, W> {
    type Error = Error;
    type Ok = ();

    fn serialize_element<T>(&mut self, element_schema: &Schema, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        if self.redacted { return Ok(()) }
        comma!(self);
        value.serialize(element_schema, self.parent)
    }

    fn end(self, _: &Schema) -> Result<Self::Ok, Self::Error> {
        self.parent.sink.write_char(']')
    }
}

pub struct FmtMapSerializer<'se, W: Write> {
    parent: &'se mut FmtSerializer<W>,
    redacted: bool,
    is_first: bool,
}
impl <'se, W: Write> FmtMapSerializer<'se, W> {
    fn new(parent: &'se mut FmtSerializer<W>, redacted: bool) -> Self {
        Self { parent, redacted, is_first: true }
    }
}
impl <W: Write> MapSerializer for FmtMapSerializer<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_entry<K, V>(&mut self, key_schema: &Schema, value_schema: &Schema, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize
    {
        if self.redacted { return Ok(())}
        comma!(self);
        key.serialize(key_schema, self.parent)?;
        self.parent.sink.write_char(':')?;
        value.serialize(value_schema, self.parent)
    }

    fn end(self, _: &Schema) -> Result<Self::Ok, Self::Error> {
        self.parent.sink.write_char('}')
    }
}

pub struct FmtStructSerializer<'se, W: Write> {
    parent: &'se mut FmtSerializer<W>,
    redacted: bool,
    is_first: bool,
}
impl <'se, W: Write> FmtStructSerializer<'se, W> {
    fn new(parent: &'se mut FmtSerializer<W>, redacted: bool) -> Self {
        Self { parent, redacted, is_first: true }
    }
}
impl <W: Write> StructSerializer for FmtStructSerializer<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_member<T>(&mut self, member_schema: &Schema, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize
    {
        if self.redacted { return Ok(())}
        comma!(self);
        self.parent.sink.write_str(&member_schema.member_name.as_ref().expect("EEK!").as_str())?;
        self.parent.sink.write_char('=')?;
        value.serialize(member_schema, self.parent)
    }

    fn end(self, schema: &Schema) -> Result<Self::Ok, Self::Error> {
        self.parent.sink.write_char(']')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::prelude;
    use crate::schema::ShapeId;
    use crate::{lazy_member_schema, traits};
    use std::sync::Arc;
    use std::sync::LazyLock;
    use crate::schema::traits::SensitiveTrait;
    use crate::serde::shapes::SerializeShape;

    static SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
        Schema::structure_builder(ShapeId::from("com.example#Shape"))
            .put_member("a", &prelude::STRING, traits![])
            .put_member("b", &prelude::STRING, traits![SensitiveTrait::new()])
            .put_member("c", &prelude::STRING, traits![])
            .build()
    });
    lazy_member_schema!(MEMBER_A, SCHEMA, "a");
    lazy_member_schema!(MEMBER_B, SCHEMA, "b");
    lazy_member_schema!(MEMBER_C, SCHEMA, "c");

    //#[derive(SerializableStruct)]
    //#[schema(SCHEMA)]
    pub(crate) struct SerializeMe {
        // #[schema(MEMBER_A)]
        pub member_a: String,
        // #[schema(MEMBER_B)]
        pub member_b: String,
        // #[schema(MEMBER_C)]
        pub member_c: Option<String>
    }

    impl SerializeShape for SerializeMe {
        fn schema(&self) -> &Schema {
            &SCHEMA
        }
    }

    impl Serialize for SerializeMe {
        fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error>
        {
            let mut ser = serializer.write_struct(schema, 2)?;
            ser.serialize_member(&MEMBER_A, &self.member_a)?;
            ser.serialize_member(&MEMBER_B, &self.member_b)?;
            ser.serialize_optional_member(&MEMBER_C, &self.member_c)?;
            ser.end(schema)
        }
    }

    #[test]
    fn fmt_serializer_simple() {
        let mut fmter = FmtSerializer::default();
        let struct_to_write = SerializeMe {
            member_a: "a".to_string(),
            member_b: "b".to_string(),
            member_c: None
        };
        struct_to_write.serialize_shape(&mut fmter).expect("serialization failed");
        assert_eq!(fmter.flush(), "Shape[a=a, b=**REDACTED**]");
    }
}