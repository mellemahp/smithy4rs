#![allow(dead_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::LazyLock;
use std::time::Instant;
use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use indexmap::IndexMap;
use num_bigint::BigInt;
use thiserror::Error;
use crate::{lazy_schema, traits};
use crate::schema::{prelude, Schema, SchemaRef, ShapeId, ShapeType};
use crate::serde::se::{ListSerializer, MapSerializer, Serializer, SerializerResult, StructSerializer};
use crate::serde::serializers::Serialize;
use crate::serde::{FmtSerializer, SerializeShape};

#[derive(Clone, PartialEq)]
pub struct Document {
    pub schema: SchemaRef,
    pub value: DocumentValue,
    /// A shape ID for a typed document.
    ///
    /// The discriminator is primarily used to implement polymorphism using documents in deserialization.
    ///
    /// *Impl note*: It is expected that protocols set the discriminator on deserialization if applicable
    pub discriminator: Option<ShapeId>,
}

/// A Smithy document type, representing untyped data from the Smithy data model.
#[derive(Clone, PartialEq)]
pub enum DocumentValue {
    Null,
    Number(NumberValue),
    Boolean(bool),
    Blob(ByteBuffer),
    String(String),
    Timestamp(Instant),
    List(Vec<Document>),
    Map(IndexMap<String, Document>),
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
    #[error("Encountered unknown error")]
    Unknown(#[from] Box<dyn Error>),
    #[default]
    #[error("Whooopsie")]
    Default
}

impl SerializeShape for Document {
    fn schema(&self) -> &SchemaRef {
        &self.schema
    }
}

impl Serialize for Document {
    fn serialize<S: Serializer>(&self, schema: &SchemaRef, serializer: &mut S) -> SerializerResult<S::Error> {
        match get_shape_type(schema)? {
            ShapeType::Blob => serializer.write_blob(schema, self.as_blob().ok_or(conversion_error("blob"))?),
            ShapeType::Boolean => serializer.write_boolean(schema, self.as_bool().ok_or(conversion_error("bool"))?),
            ShapeType::String => serializer.write_string(schema, self.as_string().ok_or(conversion_error("string"))?),
            ShapeType::Timestamp => serializer.write_timestamp(schema, self.as_timestamp().ok_or(conversion_error("timestamp"))?),
            ShapeType::Byte => serializer.write_byte(schema, self.as_byte().ok_or(conversion_error("byte"))?),
            ShapeType::Short => serializer.write_short(schema, self.as_short().ok_or(conversion_error("short"))?),
            ShapeType::Integer => serializer.write_integer(schema, self.as_integer().ok_or(conversion_error("integer"))?),
            ShapeType::Long => serializer.write_long(schema, self.as_long().ok_or(conversion_error("long"))?),
            ShapeType::Float => serializer.write_float(schema, self.as_float().ok_or(conversion_error("float"))?),
            ShapeType::Double => serializer.write_double(schema, self.as_double().ok_or(conversion_error("double"))?),
            ShapeType::BigInteger => serializer.write_big_integer(schema, &self.as_big_integer().ok_or(conversion_error("big integer"))?),
            ShapeType::BigDecimal => serializer.write_big_decimal(schema, &self.as_big_decimal().ok_or(conversion_error("big decimal"))?),
            ShapeType::Document => serializer.write_document(schema, &self),
            // TODO: These wont work RN. Need to implement.
            ShapeType::Enum => serializer.write_string(schema, self.as_string().ok_or(conversion_error("enum"))?),
            ShapeType::IntEnum => serializer.write_integer(schema, self.as_integer().ok_or(conversion_error("intEnum"))?),
            ShapeType::List => self.as_list().ok_or(conversion_error("list"))?.serialize(schema, serializer),
            ShapeType::Map => self.as_map().ok_or(conversion_error("map"))?.serialize(schema, serializer),
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
            },
            // TODO: Raise _some_ error?
            _ => panic!("Unsupported shape type"),
        }
    }
}

