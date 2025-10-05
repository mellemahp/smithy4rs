use std::{
    cmp::PartialEq,
    fmt::{Display, Error},
    io,
    time::Instant,
};

use thiserror::Error;

use crate::{
    BigDecimal, BigInt, ByteBuffer,
    prelude::SensitiveTrait,
    schema::{Document, SchemaRef},
    serde::{
        fmt::FmtError::Custom,
        se::{ListSerializer, MapSerializer, SerializeWithSchema, Serializer, StructSerializer},
    },
};

const REDACTED_STRING: &str = "**REDACTED**";
macro_rules! redact {
    ($self:ident, $schema:ident, $expr:expr) => {
        if $schema.contains_type::<SensitiveTrait>() {
            $self
                .writer
                .write_all(REDACTED_STRING.as_ref())
                .map_err(FmtError::Io)
        } else {
            $expr.map_err(FmtError::Io)
        }
    };
}

macro_rules! redact_aggregate {
    ($self:ident, $schema:ident) => {
        if $schema.contains_type::<SensitiveTrait>() {
            $self
                .writer
                .write_all(REDACTED_STRING.as_ref())
                .map_err(FmtError::Io)?;
            Ok(InnerFmtSerializer {
                ser: $self,
                state: State::Redacted,
            })
        } else {
            Ok(InnerFmtSerializer {
                ser: $self,
                state: State::First,
            })
        }
    };
}

macro_rules! start_text {
    ($self:ident) => {
        if $self.state == State::First {
            $self.state = State::Rest;
        } else if $self.state == State::Redacted {
            /* Skip redacted lists */
            return Ok(());
        } else {
            $self.ser.writer.write_all(b", ").map_err(FmtError::Io)?;
        }
    };
}

#[derive(Error, Debug)]
pub enum FmtError {
    #[error(transparent)]
    Fmt(#[from] Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Encountered unknown error")]
    Unknown(#[from] Box<dyn std::error::Error>),
    #[error("Serialization error: {0}")]
    Custom(String),
}
use crate::serde::se::Error as SerdeError;
impl SerdeError for FmtError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Custom(msg.to_string())
    }
}

// TODO: Update to just accept structs with schema.
#[allow(dead_code)]
pub fn to_string<T: SerializeWithSchema + ?Sized>(
    schema: &SchemaRef,
    value: &T,
) -> Result<String, FmtError> {
    let mut writer: Vec<u8> = Vec::with_capacity(128);
    let mut ser = FmtSerializer::new(&mut writer);
    value
        .serialize_with_schema(schema, &mut ser)
        .map_err(|e| FmtError::Unknown(e.into()))?;
    String::from_utf8(writer).map_err(|e| FmtError::Unknown(e.into()))
}

/// Serializer used to format and print a shape.
pub struct FmtSerializer<W: io::Write> {
    pub(super) writer: W,
}

impl<W: io::Write> FmtSerializer<W> {
    pub const fn new(writer: W) -> Self {
        FmtSerializer { writer }
    }
}

impl<'a, W: io::Write> Serializer for &'a mut FmtSerializer<W> {
    type Error = FmtError;
    type Ok = ();

    type SerializeList = InnerFmtSerializer<'a, W>;
    type SerializeMap = InnerFmtSerializer<'a, W>;
    type SerializeStruct = InnerFmtSerializer<'a, W>;

    #[inline]
    fn write_struct(
        self,
        schema: &SchemaRef,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.writer.write_all(schema.id().name().as_ref())?;
        self.writer.write_all(b"[").map_err(FmtError::Io)?;
        redact_aggregate!(self, schema)
    }

    #[inline]
    fn write_map(self, schema: &SchemaRef, _: usize) -> Result<Self::SerializeMap, Self::Error> {
        self.writer.write_all(b"{").map_err(FmtError::Io)?;
        redact_aggregate!(self, schema)
    }

    #[inline]
    fn write_list(self, schema: &SchemaRef, _: usize) -> Result<Self::SerializeList, Self::Error> {
        self.writer.write_all(b"[").map_err(FmtError::Io)?;
        redact_aggregate!(self, schema)
    }

