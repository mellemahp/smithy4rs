#![allow(dead_code, unused_imports, unused_variables)]

use std::{collections::HashMap, fmt::Display, marker::PhantomData, sync::Arc};

use indexmap::IndexMap;
use thiserror::Error;

use crate::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    prelude::{BIG_DECIMAL, BIG_INTEGER, BOOLEAN, BYTE},
    schema::{
        DefaultDocumentValue, Document, DocumentError, DocumentImpl, DocumentValue,
        LIST_DOCUMENT_SCHEMA, MAP_DOCUMENT_SCHEMA, NumberFloat, NumberInteger, NumberValue, Schema,
        SchemaRef, SchemaShape, ShapeId, ShapeType, StaticSchemaShape, TraitList, get_shape_type,
    },
    serde::{
        Buildable, ShapeBuilder,
        de::Deserializer,
        deserializers::DeserializeWithSchema,
        se::{ListSerializer, MapSerializer, Serializer, StructSerializer},
        serializers::{Error, SerializableShape, SerializeWithSchema},
    },
};
// ============================================================================
// Serialization
// ============================================================================

impl SerializeWithSchema for Document {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // TODO(errors): Handle exceptions?
        match get_shape_type(schema).unwrap() {
            ShapeType::Blob => serializer.write_blob(schema, self.value.as_blob().unwrap()),
            ShapeType::Boolean => serializer.write_boolean(schema, self.value.as_bool().unwrap()),
            ShapeType::String => serializer.write_string(schema, self.value.as_string().unwrap()),
            ShapeType::Timestamp => {
                serializer.write_timestamp(schema, self.value.as_timestamp().unwrap())
            }
            ShapeType::Byte => serializer.write_byte(schema, self.value.as_byte().unwrap()),
            ShapeType::Short => serializer.write_short(schema, self.value.as_short().unwrap()),
            ShapeType::Integer => {
                serializer.write_integer(schema, self.value.as_integer().unwrap())
            }
            ShapeType::Long => serializer.write_long(schema, self.value.as_long().unwrap()),
            ShapeType::Float => serializer.write_float(schema, self.value.as_float().unwrap()),
            ShapeType::Double => serializer.write_double(schema, self.value.as_double().unwrap()),
            ShapeType::BigInteger => {
                serializer.write_big_integer(schema, self.value.as_big_integer().unwrap())
            }
            ShapeType::BigDecimal => {
                serializer.write_big_decimal(schema, self.value.as_big_decimal().unwrap())
            }
            ShapeType::Document => serializer.write_document(schema, self),
            ShapeType::Enum => serializer.write_string(schema, self.value.as_string().unwrap()),
            ShapeType::IntEnum => {
                serializer.write_integer(schema, self.value.as_integer().unwrap())
            }
            ShapeType::List => self
                .value
                .as_list()
                .unwrap()
                .serialize_with_schema(schema, serializer),
            ShapeType::Map => self
                .value
                .as_map()
                .unwrap()
                .serialize_with_schema(schema, serializer),
            ShapeType::Structure | ShapeType::Union => {
                let document_map = self.value.as_map().unwrap();
                let mut struct_serializer = serializer.write_struct(schema, self.size())?;
                if let Some(discriminator) = &self.discriminator {
                    struct_serializer.serialize_discriminator(discriminator)?;
                }
                for (key, value) in document_map {
                    if let Some(member_schema) = schema.get_member(key) {
                        struct_serializer.serialize_member(member_schema, value)?;
                    } else {
                        // TODO(unknown members) Should unknown members be allowed?
                        todo!("Add some logging on unknown members");
                    }
                }
                struct_serializer.end(schema)
            }
            _ => Err(Error::custom("Unsupported shape type")),
        }
    }
}

impl<T> From<T> for Document
where
    T: StaticSchemaShape + SerializeWithSchema,
{
    fn from(shape: T) -> Self {
        shape
            .serialize_with_schema(T::schema(), DocumentParser)
            .expect(
                "Infallible conversion from StaticSchemaShape to Document failed - this is a bug",
            )
    }
}

impl Document {
    /// Convert any serializable (schema-based) shape to a Document.
    ///
    /// # Errors
    ///
    /// Returns `DocumentError` if the shape cannot be serialized to a document,
    /// typically due to schema mismatches or validation failures.
    ///
    pub fn from<T: SchemaShape + SerializeWithSchema>(shape: T) -> Result<Self, DocumentError> {
        shape.serialize_with_schema(shape.schema(), DocumentParser)
    }
}