fn get_shape_type(schema: &SchemaRef) -> Result<&ShapeType, Box<dyn Error>> {
    let mut shape_type = schema.shape_type();
    if shape_type == &ShapeType::Member {
        let Some(member) = schema.as_member() else {
            // TODO: Real error
            return Err(conversion_error("Expected memberSchema for member shape type"))
        };
        Ok(member.target.shape_type())
    } else {
        Ok(shape_type)
    }
}

fn conversion_error(expected: &'static str) -> Box<dyn Error> {
    Box::new(DocumentError::DocumentConversion(expected.to_string())) as Box<dyn Error>
}

impl Document {
    pub fn of(value: impl Into<Document>) -> Self {
        value.into()
    }
}

impl Document {
    pub fn value(&self) -> &DocumentValue {
        &self.value
    }

    pub fn size(&self) -> usize {
        match self.value {
            DocumentValue::List(ref array) => array.len(),
            DocumentValue::Map(ref map) => map.len(),
            DocumentValue::Null => 0,
            _ => 1,
        }
    }

    #[allow(unused_variables)]
    pub fn of_shape(shape: impl SerializeShape) -> Result<Self, DocumentError> {
        todo!()
    }

    /// Get the blob value of the Document if it is a blob.
    pub fn as_blob(&self) -> Option<&ByteBuffer> {
        if let DocumentValue::Blob(b) = &self.value {
            Some(b)
        } else {
            None
        }
    }

    /// Get the boolean value of the Document if it is a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        if let &DocumentValue::Boolean(b) = &self.value {
            Some(b)
        } else {
            None
        }
    }


    /// Get the string value of the Document if it is a string.
    pub fn as_string(&self) -> Option<&String> {
        if let DocumentValue::String(s) = &self.value {
            Some(s)
        } else {
            None
        }
    }

    /// Get the timestamp value of the Document if it is a timestamp.
    pub fn as_timestamp(&self) -> Option<&Instant> {
        todo!()
    }


    /// Get the byte value of the Document if it is a byte or can be converted into one.
    pub fn as_byte(&self) -> Option<i8> {
        match &self.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Some(b),
                &NumberInteger::Short(s) => Some(s as i8),
                &NumberInteger::Integer(i) => Some(i as i8),
                &NumberInteger::Long(l) => Some(l as i8),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
            },
            _ => None,
        }
    }

    /// Get the short value of the Document if it is a short or can be converted into one.
    pub fn as_short(&self) -> Option<i16> {
        match &self.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Some(b as i16),
                &NumberInteger::Short(s) => Some(s),
                &NumberInteger::Integer(i) => Some(i as i16),
                &NumberInteger::Long(l) => Some(l as i16),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
            },
            _ => None,
        }
    }

    /// Get the integer value of the Document if it is an integer or can be converted into one.
    pub fn as_integer(&self) -> Option<i32> {
        match &self.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Some(b as i32),
                &NumberInteger::Short(s) => Some(s as i32),
                &NumberInteger::Integer(i) => Some(i),
                &NumberInteger::Long(l) => Some(l as i32),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
            },
            _ => None,
        }
    }

    /// Get the long value of the Document if it is a long or can be converted into one.
    pub fn as_long(&self) -> Option<i64> {
        match &self.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Some(b as i64),
                &NumberInteger::Short(s) => Some(s as i64),
                &NumberInteger::Integer(i) => Some(i as i64),
                &NumberInteger::Long(l) => Some(l),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
            },
            _ => None,
        }
    }

    /// Get the float value of the Document if it is a float or can be converted into one.
    pub fn as_float(&self) -> Option<f32> {
        match &self.value {
            DocumentValue::Number(NumberValue::Float(nf)) => match nf {
                &NumberFloat::Float(f) => Some(f),
                &NumberFloat::Double(d) => Some(d as f32),
                NumberFloat::BigDecimal(_) => todo!(),
            },
            _ => None,
        }
    }

    /// Get the decimal value of the Document if it is a decimal or can be converted into one.
    pub fn as_double(&self) -> Option<f64> {
        match &self.value {
            DocumentValue::Number(NumberValue::Float(nf)) => match nf {
                &NumberFloat::Float(f) => Some(f as f64),
                &NumberFloat::Double(d) => Some(d),
                NumberFloat::BigDecimal(_) => todo!(),
            },
            _ => None,
        }
    }

    pub fn as_big_integer(&self) -> Option<BigInt> {
        todo!()
    }

    pub fn as_big_decimal(&self) -> Option<BigDecimal> {
        todo!()
    }

    pub fn as_list(&self) -> Option<&Vec<Document>> {
        if let DocumentValue::List(document_list) = &self.value {
           Some(document_list)
        } else {
            None
        }
    }

    pub fn as_map(&self) -> Option<&IndexMap<String, Document>> {
        if let DocumentValue::Map(document_map) = &self.value {
            Some(document_map)
        } else {
            None
        }
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fmter = FmtSerializer::default();
        match self.serialize_shape(&mut fmter) {
            Ok(_) => f.write_str(&fmter.flush()),
            Err(_) => Err(std::fmt::Error),
        }
    }
}

