#![allow(dead_code)]

use std::error::Error;

use bigdecimal::ToPrimitive;
use indexmap::IndexMap;
use thiserror::Error;

use crate::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    prelude::*,
    schema::{Schema, SchemaRef, SchemaShape, ShapeId, ShapeType},
    smithy,
};

/// A Smithy document type, representing untyped data from the Smithy data model.
///
/// Document types are a protocol-agnostic view of untyped data. Protocols should attempt
/// to smooth over protocol incompatibilities with the Smithy data model.
///
#[derive(Clone, PartialEq, Debug)]
pub struct Document {
    pub(crate) schema: SchemaRef,
    pub(crate) value: DocumentValue,
    pub(crate) discriminator: Option<ShapeId>,
}

impl Document {
    /// Get the Schema of the document
    #[must_use]
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Get the value of the document
    #[must_use]
    pub fn value(&self) -> &DocumentValue {
        &self.value
    }

    /// Get the discriminator (type ID) of a type document
    ///
    /// The discriminator is primarily used to implement polymorphism using documents in deserialization.
    ///
    /// <div class="warning">
    /// It is expected that protocols set the discriminator on deserialization if applicable
    /// </div>
    #[must_use]
    pub fn discriminator(&self) -> Option<&ShapeId> {
        self.discriminator.as_ref()
    }

    /// Get the size of the document.
    ///
    /// <div class ="note">
    /// Scalar documents always return a size of 1
    /// </div>
    #[must_use]
    pub fn size(&self) -> usize {
        match self.value {
            DocumentValue::List(ref array) => array.len(),
            DocumentValue::Map(ref map) => map.len(),
            DocumentValue::Null => 0,
            _ => 1,
        }
    }
}

impl SchemaShape for Document {
    fn schema(&self) -> &SchemaRef {
        &self.schema
    }
}

// TODO: Should documents implement iterators?

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
/// <div class ="note">
/// `IntEnum` shapes are represented as integers in the Smithy data model.
/// </div>
#[derive(Debug, Clone, PartialEq)]
pub enum NumberValue {
    Integer(NumberInteger),
    Float(NumberFloat),
}

impl NumberValue {
    pub const fn from_i8(value: i8) -> Self {
        Self::Integer(NumberInteger::Byte(value))
    }

    pub const fn from_i16(value: i16) -> Self {
        Self::Integer(NumberInteger::Short(value))
    }

    pub const fn from_i32(value: i32) -> Self {
        Self::Integer(NumberInteger::Integer(value))
    }

    pub const fn from_i64(value: i64) -> Self {
        Self::Integer(NumberInteger::Long(value))
    }

    pub const fn from_big_int(value: BigInt) -> Self {
        Self::Integer(NumberInteger::BigInt(value))
    }

    pub const fn from_f32(value: f32) -> Self {
        Self::Float(NumberFloat::Float(value))
    }

    pub const fn from_f64(value: f64) -> Self {
        Self::Float(NumberFloat::Double(value))
    }

    pub const fn from_big_decimal(value: BigDecimal) -> Self {
        Self::Float(NumberFloat::BigDecimal(value))
    }
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

#[derive(Error, Debug, Default)]
pub enum DocumentError {
    #[error("Failed to convert document to type {0}")]
    DocumentSerialization(String),
    #[error("Failed to convert document to type {0}")]
    DocumentConversion(String),
    #[error("Encountered unknown error")]
    Unknown(#[from] Box<dyn Error>),
    #[error("Encountered error: {0}")]
    CustomError(String),
    #[default]
    #[error("Whooopsie")]
    Default,
}

impl crate::serde::de::Error for DocumentError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        DocumentError::CustomError(msg.to_string())
    }
}

/// Get the shape type of the Document
///
/// If the Document is a member, then returns the type of the member target.
pub(crate) fn get_shape_type(schema: &Schema) -> Result<&ShapeType, Box<dyn Error>> {
    let shape_type = schema.shape_type();
    if shape_type == &ShapeType::Member {
        let Some(member) = schema.as_member() else {
            return Err(conversion_error(
                "Expected memberSchema for member shape type",
            ));
        };
        Ok(member.target.shape_type())
    } else {
        Ok(shape_type)
    }
}

pub(crate) fn conversion_error(expected: &'static str) -> Box<dyn Error> {
    Box::new(DocumentError::DocumentConversion(expected.to_string()))
}

// ============================================================================
// Document Number Comparison
// ============================================================================

// TODO(numeric comparisons): Add comparisons between numeric types.

// ============================================================================
// AS-ers to borrow document value as type if possible
// ============================================================================
impl Document {
    /// Get the blob value of the Document if it is a blob.
    #[must_use]
    pub fn as_blob(&self) -> Option<&ByteBuffer> {
        if let DocumentValue::Blob(b) = &self.value {
            Some(b)
        } else {
            None
        }
    }

