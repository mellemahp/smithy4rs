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

use std::{
    error::Error,
    fmt::{Debug, Formatter},
    sync::LazyLock,
};

use bigdecimal::ToPrimitive;
use indexmap::IndexMap;
use thiserror::Error;

use crate::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    prelude::*,
    schema::{SchemaRef, SchemaShape, ShapeId, ShapeType},
    smithy,
};

// ============================================================================
// Base Document Wrapper and trait
// ============================================================================

/// A Smithy document type, representing untyped data from the Smithy data model.
///
/// Document types are a protocol-agnostic view of untyped data. Protocols should attempt
/// to smooth over protocol incompatibilities with the Smithy data model.
///
pub trait Document: SchemaShape + Send + Sync {
    /// Get the discriminator (type ID) of a type document
    ///
    /// The discriminator is primarily used to implement polymorphism using documents in deserialization.
    ///
    /// <div class="warning">
    /// It is expected that protocols set the discriminator on deserialization if applicable
    /// </div>
    #[must_use]
    fn discriminator(&self) -> Option<&ShapeId>;

    // ========== Basic info Methods ==========

    /// Get the Shape Type for the underlying contents of the document.
    ///
    /// The type returned from this method will differ from the type of the document (which will
    /// always be [`ShapeType::Document`]).
    ///
    /// ### Enums
    /// - `enum` shapes: Enum shapes are treated as a `string`, and variants can be found in
    ///   the corresponding schema for the document.
    ///  - `intEnum` shapes: Enum shapes are treated as an `integer`, and variants can be found in
    ///    the corresponding schema for the document.
    #[must_use]
    #[cold] // cold as this is mostly called for testing.
    fn get_type(&self) -> Option<&ShapeType>;

    /// Get the number of elements in an array document, or the number of key value pairs in a map document.
    ///
    /// *NOTE*: Should return `0` for all other
    #[must_use]
    fn size(&self) -> usize;

    // ========== Borrowed data conversions ==========

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
    fn as_list(&self) -> Option<&Vec<Box<dyn Document>>>;

    /// Get the map contents of the Document if it is a map.
    ///
    /// Returns `None` if the document is not a map.
    #[must_use]
    fn as_map(&self) -> Option<&IndexMap<String, Box<dyn Document>>>;

    /// Returns true if the document represents a `Null` value.
    #[must_use]
    fn is_null(&self) -> bool;

    // ========== Owned data conversions ==========

    /// Convert the [`Document`] to a `blob`.
    ///
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `blob` ([`ByteBuffer`]) value.
    fn into_blob(self: Box<Self>) -> Result<ByteBuffer, DocumentError>;

    /// Convert the [`Document`] to a `boolean`.
    ///
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `boolean` value.
    fn into_bool(self: Box<Self>) -> Result<bool, DocumentError>;

    /// Convert the [`Document`] to a `string`.
    ///
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `string` value.
    fn into_string(self: Box<Self>) -> Result<String, DocumentError>;

    /// Convert the [`Document`] to a `timestamp`.
    ///
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `timestamp` ([`Instant`]) value.
    fn into_timestamp(self: Box<Self>) -> Result<Instant, DocumentError>;

    /// Convert the [`Document`] to a `byte`.
    ///
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `byte` (`i8`) value.
    fn into_byte(self: Box<Self>) -> Result<i8, DocumentError>;

    /// Convert the [`Document`] to a `short`.
    ///
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `short` (`i16`) value.
    fn into_short(self: Box<Self>) -> Result<i16, DocumentError>;

    /// Convert the [`Document`] to an `integer`.
    ///
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `integer` (`i32`) value.
    fn into_integer(self: Box<Self>) -> Result<i32, DocumentError>;

    /// Convert the [`Document`] to a `long`.
    ///
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `long` (`i64`) value.
    fn into_long(self: Box<Self>) -> Result<i64, DocumentError>;

    /// Convert the [`Document`] to a `float`.
    ///
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `float` (`f32`) value.
    fn into_float(self: Box<Self>) -> Result<f32, DocumentError>;