// // ====== INTO conversions =====
// // TODO: Macro-ify these?
impl TryFrom<Document> for ByteBuffer {
    type Error = DocumentError;

    fn try_from(value: Document) -> Result<Self, Self::Error> {
        if let DocumentValue::Blob(b) = value.value {
            Ok(b)
        } else {
            Err(DocumentError::DocumentConversion("blob".to_string()))
        }
    }
}

impl TryFrom<Document> for bool {
    type Error = DocumentError;

    fn try_from(value: Document) -> Result<Self, Self::Error> {
        if let DocumentValue::Boolean(b) = value.value {
            Ok(b)
        } else {
            Err(DocumentError::DocumentConversion("boolean".to_string()))
        }
    }
}

impl TryFrom<Document> for String {
    type Error = DocumentError;

    fn try_from(value: Document) -> Result<Self, Self::Error> {
        if let DocumentValue::String(s) = value.value {
            Ok(s)
        } else {
            Err(DocumentError::DocumentConversion("string".to_string()))
        }
    }
}

impl TryFrom<Document> for Instant {
    type Error = DocumentError;

    fn try_from(_: Document) -> Result<Self, Self::Error> {
        todo!()
    }
}

// TODO: Make Number conversions smarter? Or does rust `as` method handle truncation and such?
impl TryFrom<Document> for i8 {
    type Error = DocumentError;

    fn try_from(value: Document) -> Result<Self, Self::Error> {
        match value.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                NumberInteger::Byte(b) => Ok(b),
                NumberInteger::Short(s) => Ok(s as i8),
                NumberInteger::Integer(i) => Ok(i as i8),
                NumberInteger::Long(l) => Ok(l as i8),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
            },
            _ => Err(DocumentError::DocumentConversion("i8".to_string())),
        }
    }
}

impl TryFrom<Document> for i16 {
    type Error = DocumentError;

    fn try_from(value: Document) -> Result<Self, Self::Error> {
        match value.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                NumberInteger::Byte(b) => Ok(b as i16),
                NumberInteger::Short(s) => Ok(s),
                NumberInteger::Integer(i) => Ok(i as i16),
                NumberInteger::Long(l) => Ok(l as i16),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
            },
            _ => Err(DocumentError::DocumentConversion("i16".to_string())),
        }
    }
}

impl TryFrom<Document> for i32 {
    type Error = DocumentError;

