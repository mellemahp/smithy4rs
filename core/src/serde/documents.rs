#![allow(dead_code, unused_imports, unused_variables)]

use std::{collections::HashMap, fmt::Display, marker::PhantomData, time::Instant};

use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use indexmap::IndexMap;
use num_bigint::BigInt;
use thiserror::Error;

use crate::{
    prelude::{BIG_DECIMAL, BIG_INTEGER, BOOLEAN, BYTE},
    schema::{
        Document, DocumentError, DocumentValue, LIST_DOCUMENT_SCHEMA, MAP_DOCUMENT_SCHEMA,
        NumberFloat, NumberInteger, NumberValue, Schema, SchemaRef, SchemaShape, ShapeId,
        ShapeType, TraitList, get_shape_type,
    },
    serde::{
        de::{DeserializeWithSchema, Deserializer, Error as DeserializationError},
        deserializers::{Deserialize, Error as DeserializerError},
        se::{ListSerializer, MapSerializer, Serialize, Serializer, StructSerializer},
        serializers::{Error as SerializerError, SerializeWithSchema},
    },
};

// Bring Error trait into scope for D::Error::custom calls
use crate::serde::de::Error as _;

/////////////////////////////////////////////////////////////////////////////////
// Serialization
/////////////////////////////////////////////////////////////////////////////////

/// Marker Trait used to differentiate between generated shapes and Documents for
/// some blanket implementations.
///
/// NOTE: In general you should not need to implement this yourself
pub trait SerializableShape: Serialize {}

impl SerializeWithSchema for Document {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // TODO: Handle exceptions?
        match get_shape_type(schema).unwrap() {
            ShapeType::Blob => serializer.write_blob(schema, self.as_blob().unwrap()),
            ShapeType::Boolean => serializer.write_boolean(schema, self.as_bool().unwrap()),
            ShapeType::String => serializer.write_string(schema, self.as_string().unwrap()),
            ShapeType::Timestamp => {
                serializer.write_timestamp(schema, self.as_timestamp().unwrap())
            }
            ShapeType::Byte => serializer.write_byte(schema, self.as_byte().unwrap()),
            ShapeType::Short => serializer.write_short(schema, self.as_short().unwrap()),
            ShapeType::Integer => serializer.write_integer(schema, self.as_integer().unwrap()),
            ShapeType::Long => serializer.write_long(schema, self.as_long().unwrap()),
            ShapeType::Float => serializer.write_float(schema, self.as_float().unwrap()),
            ShapeType::Double => serializer.write_double(schema, self.as_double().unwrap()),
            ShapeType::BigInteger => {
                serializer.write_big_integer(schema, self.as_big_integer().unwrap())
            }
            ShapeType::BigDecimal => {
                serializer.write_big_decimal(schema, self.as_big_decimal().unwrap())
            }
            ShapeType::Document => serializer.write_document(schema, self),
            ShapeType::Enum => serializer.write_string(schema, self.as_string().unwrap()),
            ShapeType::IntEnum => serializer.write_integer(schema, self.as_integer().unwrap()),
            ShapeType::List => self
                .as_list()
                .unwrap()
                .serialize_with_schema(schema, serializer),
            ShapeType::Map => self
                .as_map()
                .unwrap()
                .serialize_with_schema(schema, serializer),
            ShapeType::Structure | ShapeType::Union => {
                let document_map = self.as_map().unwrap();
                let mut struct_serializer = serializer.write_struct(schema, self.size())?;
                if let Some(discriminator) = &self.discriminator {
                    struct_serializer.serialize_discriminator(discriminator)?;
                }
                for (key, value) in document_map {
                    if let Some(member_schema) = schema.get_member(key) {
                        struct_serializer.serialize_member(member_schema, value)?;
                    } else {
                        // TODO: Should unknown members be allowed???
                        todo!("Add some logging on unknown members");
                    }
                }
                struct_serializer.end(schema)
            }
            _ => Err(SerializerError::custom("Unsupported shape type")),
        }
    }
}

impl<T: SerializableShape> From<T> for Document {
    fn from(shape: T) -> Self {
        // TODO: should this be fallible?
        shape.serialize(DocumentSerializer).unwrap()
    }
}

impl SerializerError for DocumentError {
    fn custom<T: Display>(msg: T) -> Self {
        DocumentError::CustomError(msg.to_string())
    }
}

impl DeserializerError for DocumentError {
    fn custom<T: Display>(msg: T) -> Self {
        DocumentError::CustomError(msg.to_string())
    }
}

