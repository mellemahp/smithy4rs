use std::fmt::{Error, Write};
use std::time::Instant;
use bigdecimal::BigDecimal;
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

    fn write_struct(&mut self, schema: &Schema, _: usize) -> Result<Self::SerializeStruct<'_>, Self::Error> {
        self.sink.write_str(schema.id().name.as_str())?;
        self.sink.write_char('[')?;
        let redacted = schema.contains_trait_type::<SensitiveTrait>();
        if redacted {
            self.sink.write_str(REDACTED_STRING)?;
        }
        Ok(FmtStructSerializer::new(self, redacted))
    }

    fn write_map(&mut self, schema: &Schema, _: usize) -> Result<Self::SerializeMap<'_>, Self::Error> {
        self.sink.write_str("{")?;
        let redacted = schema.contains_trait_type::<SensitiveTrait>();
        if redacted {
            self.sink.write_str(REDACTED_STRING)?;
        }
        Ok(FmtMapSerializer::new(self, redacted))
    }

    fn write_list(&mut self, schema: &Schema, _: usize) -> Result<Self::SerializeList<'_>, Self::Error> {
        self.sink.write_str("[")?;
        let redacted = schema.contains_trait_type::<SensitiveTrait>();
        if redacted {
            self.sink.write_str(REDACTED_STRING)?;
        }
        Ok(FmtListSerialize::new(self, redacted))
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

    fn write_blob(&mut self, _: &Schema, _: &ByteBuffer) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_timestamp(&mut self, schema: &Schema, value: &Instant) -> Result<(), Self::Error> {
        // TODO: This is incorrect and needs to be fixed. Just to get all branches running
        redact!(self,schema,self.sink.write_str(value.elapsed().as_secs().to_string().as_str()))
    }

    fn write_document(&mut self, _: &Schema, _: &Document) -> Result<(), Self::Error> {
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
        let Schema::Member(me) = member_schema else {
            panic!("Expected member schema!");
        };
        self.parent.sink.write_str(me.name.as_str())?;
        self.parent.sink.write_char('=')?;
        value.serialize(member_schema, self.parent)
    }

    fn end(self, _: &Schema) -> Result<Self::Ok, Self::Error> {
        self.parent.sink.write_char(']')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{prelude, Ref};
    use crate::schema::ShapeId;
    use crate::{lazy_member_schema, lazy_schema, traits};
    use std::sync::LazyLock;
    use indexmap::IndexMap;
    use crate::schema::traits::SensitiveTrait;
    use crate::serde::shapes::SerializeShape;

    lazy_schema!(MAP_SCHEMA, Schema::map_builder(ShapeId::from("com.example#Map"))
        .put_member("key", &prelude::STRING, traits![])
        .put_member("value", &prelude::STRING, traits![])
        .build()
    );
    lazy_schema!(LIST_SCHEMA, Schema::list_builder(ShapeId::from("com.example#List"))
        .put_member("member", &prelude::STRING, traits![])
        .build()
    );
    lazy_schema!(SCHEMA, Schema::structure_builder(ShapeId::from("com.example#Shape"))
            .put_member("a", &prelude::STRING, traits![])
            .put_member("b", &prelude::STRING, traits![SensitiveTrait::new()])
            .put_member("c", &prelude::STRING, traits![])
            .put_member("map", &MAP_SCHEMA, traits![])
            .put_member("list", &LIST_SCHEMA, traits![])
            .build()
    );
    lazy_member_schema!(MEMBER_A, SCHEMA, "a");
    lazy_member_schema!(MEMBER_B, SCHEMA, "b");
    lazy_member_schema!(MEMBER_C, SCHEMA, "c");
    lazy_member_schema!(MEMBER_LIST, SCHEMA, "list");
    lazy_member_schema!(MEMBER_MAP, SCHEMA, "map");

    lazy_schema!(REDACTED_AGGREGATES, Schema::structure_builder(ShapeId::from("com.example#Shape"))
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
    impl SerializeShape for RedactMe {
        fn schema(&self) -> &Schema {
            &REDACTED_AGGREGATES
        }
    }

    impl Serialize for RedactMe {
        fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error>
        {
            let mut ser = serializer.write_struct(schema, 2)?;
            ser.serialize_member(&MEMBER_LIST_REDACT, &self.member_list)?;
            ser.serialize_member(&MEMBER_MAP_REDACT, &self.member_map)?;
            ser.end(schema)
        }
    }

    #[test]
    fn fmt_serializer_all() {
        let mut fmter = FmtSerializer::default();
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
        struct_to_write.serialize_shape(&mut fmter).expect("serialization failed");
        assert_eq!(fmter.flush(), "Shape[a=a, b=**REDACTED**, c=c, list=[a, b], map={a:b}]");
    }

    #[test]
    fn fmt_serializer_omits_none() {
        let mut fmter = FmtSerializer::default();
        let struct_to_write = SerializeMe {
            member_a: "a".to_string(),
            member_b: "b".to_string(),
            member_optional: None,
            member_list: Vec::new(),
            member_map: IndexMap::new(),
        };
        struct_to_write.serialize_shape(&mut fmter).expect("serialization failed");
        assert_eq!(fmter.flush(), "Shape[a=a, b=**REDACTED**, list=[], map={}]");
    }

    #[test]
    fn redacts_aggregates() {
        let mut fmter = FmtSerializer::default();
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let list = vec!["a".to_string(), "b".to_string()];
        let struct_to_write = RedactMe {
            member_list: list,
            member_map: map,
        };
        struct_to_write.serialize_shape(&mut fmter).expect("serialization failed");
        assert_eq!(fmter.flush(), "Shape[list=[**REDACTED**], map={**REDACTED**}]");
    }


}