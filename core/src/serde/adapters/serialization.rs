use std::{
    any::type_name,
    error::Error as StdError,
    fmt::{Debug, Display, Formatter},
};

use serde::{
    Serialize,
    ser::{Error as SerdeError, SerializeMap, SerializeSeq, SerializeStruct},
};
use static_str_ops::staticize;

use crate::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    prelude::{JsonNameTrait, XmlAttributeTrait, XmlNameTrait},
    schema::{Document, SchemaRef},
    serde::{
        se::{ListSerializer, MapSerializer, SerializeWithSchema, StructSerializer},
        serializers::{Error, Serializer},
    },
};

//========================================================================
// Errors
//========================================================================

/// Wrapper type that bridges `serde` and `smithy` Serialization error types.
#[derive(Debug)]
#[repr(transparent)]
pub struct SerErrorWrapper<E: SerdeError>(E);
impl<E: SerdeError> SerErrorWrapper<E> {
    #[inline]
    pub fn inner(self) -> E {
        self.0
    }
}
impl<E: SerdeError> Display for SerErrorWrapper<E> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
impl<E: SerdeError> StdError for SerErrorWrapper<E> {}
impl<E: SerdeError> Error for SerErrorWrapper<E> {
    #[inline]
    fn custom<T: Display>(msg: T) -> Self {
        SerErrorWrapper(E::custom(msg))
    }
}
impl<E: SerdeError> From<E> for SerErrorWrapper<E> {
    #[inline]
    fn from(e: E) -> Self {
        SerErrorWrapper(e)
    }
}

//========================================================================
// Serialization Adapter
//========================================================================

/// Adapter that bridges between `serde` serialization and schema-guided
/// serialization.
///
/// ## Supported Protocol traits
/// By default we support the following built-in protocol traitsL
/// 1. `jsonName` - Renames members in `json` serde serializers
/// 2. `xmlName` - Renames shapes and members in `xml` serde serializers
/// 3. `xmlAttribute` - Treats a member as an attribute for `xml` serde serializers (appends: `@:` to member name)
///
/// **NOTE**: The `xmlNamespace` trait is not supported by this adapter. This
/// is because `serde`'s XML implementation requires namespaces to be set in
/// the serializer config directly.
///
/// ## Generated Shapes
/// This structure is used inside generated `serde::serialize` implementations
/// when the `serde-adapter` feature is enabled.
pub struct SerAdapter<S: serde::Serializer> {
    serializer: S,
    mapper: NameMapper,
}
impl<S: serde::Serializer> SerAdapter<S> {
    /// Create a new schema-guided serialization adapter for a [`serde::Serializer`]
    pub fn new(serializer: S) -> Self {
        SerAdapter {
            serializer,
            mapper: NameMapper::new::<S>(),
        }
    }
}

/// Applies name mapping to support default protocol traits.
enum NameMapper {
    Json,
    Xml,
    Default,
}
impl NameMapper {
    fn new<T: serde::Serializer>() -> Self {
        if type_name::<T>().contains("json") {
            NameMapper::Json
        } else if type_name::<T>().contains("xml") {
            NameMapper::Xml
        } else {
            NameMapper::Default
        }
    }

    fn get_struct_name(&self, schema: &SchemaRef) -> &'static str {
        if matches!(self, NameMapper::Xml)
            && let Some(xml_name) = schema.get_trait_as::<XmlNameTrait>()
        {
            return staticize(xml_name.name());
        }
        staticize(schema.id().name())
    }

    fn get_member_name<S: StructSerializer>(
        &self,
        schema: &SchemaRef,
    ) -> Result<&'static str, S::Error> {
        let Some(me) = schema.as_member() else {
            return Err(S::Error::custom(
                "Expected member schema when serializing struct field",
            ));
        };
        match self {
            NameMapper::Json => {
                // Rename based on JSON Traits, if present
                Ok(schema
                    .get_trait_as::<JsonNameTrait>()
                    .map_or_else(|| staticize(me.name()), |val| staticize(val.name())))
            }
            NameMapper::Xml => {
                // Rename based on JSON Traits
                let name = schema
                    .get_trait_as::<XmlNameTrait>()
                    .map_or_else(|| me.name(), |val| val.name());
                // Add attribute prefix if applicable
                if schema.contains_type::<XmlAttributeTrait>() {
                    return Ok(staticize(format!("@{name}")));
                }
                Ok(staticize(name))
            }
            NameMapper::Default => Ok(staticize(me.name())),
        }
    }
}