struct DocumentSerializer;
// TODO: Should this have schema type validation?
impl Serializer for DocumentSerializer {
    // TODO: Error
    type Error = DocumentError;
    type Ok = Document;
    type SerializeList = Document;
    type SerializeMap = Document;
    type SerializeStruct = Document;

    fn write_struct(
        self,
        schema: &SchemaRef,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::Map(IndexMap::with_capacity(len)),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_map(self, schema: &SchemaRef, len: usize) -> Result<Self::SerializeMap, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::Map(IndexMap::with_capacity(len)),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_list(
        self,
        schema: &SchemaRef,
        len: usize,
    ) -> Result<Self::SerializeList, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::List(Vec::with_capacity(len)),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_boolean(self, schema: &SchemaRef, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::Boolean(value),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_byte(self, schema: &SchemaRef, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::from_i8(value)),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_short(self, schema: &SchemaRef, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::from_i16(value)),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_integer(self, schema: &SchemaRef, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::from_i32(value)),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_long(self, schema: &SchemaRef, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::from_i64(value)),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_float(self, schema: &SchemaRef, value: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::from_f32(value)),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_double(self, schema: &SchemaRef, value: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::from_f64(value)),
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
            value: DocumentValue::Number(NumberValue::from_big_int(value.clone())),
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
            value: DocumentValue::Number(NumberValue::from_big_decimal(value.clone())),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_string(self, schema: &SchemaRef, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::String(value.to_owned()),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_blob(self, schema: &SchemaRef, value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::Blob(value.clone()),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_timestamp(self, schema: &SchemaRef, value: &Instant) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::Timestamp(*value),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_document(self, schema: &SchemaRef, value: &Document) -> Result<Self::Ok, Self::Error> {
        Ok(value.clone())
    }

    fn write_null(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::Null,
            discriminator: Some(schema.id().clone()),
        })
    }

    fn skip(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl ListSerializer for Document {
    // TODO: Errors
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
        let DocumentValue::List(list) = &mut self.value else {
            return Err(DocumentError::DocumentConversion(
                "Could not convert document to list.".to_string(),
            ));
        };
        let el = value.serialize_with_schema(element_schema, DocumentSerializer)?;
        list.push(el);
        Ok(())
    }

    fn end(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(self)
    }
}

impl MapSerializer for Document {
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
        let DocumentValue::Map(map) = &mut self.value else {
            return Err(DocumentError::DocumentConversion(
                "Could not convert document to Map.".to_string(),
            ));
        };
        let Some(me) = key_schema.as_member() else {
            return Err(DocumentError::DocumentConversion(
                "Expected `key` schema.".to_string(),
            ));
        };
        let val = value.serialize_with_schema(value_schema, DocumentSerializer)?;
        map.insert(me.name.clone(), val);
        Ok(())
    }

    fn end(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(self)
    }
}

impl StructSerializer for Document {
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
        let DocumentValue::Map(map) = &mut self.value else {
            return Err(DocumentError::DocumentConversion(
                "Expected map document".to_string(),
            ));
        };
        let Some(me) = member_schema.as_member() else {
            return Err(DocumentError::DocumentConversion(
                "Expected member schema!".to_string(),
            ));
        };
        let val = value.serialize_with_schema(member_schema, DocumentSerializer)?;
        map.insert(me.name.clone(), val);
        Ok(())
    }

    fn end(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(self)
    }
}

/////////////////////////////////////////////////////////////////////////////////
// Deserialization
/////////////////////////////////////////////////////////////////////////////////
pub trait DeserializableShape: Deserialize {} // TODO: not needed perhaps

impl DeserializeWithSchema for Document {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        // Use schema introspection to determine how to deserialize
        match get_shape_type(schema).map_err(|e| D::Error::custom(e.to_string()))? {
            ShapeType::Boolean => {
                let value = deserializer.read_boolean(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Boolean(value),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Byte => {
                let value = deserializer.read_byte(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Number(NumberValue::from_i8(value)),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Short => {
                let value = deserializer.read_short(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Number(NumberValue::from_i16(value)),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Integer => {
                let value = deserializer.read_integer(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Number(NumberValue::from_i32(value)),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Long => {
                let value = deserializer.read_long(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Number(NumberValue::from_i64(value)),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Float => {
                let value = deserializer.read_float(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Number(NumberValue::from_f32(value)),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Double => {
                let value = deserializer.read_double(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Number(NumberValue::from_f64(value)),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::BigInteger => {
                let value = deserializer.read_big_integer(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Number(NumberValue::from_big_int(value)),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::BigDecimal => {
                let value = deserializer.read_big_decimal(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Number(NumberValue::from_big_decimal(value)),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::String => {
                let value = deserializer.read_string(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::String(value),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Blob => {
                let value = deserializer.read_blob(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Blob(value),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Timestamp => {
                let value = deserializer.read_timestamp(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Timestamp(value),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Document => deserializer.read_document(schema),
            ShapeType::Enum => {
                // Enums are represented as strings in documents
                let value = deserializer.read_string(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::String(value),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::IntEnum => {
                // IntEnums are represented as integers in documents
                let value = deserializer.read_integer(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Number(NumberValue::from_i32(value)),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::List => {
                let value = deserializer.read_list(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::List(value),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Map => {
                let value = deserializer.read_map(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Map(value),
                    discriminator: Some(schema.id().clone()),
                })            }
            ShapeType::Structure | ShapeType::Union => {
                let value = deserializer.read_struct(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Map(value),
                    discriminator: Some(schema.id().clone()),
                })
            }
            _ => Err(D::Error::custom(
                "Unsupported shape type for document deserialization",
            )),
        }
    }
}

/// Document deserializer
/// TODO: Should we validate the document using the schema?
pub struct DocumentDeserializer<'doc> {
    document: &'doc Document,
}

impl<'doc> DocumentDeserializer<'doc> {
    pub fn new(document: &'doc Document) -> Self {
        Self { document }
    }
}

impl<'de, 'doc> Deserializer<'de> for DocumentDeserializer<'doc> {
    type Error = DocumentError;

    fn read_struct<T: DeserializeWithSchema>(self, schema: &SchemaRef) -> Result<T, Self::Error> {
        let Some(map) = self.document.as_map() else {
            return Err(DocumentError::DocumentConversion(
                "Expected map document for struct".to_string(),
            ));
        };
        
        for (member_name, member_schema) in schema.members() {
            let Some(member) = member_schema.as_member() else {
                return Err(DocumentError::DocumentConversion(
                    "Expected member schema".to_string(),
                ));
            };
            
            let Some(member_document) = map.get(member_name) else {
                return Err(DocumentError::DocumentConversion(
                    format!("Missing member '{}'", member_name),
                ));
            };
    
            let deserializer = DocumentDeserializer::new(member_document);
            let value = Document::deserialize_with_schema(member_schema, deserializer)?;
        };
        
        todo!()
    }

    fn read_map<K: DeserializeWithSchema, V: DeserializeWithSchema>(
        self,
        schema: &SchemaRef,
    ) -> Result<IndexMap<K, V>, Self::Error> {
        todo!()
    }

    fn read_list<T: DeserializeWithSchema>(
        self,
        schema: &SchemaRef,
    ) -> Result<Vec<T>, Self::Error> {
        let Some(list) = self.document.as_list() else {
            return Err(DocumentError::DocumentConversion(
                "Expected list document for list".to_string(),
            ));
        };

        let element_schema = schema.expect_member("member");
        let mut result = Vec::new();

        for document in list {
            let deserializer = DocumentDeserializer::new(document);
            let element = T::deserialize_with_schema(element_schema, deserializer)?;
            result.push(element);
        }

        Ok(result)
    }

    fn read_boolean(self, schema: &SchemaRef) -> Result<bool, Self::Error> {
        self.document.as_bool().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected boolean document".to_string())
        })
    }

    fn read_byte(self, schema: &SchemaRef) -> Result<i8, Self::Error> {
        self.document
            .as_byte()
            .ok_or_else(|| DocumentError::DocumentConversion("Expected byte document".to_string()))
    }

    fn read_short(self, schema: &SchemaRef) -> Result<i16, Self::Error> {
        self.document
            .as_short()
            .ok_or_else(|| DocumentError::DocumentConversion("Expected short document".to_string()))
    }

    fn read_integer(self, schema: &SchemaRef) -> Result<i32, Self::Error> {
        self.document.as_integer().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected integer document".to_string())
        })
    }

    fn read_long(self, schema: &SchemaRef) -> Result<i64, Self::Error> {
        self.document
            .as_long()
            .ok_or_else(|| DocumentError::DocumentConversion("Expected long document".to_string()))
    }

    fn read_float(self, schema: &SchemaRef) -> Result<f32, Self::Error> {
        self.document
            .as_float()
            .ok_or_else(|| DocumentError::DocumentConversion("Expected float document".to_string()))
    }

    fn read_double(self, schema: &SchemaRef) -> Result<f64, Self::Error> {
        self.document.as_double().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected double document".to_string())
        })
    }

    fn read_big_integer(self, schema: &SchemaRef) -> Result<BigInt, Self::Error> {
        self.document.as_big_integer().cloned().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected big integer document".to_string())
        })
    }

    fn read_big_decimal(self, schema: &SchemaRef) -> Result<BigDecimal, Self::Error> {
        self.document.as_big_decimal().cloned().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected big decimal document".to_string())
        })
    }

    fn read_string(self, schema: &SchemaRef) -> Result<String, Self::Error> {
        self.document.as_string().cloned().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected string document".to_string())
        })
    }

    fn read_blob(self, schema: &SchemaRef) -> Result<ByteBuffer, Self::Error> {
        self.document
            .as_blob()
            .cloned()
            .ok_or_else(|| DocumentError::DocumentConversion("Expected blob document".to_string()))
    }

    fn read_timestamp(self, schema: &SchemaRef) -> Result<Instant, Self::Error> {
        self.document.as_timestamp().cloned().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected timestamp document".to_string())
        })
    }

    fn read_document(self, schema: &SchemaRef) -> Result<Document, Self::Error> {
        Ok(self.document.clone())
    }

    fn read_null(self, schema: &SchemaRef) -> Result<(), Self::Error> {
        match self.document.value() {
            DocumentValue::Null => Ok(()),
            _ => Err(DocumentError::DocumentConversion(
                "Expected null document".to_string(),
            )),
        }
    }

    fn is_null(&self) -> bool {
        matches!(self.document.value(), DocumentValue::Null)
    }
}

/// Helper function to deserialize from a Document to any type
pub fn from_document<T: DeserializeWithSchema>(
    document: &Document,
    schema: &SchemaRef,
) -> Result<T, DocumentError> {
    let deserializer = DocumentDeserializer::new(document);
    T::deserialize_with_schema(schema, deserializer)
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use smithy4rs_core_derive::SerializableStruct;

    use super::*;
    use crate::{
        lazy_schema,
        prelude::*,
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
        Schema::list_builder("com.example#Map", traits![]),
        ("member", STRING, traits![])
    );
    lazy_schema!(
        SCHEMA,
        Schema::structure_builder(ShapeId::from("com.example#Shape"), traits![]),
        (MEMBER_A, "a", STRING, traits![]),
        (MEMBER_B, "b", STRING, traits![]),
        (MEMBER_C, "c", STRING, traits![]),
        (MEMBER_LIST, "list", LIST_SCHEMA, traits![]),
        (MEMBER_MAP, "map", MAP_SCHEMA, traits![])
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
        if let DocumentValue::Map(members) = document.value {
            assert!(members.contains_key("a"));
            if let DocumentValue::String(str) = &members.get("a").unwrap().value {
                assert_eq!(str, &String::from("a"));
            } else {
                panic!("Expected String")
            }
            assert!(members.contains_key("b"));
            if let DocumentValue::String(str) = &members.get("b").unwrap().value {
                assert_eq!(str, &String::from("b"));
            } else {
                panic!("Expected String")
            }
            assert!(members.contains_key("c"));
            if let DocumentValue::String(str) = &members.get("c").unwrap().value {
                assert_eq!(str, &String::from("c"));
            } else {
                panic!("Expected String")
            }
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
        let val: &Schema = &STRING;
        assert_eq!(document_str.schema(), val);
    }

    #[test]
    fn document_deserializer_roundtrip() {
        // Test that DocumentSerializer -> DocumentDeserializer roundtrip works
        let original_value = "test_string";

        // Serialize to document
        let document: Document = original_value.into();

        // Deserialize back from document
        let deserialized: String = from_document(&document, &STRING).unwrap();

        assert_eq!(original_value, deserialized);
    }

    #[test]
    fn document_deserializer_integer() {
        let original_value = 42i32;

        let document: Document = original_value.into();
        let deserialized: i32 = from_document(&document, &INTEGER).unwrap();

        assert_eq!(original_value, deserialized);
    }

    #[test]
    fn document_deserializer_list_no_cloning() {
        // Test that list deserialization works without cloning the entire list
        let original_list = vec!["item1", "item2", "item3"];
        let document: Document = original_list.clone().into();

        // Deserialize back - this should use references to list elements, not clone them
        let deserialized: Vec<String> = from_document(&document, &LIST_DOCUMENT_SCHEMA).unwrap();

        assert_eq!(deserialized.len(), 3);
        assert_eq!(deserialized[0], "item1");
        assert_eq!(deserialized[1], "item2");
        assert_eq!(deserialized[2], "item3");
    }
}
