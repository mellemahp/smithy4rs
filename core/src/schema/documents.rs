#![allow(dead_code)]
//! # Document Types
//! [Documents](https://smithy.io/2.0/spec/simple-types.html#document) are a protocol-agnostic
//! representation of untyped data in the smithy data model. They function as a kind of "any" type.
//!
//! ### Smithy Data Model
//! The Smithy data model consists of:
//! - Numbers: `byte`, `short`, `integer`, `long`, `float`, `double`, `bigInteger`, `bigDecimal`.
//!   `IntEnum` shapes are represented as integers in the Smithy data model.
//! - `boolean`
//! - `blob`
//! - `string`: `enum` shapes are represented as strings in the Smithy data model
//! - `timestamp`: Represented as an [`Instant`]
//! - `list`: list of Documents
//! - `map`: map of int|long|string keys to Document values
//! - `struct`: structure or union
//!
//! ## Document Properties
//! 1. Shape Conversion - All generated shapes should be able to be converted to/from a document
//! 2. Lossless Serialization: Documents created from a shape should serialize exactly the same as the shape they are created from. I.e.
//!
//! ```rust,ignore
//!    // These two should be equivalent
//!    my_smithy_shape.serialize(serializer)?;
//!    my_smithy_shape.as_document().serialize(serializer)?;
//! ```
//!
//! 3. Lossless Deserialization: Deserializing to a document then converting to a shape should be the same as deserializing to that shape. I.e. :
//!
//! ```rust,ignore
//!    // These two should be equivalent
//!    let my_shape = deserializer::deserialize::<MyShape>::();
//!    let my_shape = deserializer::deserialize::<Document>::().into();
//! ```
//! This is particularly important for dealing with errors, events, and over-the-wire polymorphism.
//!
//! 4. Protocol Smoothing: Documents should try to smooth over any protocol-specific incompatibilities with the smithy data model.
//! 5. Discriminators - Documents with an encloded type should be able to be (de)serialized with data that identifies their
//!
//! ### Protocol Smoothing
//! Because Document types are a protocol-agnostic view of untyped data, Protocol codecs should attempt to smooth over
//! any protocol-specific incompatibilities with the Smithy data model when deserializing into a document type.
//!
//! For example, If a protocol represents a blob as a base64 encoded string, then deserializing the value into a
//! document should base64 decode the value and use the underlying bytes as the blob value in the document.
//! When the document is re-serialized by the same protocol, the protocol should handle conversion from a byte
//! buffer back to a base64 encoded string.
//!
//! ### Lossless serialization
//! Point two *REQUIRES* that documents created from a shape have the exact same schema as that shape.
//!
//! A good example of why is to consider the following Smithy Shape:
//!
//! ```smithy
//! structure MyStruct {
//!     @jsonName("foo")
//!     bar: String
//! }
//! ```
//!
//! When we go to serialize an instance of this shape with a JSON protocol we would expect that the member `bar`
//! is renamed to `foo` in the serialized JSON data:
//!
//! ```rust,ignore
//! let myStructInstance = MyStruct::builder().bar("my-string").build()?;
//! myStructInstance.serialize(jsonSerializer);  // -> Yields `{ "foo": "my-string" }`
//! ```
//!
//! The JSON protocol will achieve this conversion by discovering the `jsonName` trait on the
//! schema of the `myStructInstance` `bar` member.
//!
//! Another good example is the case of a `@sensitive` field on a structure:
//!
//! ```smithy
//! structure MyStructRedacted {
//!    @sensitive
//!    password: String
//! }
//! ```
//!
//! `Display`-ing such a structure should result in the `@sensitive` field being redacted:
//! ```rust,ignore
//! let myStructInstance = MyStruct::builder().password("Secret".to_string()).build()?;
//! assert_eq!("MyStruct[password:**REDACTED**]", format!("{myStructInstance}"))
//! ```
//!
//! We do _not_ want conversion to a document to suddenly start leaking a sensitive field.
//!
//! In order to retain trait information, Documents created from a shape MUST retain that shape’s Schema.
//!
//! ### Discriminators
//! Document types may have a `ShapeId` that indicates the type they correspond to.
//! This ID can be serialized to allow consumers to handle over the wire polymorphism (primarily for over-the wire polymorphism).
//! Typed documents must return the shape ID of the enclosed shape.
//!
//! For example, let’s say we convert a shape of type `com.example#MyShape` into a document type.
//! The document would then store the `ShapeID` as its discriminator.
//!
//! Serializing this document with a JSON protocol might result in a field `__type` being added to the output JSON :
//!
//! ```json
//! {
//!   "__type": "com.example#MyShape",
//!   "other": {}
//! }
//! ```
//!
//! <div class="warning">
//! Discriminators are only really useful for documents (or other structs) being serialized as Structure and Union types.
//! As such they, are serialized by the [`StructSerializer::serialize_discriminator`] method.
//! </div>
//!
//!
//! Similarly, a deserializer might want to pull the `__type` data from a JSON blob when deserializing into a Document type.
//!
//!
//! ## Lossless Deserialization
//!
//! Let’s say we want to deserialize an error type that has the schema:
//! ```smithy
//! @error("client")
//! @http
//! struct MyClientError {
//!    @required
//!    message: String
//!    @jsonName("foo")
//!    bar: String
//! }
//! ```
//!
//! Over the wire we might get a JSON body like:
//!
//! ```json
//! {
//!   "__type": "com.example#MyClientError",
//!   "message": "Something broke pretty bad!",
//!   "foo": "quux!"
//! }
//! ```
//!
//! Because the error response _could_ be one of multiple error types, we don't know the type
//! to deserialize it to in advance.
//! We deserialize the JSON data to a Document, extract the discriminator, and use it to:
//!
//! ```rust,ignore
//! let error_document = Document::deserialize(json_deserializer)?;
//! let builder = error_document.get_builder(error_document.discriminator());
//! let output = builder.deserialize(error_document)?.build();
//! // This sort of type-erasure thing is hard in rust. Maybe using an enum or something
//! // to cast could work? Out of scope for this discussion
//! ```
//!
//! The initial `error_document` has no schema information (beyond the base `Document` schema), so it does not perform any protocol-specific
//! conversions during deserialization (i.e. it won't convert the field name `foo` to `bar` based on the `@jsonName` trait).
//!
//! However, when we deserialize the document into the final Error type we need to execute the protocol-specific handling of the `jsonName` trait.
//!

