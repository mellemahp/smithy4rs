#![allow(dead_code)]

use crate::schema::{Schema, SchemaRef, ShapeId, ShapeType, prelude};
use crate::serde::se::{
    ListSerializer, MapSerializer, Serializer, SerializerResult, StructSerializer,
};
use crate::serde::serializers::Serialize;
use crate::serde::{FmtSerializer, SerializeShape};
use crate::{lazy_schema, traits};
use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use indexmap::IndexMap;
use num_bigint::BigInt;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::LazyLock;
use std::time::Instant;
use thiserror::Error;

/// Marker trait to distinguish documents from other [`SerializeShape`]'s
pub trait DynamicShape: Sized {}
impl DynamicShape for Document {}

impl SerializeShape for Document {
    fn schema(&self) -> &SchemaRef {
        &self.schema
    }
}

impl Serialize for Document {
    fn serialize<S: Serializer>(
        &self,
        schema: &SchemaRef,
        serializer: &mut S,
    ) -> SerializerResult<S::Error> {
        match get_shape_type(schema)? {
            ShapeType::Blob => {
                serializer.write_blob(schema, self.as_blob().ok_or(conversion_error("blob"))?)
            }
            ShapeType::Boolean => {
                serializer.write_boolean(schema, self.as_bool().ok_or(conversion_error("bool"))?)
            }
            ShapeType::String => {
                serializer.write_string(schema, self.as_string().ok_or(conversion_error("string"))?)
            }
            ShapeType::Timestamp => serializer.write_timestamp(
                schema,
                self.as_timestamp().ok_or(conversion_error("timestamp"))?,
            ),
            ShapeType::Byte => {
                serializer.write_byte(schema, self.as_byte().ok_or(conversion_error("byte"))?)
            }
            ShapeType::Short => {
                serializer.write_short(schema, self.as_short().ok_or(conversion_error("short"))?)
            }
            ShapeType::Integer => serializer.write_integer(
                schema,
                self.as_integer().ok_or(conversion_error("integer"))?,
            ),
            ShapeType::Long => {
                serializer.write_long(schema, self.as_long().ok_or(conversion_error("long"))?)
            }
            ShapeType::Float => {
                serializer.write_float(schema, self.as_float().ok_or(conversion_error("float"))?)
            }
            ShapeType::Double => {
                serializer.write_double(schema, self.as_double().ok_or(conversion_error("double"))?)
            }
            ShapeType::BigInteger => serializer.write_big_integer(
                schema,
                &self
                    .as_big_integer()
                    .ok_or(conversion_error("big integer"))?,
            ),
            ShapeType::BigDecimal => serializer.write_big_decimal(
                schema,
                &self
                    .as_big_decimal()
                    .ok_or(conversion_error("big decimal"))?,
            ),
            ShapeType::Document => serializer.write_document(schema, &self),
            // TODO: These wont work RN. Need to implement.
            ShapeType::Enum => {
                serializer.write_string(schema, self.as_string().ok_or(conversion_error("enum"))?)
            }
            ShapeType::IntEnum => serializer.write_integer(
                schema,
                self.as_integer().ok_or(conversion_error("intEnum"))?,
            ),
            ShapeType::List => self
                .as_list()
                .ok_or(conversion_error("list"))?
                .serialize(schema, serializer),
            ShapeType::Map => self
                .as_map()
                .ok_or(conversion_error("map"))?
                .serialize(schema, serializer),
            ShapeType::Structure | ShapeType::Union => {
                let document_map = self.as_map().ok_or(conversion_error("map"))?;
                let mut struct_serializer = serializer.write_struct(schema, self.size())?;
                for (key, value) in document_map {
                    // TODO should this panic on unknown members? Probably fine to just ignore
                    if let Some(member_schema) = schema.get_member(key) {
                        struct_serializer.serialize_member(&*member_schema, value)?;
                    }
                }
                struct_serializer.end(schema)
            }
            // TODO: Raise _some_ error?
            _ => panic!("Unsupported shape type"),
        }
    }
}

fn get_shape_type(schema: &SchemaRef) -> Result<&ShapeType, Box<dyn Error>> {
    let shape_type = schema.shape_type();
    if shape_type == &ShapeType::Member {
        let Some(member) = schema.as_member() else {
            // TODO: Real error
            return Err(conversion_error(
                "Expected memberSchema for member shape type",
            ));
        };
        Ok(member.target.shape_type())
    } else {
        Ok(shape_type)
    }
}

fn conversion_error(expected: &'static str) -> Box<dyn Error> {
    Box::new(DocumentError::DocumentConversion(expected.to_string())) as Box<dyn Error>
}

