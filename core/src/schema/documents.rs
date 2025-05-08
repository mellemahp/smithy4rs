use std::collections::HashMap;
use std::time::Instant;
use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;

// use crate::schema::prelude;
// use crate::schema::shapes::{ShapeId, ShapeType};
// use crate::schema::Schema;
// use crate::serde::se::{Serialize, SerializableStruct, Serializer};
// use bigdecimal::BigDecimal;
// use bytebuffer::ByteBuffer;
// use num_bigint::BigInt;
// use std::collections::HashMap;
// use std::time::Instant;
use thiserror::Error;
use crate::schema::{Schema, ShapeId};

//
#[derive(Error, Debug, Default)]
pub enum DocumentError {
    #[error("Failed to convert document to type {0}")]
    DocumentSerialization(String),
    #[error("Failed to convert document to type {0}")]
    DocumentConversion(String),
    #[default]
    #[error("Whooopsie")]
    Default
}
//
// /// A Smithy document type, representing untyped data from the Smithy data model.
// ///
// /// *Note*: Document implementations are protocol specific
#[derive(Clone)]
pub enum DocumentValue<'doc> {
    Null,
    Number(NumberValue),
    Boolean(bool),
    Blob(ByteBuffer),
    String(String),
    Timestamp(Instant),
    Array(Vec<Document<'doc>>),
    Map(HashMap<String, Document<'doc>>),
}

/// Represents numbers in the smithy data model
///
/// Smithy numbers types include: byte, short, integer, long, float, double, bigInteger, bigDecimal.
///
/// *Note*: IntEnum shapes are represented as integers in the Smithy data model.
#[derive(Debug, Clone, PartialEq)]
pub enum NumberValue {
    Integer(NumberInteger),
    Float(NumberFloat),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberInteger {
    Byte(i8),
    Short(i16),
    Integer(i32),
    Long(i64),
    BigInt(BigInt),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberFloat {
    Float(f32),
    Double(f64),
    BigDecimal(BigDecimal),
}
//
// impl Serialize for Document<'_> {
//     fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
//         let schema = self.schema;
//         // TODO: Is there any way to avoid copy???
//         match schema.shape_type {
//             ShapeType::Blob => serializer.write_blob(schema, &self.try_into()?),
//             ShapeType::Boolean => serializer.write_boolean(schema, self.try_into()?),
//             ShapeType::String => serializer.write_string(schema, &self.try_into()?),
//             ShapeType::Timestamp => serializer.write_timestamp(schema, &self.try_into()?),
//             ShapeType::Byte => serializer.write_byte(schema, self.try_into()?),
//             ShapeType::Short => serializer.write_short(schema, self.try_into()?),
//             ShapeType::Integer => serializer.write_integer(schema, self.try_into()?),
//             ShapeType::Long => serializer.write_long(schema, self.try_into()?),
//             ShapeType::Float => serializer.write_integer(schema, self.try_into()?),
//             ShapeType::Double => serializer.write_double(schema, self.try_into()?),
//             ShapeType::BigInteger => serializer.write_big_integer(schema, &self.try_into()?),
//             ShapeType::BigDecimal => serializer.write_big_integer(schema, &self.try_into()?),
//             ShapeType::Document => serializer.write_document(schema, &self),
//             ShapeType::Enum => serializer.write_string(schema, &self.try_into()?),
//             ShapeType::IntEnum => serializer.write_integer(schema, self.try_into()?),
//             ShapeType::List => serializer.write_list(
//                 schema,
//                 &mut self.value.try_into_list()?.iter(),
//                 DocumentListConsumer,
//             ),
//             ShapeType::Map => todo!(),
//             ShapeType::Structure => serializer.write_struct(schema, self),
//             ShapeType::Union => serializer.write_struct(schema, self),
//             _ => Err(Default::default()),
//         }
//     }
// }
//
// struct DocumentListConsumer;
// impl ListItemConsumer<&Document<'_>> for DocumentListConsumer {
//     fn write_item<S: Serializer>(item: &Document<'_>, serializer: &mut S) -> Result<(), S::Error> {
//         match item.value {
//             DocumentValue::Null => serializer.write_null(item.schema),
//             _ => item.serialize(serializer),
//         }
//     }
// }
//
// impl<'doc> SerializableStruct for Document<'doc> {
//     fn schema(&self) -> &'doc Schema<'doc> {
//         self.schema
//     }
//
//     fn serialize_members<S: Serializer>(&self, _: &mut S) -> Result<(), S::Error> {
//         todo!()
//     }
// }
//
#[derive(Clone)]
pub struct Document<'doc> {
    pub schema: &'doc Schema<'doc>,
    pub(crate) value: DocumentValue<'doc>,
    // NOTE: It is expected that protocols set these!
    pub discriminator: Option<&'doc ShapeId>,
}
//
// impl Document<'_> {
//     pub fn size(&self) -> usize {
//         match self.value {
//             DocumentValue::Array(ref array) => array.len(),
//             DocumentValue::Map(ref map) => map.len(),
//             DocumentValue::Null => 0,
//             _ => 1,
//         }
//     }
//
//     pub fn value(&self) -> &DocumentValue<'_> {
//         &self.value
//     }
//
//     pub fn of(shape: impl Serialize) -> Result<Self, DocumentError> {
//         let mut parser = DocumentParser::new();
//         shape.serialize(&mut parser)?;
//         Ok(parser.result())
//     }
// }
//
// impl <'doc> Document<'doc> {
//     pub fn string_document(schema: &'doc Schema, value: &str) -> Self {
//         Document {
//             schema,
//             value: DocumentValue::String(value.to_string()),
//             discriminator: None,
//         }
//     }
// }
//
// // TODO: Could these be just normal TryFrom impls?
// impl<'doc> DocumentValue<'doc> {
//     fn try_into_list(&self) -> Result<&[Document<'doc>], DocumentError> {
//         if let &Self::Array(arr) = &self {
//             Ok(arr)
//         } else {
//             Err(DocumentError::DocumentConversion("list".to_string()))
//         }
//     }
//
//     #[allow(dead_code)]
//     fn try_into_map(&self) -> Result<&HashMap<String, Document<'doc>>, DocumentError> {
//         if let &Self::Map(map) = &self {
//             Ok(map)
//         } else {
//             Err(DocumentError::DocumentSerialization("map".to_string()))
//         }
//     }
// }
//
// // ====== INTO conversions =====
// // TODO: Macro-ify these?
// impl TryFrom<&Document<'_>> for ByteBuffer {
//     type Error = DocumentError;
//
//     fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
//         if let DocumentValue::Blob(b) = &value.value {
//             Ok(b.clone())
//         } else {
//             Err(DocumentError::DocumentSerialization("blob".to_string()))
//         }
//     }
// }
//
// impl TryFrom<&Document<'_>> for bool {
//     type Error = DocumentError;
//
//     fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
//         if let &DocumentValue::Boolean(b) = &value.value {
//             Ok(b)
//         } else {
//             Err(DocumentError::DocumentSerialization("boolean".to_string()))
//         }
//     }
// }
//
// impl TryFrom<&Document<'_>> for String {
//     type Error = DocumentError;
//
//     fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
//         if let DocumentValue::String(s) = &value.value {
//             Ok(s.clone())
//         } else {
//             Err(DocumentError::DocumentSerialization("string".to_string()))
//         }
//     }
// }
//
// impl TryFrom<&Document<'_>> for Instant {
//     type Error = DocumentError;
//
//     fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
//         if let &DocumentValue::Timestamp(ts) = &value.value {
//             Ok(ts)
//         } else {
//             Err(DocumentError::DocumentSerialization(
//                 "timestamp".to_string(),
//             ))
//         }
//     }
// }
//
// // TODO: Make Number conversions actually smart!
// impl TryFrom<&Document<'_>> for i8 {
//     type Error = DocumentError;
//
//     fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
//         match &value.value {
//             DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
//                 &NumberInteger::Byte(b) => Ok(b as i8),
//                 &NumberInteger::Short(s) => Ok(s as i8),
//                 &NumberInteger::Integer(i) => Ok(i as i8),
//                 &NumberInteger::Long(l) => Ok(l as i8),
//                 NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
//             },
//             _ => Err(DocumentError::DocumentSerialization("i8".to_string())),
//         }
//     }
// }
//
// impl TryFrom<&Document<'_>> for i16 {
//     type Error = DocumentError;
//
//     fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
//         match &value.value {
//             DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
//                 &NumberInteger::Byte(b) => Ok(b as i16),
//                 &NumberInteger::Short(s) => Ok(s),
//                 &NumberInteger::Integer(i) => Ok(i as i16),
//                 &NumberInteger::Long(l) => Ok(l as i16),
//                 NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
//             },
//             _ => Err(DocumentError::DocumentSerialization("i16".to_string())),
//         }
//     }
// }
//
// impl TryFrom<&Document<'_>> for i32 {
//     type Error = DocumentError;
//
//     fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
//         match &value.value {
//             DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
//                 &NumberInteger::Byte(b) => Ok(b as i32),
//                 &NumberInteger::Short(s) => Ok(s as i32),
//                 &NumberInteger::Integer(i) => Ok(i),
//                 &NumberInteger::Long(l) => Ok(l as i32),
//                 NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
//             },
//             _ => Err(DocumentError::DocumentSerialization("i32".to_string())),
//         }
//     }
// }
//
// impl TryFrom<&Document<'_>> for f32 {
//     type Error = DocumentError;
//
//     fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
//         match &value.value {
//             DocumentValue::Number(NumberValue::Float(nf)) => match nf {
//                 &NumberFloat::Float(f) => Ok(f),
//                 &NumberFloat::Double(d) => Ok(d as f32),
//                 NumberFloat::BigDecimal(_) => todo!(),
//             },
//             _ => Err(DocumentError::DocumentSerialization("f32".to_string())),
//         }
//     }
// }
//
// impl TryFrom<&Document<'_>> for f64 {
//     type Error = DocumentError;
//
//     fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
//         match &value.value {
//             DocumentValue::Number(NumberValue::Float(nf)) => match nf {
//                 &NumberFloat::Float(f) => Ok(f as f64),
//                 &NumberFloat::Double(d) => Ok(d),
//                 NumberFloat::BigDecimal(_) => todo!(),
//             },
//             _ => Err(DocumentError::DocumentSerialization("f64".to_string())),
//         }
//     }
// }
//
// // TODO: Maybe these could be made more generic?
// impl TryFrom<&Document<'_>> for i64 {
//     type Error = DocumentError;
//
//     fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
//         match &value.value {
//             DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
//                 &NumberInteger::Byte(b) => Ok(b as i64),
//                 &NumberInteger::Short(s) => Ok(s as i64),
//                 &NumberInteger::Integer(i) => Ok(i as i64),
//                 &NumberInteger::Long(l) => Ok(l),
//                 NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
//             },
//             _ => Err(DocumentError::DocumentSerialization("i64".to_string())),
//         }
//     }
// }
//
// impl TryFrom<&Document<'_>> for BigInt {
//     type Error = DocumentError;
//
//     fn try_from(_: &Document<'_>) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }
//
// impl TryFrom<&Document<'_>> for BigDecimal {
//     type Error = DocumentError;
//
//     fn try_from(_: &Document<'_>) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }
//
// // FROM impls
// impl From<i32> for Document<'_> {
//     fn from(value: i32) -> Self {
//         Document {
//             schema: &prelude::INTEGER,
//             value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Integer(value))),
//             discriminator: None,
//         }
//     }
// }
//
// impl From<&str> for Document<'_> {
//     fn from(value: &str) -> Self {
//         Document {
//             schema: &prelude::STRING,
//             value: DocumentValue::String(value.to_string()),
//             discriminator: None,
//         }
//     }
// }
//
// impl From<String> for Document<'_> {
//     fn from(value: String) -> Self {
//         Document {
//             schema: &prelude::STRING,
//             value: DocumentValue::String(value),
//             discriminator: None,
//         }
//     }
// }
//
// // TODO: Rest of these conversions!
//
// struct DocumentParser<'parser> {
//     result: Option<Document<'parser>>,
// }
// impl DocumentParser<'_> {
//     pub(super) fn new() -> Self {
//         Self { result: None }
//     }
//
//     pub(super) fn result<'a>(self) -> Document<'a> {
//         let Some(result) = self.result else {
//             unreachable!("Document parser should always have a result by the time this is called.");
//         };
//         // TODO: can this be avoided?
//         //result.clone()
//         todo!()
//     }
// }
//
// impl Serializer for DocumentParser<'_> {
//     type Error = DocumentError;
//
//     fn write_struct<T: SerializableStruct>(&mut self, schema: &Schema, structure: &T) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_map<K, V, C: MapEntryConsumer<K, V>>(&mut self, schema: &Schema, map_state: impl Iterator<Item=(K, V)> + ExactSizeIterator, consumer: C) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_map_entry<K, V, C: MapEntryConsumer<K, V>>(&mut self, schema: &Schema, key: K, value: V, consumer: &C) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_list<I, C: ListItemConsumer<I>>(&mut self, schema: &Schema, list_state: impl Iterator<Item=I> + ExactSizeIterator, consumer: C) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_byte(&mut self, schema: &Schema, value: i8) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_short(&mut self, schema: &Schema, value: i16) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_integer(&mut self, schema: &Schema, value: i32) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_long(&mut self, schema: &Schema, value: i64) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_float(&mut self, schema: &Schema, value: f32) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_double(&mut self, schema: &Schema, value: f64) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_big_integer(&mut self, schema: &Schema, value: &BigInt) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_big_decimal(&mut self, schema: &Schema, value: &BigDecimal) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_string(&mut self, schema: &Schema, value: &String) -> Result<(), Self::Error> {
//         // self.result = Some(Document::string_document(schema, value));
//         Ok(())
//     }
//
//     fn write_blob(&mut self, schema: &Schema, value: &ByteBuffer) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_timestamp(&mut self, schema: &Schema, value: &Instant) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_document(&mut self, schema: &Schema, value: &Document) -> Result<(), Self::Error> {
//         todo!()
//     }
//
//     fn write_null(&mut self, schema: &Schema) -> Result<(), Self::Error> {
//         todo!()
//     }
// }