use std::error::Error;
use std::fmt::{Debug, Formatter};
use bigdecimal::ToPrimitive;
use indexmap::IndexMap;
use thiserror::Error;

use crate::{BigDecimal, BigInt, ByteBuffer, Instant, prelude::*, schema::{Schema, SchemaRef, SchemaShape, ShapeId, ShapeType}, smithy, Ref};

// ============================================================================
// Base Document Wrapper and trait
// ============================================================================

pub type DocumentImpl = Ref<dyn DocumentValue>;

/// A Smithy document type, representing untyped data from the Smithy data model.
///
/// Document types are a protocol-agnostic view of untyped data. Protocols should attempt
/// to smooth over protocol incompatibilities with the Smithy data model.
///
#[derive(Clone, Debug)]
pub struct Document {
    pub(crate) schema: SchemaRef,
    pub(crate) value: DocumentImpl,
    pub(crate) discriminator: Option<ShapeId>,
}

impl PartialEq for Document {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl Document {
    /// Get the Schema of the document
    #[must_use]
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Get the value of the document
    #[must_use]
    pub fn value(&self) -> &DocumentImpl {
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
    /// **NOTE**: Scalar documents always return a size of 1
    /// </div>
    #[must_use]
    pub fn size(&self) -> usize {
        self.value.size()
    }
}

impl SchemaShape for Document {
    fn schema(&self) -> &SchemaRef {
        &self.schema
    }
}

// TODO: Should documents implement iterators?

/// A Smithy document type, representing untyped data from the Smithy data model.
///
/// TODO: DOCS ON MODEL
pub trait DocumentValue: Send + Sync  {
    /// Get the Shape Type for the underlying contents of the document.
    ///
    /// The type returned from this method will differ from the type of the document (which will
    /// be [`ShapeType::Document`]).
    ///
    /// ### Enums
    /// - `enum` shapes: Enum shapes are treated as a `string`, and variants can be found in
    ///   the corresponding schema for the document.
    ///  - `intEnum` shapes: Enum shapes are treated as an `integer`, and variants can be found in
    ///     the corresponding schema for the document.
    #[must_use]
    fn get_type(&self) -> &ShapeType;