    fn try_from(value: Document) -> Result<Self, Self::Error> {
        match value.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                NumberInteger::Byte(b) => Ok(b as i32),
                NumberInteger::Short(s) => Ok(s as i32),
                NumberInteger::Integer(i) => Ok(i),
                NumberInteger::Long(l) => Ok(l as i32),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
            },
            _ => Err(DocumentError::DocumentConversion("i32".to_string())),
        }
    }
}

impl TryFrom<Document> for i64 {
    type Error = DocumentError;

    fn try_from(value: Document) -> Result<Self, Self::Error> {
        match value.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                NumberInteger::Byte(b) => Ok(b as i64),
                NumberInteger::Short(s) => Ok(s as i64),
                NumberInteger::Integer(i) => Ok(i as i64),
                NumberInteger::Long(l) => Ok(l),
                NumberInteger::BigInt(_) => todo!("Support conversion if possible"),
            },
            _ => Err(DocumentError::DocumentConversion("i64".to_string())),
        }
    }
}


impl TryFrom<Document> for f32 {
    type Error = DocumentError;

    fn try_from(value: Document) -> Result<Self, Self::Error> {
        match value.value {
            DocumentValue::Number(NumberValue::Float(nf)) => match nf {
                NumberFloat::Float(f) => Ok(f),
                NumberFloat::Double(d) => Ok(d as f32),
                NumberFloat::BigDecimal(_) => todo!(),
            },
            _ => Err(DocumentError::DocumentConversion("f32".to_string())),
        }
    }
}

impl TryFrom<Document> for f64 {
    type Error = DocumentError;

    fn try_from(value: Document) -> Result<Self, Self::Error> {
        match value.value {
            DocumentValue::Number(NumberValue::Float(nf)) => match nf {
                NumberFloat::Float(f) => Ok(f as f64),
                NumberFloat::Double(d) => Ok(d),
                NumberFloat::BigDecimal(_) => todo!(),
            },
            _ => Err(DocumentError::DocumentConversion("f64".to_string())),
        }
    }
}


impl TryFrom<&Document> for BigInt {
    type Error = DocumentError;

    fn try_from(_: &Document) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<&Document> for BigDecimal {
    type Error = DocumentError;

    fn try_from(_: &Document) -> Result<Self, Self::Error> {
        todo!()
    }
}

// ==== FROM impls ====
impl From<bool> for Document {
    fn from(value: bool) -> Self {
        Document {
            schema: prelude::BOOLEAN.clone(),
            value: DocumentValue::Boolean(value),
            discriminator: None,
        }
    }
}

impl From<i8> for Document {
    fn from(value: i8) -> Self {
        Document {
            schema: prelude::BYTE.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Byte(value))),
            discriminator: None,
        }
    }
}

impl From<i16> for Document {
    fn from(value: i16) -> Self {
        Document {
            schema: prelude::SHORT.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Short(value))),
            discriminator: None,
        }
    }
}

impl From<i32> for Document {
    fn from(value: i32) -> Self {
        Document {
            schema: prelude::INTEGER.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Integer(value))),
            discriminator: None,
        }
    }
}

impl From<i64> for Document {
    fn from(value: i64) -> Self {
        Document {
            schema: prelude::LONG.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Long(value))),
            discriminator: None,
        }
    }
}

impl From<f32> for Document {
    fn from(value: f32) -> Self {
        Document {
            schema: prelude::FLOAT.clone(),
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::Float(value))),
            discriminator: None,
        }
    }
}

impl From<f64> for Document {
    fn from(value: f64) -> Self {
        Document {
            schema: prelude::FLOAT.clone(),
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::Double(value))),
            discriminator: None,
        }
    }
}

impl From<&str> for Document {
    fn from(value: &str) -> Self {
        Document {
            schema: prelude::STRING.clone(),
            value: DocumentValue::String(value.to_string()),
            discriminator: None,
        }
    }
}

impl From<BigInt> for Document {
    fn from(value: BigInt) -> Self {
        Document {
            schema: prelude::BIG_INTEGER.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::BigInt(value))),
            discriminator: None,
        }
    }
}