pub trait AsDocument: SerializeShape {
    fn as_document(&self) -> Result<Document, DocumentError>
    where
        Self: Sized,
    {
        let mut doc_parser = DocumentParser::new();
        self.serialize_shape(&mut doc_parser)?;
        Ok(doc_parser.result()?)
    }
}
impl<T: SerializeShape> AsDocument for T {}

struct DocumentParser {
    document: Option<Document>,
}
impl DocumentParser {
    pub(super) fn new() -> Self {
        DocumentParser { document: None }
    }

    pub fn set_document(&mut self, document: Document) {
        self.document = Some(document);
    }

    pub(super) fn result(self) -> Result<Document, DocumentError> {
        self.document.ok_or(DocumentError::DocumentSerialization(
            "Serialization did not set document value".to_string(),
        ))
    }
}

impl Serializer for DocumentParser {
    type Error = DocumentError;
    type SerializeList<'l>
        = DocumentListParser<'l>
    where
        Self: 'l;
    type SerializeMap<'m>
        = DocumentMapParser<'m>
    where
        Self: 'm;
    type SerializeStruct<'s>
        = DocumentMapParser<'s>
    where
        Self: 's;

    // TODO: Use len
    fn write_struct(
        &mut self,
        schema: &SchemaRef,
        _: usize,
    ) -> Result<Self::SerializeStruct<'_>, Self::Error> {
        Ok(DocumentMapParser::new(self, schema))
    }

    // TODO: Use len
    fn write_map(
        &mut self,
        schema: &SchemaRef,
        _: usize,
    ) -> Result<Self::SerializeMap<'_>, Self::Error> {
        Ok(DocumentMapParser::new(self, schema))
    }

    fn write_list(
        &mut self,
        schema: &SchemaRef,
        len: usize,
    ) -> Result<Self::SerializeList<'_>, Self::Error> {
        Ok(DocumentListParser::new(self, schema, len))
    }

    fn write_boolean(&mut self, schema: &SchemaRef, value: bool) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Boolean(value),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_byte(&mut self, schema: &SchemaRef, value: i8) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Byte(value))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_short(&mut self, schema: &SchemaRef, value: i16) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Short(value))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_integer(&mut self, schema: &SchemaRef, value: i32) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Integer(value))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_long(&mut self, schema: &SchemaRef, value: i64) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Long(value))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_float(&mut self, schema: &SchemaRef, value: f32) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::Float(value))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_double(&mut self, schema: &SchemaRef, value: f64) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::Double(value))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_big_integer(
        &mut self,
        schema: &SchemaRef,
        value: &BigInt,
    ) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::BigInt(
                value.clone(),
            ))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_big_decimal(
        &mut self,
        schema: &SchemaRef,
        value: &BigDecimal,
    ) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::BigDecimal(
                value.clone(),
            ))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_string(
        &mut self,
        schema: &SchemaRef,
        value: &String,
    ) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::String(value.clone()),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_blob(
        &mut self,
        schema: &SchemaRef,
        value: &ByteBuffer,
    ) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Blob(value.clone()),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_timestamp(
        &mut self,
        schema: &SchemaRef,
        value: &Instant,
    ) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Timestamp(value.clone()),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_document(
        &mut self,
        _schema: &SchemaRef,
        _value: &Document,
    ) -> SerializerResult<Self::Error> {
        todo!()
    }

    fn write_null(&mut self, _schema: &SchemaRef) -> SerializerResult<Self::Error> {
        todo!()
    }

    fn skip(&mut self, _schema: &SchemaRef) -> SerializerResult<Self::Error> {
        todo!()
    }
}

struct DocumentListParser<'lp> {
    parent: &'lp mut DocumentParser,
    document: Document,
}
impl<'lp> DocumentListParser<'lp> {
    pub(super) fn new(parent: &'lp mut DocumentParser, schema: &SchemaRef, len: usize) -> Self {
        DocumentListParser {
            parent,
            document: Document {
                schema: schema.clone(),
                value: DocumentValue::List(Vec::with_capacity(len)),
                discriminator: Some(schema.id().clone()),
            },
        }
    }
}
impl ListSerializer for DocumentListParser<'_> {
    type Error = DocumentError;

    fn serialize_element<T>(
        &mut self,
        element_schema: &SchemaRef,
        value: &T,
    ) -> SerializerResult<Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut element_parser = DocumentParser::new();
        value.serialize(element_schema, &mut element_parser)?;
        if let DocumentValue::List(list) = &mut self.document.value {
            list.push(element_parser.result()?);
            Ok(())
        } else {
            Err(DocumentError::DocumentSerialization(
                "expected a list".to_string(),
            ))
        }
    }

    fn end(self, _: &SchemaRef) -> SerializerResult<Self::Error> {
        self.parent.set_document(self.document);
        Ok(())
    }
}