    /// Get the number of elements in an array document, or the number of key value pairs in a map document.
    ///
    /// *NOTE*: Should return `0` for all other
    #[must_use]
    fn size(&self) -> usize;

    /// Get the `blob` value of the Document if it is a `blob`.
    ///
    /// Returns `None` if the document could not be converted to a
    /// `blob` ([`ByteBuffer`]) value.
    #[must_use]
    fn as_blob(&self) -> Option<&ByteBuffer>;

    /// Get the `boolean` value of the Document if it is a `boolean` or can be converted into one.
    ///
    /// Returns `None` if the document could not be converted to a
    /// `boolean` value.
    #[must_use]
    fn as_bool(&self) -> Option<bool>;

    /// Get the `string` value of the Document if it is a `string` or can be converted into one.
    ///
    /// Returns `None` if the document could not be converted to a
    /// `string` value.
    #[must_use]
    fn as_string(&self) -> Option<&str>;

    /// Get the `timestamp` value of the Document if it is a `timestamp` or can be converted into one.
    ///
    /// Returns `None` if the document could not be converted to a
    /// `timestamp` ([`Instant`]) value.
    #[must_use]
    fn as_timestamp(&self) -> Option<&Instant>;

    /// Get the `byte` value of the Document if it is a `byte` or can be converted into one.
    ///
    /// Returns `None` if the document could not be converted to a
    /// `byte` (`i8`) value.
    #[must_use]
    fn as_byte(&self) -> Option<i8>;

    /// Get the `short` value of the Document if it is a `short` or can be converted into one.
    ///
    /// Returns `None` if the document could not be converted to a
    /// `short` (`i16`) value.
    #[must_use]
    fn as_short(&self) -> Option<i16>;

    /// Get the `integer` value of the Document if it is an `integer` or can be converted into one.
    ///
    /// Returns `None` if the document could not be converted to an
    /// `integer` (`i32`) value.
    #[must_use]
    fn as_integer(&self) -> Option<i32>;

    /// Get the `long` value of the Document if it is a `long` or can be converted into one.
    ///
    /// Returns `None` if the document could not be converted to a
    /// `long` (`i64`) value.
    #[must_use]
    fn as_long(&self) -> Option<i64>;

    /// Get the `float` value of the Document if it is a `float` or can be converted into one.
    ///
    /// Returns `None` if the document could not be converted to
    /// `float` (`f32`) value.
    #[must_use]
    fn as_float(&self) -> Option<f32>;

    /// Get the `decimal` value of the Document if it is a `decimal` or can be converted into one.
    ///
    /// Returns `None` if the document could not be converted to
    /// `double` (`f64`) value.
    #[must_use]
    fn as_double(&self) -> Option<f64>;

    /// Get the `bigInteger` value of the Document if it is a `bigInteger` or can be converted into one.
    ///
    /// Returns `None` if the document could not be converted to
    /// `bigInteger` ([`BigInt`]) value.
    #[must_use]
    fn as_big_integer(&self) -> Option<&BigInt>;

    /// Get the `bigDecimal` value of the Document if it is a `bigDecimal` or can be converted into one.
    ///
    /// Returns `None` if the document could not be converted to
    /// `bigDecimal` ([`BigDecimal`]) value.
    #[must_use]
    fn as_big_decimal(&self) -> Option<&BigDecimal>;

