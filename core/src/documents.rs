
use std::collections::HashMap;
use std::time::Instant;
use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;
use thiserror::Error;
use crate::schema::Schema;
use crate::serde::de::Deserializable;
use crate::serde::deserializers::Deserializer;
use crate::serde::se::{Serializable, SerializableStruct, Serializer};
use crate::serde::serializers::ListItemConsumer;
use crate::shapes::{ShapeId, ShapeType};

#[derive(Error, Debug)]
pub enum DocumentError {
    #[error("Failed to convert document to type {0}")]
    DocumentSerialization(String),
    #[error("Failed to convert document to type {0}")]
    DocumentConversion(String),
}


/// A Smithy document type, representing untyped data from the Smithy data model.
///
/// *Note*: Document implementations are protocol specific
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub enum NumberValue {
    Integer(NumberInteger),
    Float(NumberFloat)
}

#[derive(Debug, Clone)]
pub enum NumberInteger {
    Byte(i8),
    Short(i16),
    Integer(i32),
    Long(i64),
    BigInt(BigInt)
}

#[derive(Debug, Clone)]
pub enum NumberFloat {
    Float(f32),
    Double(f64),
    BigDecimal(BigDecimal)
}

// TODO: Could these be just normal TryFrom impls?
impl <'doc> DocumentValue<'doc> {
    fn try_into_list(self) -> Result<Vec<Document<'doc>>, DocumentError> {
        if let Self::Array(arr) = self {
            Ok(arr)
        } else {
            Err(DocumentError::DocumentConversion("list".to_string()))
        }
    }

    fn try_into_map(self) -> Result<HashMap<String, Document<'doc>>, DocumentError> {
        if let Self::Map(map) = self {
            Ok(map)
        } else {
            Err(DocumentError::DocumentSerialization("map".to_string()))
        }
    }
}

impl TryFrom<DocumentValue<'_>> for ByteBuffer {
    type Error = DocumentError;

    fn try_from(value: DocumentValue<'_>) -> Result<Self, Self::Error> {
        if let DocumentValue::Blob(b) = value {
            Ok(b)
        } else {
            Err(DocumentError::DocumentSerialization("blob".to_string()))
        }
    }
}

impl TryFrom<DocumentValue<'_>> for bool {
    type Error = DocumentError;

    fn try_from(value: DocumentValue<'_>) -> Result<Self, Self::Error> {
        if let DocumentValue::Boolean(b) = value {
            Ok(b)
        } else {
            Err(DocumentError::DocumentSerialization("boolean".to_string()))
        }
    }
}

// TODO: Macro-ify these?
impl TryFrom<DocumentValue<'_>> for String {
    type Error = DocumentError;

    fn try_from(value: DocumentValue<'_>) -> Result<Self, Self::Error> {
        if let DocumentValue::String(s) = value {
            Ok(s)
        } else {
            Err(DocumentError::DocumentSerialization("string".to_string()))
        }
    }
}

impl TryFrom<DocumentValue<'_>> for Instant {
    type Error = DocumentError;

    fn try_from(value: DocumentValue<'_>) -> Result<Self, Self::Error> {
        if let DocumentValue::Timestamp(ts) = value {
            Ok(ts)
        } else {
            Err(DocumentError::DocumentSerialization("timestamp".to_string()))
        }
    }
}

// TODO: Make Number conversions actually smart!
impl TryFrom<DocumentValue<'_>> for i8 {
    type Error = DocumentError;

    fn try_from(value: DocumentValue<'_>) -> Result<Self, Self::Error> {
        match value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                NumberInteger::Byte(b) => Ok(b as i8),
                NumberInteger::Short(s) => Ok(s as i8),
                NumberInteger::Integer(i) => Ok(i as i8),
                NumberInteger::Long(l) => Ok(l as i8),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible")
            },
            _ => Err(DocumentError::DocumentSerialization("i8".to_string()))
        }
    }
}

impl TryFrom<DocumentValue<'_>> for i16 {
    type Error = DocumentError;

    fn try_from(value: DocumentValue<'_>) -> Result<Self, Self::Error> {
        match value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                NumberInteger::Byte(b) => Ok(b as i16),
                NumberInteger::Short(s) => Ok(s),
                NumberInteger::Integer(i) => Ok(i as i16),
                NumberInteger::Long(l) => Ok(l as i16),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible")
            },
            _ => Err(DocumentError::DocumentSerialization("i16".to_string()))
        }
    }
}

impl TryFrom<DocumentValue<'_>> for i32 {
    type Error = DocumentError;

    fn try_from(value: DocumentValue<'_>) -> Result<Self, Self::Error> {
        match value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                NumberInteger::Byte(b) => Ok(b as i32),
                NumberInteger::Short(s) => Ok(s as i32),
                NumberInteger::Integer(i) => Ok(i),
                NumberInteger::Long(l) => Ok(l as i32),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible")
            },
            _ => Err(DocumentError::DocumentSerialization("i32".to_string()))
        }
    }
}