impl From<BigDecimal> for Document {
    fn from(value: BigDecimal) -> Self {
        Document {
            schema: prelude::BIG_DECIMAL.clone(),
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::BigDecimal(value))),
            discriminator: None,
        }
    }
}

impl From<ByteBuffer> for Document {
    fn from(value: ByteBuffer) -> Self {
        Document {
            schema: prelude::BLOB.clone(),
            value: DocumentValue::Blob(value),
            discriminator: None,
        }
    }
}

impl From<String> for Document {
    fn from(value: String) -> Self {
        Document {
            schema: prelude::STRING.clone(),
            value: DocumentValue::String(value),
            discriminator: None,
        }
    }
}

lazy_schema!(LIST_DOCUMENT_SCHEMA, Schema::list_builder(prelude::DOCUMENT.id().clone())
    .put_member("member", &prelude::DOCUMENT, traits![])
    .build()
);
impl <'a> From<Vec<Document>> for Document {
    fn from(value: Vec<Document>) -> Self {
        Document {
            schema: LIST_DOCUMENT_SCHEMA.clone(),
            value: DocumentValue::List(value),
            discriminator: None,
        }
    }
}

lazy_schema!(MAP_DOCUMENT_SCHEMA, Schema::list_builder(prelude::DOCUMENT.id().clone())
    .put_member("key", &prelude::STRING, traits![])
    .put_member("value", &prelude::DOCUMENT, traits![])
    .build()
);
impl <'a> From<IndexMap<String, Document>> for Document {
    fn from(value: IndexMap<String, Document>) -> Self {
        Document {
            schema: MAP_DOCUMENT_SCHEMA.clone(),
            value: DocumentValue::Map(value),
            discriminator: None,
        }
    }
}

// TODO: How to make this `of` implementation work for Serializable struct?
// impl <'a, T: SerializeShape + Serialize> From<T> for Document {
//     fn from(value: T) -> Self {
//         todo!()
//     }
// }

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
       self.document.ok_or(DocumentError::DocumentSerialization("Serialization did not set document value".to_string()))
    }
}

impl Serializer for DocumentParser {
    type Error = DocumentError;
    type SerializeList<'l> = DocumentListParser<'l>
    where Self: 'l;
    type SerializeMap<'m> = DocumentMapParser<'m>
    where Self: 'm;
    type SerializeStruct<'s> = DocumentMapParser<'s>
    where Self: 's;

    // TODO: Use len
    fn write_struct(&mut self, schema: &SchemaRef, _: usize) -> Result<Self::SerializeStruct<'_>, Self::Error> {
        Ok(DocumentMapParser::new(self, schema))
    }

    // TODO: Use len
    fn write_map(&mut self, schema: &SchemaRef, _: usize) -> Result<Self::SerializeMap<'_>, Self::Error> {
        Ok(DocumentMapParser::new(self, schema))
    }

    fn write_list(&mut self, schema: &SchemaRef, len: usize) -> Result<Self::SerializeList<'_>, Self::Error> {
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

    fn write_big_integer(&mut self, schema: &SchemaRef, value: &BigInt) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::BigInt(value.clone()))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_big_decimal(&mut self, schema: &SchemaRef, value: &BigDecimal) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::BigDecimal(value.clone()))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_string(&mut self, schema: &SchemaRef, value: &String) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::String(value.clone()),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_blob(&mut self, schema: &SchemaRef, value: &ByteBuffer) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Blob(value.clone()),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_timestamp(&mut self, schema: &SchemaRef, value: &Instant) -> SerializerResult<Self::Error> {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Timestamp(value.clone()),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_document(&mut self, schema: &SchemaRef, value: &Document) -> SerializerResult<Self::Error> {
        todo!()
    }

    fn write_null(&mut self, schema: &SchemaRef) -> SerializerResult<Self::Error> {
        todo!()
    }

    fn skip(&mut self, schema: &SchemaRef) -> SerializerResult<Self::Error> {
        todo!()
    }
}

