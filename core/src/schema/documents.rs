#![allow(dead_code)]

use std::error::Error;
use std::sync::LazyLock;
use std::time::Instant;
use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use indexmap::IndexMap;
use num_bigint::BigInt;
use thiserror::Error;
use crate::{lazy_schema, traits};
use crate::schema::{prelude, Ref, Schema, ShapeId, ShapeType};
use crate::serde::se::{Serializer, StructSerializer};
use crate::serde::serializers::Serialize;
use crate::serde::SerializeShape;

#[derive(Clone)]
pub struct Document<'doc> {
    pub schema: &'doc Schema<'doc>,
    pub value: DocumentValue<'doc>,
    /// A shape ID for a typed document.
    ///
    /// The discriminator is primarily used to implement polymorphism using documents in deserialization.
    ///
    /// *Impl note*: It is expected that protocols set the discriminator on deserialization if applicable
    pub discriminator: Option<ShapeId>,
}

/// A Smithy document type, representing untyped data from the Smithy data model.
#[derive(Clone)]
pub enum DocumentValue<'doc> {
    Null,
    Number(NumberValue),
    Boolean(bool),
    Blob(ByteBuffer),
    String(String),
    Timestamp(Instant),
    Array(Vec<Document<'doc>>),
    Map(IndexMap<String, Document<'doc>>),
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

impl SerializeShape for Document<'_> {
    fn schema(&self) -> &Schema {
        self.schema
    }
}

impl Serialize for Document<'_> {
    fn serialize<S: Serializer>(&self, schema: &Schema, serializer: &mut S) -> Result<S::Ok, S::Error> {
        match schema.shape_type() {
            ShapeType::Blob => serializer.write_blob(schema, &self.try_into().map_err(|err| Box::new(err) as Box<dyn Error>)?),
            ShapeType::Boolean => serializer.write_boolean(schema, self.try_into().map_err(|err| Box::new(err) as Box<dyn Error>)?),
            ShapeType::String => serializer.write_string(schema, &self.try_into().map_err(|err| Box::new(err) as Box<dyn Error>)?),
            ShapeType::Timestamp => serializer.write_timestamp(schema, &self.try_into().map_err(|err| Box::new(err) as Box<dyn Error>)?),
            ShapeType::Byte => serializer.write_byte(schema, self.try_into().map_err(|err| Box::new(err) as Box<dyn Error>)?),
            ShapeType::Short => serializer.write_short(schema, self.try_into().map_err(|err| Box::new(err) as Box<dyn Error>)?),
            ShapeType::Integer => serializer.write_integer(schema, self.try_into().map_err(|err| Box::new(err) as Box<dyn Error>)?),
            ShapeType::Long => serializer.write_long(schema, self.try_into().map_err(|err| Box::new(err) as Box<dyn Error>)?),
            ShapeType::Float => serializer.write_integer(schema, self.try_into().map_err(|err| Box::new(err) as Box<dyn Error>)?),
            ShapeType::Double => serializer.write_double(schema, self.try_into().map_err(|err| Box::new(err) as Box<dyn Error>)?),
            ShapeType::BigInteger => serializer.write_big_integer(schema, &self.try_into().map_err(|err| Box::new(err) as Box<dyn Error>)?),
            ShapeType::BigDecimal => serializer.write_big_integer(schema, &self.try_into().map_err(|err| Box::new(err) as Box<dyn Error>)?),
            ShapeType::Document => serializer.write_document(schema, &self),
            ShapeType::Enum => serializer.write_string(schema, &self.try_into().map_err(|err| Box::new(err) as Box<dyn Error>)?),
            ShapeType::IntEnum => serializer.write_integer(schema, self.try_into().map_err(|err| Box::new(err) as Box<dyn Error>)?),
            ShapeType::List => {
                if let DocumentValue::Array(document_list) = &self.value {
                    document_list.serialize(schema, serializer)
                } else {
                    Err(Box::new(DocumentError::DocumentSerialization("Could not serlialize document as list".to_string())) as Box<dyn Error>)?
                }
            },
            ShapeType::Map => {
                if let DocumentValue::Map(document_map) = &self.value {
                    document_map.serialize(schema, serializer)
                } else {
                    Err(Box::new(DocumentError::DocumentSerialization("Could not serialize document as map".to_string())) as Box<dyn Error>)?
                }
            },
            ShapeType::Structure | ShapeType::Union => {
                if let DocumentValue::Map(document_map) = &self.value {
                    let mut struct_serializer = serializer.write_struct(schema, self.size())?;
                    for (key, value) in document_map {
                        // TODO should this panic on unknown members? Probably fine to just ignore
                        if let Some(member_schema) = schema.get_member(key) {
                            struct_serializer.serialize_member(member_schema.as_ref(), value)?;
                        }
                    }
                    struct_serializer.end(schema)
                } else {
                    Err(Box::new(DocumentError::DocumentSerialization("Could not serialize document as struct".to_string())) as Box<dyn Error>)?
                }
            },
            // TODO: Raise _some_ error?
            _ => panic!("Service and member types not supported")
        }
    }
}