    /// Convert the [`Document`] to a `double`.
    ///
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `double` (`f64`) value.
    fn into_double(self: Box<Self>) -> Result<f64, DocumentError>;

    /// Convert the [`Document`] to a `bigInteger`.
    ///
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `bigInteger` ([`BigInt`]) value.
    fn into_big_integer(self: Box<Self>) -> Result<BigInt, DocumentError>;

    /// Convert the [`Document`] to a `bigDecimal`.
    ///
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `bigDecimal` ([`BigDecimal`]) value.
    fn into_big_decimal(self: Box<Self>) -> Result<BigDecimal, DocumentError>;

    /// Convert the [`Document`] to a `list`.
    ///
    /// Returns a [`DocumentError`] if the document is not a list.
    fn into_list(self: Box<Self>) -> Result<Vec<Box<dyn Document>>, DocumentError>;

    /// Convert the [`Document`] to a `map`.
    ///
    /// Returns a [`DocumentError`] if the document is not a map.
    fn into_map(self: Box<Self>) -> Result<IndexMap<String, Box<dyn Document>>, DocumentError>;

    /// Clone implementation necessary to satisfy the compiler
    fn box_clone(&self) -> Box<dyn Document>;
}

// ================================================================
// Blanket impl for dyn Documents
// ================================================================

impl PartialEq for dyn Document {
    fn eq(&self, other: &Self) -> bool {
        if self.schema() != other.schema() {
            return false;
        }
        if self.discriminator() != other.discriminator() {
            return false;
        }
        // TODO(numerical comparison): We should compare compatible numeric types
        // if possible. This is just a placeholder to get test working.
        match (self.get_type(), other.get_type()) {
            (Some(ShapeType::String), Some(ShapeType::String)) => {
                self.as_string() == other.as_string()
            }
            (Some(ShapeType::Blob), Some(ShapeType::Blob)) => self.as_blob() == other.as_blob(),
            (Some(ShapeType::Boolean), Some(ShapeType::Boolean)) => {
                self.as_bool() == other.as_bool()
            }
            (Some(ShapeType::Timestamp), Some(ShapeType::Timestamp)) => {
                self.as_timestamp() == other.as_timestamp()
            }
            (Some(ShapeType::Byte), Some(ShapeType::Byte)) => self.as_byte() == other.as_byte(),
            (Some(ShapeType::Short), Some(ShapeType::Short)) => self.as_short() == other.as_short(),
            (Some(ShapeType::Integer), Some(ShapeType::Integer)) => {
                self.as_integer() == other.as_integer()
            }
            (Some(ShapeType::Long), Some(ShapeType::Long)) => self.as_long() == other.as_long(),
            (Some(ShapeType::Float), Some(ShapeType::Float)) => self.as_float() == other.as_float(),
            (Some(ShapeType::Double), Some(ShapeType::Double)) => {
                self.as_double() == other.as_double()
            }
            (Some(ShapeType::BigInteger), Some(ShapeType::BigInteger)) => {
                self.as_big_integer() == other.as_big_integer()
            }
            (Some(ShapeType::BigDecimal), Some(ShapeType::BigDecimal)) => {
                self.as_big_decimal() == other.as_big_decimal()
            }
            (Some(ShapeType::List), Some(ShapeType::List)) => self.as_map() == other.as_map(),
            (Some(ShapeType::Map), Some(ShapeType::Map)) => self.as_map() == other.as_map(),
            (None, None) => true,
            _ => false,
        }
    }
}

impl Clone for Box<dyn Document> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl Debug for dyn Document + 'static {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO(document debug): Add actual impl
        write!(f, "[DocumentValue]")
    }
}

// ============================================================================
// Document Errors
// ============================================================================

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

// ============================================================================
// Default Document Implementation
// ============================================================================
type BoxedDoc = Box<dyn Document>;

