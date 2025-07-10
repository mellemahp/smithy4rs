
// TODO: Remove
#![allow(unused_variables)]

use std::time::Instant;
use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct};
use static_str_ops::staticize;
use crate::schema::{Document, SchemaRef};
use crate::serde::se::{ListSerializer, MapSerializer, Serialize, StructSerializer};
use crate::serde::serializers::Serializer;

struct SerdeAdapter<S: serde::Serializer> {
    serializer: S,
}
impl <S: serde::Serializer> SerdeAdapter<S> {
    fn new(serializer: S) -> Self {
        SerdeAdapter { serializer }
    }
}

impl <S: serde::Serializer> Serializer for SerdeAdapter<S> {
    type Error = S::Error;
    type Ok = S::Ok;
    type SerializeList = ListSerializeAdapter<S>;
    type SerializeMap = MapSerializerAdapter<S>;
    type SerializeStruct = StructSerializerAdapter<S>;

    #[inline]
    fn write_struct(self, schema: &SchemaRef, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        let struct_name = staticize(schema.id().name());
        let struct_ser = self.serializer.serialize_struct(struct_name, len)?;
        Ok(StructSerializerAdapter::new(struct_ser))
    }

    #[inline]
    fn write_map(self, _schema: &SchemaRef, len: usize) -> Result<Self::SerializeMap, Self::Error> {
        let map_ser = self.serializer.serialize_map(Some(len))?;
        Ok(MapSerializerAdapter::new(map_ser))
    }

    #[inline]
    fn write_list(self, _schema: &SchemaRef, len: usize) -> Result<Self::SerializeList, Self::Error> {
        let list_ser = self.serializer.serialize_seq(Some(len))?;
        Ok(ListSerializeAdapter::new(list_ser))
    }

    #[inline]
    fn write_boolean(self, schema: &SchemaRef, value: bool) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_bool(value)
    }

    #[inline]
    fn write_byte(self, _: &SchemaRef, value: i8) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_i8(value)
    }

    #[inline]
    fn write_short(self, _: &SchemaRef, value: i16) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_i16(value)
    }

    #[inline]
    fn write_integer(self, _: &SchemaRef, value: i32) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_i32(value)
    }

    #[inline]
    fn write_long(self, _: &SchemaRef, value: i64) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_i64(value)
    }

    #[inline]
    fn write_float(self, _: &SchemaRef, value: f32) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_f32(value)
    }

    #[inline]
    fn write_double(self, _: &SchemaRef, value: f64) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_f64(value)
    }

    #[inline]
    fn write_big_integer(self, schema: &SchemaRef, value: &BigInt) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    #[inline]
    fn write_big_decimal(self, schema: &SchemaRef, value: &BigDecimal) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    #[inline]
    fn write_string(self, _: &SchemaRef, value: &String) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_str(value.as_str())
    }

    #[inline]
    fn write_blob(self, schema: &SchemaRef, value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    #[inline]
    fn write_timestamp(self, schema: &SchemaRef, value: &Instant) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    #[inline]
    fn write_document(self, schema: &SchemaRef, value: &Document) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    #[inline]
    fn write_null(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_none()
    }

    fn skip(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_none()
    }
}

pub struct ListSerializeAdapter<S: serde::Serializer> {
    serializer: S::SerializeSeq,
}
impl <S: serde::Serializer> ListSerializeAdapter<S> {
    fn new(serializer: S::SerializeSeq) -> Self {
        Self { serializer }
    }
}
impl <S: serde::Serializer> ListSerializer for ListSerializeAdapter<S> {
    type Error = S::Error;
    type Ok = S::Ok;

    #[inline]
    fn serialize_element<T>(
        &mut self,
        value_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serializer.serialize_element(&ValueWrapper(value_schema, value))
    }

    #[inline]
    fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.serializer.end()
    }
}

pub struct MapSerializerAdapter<S: serde::Serializer> {
    serializer: S::SerializeMap
}
impl <S: serde::Serializer> MapSerializerAdapter<S> {
    fn new(serializer: S::SerializeMap) -> Self {
        Self { serializer }
    }
}
impl <S: serde::Serializer> MapSerializer for MapSerializerAdapter<S> {
    type Error = S::Error;
    type Ok = S::Ok;

