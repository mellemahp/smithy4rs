use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter},
    time::Instant,
};

use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;
use serde::ser::{Error as SerdeError, SerializeMap, SerializeSeq, SerializeStruct};
use static_str_ops::staticize;

use crate::{
    schema::{Document, SchemaRef},
    serde::{
        se::{ListSerializer, MapSerializer, SerializeWithSchema, StructSerializer},
        serializers::{Error, Serializer},
    },
};
// TODO: This should all be behind a feature flag so serde is not
//       required for all consumers.
struct SerdeAdapter<S: serde::Serializer> {
    serializer: S,
}
impl<S: serde::Serializer> SerdeAdapter<S> {
    fn new(serializer: S) -> Self {
        SerdeAdapter { serializer }
    }
}

#[derive(Debug)]
pub struct SerdeErrorWrapper<E: SerdeError>(E);
impl<E: SerdeError> Display for SerdeErrorWrapper<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
impl<E: SerdeError> StdError for SerdeErrorWrapper<E> {}
impl<E: SerdeError> Error for SerdeErrorWrapper<E> {
    fn custom<T: Display>(msg: T) -> Self {
        SerdeErrorWrapper(E::custom(msg))
    }
}

impl<E: SerdeError> From<E> for SerdeErrorWrapper<E> {
    fn from(e: E) -> Self {
        SerdeErrorWrapper(e)
    }
}

impl<S: serde::Serializer> Serializer for SerdeAdapter<S> {
    type Error = SerdeErrorWrapper<S::Error>;
    type Ok = S::Ok;
    type SerializeList = ListSerializeAdapter<S>;
    type SerializeMap = MapSerializerAdapter<S>;
    type SerializeStruct = StructSerializerAdapter<S>;

    #[inline]
    fn write_struct(
        self,
        schema: &SchemaRef,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
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
    fn write_list(
        self,
        _schema: &SchemaRef,
        len: usize,
    ) -> Result<Self::SerializeList, Self::Error> {
        let list_ser = self.serializer.serialize_seq(Some(len))?;
        Ok(ListSerializeAdapter::new(list_ser))
    }

    #[inline]
    fn write_boolean(self, _: &SchemaRef, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.serialize_bool(value)?)
    }

    #[inline]
    fn write_byte(self, _: &SchemaRef, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.serialize_i8(value)?)
    }

    #[inline]
    fn write_short(self, _: &SchemaRef, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.serialize_i16(value)?)
    }

    #[inline]
    fn write_integer(self, _: &SchemaRef, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.serialize_i32(value)?)
    }

    #[inline]
    fn write_long(self, _: &SchemaRef, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.serialize_i64(value)?)
    }

    #[inline]
    fn write_float(self, _: &SchemaRef, value: f32) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.serialize_f32(value)?)
    }

    #[inline]
    fn write_double(self, _: &SchemaRef, value: f64) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.serialize_f64(value)?)
    }

    #[inline]
    fn write_big_integer(
        self,
        _schema: &SchemaRef,
        _value: &BigInt,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    #[inline]
    fn write_big_decimal(
        self,
        _schema: &SchemaRef,
        _value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    #[inline]
    fn write_string(self, _: &SchemaRef, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.serialize_str(value)?)
    }

    #[inline]
    fn write_blob(self, _: &SchemaRef, _value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    #[inline]
    fn write_timestamp(self, _: &SchemaRef, _value: &Instant) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    #[inline]
    fn write_document(self, _: &SchemaRef, _value: &Document) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    #[inline]
    fn write_null(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.serialize_none()?)
    }

    fn skip(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.serialize_none()?)
    }
}

pub struct ListSerializeAdapter<S: serde::Serializer> {
    serializer: S::SerializeSeq,
}
impl<S: serde::Serializer> ListSerializeAdapter<S> {
    fn new(serializer: S::SerializeSeq) -> Self {
        Self { serializer }
    }
}
impl<S: serde::Serializer> ListSerializer for ListSerializeAdapter<S> {
    type Error = SerdeErrorWrapper<S::Error>;
    type Ok = S::Ok;

    #[inline]
    fn serialize_element<T>(
        &mut self,
        value_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
    {
        Ok(self
            .serializer
            .serialize_element(&ValueWrapper(value_schema, value))?)
    }

    #[inline]
    fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.end()?)
    }
}

pub struct MapSerializerAdapter<S: serde::Serializer> {
    serializer: S::SerializeMap,
}
impl<S: serde::Serializer> MapSerializerAdapter<S> {
    fn new(serializer: S::SerializeMap) -> Self {
        Self { serializer }
    }
}
impl<S: serde::Serializer> MapSerializer for MapSerializerAdapter<S> {
    type Error = SerdeErrorWrapper<S::Error>;
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
        K: SerializeWithSchema + ?Sized,
        V: SerializeWithSchema + ?Sized,
    {
        Ok(self.serializer.serialize_entry(
            &ValueWrapper(key_schema, key),
            &ValueWrapper(value_schema, value),
        )?)
    }

    #[inline]
    fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.end()?)
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
    type Error = SerdeErrorWrapper<S::Error>;
    type Ok = S::Ok;

    #[inline]
    fn serialize_member<T>(
        &mut self,
        member_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema + ?Sized,
    {
        // TODO: How to handle error?
        let Some(me) = member_schema.as_member() else {
            panic!("Expected member schema!");
        };
        Ok(self
            .serializer
            .serialize_field(staticize(&me.name), &ValueWrapper(member_schema, value))?)
    }

    #[inline]
    fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.end()?)
    }
}

struct ValueWrapper<'a, T: SerializeWithSchema + ?Sized>(&'a SchemaRef, &'a T);
impl<T: SerializeWithSchema + ?Sized> serde::Serialize for ValueWrapper<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.1
            .serialize_with_schema(self.0, SerdeAdapter::new(serializer))
            .map_err(|wrapper| wrapper.0)
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
        prelude::*,
        schema::{Schema, SchemaRef, ShapeId},
        traits,
    };
    use crate::schema::SchemaShape;

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
        Schema::structure_builder(ShapeId::from("com.example#Test"), traits![]),
        (MEMBER_A, "a", STRING, traits![]),
        (MEMBER_B, "b", STRING, traits![]),
        (MEMBER_MAP, "map", MAP_SCHEMA, traits![]),
        (MEMBER_LIST, "list", LIST_SCHEMA, traits![])
    );

    #[derive(SerializableStruct)]
    #[smithy_schema(SCHEMA)]
    struct Test {
        #[smithy_schema(MEMBER_A)]
        a: String,
        #[smithy_schema(MEMBER_B)]
        b: String,
        #[smithy_schema(MEMBER_LIST)]
        member_list: Vec<String>,
        #[smithy_schema(MEMBER_MAP)]
        member_map: IndexMap<String, String>,
    }

    impl serde::Serialize for Test {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let adapter = SerdeAdapter::new(serializer);
            self.serialize_with_schema(self.schema(), adapter)
                .map_err(|wrapper| wrapper.0)
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
            member_map: map,
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