    #[inline]
    fn write_boolean(self, schema: &SchemaRef, value: bool) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.writer.write_all(value.to_string().as_ref())
        )
    }

    #[inline]
    fn write_byte(self, schema: &SchemaRef, value: i8) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.writer.write_all(value.to_string().as_str().as_ref())
        )
    }

    #[inline]
    fn write_short(self, schema: &SchemaRef, value: i16) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.writer.write_all(value.to_string().as_str().as_ref())
        )
    }

    #[inline]
    fn write_integer(self, schema: &SchemaRef, value: i32) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.writer.write_all(value.to_string().as_str().as_ref())
        )
    }

    #[inline]
    fn write_long(self, schema: &SchemaRef, value: i64) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.writer.write_all(value.to_string().as_str().as_ref())
        )
    }

    #[inline]
    fn write_float(self, schema: &SchemaRef, value: f32) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.writer.write_all(value.to_string().as_str().as_ref())
        )
    }

    #[inline]
    fn write_double(self, schema: &SchemaRef, value: f64) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.writer.write_all(value.to_string().as_str().as_ref())
        )
    }

    #[inline]
    fn write_big_integer(self, schema: &SchemaRef, value: &BigInt) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.writer.write_all(value.to_string().as_str().as_ref())
        )
    }

    #[inline]
    fn write_big_decimal(self, schema: &SchemaRef, value: &BigDecimal) -> Result<(), Self::Error> {
        redact!(
            self,
            schema,
            self.writer.write_all(value.to_string().as_str().as_ref())
        )
    }

    #[inline]
    fn write_string(self, schema: &SchemaRef, value: &str) -> Result<(), Self::Error> {
        redact!(self, schema, self.writer.write_all(value.as_ref()))
    }

    #[inline]
    fn write_blob(self, _: &SchemaRef, _: &ByteBuffer) -> Result<(), Self::Error> {
        todo!()
    }

    #[inline]
    fn write_timestamp(self, schema: &SchemaRef, value: &Instant) -> Result<(), Self::Error> {
        // TODO: This is incorrect and needs to be fixed. Just to get all branches running
        redact!(
            self,
            schema,
            self.writer
                .write_all(value.elapsed().as_secs().to_string().as_str().as_ref())
        )
    }

    #[inline]
    fn write_document(self, _: &SchemaRef, _: &Document) -> Result<(), Self::Error> {
        todo!()
    }

    #[inline]
    fn write_null(self, schema: &SchemaRef) -> Result<(), Self::Error> {
        redact!(self, schema, self.writer.write_all(b"null"))
    }

    #[inline]
    fn skip(self, _: &SchemaRef) -> Result<(), Self::Error> {
        /* Do not write anything on non-present fields */
        Ok(())
    }

    #[inline]
    fn flush(self) -> Result<(), Self::Error> {
        // Does nothing for string serializer
        Ok(())
    }
}

// TODO: Add formatter for control characters?
// Must be public to satisfy "leaking" of internal types.
#[doc(hidden)]
pub struct InnerFmtSerializer<'a, W: 'a>
where
    W: io::Write,
{
    ser: &'a mut FmtSerializer<W>,
    state: State,
}

#[derive(PartialEq, Eq)]
#[doc(hidden)]
pub enum State {
    First,
    Rest,
    Redacted,
}

impl<W> ListSerializer for InnerFmtSerializer<'_, W>
where
    W: io::Write,
{
    type Error = FmtError;
    type Ok = ();

    #[inline]
    fn serialize_element<T>(
        &mut self,
        element_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
    {
        start_text!(self);
        value.serialize_with_schema(element_schema, &mut *self.ser)
    }

    #[inline]
    fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.ser.writer.write_all(b"]").map_err(FmtError::Io)
    }
}

impl<W> MapSerializer for InnerFmtSerializer<'_, W>
where
    W: io::Write,
{
    type Error = FmtError;
    type Ok = ();

    #[inline]
    fn serialize_entry<K, V>(
        &mut self,
        key_schema: &SchemaRef,
        value_schema: &SchemaRef,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: ?Sized + SerializeWithSchema,
        V: ?Sized + SerializeWithSchema,
    {
        start_text!(self);
        key.serialize_with_schema(key_schema, &mut *self.ser)?;
        self.ser.writer.write_all(b":").map_err(FmtError::Io)?;
        value.serialize_with_schema(value_schema, &mut *self.ser)
    }

    #[inline]
    fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.ser.writer.write_all(b"}").map_err(FmtError::Io)
    }
}

