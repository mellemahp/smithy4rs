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
use crate::schema::{prelude, Schema, ShapeId, ShapeType, StructSchema};
use crate::serde::se::{ListSerializer, MapSerializer, Serializer, SerializerResult, StructSerializer};
use crate::serde::serializers::Serialize;
use crate::serde::SerializeShape;

#[derive(Clone, PartialEq)]
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
#[derive(Clone, PartialEq)]
pub enum DocumentValue<'doc> {
    Null,
    Number(NumberValue),
    Boolean(bool),
    Blob(ByteBuffer),
    String(String),
    Timestamp(Instant),
    List(Vec<Document<'doc>>),
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
    #[error("Encountered unknown error")]
    Unknown(#[from] Box<dyn Error>),
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
    fn serialize<'a, S: Serializer<'a>>(&self, schema: &'a Schema, serializer: &mut S) -> SerializerResult<S::Error> {
        match schema.shape_type() {
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
            _ => panic!("Service and member types not supported")
        }
    }
}

fn conversion_error(expected: &'static str) -> Box<dyn Error> {
    Box::new(DocumentError::DocumentConversion(expected.to_string())) as Box<dyn Error>
}

impl <'doc> Document<'doc> {
    pub fn of(value: impl Into<Document<'doc>>) -> Self {
        value.into()
    }
}

impl Document<'_> {
    pub fn value(&self) -> &DocumentValue<'_> {
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

// // ====== INTO conversions =====
// // TODO: Macro-ify these?
impl TryFrom<Document<'_>> for ByteBuffer {
    type Error = DocumentError;

    fn try_from(value: Document<'_>) -> Result<Self, Self::Error> {
        if let DocumentValue::Blob(b) = value.value {
            Ok(b)
        } else {
            Err(DocumentError::DocumentConversion("blob".to_string()))
        }
    }
}

impl TryFrom<Document<'_>> for bool {
    type Error = DocumentError;

    fn try_from(value: Document<'_>) -> Result<Self, Self::Error> {
        if let DocumentValue::Boolean(b) = value.value {
            Ok(b)
        } else {
            Err(DocumentError::DocumentConversion("boolean".to_string()))
        }
    }
}

impl TryFrom<Document<'_>> for String {
    type Error = DocumentError;

    fn try_from(value: Document<'_>) -> Result<Self, Self::Error> {
        if let DocumentValue::String(s) = value.value {
            Ok(s)
        } else {
            Err(DocumentError::DocumentConversion("string".to_string()))
        }
    }
}

impl TryFrom<Document<'_>> for Instant {
    type Error = DocumentError;

    fn try_from(_: Document<'_>) -> Result<Self, Self::Error> {
        todo!()
    }
}

// TODO: Make Number conversions smarter? Or does rust `as` method handle truncation and such?
impl TryFrom<Document<'_>> for i8 {
    type Error = DocumentError;

    fn try_from(value: Document<'_>) -> Result<Self, Self::Error> {
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

impl TryFrom<Document<'_>> for i16 {
    type Error = DocumentError;

    fn try_from(value: Document<'_>) -> Result<Self, Self::Error> {
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

impl TryFrom<Document<'_>> for i32 {
    type Error = DocumentError;

    fn try_from(value: Document<'_>) -> Result<Self, Self::Error> {
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

impl TryFrom<Document<'_>> for i64 {
    type Error = DocumentError;

    fn try_from(value: Document<'_>) -> Result<Self, Self::Error> {
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


impl TryFrom<Document<'_>> for f32 {
    type Error = DocumentError;

    fn try_from(value: Document<'_>) -> Result<Self, Self::Error> {
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

impl TryFrom<Document<'_>> for f64 {
    type Error = DocumentError;

    fn try_from(value: Document<'_>) -> Result<Self, Self::Error> {
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
        Document {
            schema: &prelude::BOOLEAN,
            value: DocumentValue::Boolean(value),
            discriminator: None,
        }
    }
}

impl From<i8> for Document<'_> {
    fn from(value: i8) -> Self {
        Document {
            schema: &prelude::BYTE,
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Byte(value))),
            discriminator: None,
        }
    }
}

impl From<i16> for Document<'_> {
    fn from(value: i16) -> Self {
        Document {
            schema: &prelude::SHORT,
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Short(value))),
            discriminator: None,
        }
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

impl From<i64> for Document<'_> {
    fn from(value: i64) -> Self {
        Document {
            schema: &prelude::LONG,
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Long(value))),
            discriminator: None,
        }
    }
}