#[derive(Clone, PartialEq, Debug)]
pub struct DefaultDocument {
    pub(crate) schema: SchemaRef,
    pub(crate) value: DefaultDocumentValue,
    pub(crate) discriminator: Option<ShapeId>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum DefaultDocumentValue { 
    Null,
    Number(NumberValue),
    Boolean(bool),
    Blob(ByteBuffer),
    String(String),
    Timestamp(Instant),
    List(Vec<BoxedDoc>),
    Map(IndexMap<String, BoxedDoc>),
}

impl SchemaShape for DefaultDocument {
    fn schema(&self) -> &SchemaRef {
        &self.schema
    }
}

impl Document for DefaultDocument {
    fn discriminator(&self) -> Option<&ShapeId> {
        self.discriminator.as_ref()
    }

    fn get_type(&self) -> Option<&ShapeType> {
        match &self.value {
            DefaultDocumentValue::Number(n) => match n {
                NumberValue::Integer(i) => match i {
                    NumberInteger::Byte(_) => Some(&ShapeType::Byte),
                    NumberInteger::Short(_) => Some(&ShapeType::Short),
                    NumberInteger::Integer(_) => Some(&ShapeType::Integer),
                    NumberInteger::Long(_) => Some(&ShapeType::Long),
                    NumberInteger::BigInt(_) => Some(&ShapeType::BigInteger),
                },
                NumberValue::Float(f) => match f {
                    NumberFloat::Float(_) => Some(&ShapeType::Float),
                    NumberFloat::Double(_) => Some(&ShapeType::Double),
                    NumberFloat::BigDecimal(_) => Some(&ShapeType::BigDecimal),
                },
            },
            DefaultDocumentValue::Boolean(_) => Some(&ShapeType::Boolean),
            DefaultDocumentValue::Blob(_) => Some(&ShapeType::Blob),
            DefaultDocumentValue::String(_) => Some(&ShapeType::String),
            DefaultDocumentValue::Timestamp(_) => Some(&ShapeType::Timestamp),
            DefaultDocumentValue::List(_) => Some(&ShapeType::List),
            DefaultDocumentValue::Map(_) => Some(&ShapeType::Map),
            _ => None,
        }
    }

    fn size(&self) -> usize {
        match &self.value {
            DefaultDocumentValue::List(v) => v.len(),
            DefaultDocumentValue::Map(v) => v.len(),
            _ => 0,
        }
    }

    fn as_blob(&self) -> Option<&ByteBuffer> {
        if let DefaultDocumentValue::Blob(b) = &self.value {
            Some(b)
        } else {
            None
        }
    }

    fn as_bool(&self) -> Option<bool> {
        if let DefaultDocumentValue::Boolean(b) = &self.value {
            Some(*b)
        } else {
            None
        }
    }

    fn as_string(&self) -> Option<&str> {
        if let DefaultDocumentValue::String(s) = &self.value {
            Some(s.as_str())
        } else {
            None
        }
    }

    fn as_timestamp(&self) -> Option<&Instant> {
        if let DefaultDocumentValue::Timestamp(t) = &self.value {
            Some(t)
        } else {
            None
        }
    }

