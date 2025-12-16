#![allow(dead_code, unused_imports, unused_variables)]

use std::{collections::HashMap, fmt::Display, marker::PhantomData};

use indexmap::IndexMap;
use thiserror::Error;

use crate::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    prelude::{BIG_DECIMAL, BIG_INTEGER, BOOLEAN, BYTE},
    schema::{
        Document, DocumentError, DocumentValue, LIST_DOCUMENT_SCHEMA, MAP_DOCUMENT_SCHEMA,
        NumberFloat, NumberInteger, NumberValue, Schema, SchemaRef, SchemaShape, ShapeId,
        ShapeType, StaticSchemaShape, TraitList, get_shape_type,
    },
    serde::{
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

    fn skip(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        // When skipping (e.g., for None values), return a null document
        Ok(Document {
            schema: schema.clone(),
            value: DocumentValue::Null,
            discriminator: Some(schema.id().clone()),
        })
    }
}

impl ListSerializer for Document {
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
        let el = value.serialize_with_schema(element_schema, DocumentParser)?;
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
        // Serialize the key to get its string representation
        let key_doc = key.serialize_with_schema(key_schema, DocumentParser)?;
        let key_str = key_doc
            .as_string()
            .ok_or_else(|| {
                DocumentError::DocumentConversion("Map key must be a string".to_string())
            })?
            .clone();

        let val = value.serialize_with_schema(value_schema, DocumentParser)?;
        map.insert(key_str, val);
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
        let val = value.serialize_with_schema(member_schema, DocumentParser)?;
        map.insert(me.name.clone(), val);
        Ok(())
    }

    fn end(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(self)
    }
}

// ============================================================================
// Deserialization
// ============================================================================

use crate::serde::de::{DeserializeWithSchema, Deserializer};

/// A deserializer that reads from a `Document`.
pub struct DocumentDeserializer<'a> {
    document: &'a Document,
}

impl<'a> DocumentDeserializer<'a> {
    pub fn new(document: &'a Document) -> Self {
        Self { document }
    }
}

