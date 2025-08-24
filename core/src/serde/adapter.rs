use core::fmt;
use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
    time::Instant,
};

use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;
use serde::{
    de::{DeserializeSeed, Error as DeError, SeqAccess, Visitor},
    ser::{Error as SerdeError, SerializeMap, SerializeSeq, SerializeStruct},
};
use static_str_ops::staticize;

use crate::{
    schema::{Document, SchemaRef},
    serde::{
        de::DeserializeWithSchema,
        deserializers::{Deserializer, Error as DeserializerError},
        se::{ListSerializer, MapSerializer, SerializeWithSchema, StructSerializer},
        serializers::{Error as SerializerError, Serializer},
    },
};
// TODO: This should all be behind a feature flag so serde is not
//       required for all consumers.
struct SerdeSerializerAdapter<S: serde::Serializer> {
    serializer: S,
}
impl<S: serde::Serializer> SerdeSerializerAdapter<S> {
    fn new(serializer: S) -> Self {
        SerdeSerializerAdapter { serializer }
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
impl<E: SerdeError> SerializerError for SerdeErrorWrapper<E> {
    fn custom<T: Display>(msg: T) -> Self {
        SerdeErrorWrapper(E::custom(msg))
    }
}

impl<E: SerdeError> From<E> for SerdeErrorWrapper<E> {
    fn from(e: E) -> Self {
        SerdeErrorWrapper(e)
    }
}

impl<S: serde::Serializer> Serializer for SerdeSerializerAdapter<S> {
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
            .serialize_with_schema(self.0, SerdeSerializerAdapter::new(serializer))
            .map_err(|wrapper| wrapper.0)
    }
}

// Deserialize
pub struct SerdeDeserializerAdapter<'de, D: serde::Deserializer<'de>> {
    deserializer: D,
    _phantom: PhantomData<&'de ()>,
}

impl<'de, D: serde::Deserializer<'de>> SerdeDeserializerAdapter<'de, D> {
    pub fn new(deserializer: D) -> Self {
        Self {
            deserializer,
            _phantom: PhantomData,
        }
    }
}

struct DeserializeWrapper<T: DeserializeWithSchema> {
    value: T,
}

impl<T: DeserializeWithSchema> DeserializeWrapper<T> {
    fn into_inner(self) -> T {
        self.value
    }
}

struct DeserializeWrapperSeed<T> {
    schema: SchemaRef,
    _phantom: PhantomData<T>,
}

impl<'de, T> DeserializeSeed<'de> for DeserializeWrapperSeed<T>
where
    T: DeserializeWithSchema,
{
    type Value = DeserializeWrapper<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let adapter = SerdeDeserializerAdapter::new(deserializer);
        let value = T::deserialize_with_schema(&self.schema, adapter).map_err(|wrapper_err| {
            match wrapper_err {
                DeErrorWrapper(original_err) => original_err,
            }
        })?;
        Ok(DeserializeWrapper { value })
    }
}

#[derive(Debug)]
pub struct DeErrorWrapper<E: DeError>(E);
impl<E: DeError> Display for DeErrorWrapper<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
impl<E: DeError> StdError for DeErrorWrapper<E> {}
impl<E: DeError> DeserializerError for DeErrorWrapper<E> {
    fn custom<T: Display>(msg: T) -> Self {
        DeErrorWrapper(E::custom(msg))
    }
}

impl<E: DeError> From<E> for DeErrorWrapper<E> {
    fn from(e: E) -> Self {
        DeErrorWrapper(e)
    }
}

impl<'de, D: serde::Deserializer<'de>> Deserializer<'de> for SerdeDeserializerAdapter<'de, D> {
    type Error = DeErrorWrapper<D::Error>;

    fn read_struct<T: DeserializeWithSchema>(self, schema: &SchemaRef) -> Result<T, Self::Error> {
        todo!()
    }

    fn read_map<K: DeserializeWithSchema, V: DeserializeWithSchema>(
        self,
        schema: &SchemaRef,
    ) -> Result<indexmap::IndexMap<K, V>, Self::Error> {
        todo!()
    }