    fn as_byte(&self) -> Option<i8> {
        match &self.value {
            DefaultDocumentValue::Number(NumberValue::Integer(ni)) => match ni {
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
        match &self.value {
            DefaultDocumentValue::Number(NumberValue::Integer(ni)) => match ni {
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
        match &self.value {
            DefaultDocumentValue::Number(NumberValue::Integer(ni)) => match ni {
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
        match &self.value {
            DefaultDocumentValue::Number(NumberValue::Integer(ni)) => match ni {
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
        match &self.value {
            DefaultDocumentValue::Number(NumberValue::Float(nf)) => match nf {
                &NumberFloat::Float(f) => Some(f),
                &NumberFloat::Double(d) => Some(d as f32),
                NumberFloat::BigDecimal(b) => b.to_f32(),
            },
            _ => None,
        }
    }

    fn as_double(&self) -> Option<f64> {
        match &self.value {
            DefaultDocumentValue::Number(NumberValue::Float(nf)) => match nf {
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

    fn as_list(&self) -> Option<&Vec<BoxedDoc>> {
        if let DefaultDocumentValue::List(document_list) = &self.value {
            Some(document_list)
        } else {
            None
        }
    }

    fn as_map(&self) -> Option<&IndexMap<String, BoxedDoc>> {
        if let DefaultDocumentValue::Map(document_map) = &self.value {
            Some(document_map)
        } else {
            None
        }
    }

    fn is_null(&self) -> bool {
        matches!(self.value, DefaultDocumentValue::Null)
    }

    fn into_blob(self: Box<Self>) -> Result<ByteBuffer, DocumentError> {
        if let DefaultDocumentValue::Blob(value) = self.value {
            Ok(value)
        } else {
            Err(DocumentError::DocumentConversion(
                "Expected blob document".to_string(),
            ))
        }
    }

    fn into_bool(self: Box<Self>) -> Result<bool, DocumentError> {
        if let DefaultDocumentValue::Boolean(value) = self.value {
            Ok(value)
        } else {
            Err(DocumentError::DocumentConversion(
                "Expected boolean document".to_string(),
            ))
        }
    }

    fn into_string(self: Box<Self>) -> Result<String, DocumentError> {
        if let DefaultDocumentValue::String(value) = self.value {
            Ok(value)
        } else {
            Err(DocumentError::DocumentConversion(
                "Expected string document".to_string(),
            ))
        }
    }

    fn into_timestamp(self: Box<Self>) -> Result<Instant, DocumentError> {
        if let DefaultDocumentValue::Timestamp(value) = self.value {
            Ok(value)
        } else {
            Err(DocumentError::DocumentConversion(
                "Expected timestamp document".to_string(),
            ))
        }
    }

    // TODO(numeric conversion): Review these numeric conversions.
    fn into_byte(self: Box<Self>) -> Result<i8, DocumentError> {
        self.as_byte()
            .ok_or_else(|| DocumentError::DocumentConversion("Expected byte document".to_string()))
    }

    fn into_short(self: Box<Self>) -> Result<i16, DocumentError> {
        self.as_short()
            .ok_or_else(|| DocumentError::DocumentConversion("Expected short document".to_string()))
    }

    fn into_integer(self: Box<Self>) -> Result<i32, DocumentError> {
        self.as_integer().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected integer document".to_string())
        })
    }

    fn into_long(self: Box<Self>) -> Result<i64, DocumentError> {
        self.as_long()
            .ok_or_else(|| DocumentError::DocumentConversion("Expected long document".to_string()))
    }

    fn into_float(self: Box<Self>) -> Result<f32, DocumentError> {
        self.as_float()
            .ok_or_else(|| DocumentError::DocumentConversion("Expected float document".to_string()))
    }

    fn into_double(self: Box<Self>) -> Result<f64, DocumentError> {
        self.as_double().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected double document".to_string())
        })
    }

    // TODO(numeric conversion): These shouldnt need to clone.
    fn into_big_integer(self: Box<Self>) -> Result<BigInt, DocumentError> {
        self.as_big_integer().cloned().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected bigInteger document".to_string())
        })
    }

    fn into_big_decimal(self: Box<Self>) -> Result<BigDecimal, DocumentError> {
        self.as_big_decimal().cloned().ok_or_else(|| {
            DocumentError::DocumentConversion("Expected bigInteger document".to_string())
        })
    }

    fn into_list(self: Box<Self>) -> Result<Vec<BoxedDoc>, DocumentError> {
        if let DefaultDocumentValue::List(value) = self.value {
            Ok(value)
        } else {
            Err(DocumentError::DocumentConversion(
                "Expected list document".to_string(),
            ))
        }
    }

    fn into_map(self: Box<Self>) -> Result<IndexMap<String, BoxedDoc>, DocumentError> {
        if let DefaultDocumentValue::Map(value) = self.value {
            Ok(value)
        } else {
            Err(DocumentError::DocumentConversion(
                "Expected map document".to_string(),
            ))
        }
    }

    fn box_clone(&self) -> BoxedDoc {
        Box::new(self.clone())
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

// ============================================================================
// Convert Document into types
// ============================================================================

impl TryFrom<BoxedDoc> for ByteBuffer {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        value.into_blob()
    }
}

impl TryFrom<BoxedDoc> for bool {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        value.into_bool()
    }
}

impl TryFrom<BoxedDoc> for String {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        value.into_string()
    }
}

impl TryFrom<BoxedDoc> for Instant {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        value.into_timestamp()
    }
}

impl TryFrom<BoxedDoc> for i8 {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        value.into_byte()
    }
}

impl TryFrom<BoxedDoc> for i16 {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        value.into_short()
    }
}

impl TryFrom<BoxedDoc> for i32 {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        value.into_integer()
    }
}

impl TryFrom<BoxedDoc> for i64 {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        value.into_long()
    }
}