impl<W> StructSerializer for InnerFmtSerializer<'_, W>
where
    W: io::Write,
{
    type Error = FmtError;
    type Ok = ();

    #[inline]
    fn serialize_member<T>(
        &mut self,
        member_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
    {
        start_text!(self);
        let Some(me) = member_schema.as_member() else {
            panic!("Expected member schema!");
        };
        self.ser
            .writer
            .write_all(me.name.as_str().as_ref())
            .map_err(FmtError::Io)?;
        self.ser.writer.write_all(b"=").map_err(FmtError::Io)?;
        value.serialize_with_schema(member_schema, &mut *self.ser)
    }

    #[inline]
    fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.ser.writer.write_all(b"]").map_err(FmtError::Io)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use indexmap::IndexMap;
    use smithy4rs_core_derive::SerializableStruct;

    use super::*;
    use crate::{
        lazy_schema,
        prelude::STRING,
        schema::{Schema, ShapeId},
        traits,
    };

    lazy_schema!(
        MAP_SCHEMA,
        Schema::map_builder(ShapeId::from("com.example#Map"), traits![]),
        ("key", STRING, traits![]),
        ("value", STRING, traits![])
    );
    lazy_schema!(
        LIST_SCHEMA,
        Schema::list_builder(ShapeId::from("com.example#List"), traits![]),
        ("member", STRING, traits![])
    );
    lazy_schema!(
        SCHEMA,
        Schema::structure_builder(ShapeId::from("com.example#Shape"), traits![]),
        (MEMBER_A, "a", STRING, traits![]),
        (MEMBER_B, "b", STRING, traits![SensitiveTrait]),
        (MEMBER_C, "c", STRING, traits![]),
        (MEMBER_MAP, "map", MAP_SCHEMA, traits![]),
        (MEMBER_LIST, "list", LIST_SCHEMA, traits![])
    );

    lazy_schema!(
        REDACTED_AGGREGATES,
        Schema::structure_builder(ShapeId::from("com.example#Shape"), traits![]),
        (
            MEMBER_MAP_REDACT,
            "map",
            MAP_SCHEMA,
            traits![SensitiveTrait]
        ),
        (
            MEMBER_LIST_REDACT,
            "list",
            LIST_SCHEMA,
            traits![SensitiveTrait]
        )
    );

    #[derive(SerializableStruct)]
    #[smithy_schema(SCHEMA)]
    pub(crate) struct SerializeMe {
        #[smithy_schema(MEMBER_A)]
        pub member_a: String,
        #[smithy_schema(MEMBER_B)]
        pub member_b: String,
        #[smithy_schema(MEMBER_C)]
        pub member_optional: Option<String>,
        #[smithy_schema(MEMBER_LIST)]
        pub member_list: Vec<String>,
        #[smithy_schema(MEMBER_MAP)]
        pub member_map: IndexMap<String, String>,
    }

    #[derive(SerializableStruct)]
    #[smithy_schema(REDACTED_AGGREGATES)]
    pub(crate) struct RedactMe {
        #[smithy_schema(MEMBER_LIST_REDACT)]
        pub member_list: Vec<String>,
        #[smithy_schema(MEMBER_MAP_REDACT)]
        pub member_map: IndexMap<String, String>,
    }

    #[test]
    fn fmt_serializer_all() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let list = vec!["a".to_string(), "b".to_string()];
        let struct_to_write = SerializeMe {
            member_a: "a".to_string(),
            member_b: "b".to_string(),
            member_optional: Some("c".to_string()),
            member_map: map,
            member_list: list,
        };
        let output = to_string(&SCHEMA, &struct_to_write).expect("serialization failed");
        assert_eq!(
            output,
            "Shape[a=a, b=**REDACTED**, c=c, list=[a, b], map={a:b}]"
        );
    }

    #[test]
    fn fmt_serializer_omits_none() {
        let struct_to_write = SerializeMe {
            member_a: "a".to_string(),
            member_b: "b".to_string(),
            member_optional: None,
            member_list: Vec::new(),
            member_map: IndexMap::new(),
        };
        let output = to_string(&SCHEMA, &struct_to_write).expect("serialization failed");
        assert_eq!(output, "Shape[a=a, b=**REDACTED**, list=[], map={}]");
    }

    #[test]
    fn redacts_aggregates() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let list = vec!["a".to_string(), "b".to_string()];
        let struct_to_write = RedactMe {
            member_list: list,
            member_map: map,
        };
        let output =
            to_string(&REDACTED_AGGREGATES, &struct_to_write).expect("serialization failed");
        assert_eq!(output, "Shape[list=[**REDACTED**], map={**REDACTED**}]");
    }

    #[test]
    fn document_conversion_retains_redaction() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let list = vec!["a".to_string(), "b".to_string()];
        let struct_to_write = RedactMe {
            member_list: list,
            member_map: map,
        };
        let document: Document = struct_to_write.into();
        let output = to_string(&REDACTED_AGGREGATES, &document).expect("serialization failed");
        assert_eq!(output, "Shape[list=[**REDACTED**], map={**REDACTED**}]");
    }
}