    /// Get the list contents of the Document if it is a list.
    ///
    /// Returns `None` if the document is not a list.
    #[must_use]
    fn as_list(&self) -> Option<&Vec<Document>>;

    /// Get the map contents of the Document if it is a map.
    ///
    /// Returns `None` if the document is not a map.
    #[must_use]
    fn as_map(&self) -> Option<&IndexMap<String, Document>>;

    /// Returns true if the document represents a `Null` value.
    #[must_use]
    fn is_null(&self) -> bool;
}

impl Debug for dyn DocumentValue + 'static {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
impl PartialEq for dyn DocumentValue + 'static {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

// ============================================================================
// Default Document Implementation
// ============================================================================

#[derive(Clone, PartialEq, Debug)]
pub enum DefaultDocumentValue {
    Null,
    Number(NumberValue),
    Boolean(bool),
    Blob(ByteBuffer),
    String(String),
    Timestamp(Instant),
    List(Vec<Document>),
    Map(IndexMap<String, Document>),
}

impl DocumentValue for DefaultDocumentValue {
    fn get_type(&self) -> &ShapeType {
        todo!()
    }

    fn size(&self) -> usize {
        todo!()
    }

    fn as_blob(&self) -> Option<&ByteBuffer> {
        if let Self::Blob(b) = &self {
            Some(b)
        } else {
            None
        }
    }

    fn as_bool(&self) -> Option<bool> {
        if let Self::Boolean(b) = &self {
            Some(*b)
        } else {
            None
        }
    }

    fn as_string(&self) -> Option<&str> {
        if let Self::String(s) = &self {
            Some(s.as_str())
        } else {
            None
        }
    }

    fn as_timestamp(&self) -> Option<&Instant> {
        if let Self::Timestamp(t) = self {
            Some(t)
        } else {
            None
        }
    }

    fn as_byte(&self) -> Option<i8> {
        match self {
            Self::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Some(b),
                &NumberInteger::Short(s) => s.try_into().ok(),
                &NumberInteger::Integer(i) => i.try_into().ok(),
                &NumberInteger::Long(l) => l.try_into().ok(),
                NumberInteger::BigInt(b) => b.to_i8(),
            },
            _ => None,
        }
    }

    fn as_short(&self) -> Option<i16> {
        match self {
            Self::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Some(b.into()),
                &NumberInteger::Short(s) => Some(s),
                &NumberInteger::Integer(i) => i.try_into().ok(),
                &NumberInteger::Long(l) => l.try_into().ok(),
                NumberInteger::BigInt(b) => b.to_i16(),
            },
            _ => None,
        }
    }