impl Error for DocumentError {
    fn custom<T: Display>(msg: T) -> Self {
        DocumentError::CustomError(msg.to_string())
    }
}

pub struct DocumentParser;
// TODO(document validation): Should this have schema type validation?
impl Serializer for DocumentParser {
    type Error = DocumentError;
    type Ok = Document;
    type SerializeList = DocumentListAccumulator;
    type SerializeMap = DocumentMapAccumulator;
    type SerializeStruct = DocumentMapAccumulator;

    fn write_struct(
        self,
        schema: &SchemaRef,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(DocumentMapAccumulator {
            schema: schema.clone(),
            values: IndexMap::with_capacity(len),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_map(self, schema: &SchemaRef, len: usize) -> Result<Self::SerializeMap, Self::Error> {
        Ok(DocumentMapAccumulator {
            schema: schema.clone(),
            values: IndexMap::with_capacity(len),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_list(
        self,
        schema: &SchemaRef,
        len: usize,
    ) -> Result<Self::SerializeList, Self::Error> {
        Ok(DocumentListAccumulator {
            schema: schema.clone(),
            values: Vec::with_capacity(len),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_boolean(self, schema: &SchemaRef, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DefaultDocumentValue::Boolean(value).into(),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_byte(self, schema: &SchemaRef, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DefaultDocumentValue::Number(NumberValue::from_i8(value)).into(),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_short(self, schema: &SchemaRef, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DefaultDocumentValue::Number(NumberValue::from_i16(value)).into(),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_integer(self, schema: &SchemaRef, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DefaultDocumentValue::Number(NumberValue::from_i32(value)).into(),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_long(self, schema: &SchemaRef, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DefaultDocumentValue::Number(NumberValue::from_i64(value)).into(),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_float(self, schema: &SchemaRef, value: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DefaultDocumentValue::Number(NumberValue::from_f32(value)).into(),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_double(self, schema: &SchemaRef, value: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DefaultDocumentValue::Number(NumberValue::from_f64(value)).into(),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_big_integer(
        self,
        schema: &SchemaRef,
        value: &BigInt,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DefaultDocumentValue::Number(NumberValue::from_big_int(value.clone())).into(),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_big_decimal(
        self,
        schema: &SchemaRef,
        value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DefaultDocumentValue::Number(NumberValue::from_big_decimal(value.clone()))
                .into(),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_string(self, schema: &SchemaRef, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DefaultDocumentValue::String(value.to_owned()).into(),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_blob(self, schema: &SchemaRef, value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DefaultDocumentValue::Blob(value.clone()).into(),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_timestamp(self, schema: &SchemaRef, value: &Instant) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DefaultDocumentValue::Timestamp(*value).into(),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_document(self, schema: &SchemaRef, value: &Document) -> Result<Self::Ok, Self::Error> {
        Ok(value.clone())
    }

    fn write_null(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DefaultDocumentValue::Null.into(),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn skip(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        // When skipping (e.g., for None values), return a null document
        Ok(Document {
            schema: schema.clone(),
            value: DefaultDocumentValue::Null.into(),
            discriminator: Some(schema.id().clone()),
        })
    }
}

#[doc(hidden)]
pub struct DocumentListAccumulator {
    schema: SchemaRef,
    values: Vec<Document>,
    discriminator: Option<ShapeId>,
}
impl ListSerializer for DocumentListAccumulator {
    type Error = DocumentError;
    type Ok = Document;

    fn serialize_element<T>(
        &mut self,
        element_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
    {
        let el = value.serialize_with_schema(element_schema, DocumentParser)?;
        self.values.push(el);
        Ok(())
    }

    fn end(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: self.schema,
            value: DefaultDocumentValue::List(self.values).into(),
            discriminator: self.discriminator,
        })
    }
}

#[doc(hidden)]
pub struct DocumentMapAccumulator {
    schema: SchemaRef,
    values: IndexMap<String, Document>,
    discriminator: Option<ShapeId>,
}
impl MapSerializer for DocumentMapAccumulator {
    type Error = DocumentError;
    type Ok = Document;

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
        // Serialize the key to get its string representation
        let key_doc = key.serialize_with_schema(key_schema, DocumentParser)?;
        let key_str = key_doc
            .value
            .as_string()
            .ok_or_else(|| {
                DocumentError::DocumentConversion("Map key must be a string".to_string())
            })?
            .to_string();

        let val = value.serialize_with_schema(value_schema, DocumentParser)?;
        self.values.insert(key_str, val);
        Ok(())
    }

    fn end(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: self.schema,
            value: DefaultDocumentValue::Map(self.values).into(),
            discriminator: self.discriminator,
        })
    }
}

impl StructSerializer for DocumentMapAccumulator {
    type Error = DocumentError;
    type Ok = Document;

    fn serialize_member<T>(
        &mut self,
        member_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
    {
        let Some(me) = member_schema.as_member() else {
            return Err(DocumentError::DocumentConversion(
                "Expected member schema!".to_string(),
            ));
        };
        let val = value.serialize_with_schema(member_schema, DocumentParser)?;
        self.values.insert(me.name.clone(), val);
        Ok(())
    }

    fn end(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: self.schema,
            value: DefaultDocumentValue::Map(self.values).into(),
            discriminator: self.discriminator,
        })
    }
}

// ============================================================================
// Deserialization
// ============================================================================

/// A deserializer that reads from a `Document`.
pub(crate) struct DocumentDeserializer {
    document: Option<Document>,
}

impl DocumentDeserializer {
    pub fn new(document: Document) -> Self {
        Self {
            document: Some(document),
        }
    }

    #[inline]
    fn get_inner<T: TryFrom<Document, Error = DocumentError>>(
        &mut self,
    ) -> Result<T, DocumentError> {
        self.document
            .take()
            .ok_or_else(|| {
                DocumentError::DocumentConversion(
                    "Encountered empty document deserializer".to_string(),
                )
            })?
            .try_into()
    }
}

// TODO: Is the expecting necessary? Is there a better way than the
impl Deserializer<'_> for DocumentDeserializer {
    type Error = DocumentError;

    #[inline]
    fn read_bool(&mut self, schema: &SchemaRef) -> Result<bool, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_byte(&mut self, schema: &SchemaRef) -> Result<i8, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_short(&mut self, schema: &SchemaRef) -> Result<i16, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_integer(&mut self, schema: &SchemaRef) -> Result<i32, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_long(&mut self, schema: &SchemaRef) -> Result<i64, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_float(&mut self, schema: &SchemaRef) -> Result<f32, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_double(&mut self, schema: &SchemaRef) -> Result<f64, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_big_integer(&mut self, schema: &SchemaRef) -> Result<BigInt, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_big_decimal(&mut self, schema: &SchemaRef) -> Result<BigDecimal, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_string(&mut self, schema: &SchemaRef) -> Result<String, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_blob(&mut self, schema: &SchemaRef) -> Result<ByteBuffer, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_timestamp(&mut self, schema: &SchemaRef) -> Result<Instant, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_document(&mut self, schema: &SchemaRef) -> Result<Document, Self::Error> {
        self.document.take().ok_or_else(|| {
            DocumentError::DocumentConversion("Encountered empty document deserializer".to_string())
        })
    }

    fn read_struct<B, F>(
        &mut self,
        schema: &SchemaRef,
        mut builder: B,
        mut consumer: F,
    ) -> Result<B, Self::Error>
    where
        F: FnMut(B, &SchemaRef, &mut Self) -> Result<B, Self::Error>,
    {
        let map: IndexMap<String, Document> = self.get_inner()?;

        // Iterate through members in the document map so we have owned values.
        // Add only values that match the provided schema.
        for (key, value) in map.into_iter() {
            if let Some(member_schema) = schema.members().get(&key) {
                let mut field_deser = DocumentDeserializer::new(value);
                builder = consumer(builder, member_schema, &mut field_deser)?;
            }
        }

        Ok(builder)
    }

    fn read_list<T, F>(
        &mut self,
        schema: &SchemaRef,
        state: &mut T,
        mut consumer: F,
    ) -> Result<(), Self::Error>
    where
        F: FnMut(&mut T, &SchemaRef, &mut Self) -> Result<(), Self::Error>,
    {
        let list: Vec<Document> = self.get_inner()?;

        let member_schema = schema.get_member("member").ok_or_else(|| {
            DocumentError::DocumentConversion("List missing member schema".to_string())
        })?;

        for element_doc in list {
            let mut elem_deser = DocumentDeserializer::new(element_doc);
            consumer(state, member_schema, &mut elem_deser)?;
        }

        Ok(())
    }

    fn read_map<T, F>(
        &mut self,
        schema: &SchemaRef,
        state: &mut T,
        mut consumer: F,
    ) -> Result<(), Self::Error>
    where
        F: FnMut(&mut T, String, &mut Self) -> Result<(), Self::Error>,
    {
        let map: IndexMap<String, Document> = self.get_inner()?;
        for (key_str, value_doc) in map {
            // Key is already a String, create deserializer for the value
            let mut value_deser = DocumentDeserializer::new(value_doc);
            consumer(state, key_str.clone(), &mut value_deser)?;
        }
        Ok(())
    }

    fn is_null(&mut self) -> bool {
        self.document
            .as_ref()
            .expect("Empty document deserializer")
            .is_null()
    }

    fn read_null(&mut self) -> Result<(), Self::Error> {
        if self.is_null() {
            Ok(())
        } else {
            Err(DocumentError::DocumentConversion(
                "Expected null document".to_string(),
            ))
        }
    }
}

// TODO(test): overhaul these to use test shapes
#[cfg(test)]
mod tests {
    use std::{str::FromStr, sync::LazyLock};

    use smithy4rs_core_derive::SmithyShape;

    use super::*;
    use crate::{
        prelude::*,
        schema::{Schema, ShapeId},
        smithy,
    };

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
    smithy!("com.example#Shape": {
        structure SCHEMA {
            A: STRING = "a"
            B: STRING = "b"
            C: STRING = "c"
            LIST: LIST_SCHEMA = "list"
            MAP: MAP_SCHEMA = "map"
        }
    });

    #[derive(SmithyShape, Clone, PartialEq, Debug)]
    #[smithy_schema(SCHEMA)]
    pub struct SerializeMe {
        #[smithy_schema(A)]
        pub member_a: String,
        #[smithy_schema(B)]
        pub member_b: String,
        #[smithy_schema(C)]
        pub member_optional: Option<String>,
        #[smithy_schema(LIST)]
        pub member_list: Vec<String>,
        #[smithy_schema(MAP)]
        pub member_map: IndexMap<String, String>,
    }

    #[test]
    fn struct_to_document() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let list = vec!["a".to_string(), "b".to_string()];
        let struct_to_convert = SerializeMe {
            member_a: "a".to_string(),
            member_b: "b".to_string(),
            member_optional: Some("c".to_string()),
            member_map: map,
            member_list: list,
        };
        let document: Document = struct_to_convert.into();
        assert_eq!(&document.discriminator.clone().unwrap(), SCHEMA.id());
        if let Some(members) = document.as_map() {
            let doc_a = &members.get("a").unwrap();
            assert_eq!(doc_a.as_string().unwrap(), "a");
            let doc_b = &members.get("b").unwrap();
            assert_eq!(doc_b.as_string().unwrap(), "b");
            let doc_c = &members.get("c").unwrap();
            assert_eq!(doc_c.as_string().unwrap(), "c");
            assert!(members.contains_key("map"));
            assert!(members.contains_key("list"));
        } else {
            panic!("Expected document");
        }
    }

    #[test]
    fn string_document_value() {
        let document_str: Document = "MyStr".into();
        let output_str = document_str.as_string().expect("string");
        assert_eq!(output_str, &"MyStr".to_string());
        let val: &SchemaRef = &STRING;
        assert_eq!(document_str.schema(), val);
    }

    #[test]
    fn number_document_values() {
        let x: &SchemaRef = &STRING;
    }

    // Roundtrip tests: value -> serialize to Document -> deserialize back to value

    #[test]
    fn roundtrip_shape() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let list = vec!["a".to_string(), "b".to_string()];
        let struct_to_convert = SerializeMe {
            member_a: "a".to_string(),
            member_b: "b".to_string(),
            member_optional: Some("c".to_string()),
            member_map: map,
            member_list: list,
        };
        let document: Document = struct_to_convert.clone().into();
        let result: SerializeMe = SerializeMeBuilder::from_document(document)
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(result, struct_to_convert);
    }

    #[test]
    fn roundtrip_bool() {
        let original = true;
        let doc: Document = original.into();
        let result: bool = doc.try_into().unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn roundtrip_string() {
        let doc: Document = "hello world".into();
        let result: String = doc.try_into().unwrap();
        assert_eq!("hello world".to_string(), result);
    }

    #[test]
    fn roundtrip_integers() {
        // Byte
        let original_byte = 127i8;
        let doc: Document = original_byte.into();
        let result: i8 = doc.try_into().unwrap();
        assert_eq!(127i8, result);

        // Short
        let original_short = 32000i16;
        let doc: Document = original_short.into();
        let result: i16 = doc.try_into().unwrap();
        assert_eq!(original_short, result);

        // Integer
        let original_int = 123456i32;
        let doc: Document = original_int.into();
        let result: i32 = doc.try_into().unwrap();
        assert_eq!(original_int, result);

        // Long
        let original_long = 9876543210i64;
        let doc: Document = original_long.into();
        let result: i64 = doc.try_into().unwrap();
        assert_eq!(original_long, result);
    }

    #[test]
    fn roundtrip_floats() {
        // Float
        let original_float = 1.2345f32;
        let doc: Document = original_float.into();
        let result: f32 = doc.try_into().unwrap();
        assert_eq!(original_float, result);

        // Double
        let original_double = 1.23456789f64;
        let doc: Document = original_double.into();
        let result: f64 = doc.try_into().unwrap();
        assert_eq!(original_double, result);
    }

    #[test]
    #[ignore = "BigDecimal/BigInteger serialization not yet implemented"]
    fn roundtrip_big_numbers() {
        // BigInteger
        let original_big_int = BigInt::from(123456789);
        let doc: Document = original_big_int.clone().into();
        let result: BigInt = doc.try_into().unwrap();
        assert_eq!(original_big_int, result);

        // BigDecimal
        let original_big_dec = BigDecimal::from_str("123.456").unwrap();
        let doc: Document = original_big_dec.clone().into();
        let result: BigDecimal = doc.try_into().unwrap();
        assert_eq!(original_big_dec, result);
    }

    #[test]
    fn roundtrip_list() {
        let original: Vec<String> = vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ];
        let doc: Document = original.clone().into();
        let result: Vec<String> = doc.try_into().unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn roundtrip_map() {
        let mut original = IndexMap::new();
        original.insert("key1".to_string(), "value1".to_string());
        original.insert("key2".to_string(), "value2".to_string());
        original.insert("key3".to_string(), "value3".to_string());

        let doc: Document = original.clone().into();
        let result: IndexMap<String, String> = doc.try_into().unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn roundtrip_struct() {
        let mut original_map = IndexMap::new();
        original_map.insert("mapkey1".to_string(), "mapvalue1".to_string());
        original_map.insert("mapkey2".to_string(), "mapvalue2".to_string());

        let original_list = vec![
            "listitem1".to_string(),
            "listitem2".to_string(),
            "listitem3".to_string(),
        ];

        let original = SerializeMe {
            member_a: "value_a".to_string(),
            member_b: "value_b".to_string(),
            member_optional: Some("value_c".to_string()),
            member_map: original_map.clone(),
            member_list: original_list.clone(),
        };

        // Serialize to document
        let doc: Document = original.into();

        // Deserialize back field by field (since we don't have a full Deserialize impl for SerializeMe)
        let doc_map = doc.as_map().expect("Should be a map");

        // Check member_a
        let member_a_doc = doc_map.get("a").expect("Should have member a").clone();
        let a_value: String = member_a_doc.try_into().unwrap();
        assert_eq!(a_value, "value_a".to_string());

        // Check member_b
        let member_b_doc = doc_map.get("b").expect("Should have member b").clone();
        let b_value: String = member_b_doc.try_into().unwrap();
        assert_eq!(b_value, "value_b");

        // Check member_optional
        let member_c_doc = doc_map.get("c").expect("Should have member c").clone();
        let c_value: String = member_c_doc.try_into().unwrap();
        assert_eq!(c_value, "value_c");

        // Check list
        let list_doc = doc_map.get("list").expect("Should have list").clone();
        let list_value: Vec<String> = list_doc.try_into().unwrap();
        assert_eq!(list_value, original_list);

        // Check map
        let map_doc = doc_map.get("map").expect("Should have map").clone();
        let map_value: IndexMap<String, String> = map_doc.try_into().unwrap();
        assert_eq!(map_value, original_map);
    }

    #[test]
    fn roundtrip_option() {
        // Some value
        let original_some: Option<String> = Some("test".to_string());
        let doc: Document = original_some.clone().into();
        let result: Option<String> = doc.try_into().unwrap();
        assert_eq!(original_some, result);

        // None value
        let original_none: Option<String> = None;
        let doc: Document = original_none.clone().into();
        let result: Option<String> = doc.try_into().unwrap();
        assert_eq!(original_none, result);
    }
}
