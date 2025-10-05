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
        ShapeType, TraitList, get_shape_type,
    },
    serde::{
        se::{ListSerializer, MapSerializer, Serialize, Serializer, StructSerializer},
        serializers::{Error, SerializeWithSchema},
    },
};

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
            _ => Err(Error::custom("Unsupported shape type")),
        }
    }
}

impl<T: SerializableShape> From<T> for Document {
    fn from(shape: T) -> Self {
        // TODO: should this be fallible?
        shape.serialize(DocumentParser).unwrap()
    }
}

impl Error for DocumentError {
    fn custom<T: Display>(msg: T) -> Self {
        DocumentError::CustomError(msg.to_string())
    }
}

struct DocumentParser;
// TODO: Should this have schema type validation?
impl Serializer for DocumentParser {
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
        let Some(me) = key_schema.as_member() else {
            return Err(DocumentError::DocumentConversion(
                "Expected `key` schema.".to_string(),
            ));
        };
        let val = value.serialize_with_schema(value_schema, DocumentParser)?;
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
        let val = value.serialize_with_schema(member_schema, DocumentParser)?;
        map.insert(me.name.clone(), val);
        Ok(())
    }

    fn end(self, schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(self)
    }
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
    fn number_document_values() {
        let x: &Schema = &STRING;
    }
}
