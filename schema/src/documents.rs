#![allow(dead_code)]

use crate::{lazy_schema, traits, Schema, SchemaRef, ShapeId, ShapeType, BigDecimal, BigInt, ByteBuffer};
use crate::prelude::*;
use indexmap::IndexMap;
use std::error::Error;
use std::sync::LazyLock;
use std::time::Instant;
use thiserror::Error;

/// A Smithy document type, representing untyped data from the Smithy data model.
///
/// Document types are a protocol-agnostic view of untyped data. Protocols should attempt
/// to smooth over protocol incompatibilities with the Smithy data model.
#[derive(Clone, PartialEq, Debug)]
pub struct Document {
    schema: SchemaRef,
    value: DocumentValue,
    discriminator: Option<ShapeId>,
}
impl Document {
    /// Get the Schema of the document
    pub fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    /// Get the value of the document
    pub fn value(&self) -> &DocumentValue {
        &self.value
    }

    /// Get the discriminator (type ID) of a type document
    ///
    /// The discriminator is primarily used to implement polymorphism using documents in deserialization.
    ///
    /// *Impl note*: It is expected that protocols set the discriminator on deserialization if applicable
    pub fn discriminator(&self) -> &Option<ShapeId> {
        &self.discriminator
    }

    /// Get the size of the document.
    ///
    /// **NOTE**: Scalar documents always return a size of 1
    pub fn size(&self) -> usize {
        match self.value {
            DocumentValue::List(ref array) => array.len(),
            DocumentValue::Map(ref map) => map.len(),
            DocumentValue::Null => 0,
            _ => 1,
        }
    }
}

// TODO: Should documents implement iterators?

// TODO: Use a blanket impl per: https://www.greyblake.com/blog/alternative-blanket-implementations-for-single-rust-trait/
/// Marker trait to distinguish documents from other [`SerializeShape`]'s
pub trait DynamicShape: Sized {}
impl DynamicShape for Document {}

/// A Smithy document type, representing untyped data from the Smithy data model.
#[derive(Clone, PartialEq, Debug)]
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

/// Represents an Integer numeric types in the Smithy data model
#[derive(Debug, Clone, PartialEq)]
pub enum NumberInteger {
    Byte(i8),
    Short(i16),
    Integer(i32),
    Long(i64),
    BigInt(BigInt),
}

/// Represents Floating-point numeric type in the Smithy data model
#[derive(Debug, Clone, PartialEq)]
pub enum NumberFloat {
    Float(f32),
    Double(f64),
    BigDecimal(BigDecimal),
}

// TODO: DOCS
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
    Default,
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

// // ====== TRY INTO conversions =====
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

impl <T: TryFrom<Document, Error=DocumentError>> TryFrom<Document> for Vec<T> {
    type Error = DocumentError;

    fn try_from(value: Document) -> Result<Self, Self::Error> {
        let DocumentValue::List(vec) = value.value else {
            return Err(DocumentError::DocumentConversion("Vec".to_string()));
        };
        let mut result: Vec<T> = Vec::new();
        for doc in vec {
            match T::try_from(doc) {
                Ok(val) => result.push(val),
                Err(e) => return Err(e),
            }
        }
        Ok(result)
    }
}

impl <T: TryFrom<Document, Error=DocumentError>> TryFrom<Document> for IndexMap<String, T> {
    type Error = DocumentError;

    fn try_from(value: Document) -> Result<Self, Self::Error> {
        let DocumentValue::Map(map) = value.value else {
            return Err(DocumentError::DocumentConversion("Map".to_string()));
        };
        let mut result: IndexMap<String, T> = IndexMap::new();
        for (key, val) in map {
            let _ = match T::try_from(val) {
                Ok(val) => result.insert(key, val),
                Err(e) => return Err(e),
            };
        }
        Ok(result)
    }
}

// ==== FROM impls ====
impl From<bool> for Document {
    fn from(value: bool) -> Self {
        Document {
            schema: BOOLEAN.clone(),
            value: DocumentValue::Boolean(value),
            discriminator: None,
        }
    }
}

impl From<i8> for Document {
    fn from(value: i8) -> Self {
        Document {
            schema: BYTE.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Byte(value))),
            discriminator: None,
        }
    }
}

impl From<i16> for Document {
    fn from(value: i16) -> Self {
        Document {
            schema: SHORT.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Short(value))),
            discriminator: None,
        }
    }
}

impl From<i32> for Document {
    fn from(value: i32) -> Self {
        Document {
            schema: INTEGER.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Integer(value))),
            discriminator: None,
        }
    }
}