struct DocumentMapParser<'mp> {
    parent: &'mp mut DocumentParser,
    document: Document,
}
impl<'mp> DocumentMapParser<'mp> {
    pub(super) fn new(parent: &'mp mut DocumentParser, schema: &SchemaRef) -> Self {
        DocumentMapParser {
            parent,
            document: Document {
                schema: schema.clone(),
                value: DocumentValue::Map(IndexMap::new()),
                discriminator: Some(schema.id().clone()),
            },
        }
    }
}

impl MapSerializer for DocumentMapParser<'_> {
    type Error = DocumentError;

    fn serialize_entry<K, V>(
        &mut self,
        key_schema: &SchemaRef,
        value_schema: &SchemaRef,
        _key: &K,
        value: &V,
    ) -> SerializerResult<Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        let Some(me) = key_schema.as_member() else {
            return Err(DocumentError::DocumentSerialization(
                "Expected member schema!".to_string(),
            ));
        };
        let mut value_serializer = DocumentParser::new();
        value.serialize(value_schema, &mut value_serializer)?;
        let DocumentValue::Map(map) = &mut self.document.value else {
            return Err(DocumentError::DocumentSerialization(
                "Expected member schema!".to_string(),
            ));
        };
        map.insert(me.name.clone(), value_serializer.result()?);
        Ok(())
    }

    fn end(self, _: &SchemaRef) -> SerializerResult<Self::Error> {
        self.parent.set_document(self.document);
        Ok(())
    }
}

impl StructSerializer for DocumentMapParser<'_> {
    type Error = DocumentError;

    fn serialize_member<T>(
        &mut self,
        member_schema: &SchemaRef,
        value: &T,
    ) -> SerializerResult<Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let Some(me) = member_schema.as_member() else {
            return Err(DocumentError::DocumentSerialization(
                "Expected member schema!".to_string(),
            ));
        };
        let mut value_serializer = DocumentParser::new();
        value.serialize(member_schema, &mut value_serializer)?;
        let DocumentValue::Map(map) = &mut self.document.value else {
            return Err(DocumentError::DocumentSerialization(
                "Expected member schema!".to_string(),
            ));
        };
        map.insert(me.name.clone(), value_serializer.result()?);
        Ok(())
    }

    fn end(self, _: &SchemaRef) -> SerializerResult<Self::Error> {
        self.parent.set_document(self.document);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lazy_member_schema;

    lazy_schema!(
        MAP_SCHEMA,
        Schema::map_builder(ShapeId::from("com.example#Map"))
            .put_member("key", &prelude::STRING, traits![])
            .put_member("value", &prelude::STRING, traits![])
            .build()
    );
    lazy_schema!(
        LIST_SCHEMA,
        Schema::list_builder(ShapeId::from("com.example#List"))
            .put_member("member", &prelude::STRING, traits![])
            .build()
    );
    lazy_schema!(
        SCHEMA,
        Schema::structure_builder(ShapeId::from("com.example#Shape"))
            .put_member("a", &prelude::STRING, traits![])
            .put_member("b", &prelude::STRING, traits![])
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
        fn schema(&self) -> &SchemaRef {
            &SCHEMA
        }
    }

    impl Serialize for SerializeMe {
        fn serialize<S: Serializer>(
            &self,
            schema: &SchemaRef,
            serializer: &mut S,
        ) -> SerializerResult<S::Error> {
            let mut ser = serializer.write_struct(schema, 2)?;
            ser.serialize_member(&MEMBER_A, &self.member_a)?;
            ser.serialize_member(&MEMBER_B, &self.member_b)?;
            ser.serialize_optional_member(&MEMBER_C, &self.member_optional)?;
            ser.serialize_member(&MEMBER_LIST, &self.member_list)?;
            ser.serialize_member(&MEMBER_MAP, &self.member_map)?;
            ser.end(schema)
        }
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
        let document = struct_to_convert.as_document().expect("Expected document");
        assert_eq!(
            document.discriminator.clone().unwrap().id,
            *struct_to_convert.schema().id().id
        );
        assert_eq!(document.schema(), struct_to_convert.schema());
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
        let document_str = Document::of("MyStr");
        let output_str = document_str.as_string().expect("string");
        assert_eq!(output_str, &"MyStr".to_string());
        //assert_eq!(document_str.schema, &*prelude::STRING);
        let document_string = Document::of("MyString".to_string());
        let output_str = document_string.as_string().expect("string");
        assert_eq!(output_str, &"MyString".to_string());
        //assert_eq!(document_string.schema, &*prelude::STRING);
    }

    #[test]
    fn number_document_values() {}
}