impl Document<'_> {
    pub fn value(&self) -> &DocumentValue<'_> {
        &self.value
    }

    pub fn size(&self) -> usize {
        match self.value {
            DocumentValue::Array(ref array) => array.len(),
            DocumentValue::Map(ref map) => map.len(),
            DocumentValue::Null => 0,
            _ => 1,
        }
    }

    #[allow(unused_variables)]
    pub fn of_shape(shape: impl SerializeShape) -> Result<Self, DocumentError> {
        todo!()
    }
}

// // ====== INTO conversions =====
// // TODO: Macro-ify these?
impl TryFrom<&Document<'_>> for ByteBuffer {
    type Error = DocumentError;

    fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
        if let DocumentValue::Blob(b) = &value.value {
            Ok(b.clone())
        } else {
            Err(DocumentError::DocumentSerialization("blob".to_string()))
        }
    }
}

impl TryFrom<&Document<'_>> for bool {
    type Error = DocumentError;

    fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
        if let &DocumentValue::Boolean(b) = &value.value {
            Ok(b)
        } else {
            Err(DocumentError::DocumentSerialization("boolean".to_string()))
        }
    }
}

impl TryFrom<&Document<'_>> for String {
    type Error = DocumentError;

    fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
        if let DocumentValue::String(s) = &value.value {
            Ok(s.clone())
        } else {
            Err(DocumentError::DocumentSerialization("string".to_string()))
        }
    }
}

impl TryFrom<&Document<'_>> for Instant {
    type Error = DocumentError;

    fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
        if let &DocumentValue::Timestamp(ts) = &value.value {
            Ok(ts)
        } else {
            Err(DocumentError::DocumentSerialization(
                "timestamp".to_string(),
            ))
        }
    }
}

// TODO: Make Number conversions actually smart!
impl TryFrom<&Document<'_>> for i8 {
    type Error = DocumentError;

    fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
        match &value.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Ok(b as i8),
                &NumberInteger::Short(s) => Ok(s as i8),
                &NumberInteger::Integer(i) => Ok(i as i8),
                &NumberInteger::Long(l) => Ok(l as i8),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
            },
            _ => Err(DocumentError::DocumentSerialization("i8".to_string())),
        }
    }
}

impl TryFrom<&Document<'_>> for i16 {
    type Error = DocumentError;

    fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
        match &value.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Ok(b as i16),
                &NumberInteger::Short(s) => Ok(s),
                &NumberInteger::Integer(i) => Ok(i as i16),
                &NumberInteger::Long(l) => Ok(l as i16),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
            },
            _ => Err(DocumentError::DocumentSerialization("i16".to_string())),
        }
    }
}

impl TryFrom<&Document<'_>> for i32 {
    type Error = DocumentError;

    fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
        match &value.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Ok(b as i32),
                &NumberInteger::Short(s) => Ok(s as i32),
                &NumberInteger::Integer(i) => Ok(i),
                &NumberInteger::Long(l) => Ok(l as i32),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
            },
            _ => Err(DocumentError::DocumentSerialization("i32".to_string())),
        }
    }
}