impl TryFrom<DocumentValue<'_>> for f32 {
    type Error = DocumentError;

    fn try_from(value: DocumentValue<'_>) -> Result<Self, Self::Error> {
        match value {
            DocumentValue::Number(NumberValue::Float(nf)) => match nf {
                NumberFloat::Float(f) => Ok(f),
                NumberFloat::Double(d) => Ok(d as f32),
                NumberFloat::BigDecimal(_) => todo!()
            },
            _ => Err(DocumentError::DocumentSerialization("f32".to_string()))
        }
    }
}

impl TryFrom<DocumentValue<'_>> for f64 {
    type Error = DocumentError;

    fn try_from(value: DocumentValue<'_>) -> Result<Self, Self::Error> {
        match value {
            DocumentValue::Number(NumberValue::Float(nf)) => match nf {
                NumberFloat::Float(f) => Ok(f as f64),
                NumberFloat::Double(d) => Ok(d),
                NumberFloat::BigDecimal(_) => todo!()
            },
            _ => Err(DocumentError::DocumentSerialization("f64".to_string()))
        }
    }
}

// TODO: Maybe these could be made more generic?
impl TryFrom<DocumentValue<'_>> for i64 {
    type Error = DocumentError;

    fn try_from(value: DocumentValue<'_>) -> Result<Self, Self::Error> {
        match value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                NumberInteger::Byte(b) => Ok(b as i64),
                NumberInteger::Short(s) => Ok(s as i64),
                NumberInteger::Integer(i) => Ok(i as i64),
                NumberInteger::Long(l) => Ok(l),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible")
            },
            _ => Err(DocumentError::DocumentSerialization("i64".to_string()))
        }
    }
}

impl TryFrom<DocumentValue<'_>> for BigInt {
    type Error = DocumentError;

    fn try_from(value: DocumentValue<'_>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<DocumentValue<'_>> for BigDecimal {
    type Error = DocumentError;

    fn try_from(value: DocumentValue<'_>) -> Result<Self, Self::Error> {
        todo!()
    }
}


impl Serializable for Document<'_> {
    fn serialize<S: Serializer>(self, serializer: &mut S) -> Result<(), S::Error> {
        let schema = self.schema;
        match schema.shape_type {
            ShapeType::Blob => serializer.write_blob(schema, self.value.try_into()?),
            ShapeType::Boolean => serializer.write_boolean(schema, self.value.try_into()?),
            ShapeType::String => serializer.write_string(schema, self.value.try_into()?),
            ShapeType::Timestamp => serializer.write_timestamp(schema, self.value.try_into()?),
            ShapeType::Byte => serializer.write_byte(schema, self.value.try_into()?),
            ShapeType::Short => serializer.write_short(schema, self.value.try_into()?),
            ShapeType::Integer => serializer.write_integer(schema, self.value.try_into()?),
            ShapeType::Long => serializer.write_long(schema, self.value.try_into()?),
            ShapeType::Float => serializer.write_integer(schema, self.value.try_into()?),
            ShapeType::Double => serializer.write_double(schema, self.value.try_into()?),
            ShapeType::BigInteger => serializer.write_big_integer(schema, self.value.try_into()?),
            ShapeType::BigDecimal => serializer.write_big_integer(schema, self.value.try_into()?),
            ShapeType::Document => serializer.write_document(schema, self),
            ShapeType::Enum => serializer.write_string(schema, self.value.try_into()?),
            ShapeType::IntEnum => serializer.write_integer(schema, self.value.try_into()?),
            ShapeType::List => serializer.write_list(schema, self.size(),self.value.try_into_list()?, DocumentListConsumer{}),
            ShapeType::Map => todo!(),
            ShapeType::Structure => serializer.write_struct(schema, self),
            ShapeType::Union => serializer.write_struct(schema, self),
            _ => Err(Default::default())
        }
    }
}

struct DocumentListConsumer{}
impl ListItemConsumer<Vec<Document<'_>>> for DocumentListConsumer {
    fn consume<S: Serializer>(item: Document, serializer: &mut S) -> Result<(), S::Error> {
        // TODO: Sparse lists?
        match item.value {
            DocumentValue::Null => serializer.write_null(item.schema),
            _ => item.serialize(serializer)
        }
    }
}

impl <'doc> SerializableStruct for Document<'doc> {
    fn schema(&self) -> &'doc Schema<'doc> {
        self.schema
    }

    fn serialize_members<S: Serializer>(self, serializer: &mut S) -> Result<(), S::Error> {
        todo!()
    }
}


#[derive(Debug, Clone)]
pub struct Document<'doc> {
    pub schema: &'doc Schema<'doc>,
    pub value: DocumentValue<'doc>,
    // NOTE: It is expected that protocols set these!
    pub discriminator: Option<&'doc ShapeId>
}

impl Document<'_> {
    fn size(&self) -> usize {
        match self.value {
            DocumentValue::Array(ref array) => array.len(),
            DocumentValue::Map(ref map) => map.len(),
            DocumentValue::Null => 0,
            _ => 1
        }
    }
}