struct DocumentListParser<'lp> {
    parent: &'lp mut DocumentParser,
    document: Document,
}
impl <'lp> DocumentListParser<'lp> {
    pub(super) fn new(parent: &'lp mut DocumentParser, schema: &SchemaRef, len: usize) -> Self {
        DocumentListParser {
            parent,
            document: Document {
                schema: schema.clone(),
                value: DocumentValue::List(Vec::with_capacity(len)),
                discriminator: Some(schema.id().clone()),
            }
        }
    }
}
impl ListSerializer for DocumentListParser<'_> {
    type Error = DocumentError;

    fn serialize_element<T>(&mut self, element_schema: &SchemaRef, value: &T) -> SerializerResult<Self::Error>
    where
        T: ?Sized + Serialize
    {
        let mut element_parser = DocumentParser::new();
        value.serialize(element_schema, &mut element_parser)?;
        if let DocumentValue::List(list) = &mut self.document.value {
            list.push(element_parser.result()?);
            Ok(())
        } else {
            Err(DocumentError::DocumentSerialization("expected a list".to_string()))
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
impl <'mp> DocumentMapParser<'mp> {
    pub(super) fn new(parent: &'mp mut DocumentParser, schema: &SchemaRef) -> Self {
        DocumentMapParser {
            parent,
            document: Document {
                schema: schema.clone(),
                value: DocumentValue::Map(IndexMap::new()),
                discriminator: Some(schema.id().clone())
            }
        }
    }
}

impl MapSerializer for DocumentMapParser<'_> {
    type Error = DocumentError;

    fn serialize_entry<K, V>(&mut self, key_schema: &SchemaRef, value_schema: &SchemaRef, key: &K, value: &V) -> SerializerResult<Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize
    {
        let Some(me) = key_schema.as_member() else {
            return Err(DocumentError::DocumentSerialization("Expected member schema!".to_string()));
        };
        let mut value_serializer = DocumentParser::new();
        value.serialize(value_schema, &mut value_serializer)?;
        let DocumentValue::Map(map) = &mut self.document.value else {
            return Err(DocumentError::DocumentSerialization("Expected member schema!".to_string()));
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

    fn serialize_member<T>(&mut self, member_schema: &SchemaRef, value: &T) -> SerializerResult<Self::Error>
    where
        T: ?Sized + Serialize
    {
        let Some(me) = member_schema.as_member() else {
            return Err(DocumentError::DocumentSerialization("Expected member schema!".to_string()));
        };
        let mut value_serializer = DocumentParser::new();
        value.serialize(member_schema, &mut value_serializer)?;
        let DocumentValue::Map(map) = &mut self.document.value else {
            return Err(DocumentError::DocumentSerialization("Expected member schema!".to_string()));
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
    use crate::lazy_member_schema;
    use super::*;

    lazy_schema!(MAP_SCHEMA, Schema::map_builder(ShapeId::from("com.example#Map"))
        .put_member("key", &prelude::STRING, traits![])
        .put_member("value", &prelude::STRING, traits![])
        .build()
    );
    lazy_schema!(LIST_SCHEMA, Schema::list_builder(ShapeId::from("com.example#List"))
        .put_member("member", &prelude::STRING, traits![])
        .build()
    );
    lazy_schema!(SCHEMA, Schema::structure_builder(ShapeId::from("com.example#Shape"))
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
        fn serialize<S: Serializer>(&self, schema: &SchemaRef, serializer: &mut S) -> SerializerResult<S::Error>
        {
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
        let mut doc_parser = DocumentParser::new();
        struct_to_convert.serialize(struct_to_convert.schema(), &mut doc_parser)
            .expect("Failed to convert to document!");
        panic!("{}", doc_parser.result().expect("Failed to convert to document!"));
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