impl TryFrom<&Document<'_>> for f32 {
    type Error = DocumentError;

    fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
        match &value.value {
            DocumentValue::Number(NumberValue::Float(nf)) => match nf {
                &NumberFloat::Float(f) => Ok(f),
                &NumberFloat::Double(d) => Ok(d as f32),
                NumberFloat::BigDecimal(_) => todo!(),
            },
            _ => Err(DocumentError::DocumentSerialization("f32".to_string())),
        }
    }
}

impl TryFrom<&Document<'_>> for f64 {
    type Error = DocumentError;

    fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
        match &value.value {
            DocumentValue::Number(NumberValue::Float(nf)) => match nf {
                &NumberFloat::Float(f) => Ok(f as f64),
                &NumberFloat::Double(d) => Ok(d),
                NumberFloat::BigDecimal(_) => todo!(),
            },
            _ => Err(DocumentError::DocumentSerialization("f64".to_string())),
        }
    }
}

// TODO: Maybe these could be made more generic?
impl TryFrom<&Document<'_>> for i64 {
    type Error = DocumentError;

    fn try_from(value: &Document<'_>) -> Result<Self, Self::Error> {
        match &value.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Ok(b as i64),
                &NumberInteger::Short(s) => Ok(s as i64),
                &NumberInteger::Integer(i) => Ok(i as i64),
                &NumberInteger::Long(l) => Ok(l),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
            },
            _ => Err(DocumentError::DocumentSerialization("i64".to_string())),
        }
    }
}

impl TryFrom<&Document<'_>> for BigInt {
    type Error = DocumentError;

    fn try_from(_: &Document<'_>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<&Document<'_>> for BigDecimal {
    type Error = DocumentError;

    fn try_from(_: &Document<'_>) -> Result<Self, Self::Error> {
        todo!()
    }
}

// ==== FROM impls ====
impl From<bool> for Document<'_> {
    fn from(value: bool) -> Self {
        todo!()
    }
}

impl From<i8> for Document {
    fn from(value: i8) -> Self {
        todo!()
    }
}

impl From<i16> for Document {
    fn from(value: i16) -> Self {
        todo!()
    }
}

impl From<i32> for Document<'_> {
    fn from(value: i32) -> Self {
        Document {
            schema: &prelude::INTEGER,
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Integer(value))),
            discriminator: None,
        }
    }
}

impl From<i64> for Document {
    fn from(value: i64) -> Self {
        todo!()
    }
}

impl From<f32> for Document {
    fn from(value: f32) -> Self {
        todo!()
    }
}

impl From<f64> for Document {
    fn from(value: f64) -> Self {
        todo!()
    }
}

impl From<&str> for Document<'_> {
    fn from(value: &str) -> Self {
        Document {
            schema: &prelude::STRING,
            value: DocumentValue::String(value.to_string()),
            discriminator: None,
        }
    }
}

impl From<BigInt> for Document {
    fn from(value: BigInt) -> Self {
        todo!()
    }
}

impl From<BigDecimal> for Document {
    fn from(value: BigDecimal) -> Self {
        todo!()
    }
}

impl From<ByteBuffer> for Document {
    fn from(value: ByteBuffer) -> Self {
        todo!()
    }
}

impl From<String> for Document<'_> {
    fn from(value: String) -> Self {
        Document {
            schema: &prelude::STRING,
            value: DocumentValue::String(value),
            discriminator: None,
        }
    }
}

pub lazy_schema!(LIST_DOCUMENT_SCHEMA, Schema::list_builder(*prelude::DOCUMENT.id().clone())
    .put_member("member", &prelude::DOCUMENT, traits![])
    .build()
);
impl From<Vec<Document<'_>>> for Document<'_> {
    fn from(value: Vec<Document<'_>>) -> Self {
        todo!()
    }
}

pub lazy_schema!(MAP_DOCUMENT_SCHEMA, Schema::list_builder(*prelude::DOCUMENT.id().clone())
    .put_member("key", &prelude::STRING, traits![])
    .put_member("value", &prelude::DOCUMENT, traits![])
    .build()
);

#[cfg(test)]
mod tests {
    use super::*;


}


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