impl TryFrom<BoxedDoc> for f32 {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        value.into_float()
    }
}

impl TryFrom<BoxedDoc> for f64 {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        value.into_double()
    }
}

impl TryFrom<BoxedDoc> for BigInt {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        value.into_big_integer()
    }
}

impl TryFrom<BoxedDoc> for BigDecimal {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        value.into_big_decimal()
    }
}

impl TryFrom<BoxedDoc> for Vec<BoxedDoc> {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        value.into_list()
    }
}

impl TryFrom<BoxedDoc> for IndexMap<String, BoxedDoc> {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        value.into_map()
    }
}

impl<T: TryFrom<BoxedDoc, Error = DocumentError>> TryFrom<BoxedDoc> for Vec<T> {
    type Error = DocumentError;

    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        let vec = value.into_list()?;
        let mut result: Vec<T> = Vec::new();
        for doc in vec {
            match T::try_from(doc.clone()) {
                Ok(val) => result.push(val),
                Err(e) => return Err(e),
            }
        }
        Ok(result)
    }
}

impl<T: TryFrom<BoxedDoc, Error = DocumentError>> TryFrom<BoxedDoc> for IndexMap<String, T> {
    type Error = DocumentError;

    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        let map = value.into_map()?;
        let mut result: IndexMap<String, T> = IndexMap::new();
        for (key, val) in map {
            let _ = match T::try_from(val.clone()) {
                Ok(val) => result.insert(key.to_string(), val),
                Err(e) => return Err(e),
            };
        }
        Ok(result)
    }
}

impl<T: TryFrom<BoxedDoc, Error = DocumentError>> TryFrom<BoxedDoc> for Option<T> {
    type Error = DocumentError;

    fn try_from(value: BoxedDoc) -> Result<Self, Self::Error> {
        if value.is_null() {
            return Ok(None);
        }
        Ok(Some(value.try_into()?))
    }
}

// ============================================================================
// Conversions INTO Default Document
// ============================================================================

impl From<DefaultDocument> for BoxedDoc {
    fn from(value: DefaultDocument) -> Self {
        Box::new(value)
    }
}

impl From<bool> for BoxedDoc {
    fn from(value: bool) -> Self {
        DefaultDocument {
            schema: BOOLEAN.clone(),
            value: DefaultDocumentValue::Boolean(value),
            discriminator: None,
        }
        .into()
    }
}

impl From<i8> for BoxedDoc {
    fn from(value: i8) -> Self {
        DefaultDocument {
            schema: BYTE.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::Byte(value))),
            discriminator: None,
        }
        .into()
    }
}

impl From<i16> for BoxedDoc {
    fn from(value: i16) -> Self {
        DefaultDocument {
            schema: SHORT.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::Short(value))),
            discriminator: None,
        }
        .into()
    }
}

impl From<i32> for BoxedDoc {
    fn from(value: i32) -> Self {
        DefaultDocument {
            schema: INTEGER.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::Integer(
                value,
            ))),
            discriminator: None,
        }
        .into()
    }
}