    #[inline]
    fn serialize_entry<K, V>(
        &mut self,
        key_schema: &SchemaRef,
        value_schema: &SchemaRef,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: Serialize + ?Sized,
        V: Serialize + ?Sized,
    {
        self.serializer.serialize_entry(
            &ValueWrapper(key_schema, key),
            &ValueWrapper(value_schema, value)
        )
    }

    #[inline]
    fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.serializer.end()
    }
}

pub struct StructSerializerAdapter<S: serde::Serializer> {
    serializer: S::SerializeStruct,
}
impl<S: serde::Serializer> StructSerializerAdapter<S> {
    fn new(serializer: S::SerializeStruct) -> Self {
        Self { serializer }
    }
}
impl<S: serde::Serializer> StructSerializer for StructSerializerAdapter<S> {
    type Error = S::Error;
    type Ok = S::Ok;

    #[inline]
    fn serialize_member<T>(
        &mut self,
        member_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        // TODO: How to handle error?
        let Some(me) = member_schema.as_member() else {
            panic!("Expected member schema!");
        };
        self.serializer.serialize_field(staticize(&me.name), &ValueWrapper(member_schema, value))
    }

    #[inline]
    fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.serializer.end()
    }
}

struct ValueWrapper<'a, T: Serialize + ?Sized>(&'a SchemaRef, &'a T);
impl <T: Serialize + ?Sized> serde::Serialize for ValueWrapper<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let adapter = SerdeAdapter::new(serializer);
        self.1.serialize(self.0, adapter)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;
    use indexmap::IndexMap;
    use crate::{lazy_member_schema, lazy_schema, traits};
    use crate::prelude::*;
    use crate::serde::se::{Serialize, Serializer};
    use crate::schema::{Schema, ShapeId};
    use super::*;

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
        Schema::structure_builder(ShapeId::from("com.example#Test"))
            .put_member("a", &STRING, traits![])
            .put_member("b", &STRING, traits![])
            .put_member("map", &MAP_SCHEMA, traits![])
            .put_member("list", &LIST_SCHEMA, traits![])
            .build()
    );
    lazy_member_schema!(MEMBER_A, SCHEMA, "a");
    lazy_member_schema!(MEMBER_B, SCHEMA, "b");
    lazy_member_schema!(MEMBER_LIST, SCHEMA, "list");
    lazy_member_schema!(MEMBER_MAP, SCHEMA, "map");

    struct Test {
        a: String,
        b: String,
        member_list: Vec<String>,
        member_map: IndexMap<String, String>,
    }
    impl Test {
        fn write_out<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.serialize(&SCHEMA, serializer)
        }
    }
    impl Serialize for Test {
        fn serialize<S: Serializer>(&self, schema: &SchemaRef, serializer: S) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 2)?;
            ser.serialize_member(&MEMBER_A, &self.a)?;
            ser.serialize_member(&MEMBER_B, &self.b)?;
            ser.serialize_member(&MEMBER_LIST, &self.member_list)?;
            ser.serialize_member(&MEMBER_MAP, &self.member_map)?;
            ser.end(schema)
        }
    }
    impl serde::Serialize for Test {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
        {
            let adapter = SerdeAdapter::new(serializer);
            self.write_out(adapter)
        }
    }

    #[test]
    fn can_use_serde_json() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        map.insert(String::from("c"), String::from("d"));
        let test = Test {
            a: "a".to_string(),
            b: "b".to_string(),
            member_list: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            member_map: map
        };
        let expected = r#"{
  "a": "a",
  "b": "b",
  "list": [
    "a",
    "b",
    "c"
  ],
  "map": {
    "a": "b",
    "c": "d"
  }
}"#;
        assert_eq!(serde_json::to_string_pretty(&test).unwrap(), expected);
        println!("{}", serde_json::to_string_pretty(&test).unwrap());
    }
}