    fn read_list<T: DeserializeWithSchema>(
        self,
        schema: &SchemaRef,
    ) -> Result<Vec<T>, Self::Error> {
        struct SeqVisitor<T> {
            element_schema: SchemaRef,
            _phantom: PhantomData<T>,
        }

        impl<'de, T: DeserializeWithSchema> Visitor<'de> for SeqVisitor<T> {
            type Value = Vec<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of values")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::new();

                while let Some(wrapper) = seq.next_element_seed(DeserializeWrapperSeed::<T> {
                    schema: self.element_schema.clone(),
                    _phantom: PhantomData,
                })? {
                    values.push(wrapper.into_inner()); // Unwrap immediately
                }
                Ok(values)
            }
        }

        let element_schema = schema.expect_member("member").clone();
        Ok(self.deserializer.deserialize_seq(SeqVisitor {
            element_schema,
            _phantom: PhantomData::<T>,
        })?)
    }

    fn read_boolean(self, schema: &SchemaRef) -> Result<bool, Self::Error> {
        struct BoolVisitor;

        impl<'de> Visitor<'de> for BoolVisitor {
            type Value = bool;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a boolean value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(value)
            }
        }

        Ok(self.deserializer.deserialize_bool(BoolVisitor)?)
    }

    fn read_byte(self, schema: &SchemaRef) -> Result<i8, Self::Error> {
        struct ByteVisitor;

        impl<'de> Visitor<'de> for ByteVisitor {
            type Value = i8;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a byte value")
            }

            fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E> {
                Ok(value)
            }
        }

        Ok(self.deserializer.deserialize_i8(ByteVisitor)?)
    }

    fn read_short(self, schema: &SchemaRef) -> Result<i16, Self::Error> {
        struct ShortVisitor;

        impl<'de> Visitor<'de> for ShortVisitor {
            type Value = i16;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a short value")
            }

            fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E> {
                Ok(value)
            }
        }

        Ok(self.deserializer.deserialize_i16(ShortVisitor)?)
    }

    fn read_integer(self, schema: &SchemaRef) -> Result<i32, Self::Error> {
        struct IntegerVisitor;

        impl<'de> Visitor<'de> for IntegerVisitor {
            type Value = i32;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("an integer value")
            }

            fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E> {
                Ok(value)
            }
        }

        Ok(self.deserializer.deserialize_i32(IntegerVisitor)?)
    }

    fn read_long(self, schema: &SchemaRef) -> Result<i64, Self::Error> {
        struct LongVisitor;

        impl<'de> Visitor<'de> for LongVisitor {
            type Value = i64;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a long value")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(value)
            }
        }

        Ok(self.deserializer.deserialize_i64(LongVisitor)?)
    }

    fn read_float(self, schema: &SchemaRef) -> Result<f32, Self::Error> {
        struct FloatVisitor;

        impl<'de> Visitor<'de> for FloatVisitor {
            type Value = f32;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a float value")
            }

            fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E> {
                Ok(value)
            }
        }

        Ok(self.deserializer.deserialize_f32(FloatVisitor)?)
    }

    fn read_double(self, schema: &SchemaRef) -> Result<f64, Self::Error> {
        struct DoubleVisitor;

        impl<'de> Visitor<'de> for DoubleVisitor {
            type Value = f64;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a double value")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
                Ok(value)
            }
        }

        Ok(self.deserializer.deserialize_f64(DoubleVisitor)?)
    }

    fn read_big_integer(self, schema: &SchemaRef) -> Result<BigInt, Self::Error> {
        todo!()
    }

    fn read_big_decimal(self, schema: &SchemaRef) -> Result<BigDecimal, Self::Error> {
        todo!()
    }

    fn read_string(self, schema: &SchemaRef) -> Result<String, Self::Error> {
        struct StringVisitor;

        impl<'de> Visitor<'de> for StringVisitor {
            type Value = String;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a string value")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
                Ok(value.to_string())
            }
        }

        Ok(self.deserializer.deserialize_string(StringVisitor)?)
    }

    fn read_blob(self, schema: &SchemaRef) -> Result<ByteBuffer, Self::Error> {
        todo!()
    }

    fn read_timestamp(self, schema: &SchemaRef) -> Result<Instant, Self::Error> {
        todo!()
    }

    fn read_document(self, schema: &SchemaRef) -> Result<Document, Self::Error> {
        todo!()
    }

    fn read_null(self, schema: &SchemaRef) -> Result<(), Self::Error> {
        struct UnitVisitor;

        impl<'de> Visitor<'de> for UnitVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a unit value")
            }
        }

        Ok(self.deserializer.deserialize_unit(UnitVisitor)?)
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
        schema::{Schema, SchemaRef, SchemaShape, ShapeId},
        serde::builders::ShapeBuilder,
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

    impl DeserializeWithSchema for Test {
        fn deserialize_with_schema<'de, D: Deserializer<'de>>(
            schema: &SchemaRef,
            deserializer: D,
        ) -> Result<Self, D::Error> {
            todo!()
        }
    }

    impl serde::Serialize for Test {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let adapter = SerdeSerializerAdapter::new(serializer);
            self.serialize_with_schema(self.schema(), adapter)
                .map_err(|wrapper| wrapper.0)
        }
    }

    // impl<'de> serde::Deserialize<'de> for Test {
    //     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    //     where
    //         D: serde::Deserializer<'de>,
    //     {
    //         let adapter = SerdeDeserializerAdapter::new(deserializer);
    //         Self::deserialize_with_schema(Self::schema(), adapter)
    //             .map_err(|wrapper| wrapper.0)
    //     }
    // }

    // struct TestBuilder {
    //     a: String,
    //     b: String,
    //     member_list: Vec<String>,
    //     member_map: IndexMap<String, String>,
    // }

    // impl<'de> ShapeBuilder<'de, Test> for TestBuilder {
    //     fn new() -> Self {
    //         todo!()
    //     }

    //     fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self, D::Error> {

    //     }

    //     fn build_with_validator<V: crate::serde::validation::Validator>(self, validator: V) -> Result<S, crate::serde::validation::ValidationErrors> {
    //         todo!()
    //     }
    // }

    // impl DeserializeWithSchema for TestBuilder {
    //     fn deserialize_with_schema<'de, D: Deserializer<'de>>(
    //         schema: &SchemaRef,
    //         deserializer: D,
    //     ) -> Result<Self, D::Error> {

    //     }
    // }

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
        // let x: Test = serde_json::from_str(&expected).unwrap();
        // assert_eq!(x.a, test.a);
        // assert_eq!(x.b, test.b);
        // assert_eq!(x.member_list, test.member_list);
        // assert_eq!(x.member_map, test.member_map);
    }

    // TODO: Might be nice to split out unit tests from bigger e2e tests

    #[test]
    fn test_read_list_string() {
        // Test deserialization of a simple string list
        let json = r#"["hello", "world", "test"]"#;
        let mut deserializer = serde_json::Deserializer::from_str(json);
        let adapter = SerdeDeserializerAdapter::new(&mut deserializer);

        let result: Vec<String> = adapter.read_list(&LIST_SCHEMA).unwrap();

        assert_eq!(result, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_read_list_empty() {
        // Test deserialization of an empty list
        let json = r#"[]"#;
        let mut deserializer = serde_json::Deserializer::from_str(json);
        let adapter = SerdeDeserializerAdapter::new(&mut deserializer);

        let result: Vec<String> = adapter.read_list(&LIST_SCHEMA).unwrap();

        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_read_list_single_element() {
        // Test deserialization of a single-element list
        let json = r#"["only"]"#;
        let mut deserializer = serde_json::Deserializer::from_str(json);
        let adapter = SerdeDeserializerAdapter::new(&mut deserializer);

        let result: Vec<String> = adapter.read_list(&LIST_SCHEMA).unwrap();

        assert_eq!(result, vec!["only"]);
    }

    #[test]
    fn test_serde_roundtrip_list() {
        // Test that we can serialize with serde and then deserialize with our adapter
        let original_list = vec!["apple", "banana", "cherry"];

        // Serialize using standard serde
        let json = serde_json::to_string(&original_list).unwrap();
        assert_eq!(json, r#"["apple","banana","cherry"]"#);

        // Deserialize using our adapter
        let mut deserializer = serde_json::Deserializer::from_str(&json);
        let adapter = SerdeDeserializerAdapter::new(&mut deserializer);
        let result: Vec<String> = adapter.read_list(&LIST_SCHEMA).unwrap();

        assert_eq!(result, original_list);
    }
}