impl From<f32> for Document<'_> {
    fn from(value: f32) -> Self {
        Document {
            schema: &prelude::FLOAT,
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::Float(value))),
            discriminator: None,
        }
    }
}

impl From<f64> for Document<'_> {
    fn from(value: f64) -> Self {
        Document {
            schema: &prelude::FLOAT,
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::Double(value))),
            discriminator: None,
        }
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

impl From<BigInt> for Document<'_> {
    fn from(value: BigInt) -> Self {
        Document {
            schema: &prelude::BIG_INTEGER,
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::BigInt(value))),
            discriminator: None,
        }
    }
}

impl From<BigDecimal> for Document<'_> {
    fn from(value: BigDecimal) -> Self {
        Document {
            schema: &prelude::BIG_DECIMAL,
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::BigDecimal(value))),
            discriminator: None,
        }
    }
}

impl From<ByteBuffer> for Document<'_> {
    fn from(value: ByteBuffer) -> Self {
        Document {
            schema: &prelude::BLOB,
            value: DocumentValue::Blob(value),
            discriminator: None,
        }
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

lazy_schema!(LIST_DOCUMENT_SCHEMA, Schema::list_builder(prelude::DOCUMENT.id().clone())
    .put_member("member", &prelude::DOCUMENT, traits![])
    .build()
);
impl <'a> From<Vec<Document<'a>>> for Document<'a> {
    fn from(value: Vec<Document<'a>>) -> Self {
        Document {
            schema: &LIST_DOCUMENT_SCHEMA,
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
impl <'a> From<IndexMap<String, Document<'a>>> for Document<'a> {
    fn from(value: IndexMap<String, Document<'a>>) -> Self {
        Document {
            schema: &MAP_DOCUMENT_SCHEMA,
            value: DocumentValue::Map(value),
            discriminator: None,
        }
    }
}

// TODO: How to make this `of` implementation work for Serializable struct?
// impl <'a, T: SerializeShape + Serialize> From<T> for Document<'a> {
//     fn from(value: T) -> Self {
//         todo!()
//     }
// }

struct DocumentParser<'parser> {
    document: Option<Document<'parser>>,
}
impl <'parser> DocumentParser<'parser> {
    pub(super) fn new() -> Self {
        DocumentParser { document: None }
    }

    pub fn set_document<'a>(&mut self, document: Document<'parser>) {
        self.document = Some(document);
    }

    pub(super) fn result<'a>(self) -> Result<Document<'parser>, DocumentError> {
       self.document.ok_or(DocumentError::DocumentSerialization("Serialization did not set document value".to_string()))
    }
}