impl<'de> Deserializer<'de> for DocumentDeserializer<'de> {
    type Error = DocumentError;

    fn read_bool(&mut self, schema: &SchemaRef) -> Result<bool, Self::Error> {
        self.document.as_bool().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected boolean document".to_string())
        })
    }

    fn read_byte(&mut self, schema: &SchemaRef) -> Result<i8, Self::Error> {
        self.document
            .as_byte()
            .ok_or_else(|| DocumentError::DocumentConversion("Expected byte document".to_string()))
    }

    fn read_short(&mut self, schema: &SchemaRef) -> Result<i16, Self::Error> {
        self.document
            .as_short()
            .ok_or_else(|| DocumentError::DocumentConversion("Expected short document".to_string()))
    }

    fn read_integer(&mut self, schema: &SchemaRef) -> Result<i32, Self::Error> {
        self.document.as_integer().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected integer document".to_string())
        })
    }

    fn read_long(&mut self, schema: &SchemaRef) -> Result<i64, Self::Error> {
        self.document
            .as_long()
            .ok_or_else(|| DocumentError::DocumentConversion("Expected long document".to_string()))
    }

    fn read_float(&mut self, schema: &SchemaRef) -> Result<f32, Self::Error> {
        self.document
            .as_float()
            .ok_or_else(|| DocumentError::DocumentConversion("Expected float document".to_string()))
    }

    fn read_double(&mut self, schema: &SchemaRef) -> Result<f64, Self::Error> {
        self.document.as_double().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected double document".to_string())
        })
    }

    fn read_big_integer(&mut self, schema: &SchemaRef) -> Result<BigInt, Self::Error> {
        self.document.as_big_integer().cloned().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected big integer document".to_string())
        })
    }

    fn read_big_decimal(&mut self, schema: &SchemaRef) -> Result<BigDecimal, Self::Error> {
        self.document.as_big_decimal().cloned().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected big decimal document".to_string())
        })
    }

    fn read_string(&mut self, schema: &SchemaRef) -> Result<String, Self::Error> {
        self.document.as_string().cloned().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected string document".to_string())
        })
    }

    fn read_blob(&mut self, schema: &SchemaRef) -> Result<ByteBuffer, Self::Error> {
        self.document
            .as_blob()
            .cloned()
            .ok_or_else(|| DocumentError::DocumentConversion("Expected blob document".to_string()))
    }

    fn read_timestamp(&mut self, schema: &SchemaRef) -> Result<Instant, Self::Error> {
        self.document.as_timestamp().copied().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected timestamp document".to_string())
        })
    }

    fn read_document(&mut self, schema: &SchemaRef) -> Result<Document, Self::Error> {
        Ok(self.document.clone())
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
        let map = self.document.as_map().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected map document for struct".to_string())
        })?;

        // Iterate through all members in the schema
        for (member_name, member_schema) in schema.members() {
            // Look up the field in the document map
            if let Some(field_doc) = map.get(member_name) {
                let mut field_deser = DocumentDeserializer::new(field_doc);
                builder = consumer(builder, member_schema, &mut field_deser)?;
            }
            // If field is missing, consumer won't be called (handles optional fields)
            // TODO(unknown members): consume unknown member?
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
        let list = self.document.as_list().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected list document".to_string())
        })?;

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
        let map = self.document.as_map().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected map document".to_string())
        })?;

        for (key_str, value_doc) in map {
            // Key is already a String, create deserializer for the value
            let mut value_deser = DocumentDeserializer::new(value_doc);
            consumer(state, key_str.clone(), &mut value_deser)?;
        }

        Ok(())
    }

    fn is_null(&mut self) -> bool {
        matches!(self.document.value, DocumentValue::Null)
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

impl<'de> DeserializeWithSchema<'de> for Document {
    fn deserialize_with_schema<D>(
        schema: &SchemaRef,
        deserializer: &mut D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use crate::serde::de::Error;

        // Dispatch based on shape type, similar to SerializeWithSchema implementation
        match get_shape_type(schema).map_err(Error::custom)? {
            ShapeType::Boolean => {
                let value = deserializer.read_bool(schema)?;
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
                    value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Byte(value))),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Short => {
                let value = deserializer.read_short(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Short(value))),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Integer | ShapeType::IntEnum => {
                let value = deserializer.read_integer(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Integer(
                        value,
                    ))),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Long => {
                let value = deserializer.read_long(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Long(value))),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Float => {
                let value = deserializer.read_float(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Number(NumberValue::Float(NumberFloat::Float(value))),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Double => {
                let value = deserializer.read_double(schema)?;
                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Number(NumberValue::Float(NumberFloat::Double(value))),
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
            ShapeType::String | ShapeType::Enum => {
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
            ShapeType::Document => deserializer.read_document(schema).map_err(Error::custom),
            ShapeType::List => {
                let mut list = Vec::new();
                let member_schema = schema
                    .get_member("member")
                    .ok_or_else(|| Error::custom("List schema missing member"))?;

                deserializer.read_list(schema, &mut list, |list, _elem_schema, de| {
                    let elem_doc = Document::deserialize_with_schema(member_schema, de)?;
                    list.push(elem_doc);
                    Ok(())
                })?;

                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::List(list),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Map => {
                let mut map = IndexMap::new();
                let value_schema = schema
                    .get_member("value")
                    .ok_or_else(|| Error::custom("Map schema missing value"))?;

                deserializer.read_map(schema, &mut map, |map, key, de| {
                    // Key is already a String, deserialize the value
                    let value_doc = Document::deserialize_with_schema(value_schema, de)?;
                    map.insert(key, value_doc);
                    Ok(())
                })?;

                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Map(map),
                    discriminator: Some(schema.id().clone()),
                })
            }
            ShapeType::Structure | ShapeType::Union => {
                let map = IndexMap::new();
                let discriminator = schema.id().clone();

                let map = deserializer.read_struct(schema, map, |mut map, member_schema, de| {
                    let member = member_schema
                        .as_member()
                        .ok_or_else(|| Error::custom("Expected member schema"))?;
                    let member_doc = Document::deserialize_with_schema(member_schema, de)?;
                    map.insert(member.name.clone(), member_doc);
                    Ok(map)
                })?;

                Ok(Document {
                    schema: schema.clone(),
                    value: DocumentValue::Map(map),
                    discriminator: Some(discriminator),
                })
            }
            _ => Err(Error::custom("Unsupported shape type for deserialization")),
        }
    }
}

// TODO(test): overhaul these to use test shapes
#[cfg(test)]
mod tests {
    use std::{str::FromStr, sync::LazyLock};

    use smithy4rs_core_derive::SmithyStruct;

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

    #[derive(SmithyStruct)]
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
    fn number_document_values() {
        let x: &Schema = &STRING;
    }

    // Roundtrip tests: value -> serialize to Document -> deserialize back to value

    #[test]
    fn roundtrip_bool() {
        let original = true;
        let doc = original
            .serialize_with_schema(&BOOLEAN, DocumentParser)
            .unwrap();
        let mut deser = DocumentDeserializer::new(&doc);
        let result = bool::deserialize_with_schema(&BOOLEAN, &mut deser).unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn roundtrip_string() {
        let original = "hello world".to_string();
        let doc = original
            .serialize_with_schema(&STRING, DocumentParser)
            .unwrap();
        let mut deser = DocumentDeserializer::new(&doc);
        let result = String::deserialize_with_schema(&STRING, &mut deser).unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn roundtrip_integers() {
        // Byte
        let original_byte: i8 = 127;
        let doc = original_byte
            .serialize_with_schema(&BYTE, DocumentParser)
            .unwrap();
        let mut deser = DocumentDeserializer::new(&doc);
        let result = i8::deserialize_with_schema(&BYTE, &mut deser).unwrap();
        assert_eq!(original_byte, result);

        // Short
        let original_short: i16 = 32000;
        let doc = original_short
            .serialize_with_schema(&SHORT, DocumentParser)
            .unwrap();
        let mut deser = DocumentDeserializer::new(&doc);
        let result = i16::deserialize_with_schema(&SHORT, &mut deser).unwrap();
        assert_eq!(original_short, result);

        // Integer
        let original_int: i32 = 123456;
        let doc = original_int
            .serialize_with_schema(&INTEGER, DocumentParser)
            .unwrap();
        let mut deser = DocumentDeserializer::new(&doc);
        let result = i32::deserialize_with_schema(&INTEGER, &mut deser).unwrap();
        assert_eq!(original_int, result);

        // Long
        let original_long: i64 = 9876543210i64;
        let doc = original_long
            .serialize_with_schema(&LONG, DocumentParser)
            .unwrap();
        let mut deser = DocumentDeserializer::new(&doc);
        let result = i64::deserialize_with_schema(&LONG, &mut deser).unwrap();
        assert_eq!(original_long, result);
    }

    #[test]
    fn roundtrip_floats() {
        // Float
        let original_float: f32 = 1.2345;
        let doc = original_float
            .serialize_with_schema(&FLOAT, DocumentParser)
            .unwrap();
        let mut deser = DocumentDeserializer::new(&doc);
        let result = f32::deserialize_with_schema(&FLOAT, &mut deser).unwrap();
        assert_eq!(original_float, result);

        // Double
        let original_double: f64 = 1.23456789;
        let doc = original_double
            .serialize_with_schema(&DOUBLE, DocumentParser)
            .unwrap();
        let mut deser = DocumentDeserializer::new(&doc);
        let result = f64::deserialize_with_schema(&DOUBLE, &mut deser).unwrap();
        assert_eq!(original_double, result);
    }

    #[test]
    #[ignore = "BigDecimal/BigInteger serialization not yet implemented"]
    fn roundtrip_big_numbers() {
        // BigInteger
        let original_big_int = BigInt::from(123456789);
        let doc = original_big_int
            .serialize_with_schema(&BIG_INTEGER, DocumentParser)
            .unwrap();
        let mut deser = DocumentDeserializer::new(&doc);
        let result = BigInt::deserialize_with_schema(&BIG_INTEGER, &mut deser).unwrap();
        assert_eq!(original_big_int, result);

        // BigDecimal
        let original_big_dec = BigDecimal::from_str("123.456").unwrap();
        let doc = original_big_dec
            .serialize_with_schema(&BIG_DECIMAL, DocumentParser)
            .unwrap();
        let mut deser = DocumentDeserializer::new(&doc);
        let result = BigDecimal::deserialize_with_schema(&BIG_DECIMAL, &mut deser).unwrap();
        assert_eq!(original_big_dec, result);
    }

    #[test]
    fn roundtrip_list() {
        let original: Vec<String> = vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ];
        let doc = original
            .serialize_with_schema(&LIST_SCHEMA, DocumentParser)
            .unwrap();
        let mut deser = DocumentDeserializer::new(&doc);
        let result = Vec::<String>::deserialize_with_schema(&LIST_SCHEMA, &mut deser).unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn roundtrip_map() {
        let mut original = IndexMap::new();
        original.insert("key1".to_string(), "value1".to_string());
        original.insert("key2".to_string(), "value2".to_string());
        original.insert("key3".to_string(), "value3".to_string());

        let doc = original
            .serialize_with_schema(&MAP_SCHEMA, DocumentParser)
            .unwrap();
        let mut deser = DocumentDeserializer::new(&doc);
        let result =
            IndexMap::<String, String>::deserialize_with_schema(&MAP_SCHEMA, &mut deser).unwrap();
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
        let member_a_doc = doc_map.get("a").expect("Should have member a");
        let mut deser_a = DocumentDeserializer::new(member_a_doc);
        let a_value = String::deserialize_with_schema(&STRING, &mut deser_a).unwrap();
        assert_eq!(a_value, "value_a");

        // Check member_b
        let member_b_doc = doc_map.get("b").expect("Should have member b");
        let mut deser_b = DocumentDeserializer::new(member_b_doc);
        let b_value = String::deserialize_with_schema(&STRING, &mut deser_b).unwrap();
        assert_eq!(b_value, "value_b");

        // Check member_optional
        let member_c_doc = doc_map.get("c").expect("Should have member c");
        let mut deser_c = DocumentDeserializer::new(member_c_doc);
        let c_value = String::deserialize_with_schema(&STRING, &mut deser_c).unwrap();
        assert_eq!(c_value, "value_c");

        // Check list
        let list_doc = doc_map.get("list").expect("Should have list");
        let mut deser_list = DocumentDeserializer::new(list_doc);
        let list_value =
            Vec::<String>::deserialize_with_schema(&LIST_SCHEMA, &mut deser_list).unwrap();
        assert_eq!(list_value, original_list);

        // Check map
        let map_doc = doc_map.get("map").expect("Should have map");
        let mut deser_map = DocumentDeserializer::new(map_doc);
        let map_value =
            IndexMap::<String, String>::deserialize_with_schema(&MAP_SCHEMA, &mut deser_map)
                .unwrap();
        assert_eq!(map_value, original_map);
    }

    #[test]
    fn roundtrip_option() {
        // Some value
        let original_some: Option<String> = Some("test".to_string());
        let doc = original_some
            .serialize_with_schema(&STRING, DocumentParser)
            .unwrap();
        let mut deser = DocumentDeserializer::new(&doc);
        let result = Option::<String>::deserialize_with_schema(&STRING, &mut deser).unwrap();
        assert_eq!(original_some, result);

        // None value
        let original_none: Option<String> = None;
        let doc = original_none
            .serialize_with_schema(&STRING, DocumentParser)
            .unwrap();
        let mut deser = DocumentDeserializer::new(&doc);
        let result = Option::<String>::deserialize_with_schema(&STRING, &mut deser).unwrap();
        assert_eq!(original_none, result);
    }
}