impl<S: serde::Serializer> Serializer for SerAdapter<S> {
    type Error = SerErrorWrapper<S::Error>;
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
        let struct_name = self.mapper.get_struct_name(schema);
        let struct_ser = self.serializer.serialize_struct(struct_name, len)?;
        Ok(StructSerializerAdapter::new(struct_ser, self.mapper))
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
        value: &BigInt,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(value.serialize(self.serializer)?)
    }

    #[inline]
    fn write_big_decimal(
        self,
        _schema: &SchemaRef,
        value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(value.serialize(self.serializer)?)
    }

    #[inline]
    fn write_string(self, _: &SchemaRef, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.serialize_str(value)?)
    }

    #[inline]
    fn write_blob(self, _: &SchemaRef, _value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        // TODO(streams): How to support data streams?
        todo!()
    }

    #[inline]
    fn write_timestamp(self, _: &SchemaRef, _value: &Instant) -> Result<Self::Ok, Self::Error> {
        // TODO(timestamp formatting): How to write timestamps with formatting traits?
        todo!()
    }

    #[inline]
    fn write_document(
        self,
        _: &SchemaRef,
        _value: &Box<dyn Document>,
    ) -> Result<Self::Ok, Self::Error> {
        // TODO(document serde): Finish implementing for document types.
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

#[doc(hidden)]
pub struct ListSerializeAdapter<S: serde::Serializer> {
    serializer: S::SerializeSeq,
}
impl<S: serde::Serializer> ListSerializeAdapter<S> {
    const fn new(serializer: S::SerializeSeq) -> Self {
        Self { serializer }
    }
}
impl<S: serde::Serializer> ListSerializer for ListSerializeAdapter<S> {
    type Error = SerErrorWrapper<S::Error>;
    type Ok = S::Ok;

    #[inline]
    fn serialize_element<T>(
        &mut self,
        value_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
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

#[doc(hidden)]
pub struct MapSerializerAdapter<S: serde::Serializer> {
    serializer: S::SerializeMap,
}
impl<S: serde::Serializer> MapSerializerAdapter<S> {
    const fn new(serializer: S::SerializeMap) -> Self {
        Self { serializer }
    }
}
impl<S: serde::Serializer> MapSerializer for MapSerializerAdapter<S> {
    type Error = SerErrorWrapper<S::Error>;
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
        K: SerializeWithSchema,
        V: SerializeWithSchema,
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

#[doc(hidden)]
pub struct StructSerializerAdapter<S: serde::Serializer> {
    serializer: S::SerializeStruct,
    mapper: NameMapper,
}
impl<S: serde::Serializer> StructSerializerAdapter<S> {
    const fn new(serializer: S::SerializeStruct, mapper: NameMapper) -> Self {
        Self { serializer, mapper }
    }
}
impl<S: serde::Serializer> StructSerializer for StructSerializerAdapter<S> {
    type Error = SerErrorWrapper<S::Error>;
    type Ok = S::Ok;

    #[inline]
    fn serialize_member<T>(
        &mut self,
        member_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        let name = self.mapper.get_member_name::<Self>(member_schema)?;
        Ok(self
            .serializer
            .serialize_field(name, &ValueWrapper(member_schema, value))?)
    }

    #[inline]
    fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(self.serializer.end()?)
    }
}

//========================================================================
// Value Wrapper
// -------------
// Wraps inner values of
//========================================================================

struct ValueWrapper<'a, T: SerializeWithSchema>(&'a SchemaRef, &'a T);
impl<T: SerializeWithSchema> Serialize for ValueWrapper<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.1
            .serialize_with_schema(self.0, SerAdapter::new(serializer))
            .map_err(|wrapper| wrapper.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{IndexMap, derive::SmithyShape, schema::prelude::*, smithy};

    smithy!("com.example#Map": {
        map MAP_SCHEMA {
            key: STRING
            value: STRING
        }
    });
    smithy!("com.example#List": {
        list LIST_SCHEMA {
            member: STRING
        }
    });
    smithy!("com.example#Test": {
        structure SCHEMA {
            A: STRING = "a"
            B: STRING = "b"
            MAP: MAP_SCHEMA = "map"
            LIST: LIST_SCHEMA = "list"
        }
    });

    #[derive(SmithyShape)]
    #[smithy_schema(SCHEMA)]
    pub struct Test {
        #[smithy_schema(A)]
        a: String,
        #[smithy_schema(B)]
        b: String,
        #[smithy_schema(LIST)]
        member_list: Vec<String>,
        #[smithy_schema(MAP)]
        member_map: IndexMap<String, String>,
    }

    fn get_test_shape() -> Test {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        map.insert(String::from("c"), String::from("d"));
        Test {
            a: "a".to_string(),
            b: "b".to_string(),
            member_list: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            member_map: map,
        }
    }

    #[test]
    fn can_use_serde_json() {
        let test = get_test_shape();
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
    }

    #[test]
    fn can_use_serde_xml() {
        let test = get_test_shape();
        let expected = r#"<?xml version="1.0" encoding="UTF-8"?><Test><a>a</a><b>b</b><list>a</list><list>b</list><list>c</list><map><a>b</a><c>d</c></map></Test>"#;
        assert_eq!(expected, serde_xml_rs::to_string(&test).unwrap());
    }

    // --------------------------------------------------------------------
    // JSON Trait support tests
    // --------------------------------------------------------------------

    smithy!("com.example#Rename": {
        structure RENAME {
            @JsonNameTrait::new("renamed");
            A: STRING = "a"
        }
    });
    #[derive(SmithyShape)]
    #[smithy_schema(RENAME)]
    pub struct TestRename {
        #[smithy_schema(A)]
        a: String,
    }

    #[test]
    fn json_name_works() {
        let rename = TestRename { a: "a".to_string() };
        let expected = r#"{
  "renamed": "a"
}"#;
        assert_eq!(serde_json::to_string_pretty(&rename).unwrap(), expected);
    }

    // --------------------------------------------------------------------
    // XML Trait support tests
    // --------------------------------------------------------------------

    smithy!("com.example#Rename": {
        structure XML_TRAITS {
            @XmlNameTrait::new("renamed");
            @XmlAttributeTrait;
            A: STRING = "a"
            @XmlNameTrait::new("int");
            B: STRING = "b"
        }
    });
    #[derive(SmithyShape)]
    #[smithy_schema(XML_TRAITS)]
    pub struct TestXml {
        #[smithy_schema(A)]
        a: String,
        #[smithy_schema(B)]
        b: i32,
    }

    #[test]
    fn xml_traits_work() {
        let rename = TestXml {
            a: "a".to_string(),
            b: 2,
        };
        let expected =
            r#"<?xml version="1.0" encoding="UTF-8"?><Rename renamed="a"><int>2</int></Rename>"#;
        assert_eq!(serde_xml_rs::to_string(&rename).unwrap(), expected);
    }
}