impl From<i64> for BoxedDoc {
    fn from(value: i64) -> Self {
        DefaultDocument {
            schema: LONG.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::Long(value))),
            discriminator: None,
        }
        .into()
    }
}

impl From<f32> for BoxedDoc {
    fn from(value: f32) -> Self {
        DefaultDocument {
            schema: FLOAT.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Float(NumberFloat::Float(value))),
            discriminator: None,
        }
        .into()
    }
}

impl From<f64> for BoxedDoc {
    fn from(value: f64) -> Self {
        DefaultDocument {
            schema: DOUBLE.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Float(NumberFloat::Double(value))),
            discriminator: None,
        }
        .into()
    }
}

impl From<&str> for BoxedDoc {
    fn from(value: &str) -> Self {
        DefaultDocument {
            schema: STRING.clone(),
            value: DefaultDocumentValue::String(value.to_string()),
            discriminator: None,
        }
        .into()
    }
}

impl From<BigInt> for BoxedDoc {
    fn from(value: BigInt) -> Self {
        DefaultDocument {
            schema: BIG_INTEGER.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::BigInt(value))),
            discriminator: None,
        }
        .into()
    }
}

impl From<BigDecimal> for BoxedDoc {
    fn from(value: BigDecimal) -> Self {
        DefaultDocument {
            schema: BIG_DECIMAL.clone(),
            value: DefaultDocumentValue::Number(NumberValue::Float(NumberFloat::BigDecimal(value))),
            discriminator: None,
        }
        .into()
    }
}

impl From<ByteBuffer> for BoxedDoc {
    fn from(value: ByteBuffer) -> Self {
        DefaultDocument {
            schema: BLOB.clone(),
            value: DefaultDocumentValue::Blob(value),
            discriminator: None,
        }
        .into()
    }
}

impl From<String> for BoxedDoc {
    fn from(value: String) -> Self {
        DefaultDocument {
            schema: STRING.clone(),
            value: DefaultDocumentValue::String(value),
            discriminator: None,
        }
        .into()
    }
}

impl From<Instant> for BoxedDoc {
    fn from(value: Instant) -> Self {
        DefaultDocument {
            schema: TIMESTAMP.clone(),
            value: DefaultDocumentValue::Timestamp(value),
            discriminator: None,
        }
        .into()
    }
}
impl From<&Instant> for BoxedDoc {
    fn from(value: &Instant) -> Self {
        DefaultDocument {
            schema: TIMESTAMP.clone(),
            value: DefaultDocumentValue::Timestamp(*value),
            discriminator: None,
        }
        .into()
    }
}

smithy!("smithy.api#Document": {
    list LIST_DOCUMENT_SCHEMA {
        member: DOCUMENT
    }
});

impl<T: Into<BoxedDoc>> From<Vec<T>> for BoxedDoc {
    fn from(value: Vec<T>) -> Self {
        let result = value.into_iter().map(Into::into).collect();
        DefaultDocument {
            schema: LIST_DOCUMENT_SCHEMA.clone(),
            value: DefaultDocumentValue::List(result),
            discriminator: None,
        }
        .into()
    }
}

smithy!("smithy.api#Document": {
    map MAP_DOCUMENT_SCHEMA {
        key: STRING
        value: DOCUMENT
    }
});
impl<T: Into<BoxedDoc>> From<IndexMap<String, T>> for BoxedDoc {
    fn from(value: IndexMap<String, T>) -> Self {
        let mut result = IndexMap::new();
        for (key, value) in value {
            result.insert(key, value.into());
        }
        DefaultDocument {
            schema: MAP_DOCUMENT_SCHEMA.clone(),
            value: DefaultDocumentValue::Map(result),
            discriminator: None,
        }
        .into()
    }
}
macro_rules! option_conversion {
    ($ty:ty, $schema:ident) => {
        impl From<Option<$ty>> for BoxedDoc {
            fn from(value: Option<$ty>) -> Self {
                value.map_or_else(
                    || {
                        DefaultDocument {
                            schema: $schema.clone(),
                            value: DefaultDocumentValue::Null,
                            discriminator: None,
                        }
                        .into()
                    },
                    Into::into,
                )
            }
        }
    };
}
option_conversion!(String, STRING);
option_conversion!(bool, BOOLEAN);
option_conversion!(Instant, TIMESTAMP);
option_conversion!(ByteBuffer, BLOB);
option_conversion!(i8, BYTE);
option_conversion!(i16, SHORT);
option_conversion!(i32, INTEGER);
option_conversion!(i64, LONG);
option_conversion!(f32, FLOAT);
option_conversion!(BigInt, BIG_INTEGER);
option_conversion!(BigDecimal, BIG_DECIMAL);