impl <'parser> Serializer<'parser> for DocumentParser<'parser> {
    type Error = DocumentError;
    type SerializeList<'l> = DocumentListParser<'l>
    where Self: 'l;
    type SerializeMap<'m> = DocumentMapParser<'m>
    where Self: 'm;
    type SerializeStruct<'s> = DocumentMapParser<'s>
    where Self: 's;

    fn write_struct(&mut self, schema: &Schema, len: usize) -> Result<Self::SerializeStruct<'_>, Self::Error> {
        todo!()
    }

    fn write_map(&mut self, schema: &Schema, len: usize) -> Result<Self::SerializeMap<'_>, Self::Error> {
        todo!()
    }

    fn write_list(&mut self, schema: &Schema, len: usize) -> Result<Self::SerializeList<'_>, Self::Error> {
        todo!()
    }

    fn write_boolean<'a>(&mut self, schema: &'a Schema, value: bool) -> SerializerResult<Self::Error>
    where
        'a: 'parser
    {
        self.set_document(Document {
            schema: schema.clone(),
            value: DocumentValue::Boolean(value),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_byte<'a>(&mut self, schema: &'a Schema, value: i8) -> SerializerResult<Self::Error>
    where
        'a: 'parser
    {
        self.document = Some(Document {
            schema,
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Byte(value))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_short<'a>(&mut self, schema: &'a Schema, value: i16) -> SerializerResult<Self::Error>
    where
        'a: 'parser
    {
        self.document = Some(Document {
            schema,
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Short(value))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_integer<'a>(&mut self, schema: &'a Schema, value: i32) -> SerializerResult<Self::Error>
    where
        'a: 'parser
    {
        self.document = Some(Document {
            schema,
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Integer(value))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_long<'a>(&mut self, schema: &'a Schema, value: i64) -> SerializerResult<Self::Error>
    where
        'a: 'parser
    {
        self.document = Some(Document {
            schema,
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Long(value))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_float<'a>(&mut self, schema: &'a Schema, value: f32) -> SerializerResult<Self::Error>
    where
        'a: 'parser
    {
        self.document = Some(Document {
            schema,
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::Float(value))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_double<'a>(&mut self, schema: &'a Schema, value: f64) -> SerializerResult<Self::Error>
    where
        'a: 'parser
    {
        self.document = Some(Document {
            schema,
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::Double(value))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_big_integer<'a>(&mut self, schema: &'a Schema, value: &BigInt) -> SerializerResult<Self::Error>
    where
        'a: 'parser
    {
        self.document = Some(Document {
            schema,
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::BigInt(value.clone()))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_big_decimal<'a>(&mut self, schema: &'a Schema, value: &BigDecimal) -> SerializerResult<Self::Error>
    where
        'a: 'parser
    {
        self.document = Some(Document {
            schema,
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::BigDecimal(value.clone()))),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_string<'a>(&mut self, schema: &'a Schema, value: &String) -> SerializerResult<Self::Error>
    where
        'a: 'parser
    {
        self.document = Some(Document {
            schema,
            value: DocumentValue::String(value.clone()),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_blob<'a>(&mut self, schema: &'a Schema, value: &ByteBuffer) -> SerializerResult<Self::Error>
    where
        'a: 'parser
    {
        self.document = Some(Document {
            schema,
            value: DocumentValue::Blob(value.clone()),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_timestamp<'a>(&mut self, schema: &'a Schema, value: &Instant) -> SerializerResult<Self::Error>
    where
        'a: 'parser
    {
        self.document = Some(Document {
            schema,
            value: DocumentValue::Timestamp(value.clone()),
            discriminator: Some(schema.id().clone()),
        });
        Ok(())
    }

    fn write_document<'a>(&mut self, schema: &'a Schema, value: &Document) -> SerializerResult<Self::Error>
    where
        'a: 'parser
    {
        todo!()
    }

    fn write_null(&mut self, schema: &Schema) -> SerializerResult<Self::Error> {
        todo!()
    }

    fn skip(&mut self, schema: &Schema) -> SerializerResult<Self::Error> {
        todo!()
    }
}

struct DocumentListParser<'parser> {
    document: Document<'parser>,
}
impl <'parser> DocumentListParser<'parser> {
    pub(super) fn new(schema: &'parser Schema) -> Self {
        DocumentListParser {
            document: Document {
                schema,
                value: DocumentValue::List(Vec::new()),
                discriminator: Some(schema.id().clone()),
            }
        }
    }
}
impl <'lp> ListSerializer<'lp> for DocumentListParser<'lp> {
    type Error = DocumentError;

    fn serialize_element<'a, T>(&mut self, element_schema: &'a Schema, value: &T) -> SerializerResult<Self::Error>
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
}

struct DocumentMapParser<'parser> {
    document: Document<'parser>,
}
impl <'parser> DocumentMapParser<'parser> {
    pub(super) fn new(schema: &'parser Schema) -> Self {
        DocumentMapParser { document: Document {
            schema,
            value: DocumentValue::Map(IndexMap::new()),
            discriminator: Some(schema.id().clone())
        }}
    }
}

impl <'ms> MapSerializer<'ms> for DocumentMapParser<'ms> {
    type Error = DocumentError;

    fn serialize_entry<'a, K, V>(&mut self, key_schema: &'a Schema, value_schema: &Schema, key: &K, value: &V) -> SerializerResult<Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize
    {
        let Schema::Member(me) = key_schema else {
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
}

impl <'s> StructSerializer<'s> for DocumentMapParser<'s> {
    type Error = DocumentError;

    fn serialize_member<'a, T>(&mut self, member_schema: &'a Schema, value: &T) -> SerializerResult<Self::Error>
    where
        T: ?Sized + Serialize,
        'a: 's
    {
        let Schema::Member(me) = member_schema else {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_document_value() {
        let document_str = Document::of("MyStr");
        let output_str = document_str.as_string().expect("string");
        assert_eq!(output_str, &"MyStr".to_string());
        assert_eq!(document_str.schema, &*prelude::STRING);
        let document_string = Document::of("MyString".to_string());
        let output_str = document_string.as_string().expect("string");
        assert_eq!(output_str, &"MyString".to_string());
        assert_eq!(document_string.schema, &*prelude::STRING);
    }

    #[test]
    fn number_document_values() {}

}