impl From<i64> for Document {
    fn from(value: i64) -> Self {
        Document {
            schema: LONG.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Long(value))),
            discriminator: None,
        }
    }
}

impl From<f32> for Document {
    fn from(value: f32) -> Self {
        Document {
            schema: FLOAT.clone(),
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::Float(value))),
            discriminator: None,
        }
    }
}

impl From<f64> for Document {
    fn from(value: f64) -> Self {
        Document {
            schema: FLOAT.clone(),
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::Double(value))),
            discriminator: None,
        }
    }
}

impl From<&str> for Document {
    fn from(value: &str) -> Self {
        Document {
            schema: STRING.clone(),
            value: DocumentValue::String(value.to_string()),
            discriminator: None,
        }
    }
}

impl From<BigInt> for Document {
    fn from(value: BigInt) -> Self {
        Document {
            schema: BIG_INTEGER.clone(),
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::BigInt(value))),
            discriminator: None,
        }
    }
}

impl From<BigDecimal> for Document {
    fn from(value: BigDecimal) -> Self {
        Document {
            schema: BIG_DECIMAL.clone(),
            value: DocumentValue::Number(NumberValue::Float(NumberFloat::BigDecimal(value))),
            discriminator: None,
        }
    }
}

impl From<ByteBuffer> for Document {
    fn from(value: ByteBuffer) -> Self {
        Document {
            schema: BLOB.clone(),
            value: DocumentValue::Blob(value),
            discriminator: None,
        }
    }
}

impl From<String> for Document {
    fn from(value: String) -> Self {
        Document {
            schema: STRING.clone(),
            value: DocumentValue::String(value),
            discriminator: None,
        }
    }
}

lazy_schema!(
    LIST_DOCUMENT_SCHEMA,
    Schema::list_builder(DOCUMENT.id().clone())
        .put_member("member", &DOCUMENT, traits![])
        .build()
);
impl<'a, T: Into<Document>> From<Vec<T>> for Document {
    fn from(value: Vec<T>) -> Self {
        let result = value.into_iter().map(Into::into).collect();
        Document {
            schema: LIST_DOCUMENT_SCHEMA.clone(),
            value: DocumentValue::List(result),
            discriminator: None,
        }
    }
}

lazy_schema!(
    MAP_DOCUMENT_SCHEMA,
    Schema::map_builder(DOCUMENT.id().clone())
        .put_member("key", &STRING, traits![])
        .put_member("value", &DOCUMENT, traits![])
        .build()
);
impl<'a, T: Into<Document>> From<IndexMap<String, T>> for Document {
    fn from(value: IndexMap<String, T>) -> Self {
        let mut result = IndexMap::new();
        for (key, value) in value.into_iter() {
            result.insert(key.into(), value.into());
        }
        Document {
            schema: MAP_DOCUMENT_SCHEMA.clone(),
            value: DocumentValue::Map(result),
            discriminator: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_document_value() {
        let document_str: Document = "MyStr".into();
        assert_eq!(document_str.schema(), &*STRING);
        let output_str: String = document_str.try_into().unwrap();
        assert_eq!(output_str, "MyStr".to_string());

        let document_string: Document = "MyString".into();
        assert_eq!(document_string.schema(), &*STRING);
        let output_string: String = document_string.try_into().unwrap();
        assert_eq!(&output_string, &"MyString");
    }

    #[test]
    fn list_document_value() {
        let vec = vec!["a", "b", "c"];
        let document_list: Document = vec.into();
        assert_eq!(document_list.schema(), &*LIST_DOCUMENT_SCHEMA);
        assert_eq!(document_list.size(), 3);
        let vec_out: Vec<String> = document_list.try_into().unwrap();
        assert_eq!(vec_out.len(), 3);
        assert_eq!(vec_out[0], "a");
        assert_eq!(vec_out[1], "b");
        assert_eq!(vec_out[2], "c");
    }

    #[test]
    fn map_document_value() {
        let mut map_in: IndexMap<String, String> = IndexMap::new();
        map_in.insert("a".to_string(), "b".to_string());
        let map_doc: Document = map_in.into();
        assert_eq!(map_doc.schema(), &*MAP_DOCUMENT_SCHEMA);
        assert_eq!(map_doc.size(), 1);

        let map_out: IndexMap<String, String> = map_doc.try_into().unwrap();
        assert_eq!(map_out.len(), 1);
        assert_eq!(map_out["a"], "b");
    }

    #[test]
    fn number_document_values() {
        // TODO
    }
}
