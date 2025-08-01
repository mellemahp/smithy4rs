#![allow(dead_code, unused_imports, unused_variables)]

use crate::schema::{Document, DocumentError, DocumentValue, NumberFloat, NumberInteger, NumberValue, SchemaRef, SchemaShape, ShapeType, get_shape_type, Schema};
use crate::serde::se::{ListSerializer, MapSerializer, Serialize, Serializer, StructSerializer};
use crate::serde::serializers::SerializeWithSchema;
use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use indexmap::IndexMap;
use num_bigint::BigInt;
use std::time::Instant;

/// Marker Trait used to differentiate between generated shapes and Documents for
/// some blanket impelementations.
///
/// NOTE: In general you should not need to implement this yourself
pub trait SerializableShape: Serialize {}

impl SerializeWithSchema for Document<'_> {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
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
            ShapeType::Document => serializer.write_document(schema, &self),
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
            // TODO: Raise _some_ error? How to map it into the serializer error?
            _ => panic!("Unsupported shape type"),
        }
    }
}

// impl<T: SerializableShape> From<T> for Document {
//     fn from(value: T) -> Self {
//         let mut doc_parser = DocumentParser::new();
//         // TODO: Should this be fallible? I think it should always work?
//         value.serialize(&mut doc_parser).unwrap();
//         doc_parser.result().unwrap()
//     }
// }
//
// struct DocumentParser {
//     document: Option<Document>,
// }
// impl DocumentParser {
//     pub(super) fn new() -> Self {
//         DocumentParser { document: None }
//     }
//
//     pub fn set_document(&mut self, document: Document) {
//         self.document = Some(document);
//     }
//
//     pub(super) fn result(self) -> Result<Document, DocumentError> {
//         self.document.ok_or(DocumentError::DocumentSerialization(
//             "Serialization did not set document value".to_string(),
//         ))
//     }
// }
//
// impl<'a> Serializer for &'a mut DocumentParser {
//     type Error = DocumentError;
//     type Ok = ();
//     type SerializeList = InnerParser<'a>;
//     type SerializeMap = InnerParser<'a>;
//     type SerializeStruct = InnerParser<'a>;
//
//     // TODO: Use len
//     fn write_struct(
//         self,
//         schema: &Schema,
//         _: usize,
//     ) -> Result<Self::SerializeStruct, Self::Error> {
//         Ok(InnerParser {
//             parser: self,
//             document: Document {
//                 schema,
//                 value: DocumentValue::Map(IndexMap::new()),
//                 discriminator: Some(schema.id().clone()),
//             },
//         })
//     }
//
//     fn write_map(self, schema: &SchemaRef, len: usize) -> Result<Self::SerializeMap, Self::Error> {
//         Ok(InnerParser {
//             parser: self,
//             document: Document {
//                 schema: schema.clone(),
//                 value: DocumentValue::Map(IndexMap::with_capacity(len)),
//                 discriminator: Some(schema.id().clone()),
//             },
//         })
//     }
//
//     fn write_list(
//         self,
//         schema: &SchemaRef,
//         len: usize,
//     ) -> Result<Self::SerializeList, Self::Error> {
//         Ok(InnerParser {
//             parser: self,
//             document: Document {
//                 schema: schema.clone(),
//                 value: DocumentValue::List(Vec::with_capacity(len)),
//                 discriminator: Some(schema.id().clone()),
//             },
//         })
//     }
//
//     fn write_boolean(self, schema: &SchemaRef, value: bool) -> Result<Self::Ok, Self::Error> {
//         self.set_document(Document {
//             schema: schema.clone(),
//             value: DocumentValue::Boolean(value),
//             discriminator: Some(schema.id().clone()),
//         });
//         Ok(())
//     }
//
//     fn write_byte(self, schema: &SchemaRef, value: i8) -> Result<Self::Ok, Self::Error> {
//         self.set_document(Document {
//             schema: schema.clone(),
//             value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Byte(value))),
//             discriminator: Some(schema.id().clone()),
//         });
//         Ok(())
//     }
//
//     fn write_short(self, schema: &SchemaRef, value: i16) -> Result<Self::Ok, Self::Error> {
//         self.set_document(Document {
//             schema: schema.clone(),
//             value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Short(value))),
//             discriminator: Some(schema.id().clone()),
//         });
//         Ok(())
//     }
//
//     fn write_integer(self, schema: &SchemaRef, value: i32) -> Result<Self::Ok, Self::Error> {
//         self.set_document(Document {
//             schema: schema.clone(),
//             value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Integer(value))),
//             discriminator: Some(schema.id().clone()),
//         });
//         Ok(())
//     }
//
//     fn write_long(self, schema: &SchemaRef, value: i64) -> Result<Self::Ok, Self::Error> {
//         self.set_document(Document {
//             schema: schema.clone(),
//             value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Long(value))),
//             discriminator: Some(schema.id().clone()),
//         });
//         Ok(())
//     }
//
//     fn write_float(self, schema: &SchemaRef, value: f32) -> Result<Self::Ok, Self::Error> {
//         self.set_document(Document {
//             schema: schema.clone(),
//             value: DocumentValue::Number(NumberValue::Float(NumberFloat::Float(value))),
//             discriminator: Some(schema.id().clone()),
//         });
//         Ok(())
//     }
//
//     fn write_double(self, schema: &SchemaRef, value: f64) -> Result<Self::Ok, Self::Error> {
//         self.set_document(Document {
//             schema: schema.clone(),
//             value: DocumentValue::Number(NumberValue::Float(NumberFloat::Double(value))),
//             discriminator: Some(schema.id().clone()),
//         });
//         Ok(())
//     }
//
//     fn write_big_integer(
//         self,
//         schema: &SchemaRef,
//         value: &BigInt,
//     ) -> Result<Self::Ok, Self::Error> {
//         self.set_document(Document {
//             schema: schema.clone(),
//             value: DocumentValue::Number(NumberValue::Integer(NumberInteger::BigInt(
//                 value.clone(),
//             ))),
//             discriminator: Some(schema.id().clone()),
//         });
//         Ok(())
//     }
//
//     fn write_big_decimal(
//         self,
//         schema: &SchemaRef,
//         value: &BigDecimal,
//     ) -> Result<Self::Ok, Self::Error> {
//         self.set_document(Document {
//             schema: schema.clone(),
//             value: DocumentValue::Number(NumberValue::Float(NumberFloat::BigDecimal(
//                 value.clone(),
//             ))),
//             discriminator: Some(schema.id().clone()),
//         });
//         Ok(())
//     }
//
//     fn write_string(self, schema: &SchemaRef, value: &str) -> Result<Self::Ok, Self::Error> {
//         self.set_document(Document {
//             schema: schema.clone(),
//             value: DocumentValue::String(value.to_string()),
//             discriminator: Some(schema.id().clone()),
//         });
//         Ok(())
//     }
//
//     fn write_blob(self, schema: &SchemaRef, value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
//         self.set_document(Document {
//             schema: schema.clone(),
//             value: DocumentValue::Blob(value.clone()),
//             discriminator: Some(schema.id().clone()),
//         });
//         Ok(())
//     }
//
//     fn write_timestamp(self, schema: &SchemaRef, value: &Instant) -> Result<Self::Ok, Self::Error> {
//         self.set_document(Document {
//             schema: schema.clone(),
//             value: DocumentValue::Timestamp(value.clone()),
//             discriminator: Some(schema.id().clone()),
//         });
//         Ok(())
//     }
//
//     fn write_document(
//         self,
//         _schema: &SchemaRef,
//         value: &Document,
//     ) -> Result<Self::Ok, Self::Error> {
//         self.document = Some(value.clone());
//         Ok(())
//     }
//
//     fn write_null(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
//         todo!()
//     }
//
//     fn skip(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
//         todo!()
//     }
// }
//
// #[doc(hidden)]
// pub struct InnerParser<'a> {
//     parser: &'a mut DocumentParser,
//     document: Document,
// }
//
// impl ListSerializer for InnerParser<'_> {
//     type Error = DocumentError;
//     type Ok = ();
//
//     fn serialize_element<T>(
//         &mut self,
//         element_schema: &SchemaRef,
//         value: &T,
//     ) -> Result<Self::Ok, Self::Error>
//     where
//         T: ?Sized + SerializeWithSchema,
//     {
//         let element_parser = DocumentParser::new();
//         value.serialize_with_schema(element_schema, &mut *self.parser)?;
//         if let DocumentValue::List(list) = &mut self.document.value {
//             list.push(element_parser.result()?);
//             Ok(())
//         } else {
//             Err(DocumentError::DocumentSerialization(
//                 "expected a list".to_string(),
//             ))
//         }
//     }
//
//     fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
//         self.parser.set_document(self.document);
//         Ok(())
//     }
// }
//
// impl MapSerializer for InnerParser<'_> {
//     type Error = DocumentError;
//     type Ok = ();
//
//     fn serialize_entry<K, V>(
//         &mut self,
//         key_schema: &SchemaRef,
//         value_schema: &SchemaRef,
//         _key: &K,
//         value: &V,
//     ) -> Result<Self::Ok, Self::Error>
//     where
//         K: ?Sized + SerializeWithSchema,
//         V: ?Sized + SerializeWithSchema,
//     {
//         let Some(me) = key_schema.as_member() else {
//             return Err(DocumentError::DocumentSerialization(
//                 "Expected member schema!".to_string(),
//             ));
//         };
//         let value_serializer = DocumentParser::new();
//         value.serialize_with_schema(value_schema, &mut *self.parser)?;
//         let DocumentValue::Map(map) = &mut self.document.value else {
//             return Err(DocumentError::DocumentSerialization(
//                 "Expected member schema!".to_string(),
//             ));
//         };
//         map.insert(me.name.clone(), value_serializer.result()?);
//         Ok(())
//     }
//
//     fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
//         self.parser.set_document(self.document);
//         Ok(())
//     }
// }
//
// impl StructSerializer for InnerParser<'_> {
//     type Error = DocumentError;
//     type Ok = ();
//
//     fn serialize_member<T>(
//         &mut self,
//         member_schema: &SchemaRef,
//         value: &T,
//     ) -> Result<Self::Ok, Self::Error>
//     where
//         T: ?Sized + SerializeWithSchema,
//     {
//         let Some(me) = member_schema.as_member() else {
//             return Err(DocumentError::DocumentSerialization(
//                 "Expected member schema!".to_string(),
//             ));
//         };
//         let value_serializer = DocumentParser::new();
//         value.serialize_with_schema(member_schema, &mut *self.parser)?;
//         let DocumentValue::Map(map) = &mut self.document.value else {
//             return Err(DocumentError::DocumentSerialization(
//                 "Expected member schema!".to_string(),
//             ));
//         };
//         map.insert(me.name.clone(), value_serializer.result()?);
//         Ok(())
//     }
//
//     fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
//         self.parser.set_document(self.document);
//         Ok(())
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::prelude::*;
//     use crate::schema::{Schema, ShapeId};
//     use crate::{lazy_member_schema, lazy_schema, traits};
//     use std::sync::LazyLock;
//
//     lazy_schema!(
//         MAP_SCHEMA,
//         Schema::map_builder(ShapeId::from("com.example#Map"))
//             .put_member("key", &STRING, traits![])
//             .put_member("value", &STRING, traits![])
//             .build()
//     );
//     lazy_schema!(
//         LIST_SCHEMA,
//         Schema::list_builder(ShapeId::from("com.example#List"))
//             .put_member("member", &STRING, traits![])
//             .build()
//     );
//     lazy_schema!(
//         SCHEMA,
//         Schema::structure_builder(ShapeId::from("com.example#Shape"))
//             .put_member("a", &STRING, traits![])
//             .put_member("b", &STRING, traits![])
//             .put_member("c", &STRING, traits![])
//             .put_member("map", &MAP_SCHEMA, traits![])
//             .put_member("list", &LIST_SCHEMA, traits![])
//             .build()
//     );
//     lazy_member_schema!(MEMBER_A, SCHEMA, "a");
//     lazy_member_schema!(MEMBER_B, SCHEMA, "b");
//     lazy_member_schema!(MEMBER_C, SCHEMA, "c");
//     lazy_member_schema!(MEMBER_LIST, SCHEMA, "list");
//     lazy_member_schema!(MEMBER_MAP, SCHEMA, "map");
//
//     pub(crate) struct SerializeMe {
//         // #[schema(MEMBER_A)]
//         pub member_a: String,
//         // #[schema(MEMBER_B)]
//         pub member_b: String,
//         // #[schema(MEMBER_C)]
//         pub member_optional: Option<String>,
//         pub member_list: Vec<String>,
//         pub member_map: IndexMap<String, String>,
//     }
//     impl SerializableShape for SerializeMe {}
//
//     impl SchemaShape for SerializeMe {
//         fn schema(&self) -> &SchemaRef {
//             &SCHEMA
//         }
//     }
//
//     impl SerializeWithSchema for SerializeMe {
//         fn serialize_with_schema<S: Serializer>(
//             &self,
//             schema: &SchemaRef,
//             serializer: S,
//         ) -> Result<S::Ok, S::Error> {
//             let mut ser = serializer.write_struct(schema, 2)?;
//             ser.serialize_member(&MEMBER_A, &self.member_a)?;
//             ser.serialize_member(&MEMBER_B, &self.member_b)?;
//             ser.serialize_optional_member(&MEMBER_C, &self.member_optional)?;
//             ser.serialize_member(&MEMBER_LIST, &self.member_list)?;
//             ser.serialize_member(&MEMBER_MAP, &self.member_map)?;
//             ser.end(schema)
//         }
//     }
//
//     #[test]
//     fn struct_to_document() {
//         let mut map = IndexMap::new();
//         map.insert(String::from("a"), String::from("b"));
//         let list = vec!["a".to_string(), "b".to_string()];
//         let struct_to_convert = SerializeMe {
//             member_a: "a".to_string(),
//             member_b: "b".to_string(),
//             member_optional: Some("c".to_string()),
//             member_map: map,
//             member_list: list,
//         };
//         let document: Document = struct_to_convert.into();
//         assert_eq!(&document.discriminator.clone().unwrap(), SCHEMA.id());
//         if let DocumentValue::Map(members) = document.value {
//             assert!(members.contains_key("a"));
//             if let DocumentValue::String(str) = &members.get("a").unwrap().value {
//                 assert_eq!(str, &String::from("a"));
//             } else {
//                 panic!("Expected String")
//             }
//             assert!(members.contains_key("b"));
//             if let DocumentValue::String(str) = &members.get("b").unwrap().value {
//                 assert_eq!(str, &String::from("b"));
//             } else {
//                 panic!("Expected String")
//             }
//             assert!(members.contains_key("c"));
//             if let DocumentValue::String(str) = &members.get("c").unwrap().value {
//                 assert_eq!(str, &String::from("c"));
//             } else {
//                 panic!("Expected String")
//             }
//             assert!(members.contains_key("map"));
//             assert!(members.contains_key("list"));
//         } else {
//             panic!("Expected document");
//         }
//     }
//
//     #[test]
//     fn string_document_value() {
//         let document_str: Document = "MyStr".into();
//         let output_str = document_str.as_string().expect("string");
//         assert_eq!(output_str, &"MyStr".to_string());
//         assert_eq!(document_str.schema(), &*STRING);
//     }
//
//     #[test]
//     fn number_document_values() {
//         let x: &Schema = &STRING;
//     }
// }