    fn as_integer(&self) -> Option<i32> {
        match self {
            Self::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Some(b.into()),
                &NumberInteger::Short(s) => Some(s.into()),
                &NumberInteger::Integer(i) => Some(i),
                &NumberInteger::Long(l) => l.try_into().ok(),
                NumberInteger::BigInt(b) => b.to_i32(),
            },
            _ => None,
        }
    }

    fn as_long(&self) -> Option<i64> {
        match self {
            Self::Number(NumberValue::Integer(ni)) => match ni {
                &NumberInteger::Byte(b) => Some(b.into()),
                &NumberInteger::Short(s) => Some(s.into()),
                &NumberInteger::Integer(i) => Some(i.into()),
                &NumberInteger::Long(l) => Some(l),
                NumberInteger::BigInt(b) => b.to_i64(),
            },
            _ => None,
        }
    }

    fn as_float(&self) -> Option<f32> {
        match self {
            Self::Number(NumberValue::Float(nf)) => match nf {
                &NumberFloat::Float(f) => Some(f),
                &NumberFloat::Double(d) => Some(d as f32),
                NumberFloat::BigDecimal(b) => b.to_f32(),
            },
            _ => None,
        }
    }

    fn as_double(&self) -> Option<f64> {
        match self {
            Self::Number(NumberValue::Float(nf)) => match nf {
                &NumberFloat::Float(f) => Some(f.into()),
                &NumberFloat::Double(d) => Some(d),
                NumberFloat::BigDecimal(b) => b.to_f64(),
            },
            _ => None,
        }
    }

    fn as_big_integer(&self) -> Option<&BigInt> {
        todo!()
    }

    fn as_big_decimal(&self) -> Option<&BigDecimal> {
        todo!()
    }

    fn as_list(&self) -> Option<&Vec<Document>> {
        if let Self::List(document_list) = self {
            Some(document_list)
        } else {
            None
        }
    }

    fn as_map(&self) -> Option<&IndexMap<String, Document>> {
        if let Self::Map(document_map) = self {
            Some(document_map)
        } else {
            None
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

/// Represents numbers in the smithy data model
///
/// Smithy numbers types include: byte, short, integer, long, float, double, bigInteger, bigDecimal.
///
/// <div class ="note">
/// **NOTE**: `IntEnum` shapes are represented as integers in the Smithy data model.
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

/// Errors that can occur when converting to/from a document type.
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
// Conversions INTO Document types
// ============================================================================
impl From<DefaultDocumentValue> for DocumentImpl {
    fn from(value: DefaultDocumentValue) -> Self {
        Ref::new(value)
    }
}

impl From<bool> for Document {
    fn from(value: bool) -> Self {
        Document {
            schema: BOOLEAN.clone(),
            value: DefaultDocumentValue::Boolean(value).into(),
            discriminator: None,
        }
    }
}

impl From<i8> for Document {
    fn from(value: i8) -> Self {
        Document {
            schema: BYTE.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::Byte(value))).into(),
            discriminator: None,
        }
    }
}

impl From<i16> for Document {
    fn from(value: i16) -> Self {
        Document {
            schema: SHORT.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::Short(value))).into(),
            discriminator: None,
        }
    }
}

impl From<i32> for Document {
    fn from(value: i32) -> Self {
        Document {
            schema: INTEGER.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::Integer(value))).into(),
            discriminator: None,
        }
    }
}

impl From<i64> for Document {
    fn from(value: i64) -> Self {
        Document {
            schema: LONG.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::Long(value))).into(),
            discriminator: None,
        }
    }
}

impl From<f32> for Document {
    fn from(value: f32) -> Self {
        Document {
            schema: FLOAT.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Float(NumberFloat::Float(value))).into(),
            discriminator: None,
        }
    }
}

impl From<f64> for Document {
    fn from(value: f64) -> Self {
        Document {
            schema: DOUBLE.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Float(NumberFloat::Double(value))).into(),
            discriminator: None,
        }
    }
}

impl From<&str> for Document {
    fn from(value: &str) -> Self {
        Document {
            schema: STRING.clone(),
            value: DefaultDocumentValue::String(value.to_string()).into(),
            discriminator: None,
        }
    }
}

impl From<BigInt> for Document {
    fn from(value: BigInt) -> Self {
        Document {
            schema: BIG_INTEGER.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::BigInt(value))).into(),
            discriminator: None,
        }
    }
}

impl From<BigDecimal> for Document {
    fn from(value: BigDecimal) -> Self {
        Document {
            schema: BIG_DECIMAL.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Float(NumberFloat::BigDecimal(value))).into(),
            discriminator: None,
        }
    }
}

impl From<ByteBuffer> for Document {
    fn from(value: ByteBuffer) -> Self {
        Document {
            schema: BLOB.clone(),
            value: DefaultDocumentValue::Blob(value).into(),
            discriminator: None,
        }
    }
}

impl From<String> for Document {
    fn from(value: String) -> Self {
        Document {
            schema: STRING.clone(),
            value: DefaultDocumentValue::String(value).into(),
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
            value: DefaultDocumentValue::List(result).into(),
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
            value: DefaultDocumentValue::Map(result).into(),
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
