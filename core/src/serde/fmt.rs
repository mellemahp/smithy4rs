use crate::prelude::SensitiveTrait;
use crate::schema::{Document, SchemaRef};
use crate::serde::se::{
    ListSerializer, MapSerializer, SerializeWithSchema, Serializer, StructSerializer,
};
use crate::{BigDecimal, BigInt, ByteBuffer};
use std::cmp::PartialEq;
use std::fmt::Error;
use std::io;
use std::time::Instant;
use thiserror::Error;

const REDACTED_STRING: &str = "**REDACTED**";
macro_rules! redact {
    ($self:ident, $schema:ident, $expr:expr) => {
        if $schema.contains_trait_type::<SensitiveTrait>() {
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
        if $schema.contains_trait_type::<SensitiveTrait>() {
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
// Must be public to statisfy "leaking" of internal types.
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
    use super::*;
    use crate::Ref;
    use crate::prelude::STRING;
    use crate::schema::{Schema, ShapeId};
    use crate::serde::se::Serialize;
    use crate::serde::shapes::SchemaShape;
    use crate::{lazy_member_schema, lazy_schema, traits};
    use indexmap::IndexMap;
    use std::sync::LazyLock;

    lazy_schema!(
        MAP_SCHEMA,
        Schema::map_builder(ShapeId::from("com.example#Map"))
            .put_member("key", &STRING, traits![])
            .put_member("value", &STRING, traits![])
            .build()
    );
    lazy_schema!(
        LIST_SCHEMA,
        Schema::list_builder(ShapeId::from("com.example#List"))
            .put_member("member", &STRING, traits![])
            .build()
    );
    lazy_schema!(
        SCHEMA,
        Schema::structure_builder(ShapeId::from("com.example#Shape"))
            .put_member("a", &STRING, traits![])
            .put_member("b", &STRING, traits![SensitiveTrait::new()])
            .put_member("c", &STRING, traits![])
            .put_member("map", &MAP_SCHEMA, traits![])
            .put_member("list", &LIST_SCHEMA, traits![])
            .build()
    );
    lazy_member_schema!(MEMBER_A, SCHEMA, "a");
    lazy_member_schema!(MEMBER_B, SCHEMA, "b");
    lazy_member_schema!(MEMBER_C, SCHEMA, "c");
    lazy_member_schema!(MEMBER_LIST, SCHEMA, "list");
    lazy_member_schema!(MEMBER_MAP, SCHEMA, "map");

    lazy_schema!(
        REDACTED_AGGREGATES,
        Schema::structure_builder(ShapeId::from("com.example#Shape"))
            .put_member("map", &MAP_SCHEMA, traits![SensitiveTrait::new()])
            .put_member("list", &LIST_SCHEMA, traits![SensitiveTrait::new()])
            .build()
    );
    lazy_member_schema!(MEMBER_LIST_REDACT, REDACTED_AGGREGATES, "list");
    lazy_member_schema!(MEMBER_MAP_REDACT, REDACTED_AGGREGATES, "map");

    //#[derive(SerializableStruct)]
    //#[schema(SCHEMA)]
    pub(crate) struct SerializeMe {
        // #[schema(MEMBER_A)]
        pub member_a: String,
        // #[schema(MEMBER_B)]
        pub member_b: String,
        // #[schema(MEMBER_C)]
        pub member_optional: Option<String>,
        pub member_list: Vec<String>,
        pub member_map: IndexMap<String, String>,
    }

    impl SchemaShape for SerializeMe {
        fn schema(&self) -> &SchemaRef {
            &SCHEMA
        }
    }

    impl SerializeWithSchema for SerializeMe {
        fn serialize_with_schema<S: Serializer>(
            &self,
            schema: &SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 2)?;
            ser.serialize_member(&MEMBER_A, &self.member_a)?;
            ser.serialize_member(&MEMBER_B, &self.member_b)?;
            ser.serialize_optional_member(&MEMBER_C, &self.member_optional)?;
            ser.serialize_member(&MEMBER_LIST, &self.member_list)?;
            ser.serialize_member(&MEMBER_MAP, &self.member_map)?;
            ser.end(schema)
        }
    }

    pub(crate) struct RedactMe {
        pub member_list: Vec<String>,
        pub member_map: IndexMap<String, String>,
    }

    impl SchemaShape for RedactMe {
        fn schema(&self) -> &SchemaRef {
            &REDACTED_AGGREGATES
        }
    }

    impl SerializeWithSchema for RedactMe {
        fn serialize_with_schema<S: Serializer>(
            &self,
            schema: &SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 2)?;
            ser.serialize_member(&MEMBER_LIST_REDACT, &self.member_list)?;
            ser.serialize_member(&MEMBER_MAP_REDACT, &self.member_map)?;
            ser.end(schema)
        }
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
}