// =========================================================================
// Null Document
// =========================================================================
pub static NULL: LazyLock<Box<dyn Document>> = LazyLock::new(|| {
    DefaultDocument {
        schema: DOCUMENT.clone(),
        value: DefaultDocumentValue::Null,
        discriminator: None,
    }
    .into()
});

// ============================================================================
// Utilities
// ============================================================================

/// Get the shape type of the Document
///
/// If the Document is a member, then returns the type of the member target.
pub(crate) fn get_shape_type(schema: &SchemaRef) -> Result<&ShapeType, Box<dyn Error>> {
    let shape_type = schema.shape_type();
    if shape_type == &ShapeType::Member {
        let Some(member) = schema.as_member() else {
            return Err(Box::new(DocumentError::DocumentConversion(
                "Expected memberSchema for member shape type".to_string(),
            )));
        };
        Ok(member.target.shape_type())
    } else {
        Ok(shape_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_document_value() {
        let document_str: BoxedDoc = "MyStr".into();
        let val: &SchemaRef = &STRING;
        assert_eq!(document_str.schema(), val);
        let output_str: String = document_str.as_string().unwrap().to_string();
        assert_eq!(output_str, "MyStr".to_string());

        let document_string: BoxedDoc = "MyString".into();
        assert_eq!(document_string.schema(), val);
        let output_string: String = document_string.as_string().unwrap().to_string();
        assert_eq!(&output_string, &"MyString");
    }

    #[test]
    fn list_document_value() {
        let vec = vec!["a", "b", "c"];
        let document_list: BoxedDoc = vec.into();
        let val: &SchemaRef = &LIST_DOCUMENT_SCHEMA;
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
        let map_doc: BoxedDoc = map_in.into();
        let val: &SchemaRef = &MAP_DOCUMENT_SCHEMA;
        assert_eq!(map_doc.schema(), val);
        assert_eq!(map_doc.size(), 1);

        let map_out: IndexMap<String, String> = map_doc.try_into().unwrap();
        assert_eq!(map_out.len(), 1);
        assert_eq!(map_out["a"], "b");
    }

    #[test]
    fn integer_document_values() {
        let byte: BoxedDoc = 1i8.into();
        let byte_val: &SchemaRef = &BYTE;
        assert_eq!(byte.schema(), byte_val);

        let short: BoxedDoc = 1i16.into();
        let short_val: &SchemaRef = &SHORT;

        assert_eq!(short.schema(), short_val);
        
        let integer: BoxedDoc = 1i32.into();
        let integer_val: &SchemaRef = &INTEGER;

        assert_eq!(integer.schema(), integer_val);
        
        let long: BoxedDoc = 1i64.into();
        let long_val: &SchemaRef = &LONG;

        assert_eq!(long.schema(), long_val);
    }

    // TODO(numeric comparisons): Add comparison checks

    #[test]
    fn float_document_values() {
        let float: BoxedDoc = 1f32.into();
        let float_val: &SchemaRef = &FLOAT;
        assert_eq!(float.schema(), float_val);

        let double: BoxedDoc = 1f64.into();
        let double_val: &SchemaRef = &DOUBLE;
        assert_eq!(double.schema(), double_val);

        let float_value: f32 = float.try_into().unwrap();
        assert_eq!(float_value, 1f32);
        let double_value: f64 = double.try_into().unwrap();
        assert_eq!(double_value, 1f64);
    }
}