    /// Get the boolean value of the Document if it is a boolean.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        if let &DocumentValue::Boolean(b) = &self.value {
            Some(b)
        } else {
            None
        }
    }

    /// Get the string value of the Document if it is a string.
    #[must_use]
    pub fn as_string(&self) -> Option<&String> {
        if let DocumentValue::String(s) = &self.value {
            Some(s)
        } else {
            None
        }
    }

    // TODO(numeric comparisons): I dont think these number conversions are right.
    //      Just placeholders for now to get things working

    /// Get the timestamp value of the Document if it is a timestamp.
    #[must_use]
    pub fn as_timestamp(&self) -> Option<&Instant> {
        match &self.value {
            DocumentValue::Timestamp(t) => Some(t),
            _ => None,
        }
    }

    /// Get the byte value of the Document if it is a byte or can be converted into one.
    #[must_use]
    pub fn as_byte(&self) -> Option<i8> {
        match &self.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Some(b),
                &NumberInteger::Short(s) => s.try_into().ok(),
                &NumberInteger::Integer(i) => i.try_into().ok(),
                &NumberInteger::Long(l) => l.try_into().ok(),
                NumberInteger::BigInt(b) => b.to_i8(),
            },
            _ => None,
        }
    }

    /// Get the short value of the Document if it is a short or can be converted into one.
    #[must_use]
    pub fn as_short(&self) -> Option<i16> {
        match &self.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Some(b.into()),
                &NumberInteger::Short(s) => Some(s),
                &NumberInteger::Integer(i) => i.try_into().ok(),
                &NumberInteger::Long(l) => l.try_into().ok(),
                NumberInteger::BigInt(b) => b.to_i16(),
            },
            _ => None,
        }
    }

    /// Get the integer value of the Document if it is an integer or can be converted into one.
    #[must_use]
    pub fn as_integer(&self) -> Option<i32> {
        match &self.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Some(b.into()),
                &NumberInteger::Short(s) => Some(s.into()),
                &NumberInteger::Integer(i) => Some(i),
                &NumberInteger::Long(l) => l.try_into().ok(),
                NumberInteger::BigInt(b) => b.to_i32(),
            },
            _ => None,
        }
    }

    /// Get the long value of the Document if it is a long or can be converted into one.
    #[must_use]
    pub fn as_long(&self) -> Option<i64> {
        match &self.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Some(b.into()),
                &NumberInteger::Short(s) => Some(s.into()),
                &NumberInteger::Integer(i) => Some(i.into()),
                &NumberInteger::Long(l) => Some(l),
                NumberInteger::BigInt(b) => b.to_i64(),
            },
            _ => None,
        }
    }

    /// Get the float value of the Document if it is a float or can be converted into one.
    #[must_use]
    pub fn as_float(&self) -> Option<f32> {
        match &self.value {
            DocumentValue::Number(NumberValue::Float(nf)) => match nf {
                &NumberFloat::Float(f) => Some(f),
                &NumberFloat::Double(d) => Some(d as f32),
                NumberFloat::BigDecimal(b) => b.to_f32(),
            },
            _ => None,
        }
    }

    /// Get the decimal value of the Document if it is a decimal or can be converted into one.
    #[must_use]
    pub fn as_double(&self) -> Option<f64> {
        match &self.value {
            DocumentValue::Number(NumberValue::Float(nf)) => match nf {
                &NumberFloat::Float(f) => Some(f.into()),
                &NumberFloat::Double(d) => Some(d),
                NumberFloat::BigDecimal(b) => b.to_f64(),
            },
            _ => None,
        }
    }

    #[must_use]
    pub fn as_big_integer(&self) -> Option<&BigInt> {
        todo!()
    }

    #[must_use]
    pub fn as_big_decimal(&self) -> Option<&BigDecimal> {
        todo!()
    }

    #[must_use]
    pub fn as_list(&self) -> Option<&Vec<Document>> {
        if let DocumentValue::List(document_list) = &self.value {
            Some(document_list)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_map(&self) -> Option<&IndexMap<String, Document>> {
        if let DocumentValue::Map(document_map) = &self.value {
            Some(document_map)
        } else {
            None
        }
    }
}

// ============================================================================
// Conversions of documents to other types
// ============================================================================

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

// TODO(numeric comparisons): Make Number conversions smarter? Or does rust `as` method handle truncation and such?
impl TryFrom<Document> for i8 {
    type Error = DocumentError;

    fn try_from(value: Document) -> Result<Self, Self::Error> {
        match value.value {
            DocumentValue::Number(NumberValue::Integer(ni)) => match ni {
                NumberInteger::Byte(b) => Ok(b),
                NumberInteger::Short(s) => Ok(s as i8),
                NumberInteger::Integer(i) => Ok(i as i8),
                NumberInteger::Long(l) => Ok(l as i8),
                NumberInteger::BigInt(b) => b
                    .to_i8()
                    .ok_or(DocumentError::DocumentConversion("i8".to_string())),
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
                NumberInteger::BigInt(b) => b
                    .to_i16()
                    .ok_or(DocumentError::DocumentConversion("i16".to_string())),
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
                NumberInteger::Byte(b) => Ok(b.into()),
                NumberInteger::Short(s) => Ok(s.into()),
                NumberInteger::Integer(i) => Ok(i),
                NumberInteger::Long(l) => Ok(l as i32),
                NumberInteger::BigInt(b) => b
                    .to_i32()
                    .ok_or(DocumentError::DocumentConversion("i32".to_string())),
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
                NumberInteger::Byte(b) => Ok(b.into()),
                NumberInteger::Short(s) => Ok(s.into()),
                NumberInteger::Integer(i) => Ok(i.into()),
                NumberInteger::Long(l) => Ok(l),
                NumberInteger::BigInt(b) => b
                    .to_i64()
                    .ok_or(DocumentError::DocumentConversion("i64".to_string())),
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
                NumberFloat::BigDecimal(b) => b
                    .to_f32()
                    .ok_or(DocumentError::DocumentConversion("f32".to_string())),
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
                NumberFloat::Float(f) => Ok(f64::from(f)),
                NumberFloat::Double(d) => Ok(d),
                NumberFloat::BigDecimal(b) => b
                    .to_f64()
                    .ok_or(DocumentError::DocumentConversion("f64".to_string())),
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

impl<T: TryFrom<Document, Error = DocumentError>> TryFrom<Document> for Vec<T> {
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

impl<T: TryFrom<Document, Error = DocumentError>> TryFrom<Document> for IndexMap<String, T> {
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

// ============================================================================
// Conversions INTO Document types
// ============================================================================

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
            schema: DOUBLE.clone(),
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

smithy!("smithy.api#Document": {
    list LIST_DOCUMENT_SCHEMA {
        member: DOCUMENT
    }
});

impl<T: Into<Document>> From<Vec<T>> for Document {
    fn from(value: Vec<T>) -> Self {
        let result = value.into_iter().map(Into::into).collect();
        Document {
            schema: LIST_DOCUMENT_SCHEMA.clone(),
            value: DocumentValue::List(result),
            discriminator: None,
        }
    }
}

smithy!("smithy.api#Document": {
    map MAP_DOCUMENT_SCHEMA {
        key: STRING
        value: DOCUMENT
    }
});
impl<T: Into<Document>> From<IndexMap<String, T>> for Document {
    fn from(value: IndexMap<String, T>) -> Self {
        let mut result = IndexMap::new();
        for (key, value) in value {
            result.insert(key, value.into());
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
        let val: &Schema = &STRING;
        assert_eq!(document_str.schema(), val);
        let output_str: String = document_str.try_into().unwrap();
        assert_eq!(output_str, "MyStr".to_string());

        let document_string: Document = "MyString".into();
        assert_eq!(document_string.schema(), val);
        let output_string: String = document_string.try_into().unwrap();
        assert_eq!(&output_string, &"MyString");
    }

    #[test]
    fn list_document_value() {
        let vec = vec!["a", "b", "c"];
        let document_list: Document = vec.into();
        let val: &Schema = &LIST_DOCUMENT_SCHEMA;
        assert_eq!(document_list.schema(), val);
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
        let val: &Schema = &MAP_DOCUMENT_SCHEMA;
        assert_eq!(map_doc.schema(), val);
        assert_eq!(map_doc.size(), 1);

        let map_out: IndexMap<String, String> = map_doc.try_into().unwrap();
        assert_eq!(map_out.len(), 1);
        assert_eq!(map_out["a"], "b");
    }

    #[test]
    fn integer_document_values() {
        let byte: Document = 1i8.into();
        let byte_val: &Schema = &BYTE;
        assert_eq!(byte.schema(), byte_val);

        let short: Document = 1i16.into();
        let short_val: &Schema = &SHORT;

        assert_eq!(short.schema(), short_val);

        let integer: Document = 1i32.into();
        let integer_val: &Schema = &INTEGER;

        assert_eq!(integer.schema(), integer_val);

        let long: Document = 1i64.into();
        let long_val: &Schema = &LONG;

        assert_eq!(long.schema(), long_val);

        let byte_value: i8 = byte.try_into().unwrap();
        assert_eq!(byte_value, 1i8);
        let short_value: i16 = short.try_into().unwrap();
        assert_eq!(short_value, 1i16);
        let integer_value: i32 = integer.try_into().unwrap();
        assert_eq!(integer_value, 1i32);
        let long_value: i64 = long.try_into().unwrap();
        assert_eq!(long_value, 1i64);
    }

    // TODO(numeric comparisons): Add comparison checks

    #[test]
    fn float_document_values() {
        let float: Document = 1f32.into();
        let float_val: &Schema = &FLOAT;
        assert_eq!(float.schema(), float_val);

        let double: Document = 1f64.into();
        let double_val: &Schema = &DOUBLE;
        assert_eq!(double.schema(), double_val);

        let float_value: f32 = float.try_into().unwrap();
        assert_eq!(float_value, 1f32);
        let double_value: f64 = double.try_into().unwrap();
        assert_eq!(double_value, 1f64);
    }
}
