use core::fmt;
use std::{error::Error, fmt::Debug, sync::LazyLock};
use std::borrow::Cow;
use std::fmt::Formatter;
use std::ops::Index;
use bigdecimal::{num_traits, FromPrimitive, ToPrimitive};
use num_bigint::ToBigInt;
use thiserror::Error;

use crate::{BigDecimal, BigInt, ByteBuffer, IndexMap, Instant, schema::{
    SchemaShape, ShapeId,
    prelude::*,
}, serde::validation::ValidationErrors, FxIndexMap};
use crate::schema::Schema;
use crate::serde::debug::DebugWrapper;
// ============================================================================
// Base Document Wrapper and trait
// ============================================================================

/// A Smithy [document](https://smithy.io/2.0/spec/simple-types.html#document) type,
/// representing untyped data in the Smithy data model.
///
/// ### Smithy Data Model
///
/// The Smithy data model consists of:
/// - Numbers: `byte`, `short`, `integer`, `long`, `float`, `double`, `bigInteger`, `bigDecimal`.
///   `IntEnum` shapes are represented as integers in the Smithy data model.
/// - `boolean`
/// - `blob`
/// - `string`: `enum` shapes are represented as strings in the Smithy data model
/// - `timestamp`: Represented as an [`Instant`]
/// - `list`: list of Documents
/// - `map`: map of int|long|string keys to Document values
/// - `struct`: structure or union
///
/// ## Document Properties
/// 1. Shape Conversion - All generated shapes should be able to be converted to/from a document
/// 2. Lossless Serialization: Documents created from a shape should serialize exactly the same as the shape they are created from.
/// 3. Lossless Deserialization: Deserializing to a document then converting to a shape should be the same as deserializing to that shape.///
/// 4. Protocol Smoothing: Documents should try to smooth over any protocol-specific incompatibilities with the smithy data model.
/// 5. Discriminators - Documents with an encloded type should be able to be (de)serialized with data that identifies their
///
/// ### Protocol-Specific implementations
/// The [`Document`] trait defines the core behavior of documents, but Documents are used as type-erased
/// trait objects (i.e. `Box<dyn Document>`).
///
/// In order to support protocol smoothing and lossless deserialization protocols should create their own [`Document`]
/// implementations that are returned by deserializers.
///
/// A default [`Document`] implementation is provided that supports conversions to/from shapes and can be used for
/// serializing Errors and other types that need over-the-wire polymorphism.
///
/// ### Protocol Smoothing
/// Because Document types are a protocol-agnostic view of untyped data, Protocol codecs should attempt to smooth over
/// any protocol-specific incompatibilities with the Smithy data model when deserializing into a document type.
///
/// For example, If a protocol represents a blob as a base64 encoded string, then deserializing the value into a
/// document should base64 decode the value and use the underlying bytes as the blob value in the document.
/// When the document is re-serialized by the same protocol, the protocol should handle conversion from a byte
/// buffer back to a base64 encoded string.
///
/// ### Lossless serialization
/// Point two *REQUIRES* that documents created from a shape have the exact same schema as that shape.
///
/// A good example of why is to consider the following Smithy Shape:
///
/// ```smithy
/// structure MyStruct {
///     @jsonName("foo")
///     bar: String
/// }
/// ```
///
/// When we go to serialize an instance of this shape with a JSON protocol we would expect that the member `bar`
/// is renamed to `foo` in the serialized JSON data:
///
/// ```rust,ignore
/// let myStructInstance = MyStruct::builder().bar("my-string").build()?;
/// myStructInstance.serialize(jsonSerializer);  // -> Yields `{ "foo": "my-string" }`
/// ```
///
/// The JSON protocol will achieve this conversion by discovering the `jsonName` trait on the
/// schema of the `myStructInstance` `bar` member.
///
/// Another good example is the case of a `@sensitive` field on a structure:
///
/// ```smithy
/// structure MyStructRedacted {
///     @sensitive
///     password: String
/// }
/// ```
///
/// `Display`-ing such a structure should result in the `@sensitive` field being redacted:
/// ```rust,ignore
/// let myStructInstance = MyStruct::builder().password("Secret".to_string()).build()?;
/// assert_eq!("MyStruct[password:**REDACTED**]", format!("{myStructInstance}"))
/// ```
///
/// We do _not_ want conversion to a document to suddenly start leaking a sensitive field.
///
/// In order to retain trait information, Documents created from a shape MUST retain that shape’s Schema.
///
/// ### Discriminators
/// Document types may have a `ShapeId` that indicates the type they correspond to.
/// This ID can be serialized to allow consumers to handle over the wire polymorphism (primarily for over-the wire polymorphism).
/// Typed documents must return the shape ID of the enclosed shape.
///
/// For example, let’s say we convert a shape of type `com.example#MyShape` into a document type.
/// The document would then store the `ShapeID` as its discriminator.
///
/// Serializing this document with a JSON protocol might result in a field `__type` being added to the output JSON :
///
/// ```json
/// {
///     "__type": "com.example#MyShape",
///     "other": {}
/// }
/// ```
///
/// <div class="warning">
/// Discriminators are only really useful for documents (or other structs) being serialized as Structure and Union types.
/// As such they, are serialized by the [`StructSerializer::serialize_discriminator`] method.
/// </div>
///
///
/// Similarly, a deserializer might want to pull the `__type` data from a JSON blob when deserializing into a Document type.
///
///
/// ## Lossless Deserialization
///
/// Let’s say we want to deserialize an error type that has the schema:
/// ```smithy
/// @error("client")
/// @http
/// struct MyClientError {
///     @required
///     message: String
///     @jsonName("foo")
///     bar: String
/// }
/// ```
///
/// Over the wire we might get a JSON body like:
///
/// ```json
/// {
///     "__type": "com.example#MyClientError",
///     "message": "Something broke pretty bad!",
///     "foo": "quux!"
/// }
/// ```
///
/// Because the error response _could_ be one of multiple error types, we don't know the type
/// to deserialize it to in advance.
/// We deserialize the JSON data to a Document, extract the discriminator, and use it to:
///
/// ```rust,ignore
/// let error_document = Document::deserialize(json_deserializer)?;
/// let builder = error_document.get_builder(error_document.discriminator());
/// let output = builder.deserialize(error_document)?.build();
/// ```
///
/// The initial `error_document` has no schema information (beyond the base `Document` schema), so it does not perform any protocol-specific
/// conversions during deserialization (i.e. it won't convert the field name `foo` to `bar` based on the `@jsonName` trait).
///
/// However, when we deserialize the document into the final Error type we need to execute the protocol-specific handling of the `jsonName` trait.
pub trait Document: Send + Sync + Debug {
    /// Get the discriminator (type ID) of a type document
    ///
    /// The discriminator is primarily used to implement polymorphism using documents in deserialization.
    ///
    /// <div class="warning">
    /// It is expected that protocols set the discriminator on deserialization if applicable
    /// </div>
    #[must_use]
    fn discriminator(&self) -> Option<&ShapeId>;

    /// Get the number of elements in an array document, or the number of key value pairs in a map document.
    ///
    /// *NOTE*: Should return `0` for all other
    #[must_use]
    fn size(&self) -> usize;

    /// Get a read-only view of the document type in the smithy data model
    fn as_variant(&self) -> DocumentVariant<'_>;

    /// Try to convert into a list.
    ///
    /// This allows us to move ownership to inner list data in deserializers.
    ///
    /// # Errors
    /// Returns an error if the document was not a list
    fn try_into_list(self: Box<Self>) -> Result<Vec<Box<dyn Document>>, DocumentError>;

    /// Try to convert into a list.
    ///
    /// This allows us to move ownership to inner list data in deserializers.
    ///
    /// # Errors
    /// Returns an error if the document was not a map or structure
    fn try_into_map(self: Box<Self>) -> Result<IndexMap<Cow<'static, str>, Box<dyn Document>>, DocumentError>;

    /// Clone implementation necessary to satisfy the compiler
    fn box_clone(&self) -> Box<dyn Document>;
}

// ================================================================
// Blanket impl for dyn Documents
// ================================================================

impl PartialEq for dyn Document {
    fn eq(&self, other: &Self) -> bool {
        if self.discriminator() != other.discriminator() {
            return false;
        }
        match (self.as_variant(), other.as_variant()) {
            (DocumentVariant::Null, DocumentVariant::Null) => true,
            (DocumentVariant::Boolean(t), DocumentVariant::Boolean(o)) => t == o,
            (DocumentVariant::Number(t), DocumentVariant::Number(o)) => t == o,
            (DocumentVariant::String(t), DocumentVariant::String(o)) => t == o,
            (DocumentVariant::List(t), DocumentVariant::List(o)) => t == o,
            (DocumentVariant::Blob(b1), DocumentVariant::Blob(b2)) => b1 == b2,
            (DocumentVariant::Timestamp(t), DocumentVariant::Timestamp(o)) => t == o,
            (DocumentVariant::Map(t), DocumentVariant::Map(o)) => t == o,
            _ => false,
        }
    }
}

impl Clone for Box<dyn Document> {
    #[inline]
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

// ================================================================
// Read-Only Document Views
// ================================================================

/// Borrow-only view of a document type
///
/// This is effectively a read-only view of the document data in the Smithy
/// data model and represents the primary access point for serializers into
/// the document data.
///
/// This allows us to use document values quickly with a single vtable
/// lookup rather than requiring multiple vtable lookups.
pub enum DocumentVariant<'document> {
    /// An empty (null) document
    Null,
    /// A `boolean` document
    Boolean(bool),
    /// A number document
    Number(NumberVariant<'document>),
    /// A `string` document
    String(&'document str),
    /// A `blob` document
    Blob(&'document ByteBuffer),
    /// A `timestamp` document
    Timestamp(Instant),
    /// A `list` document
    List(&'document [Box<dyn Document>]),
    /// A `map` document
    Map(&'document IndexMap<Cow<'static, str>, Box<dyn Document>>),
}

/// A numeric document type in the Smithy data model
pub enum NumberVariant<'document> {
    /// Smithy `byte` type
    Byte(i8),
    /// Smithy `short` type
    Short(i16),
    /// Smithy `integer` type
    Integer(i32),
    /// Smithy `long` type
    Long(i64),
    /// Smithy `big_integer` type
    BigInt(&'document BigInt),
    /// Smithy `float` type
    Float(f32),
    /// Smithy `double` type
    Double(f64),
    /// Smithy `big_decimal` type
    BigDecimal(&'document BigDecimal),
}

macro_rules! into_variant_d {
    ($variant:ident, $value:expr, $msg:expr) => {
        match $value {
            DocumentVariant::$variant(v) => Ok(v),
            _ => Err(DocumentError::DocumentConversion($msg.to_string()))
        }
    };
}

macro_rules! to_int_d {
    ($target_type:ident, $to_method:ident, $value:expr) => {{
        let err = || DocumentError::DocumentConversion(
            concat!("Invalid conversion to ", stringify!($target_type)).to_string()
        );
        match *$value {
            DocumentVariant::Number(NumberVariant::Byte(n)) => n.try_into().map_err(|_| err()),
            DocumentVariant::Number(NumberVariant::Short(n)) => n.try_into().map_err(|_| err()),
            DocumentVariant::Number(NumberVariant::Integer(n)) => n.try_into().map_err(|_| err()),
            DocumentVariant::Number(NumberVariant::Long(n)) => n.try_into().map_err(|_| err()),
            DocumentVariant::Number(NumberVariant::BigInt(ref b)) => b.$to_method().ok_or_else(err),
            _ => Err(err()),
        }
    }};
}

macro_rules! to_float_d {
    ($target_type:ident, $to_method:ident, $value:expr) => {{
        let err = || DocumentError::DocumentConversion(
            concat!("Invalid conversion to ", stringify!($target_type)).to_string()
        );

        match *$value {
            DocumentVariant::Number(NumberVariant::Float(f)) => num_traits::cast(f).ok_or_else(err),
            DocumentVariant::Number(NumberVariant::Double(d)) => num_traits::cast(d).ok_or_else(err),
            DocumentVariant::Number(NumberVariant::BigDecimal(b)) => b.$to_method().ok_or_else(err),
            _ => Err(err()),
        }
    }};
}


impl <'document> DocumentVariant<'document> {
    /// Get the `boolean` value of the Document if it is a `boolean` or can be converted into one.
    ///
    /// # Error
    /// Returns a `DocumentError` if the document could not be converted to a `boolean`.
    #[inline]
    pub(crate) fn to_bool(&self) -> Result<bool, DocumentError> {
        match self {
            &DocumentVariant::Boolean(b) => Ok(b),
            _ => Err(DocumentError::DocumentConversion("Expected Boolean Document".to_string()))
        }
    }

    /// Get the `blob` value of the Document if it is a `blob` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `blob`.
    #[inline]
    pub fn to_blob(&self) -> Result<&'document ByteBuffer, DocumentError>  {
        into_variant_d!(Blob, self, "Expected Blob Document")
    }

    /// Get the `string` value of the Document if it is a `string` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `string`.
    #[inline]
    pub fn to_str(&self) -> Result<&'document str, DocumentError> {
        into_variant_d!(String, self, "Expected String Document")
    }

    /// Get the `timestamp` value of the Document if it is a `timestamp` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `timestamp`.
    #[inline]
    pub fn to_timestamp(&self) -> Result<Instant, DocumentError> {
        match self {
            &DocumentVariant::Timestamp(t) => Ok(t),
            _ => Err(DocumentError::DocumentConversion("Expected timestamp Document".to_string()))
        }
    }

    /// Get the `byte` value of the Document if it is a `byte` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `byte`.
    #[inline]
    pub fn to_byte(&self) -> Result<i8, DocumentError> {
        to_int_d!(i8, to_i8, self)
    }

    /// Get the `short` value of the Document if it is a `short` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `short`.
    #[inline]
    pub fn to_short(&self) -> Result<i16, DocumentError> {
        to_int_d!(i16, to_i16, self)
    }

    /// Get the `integer` value of the Document if it is a `integer` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `integer`.
    #[inline]
    pub fn to_integer(&self) -> Result<i32, DocumentError> {
        to_int_d!(i32, to_i32, self)
    }

    /// Get the `long` value of the Document if it is a `long` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `long`.
    #[inline]
    pub fn to_long(&self) -> Result<i64, DocumentError> {
        to_int_d!(i64, to_i64, self)
    }

    /// Get the `big_integer` value of the Document if it is a `big_integer` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `big_integer`.
    pub fn to_big_integer(&self) -> Result<Cow<'document, BigInt>, DocumentError> {
        let err = || DocumentError::DocumentConversion(
            "Invalid conversion to BigInt".to_string()
        );

        match *self {
            DocumentVariant::Number(NumberVariant::BigInt(b)) => Ok(Cow::Borrowed(b)),
            DocumentVariant::Number(NumberVariant::Byte(b)) => Ok(Cow::Owned(BigInt::from(b))),
            DocumentVariant::Number(NumberVariant::Short(s)) => Ok(Cow::Owned(BigInt::from(s))),
            DocumentVariant::Number(NumberVariant::Integer(i)) => Ok(Cow::Owned(BigInt::from(i))),
            DocumentVariant::Number(NumberVariant::Long(l)) => Ok(Cow::Owned(BigInt::from(l))),
            DocumentVariant::Number(NumberVariant::Float(f)) => {
                let b = BigInt::from_f32(f).ok_or_else(err)?;
                Ok(Cow::Owned(b))
            },
            DocumentVariant::Number(NumberVariant::Double(d)) => {
                let b = BigInt::from_f64(d).ok_or_else(err)?;
                Ok(Cow::Owned(b))
            },
            DocumentVariant::Number(NumberVariant::BigDecimal(b)) => {
                let b = b.to_bigint().ok_or_else(err)?;
                Ok(Cow::Owned(b))
            },
            _ => Err(err()),
        }
    }

    /// Get the `float` value of the Document if it is a `float` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `float`.
    #[inline]
    pub fn to_float(&self) -> Result<f32, DocumentError> {
        to_float_d!(f32, to_f32, self)
    }

    /// Get the `double` value of the Document if it is a `double` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `double`.
    #[inline]
    pub fn to_double(&self) -> Result<f64, DocumentError> {
        to_float_d!(f64, to_f64, self)
    }

    /// Get the `big_decimal` value of the Document if it is a `big_decimal` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `big_decimal`.
    pub fn to_big_decimal(&self) -> Result<Cow<'document, BigDecimal>, DocumentError> {
        let err = || DocumentError::DocumentConversion(
            "Invalid conversion to BigDecimal".to_string()
        );

        match *self {
            DocumentVariant::Number(NumberVariant::BigDecimal(b)) => Ok(Cow::Borrowed(b)),
            DocumentVariant::Number(NumberVariant::BigInt(b)) => {
                let bd = BigDecimal::from_bigint(b.clone(), 0);
                Ok(Cow::Owned(bd))
            },
            DocumentVariant::Number(NumberVariant::Byte(n)) => Ok(Cow::Owned(BigDecimal::from(n))),
            DocumentVariant::Number(NumberVariant::Short(n)) => Ok(Cow::Owned(BigDecimal::from(n))),
            DocumentVariant::Number(NumberVariant::Integer(n)) => Ok(Cow::Owned(BigDecimal::from(n))),
            DocumentVariant::Number(NumberVariant::Long(n)) => Ok(Cow::Owned(BigDecimal::from(n))),
            DocumentVariant::Number(NumberVariant::Float(f)) => {
                let d = BigDecimal::from_f32(f).ok_or_else(err)?;
                Ok(Cow::Owned(d))
            },
            DocumentVariant::Number(NumberVariant::Double(d)) => {
                let d = BigDecimal::from_f64(d).ok_or_else(err)?;
                Ok(Cow::Owned(d))
            },
            _ => Err(err()),
        }
    }

    /// Get the `list` value of the Document if it is a `list`.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `list`.
    #[inline]
    pub fn to_list(&self) -> Result<&'document [Box<dyn Document>], DocumentError> {
        into_variant_d!(List, self, "Expected List")
    }

    /// Get the `big_decimal` value of the Document if it is a `map` or `structure`.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `map`.
    #[inline]
    pub fn to_map(&self) -> Result<&'document IndexMap<Cow<'static, str>, Box<dyn Document>>, DocumentError> {
        into_variant_d!(Map, self, "Expected Map")
    }
}

impl PartialEq for NumberVariant<'_> {
    fn eq(&self, other: &Self) -> bool {
        // TODO: Add all combos!
        match (self, other) {
            (NumberVariant::BigInt(a), NumberVariant::BigInt(b)) => a == b,
            _ => false
        }
    }
}

// ============================================================================
// Document Errors
// ============================================================================

/// Errors that can occur when converting to/from a document type.
#[derive(Error, Debug)]
pub enum DocumentError {
    /// Document failed to serialize to a data sink
    #[error("Failed to convert document to type {0}")]
    DocumentSerialization(String),
    /// Could not convert a type _into_ a document
    #[error("Failed to convert document to type {0}")]
    DocumentConversion(String),
    /// An unknown error
    #[error("Encountered unknown error")]
    Unknown(#[from] Box<dyn Error>),
    /// Encountered validation error while deserializing document
    #[error("Invalid Document")]
    Invalid(#[from] ValidationErrors),
    /// A custom error
    #[error("Encountered error: {0}")]
    CustomError(String),
}

impl crate::serde::de::Error for DocumentError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        DocumentError::CustomError(msg.to_string())
    }
}

// ============================================================================
// Static Document Implementation
// ============================================================================

/// Static/Default Document Implementation
///
/// This implementation is used for in-place conversion of generated shapes to
/// documents.
///
/// In order to support proper debug printing and serialization of these documents
/// they include a reference to the static schema of the corresponding generated shape.
/// For documents instantiated by a deserializer users should always prefer a
/// schema-less document type specific to that deserializer
///
/// TODO: Include examples!
#[derive(Clone, PartialEq)]
pub struct StaticDocument {
    /// Reference to the static schema for the shape this was generated from
    pub schema: &'static Schema,
    /// Document value
    pub value: Value
}

impl Debug for StaticDocument {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("Document");
        let d: &dyn Document = self;
        s.field("value", &DebugWrapper::new(self.schema, &d));
        self.discriminator()
            .map(|v| s.field("discriminator", &v.id()));
        s.finish()
    }
}

impl SchemaShape for StaticDocument {
    fn schema(&self) -> &Schema {
        self.schema
    }
}

/// Possible Values for this document
///
/// See: <https://smithy.io/2.0/spec/simple-types.html#document>
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    /// An Empty document
    Null,
    /// A numeric document
    Number(Number),
    /// A boolean document
    Boolean(bool),
    /// A blob document
    Blob(ByteBuffer),
    /// A string document
    String(String),
    /// A timestamp document
    Timestamp(Instant),
    /// A list document
    List(Vec<Box<dyn Document>>),
    /// A map document
    Map(IndexMap<Cow<'static, str>, Box<dyn Document>>),
}

/// Represents numbers in the smithy data model
///
/// Smithy numbers types include: byte, short, integer, long, float, double, bigInteger, bigDecimal.
///
/// <div class ="note">
/// **NOTE**: `IntEnum` shapes are represented as integers in the Smithy data model.
/// </div>
#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    /// Smithy `byte` type
    Byte(i8),
    /// Smithy `short` type
    Short(i16),
    /// Smithy `integer` type
    Integer(i32),
    /// Smithy `long` type
    Long(i64),
    /// Smithy `big_integer` type
    BigInt(BigInt),
    /// Smithy `float` type
    Float(f32),
    /// Smithy `double` type
    Double(f64),
    /// Smithy `big_decimal` type
    BigDecimal(BigDecimal),
}

impl Document for StaticDocument {
    #[inline]
    fn discriminator(&self) -> Option<&ShapeId> {
        // A static document ALWAYS has a discriminator
        // as it was generated from a shape with a concrete ID
        Some(self.schema.id())
    }

    #[inline]
    fn size(&self) -> usize {
        match &self.value {
            Value::List(v) => v.len(),
            Value::Map(v) => v.len(),
            _ => 0,
        }
    }

    #[inline]
    fn as_variant(&self) -> DocumentVariant<'_> {
        match &self.value {
            Value::Null => DocumentVariant::Null,
            Value::Number(n) => {
                match n {
                    Number::Byte(b) => DocumentVariant::Number(NumberVariant::Byte(*b)),
                    Number::Short(s) => DocumentVariant::Number(NumberVariant::Short(*s)),
                    Number::Integer(i) => DocumentVariant::Number(NumberVariant::Integer(*i)),
                    Number::Long(l) => DocumentVariant::Number(NumberVariant::Long(*l)),
                    Number::BigInt(bi) => DocumentVariant::Number(NumberVariant::BigInt(bi)),
                    Number::Float(f) => DocumentVariant::Number(NumberVariant::Float(*f)),
                    Number::Double(d) => DocumentVariant::Number(NumberVariant::Double(*d)),
                    Number::BigDecimal(bd) => DocumentVariant::Number(NumberVariant::BigDecimal(bd))
                }
            }
            Value::Boolean(b) => DocumentVariant::Boolean(*b),
            Value::Blob(bb) => DocumentVariant::Blob(bb),
            Value::String(s) => DocumentVariant::String(s),
            Value::Timestamp(i) => DocumentVariant::Timestamp(*i),
            Value::List(l) => DocumentVariant::List(l),
            Value::Map(m) => DocumentVariant::Map(m),
        }
    }

    #[inline]
    fn try_into_list(self: Box<Self>) -> Result<Vec<Box<dyn Document>>, DocumentError> {
        if let Value::List(l) = self.value {
            Ok(l)
        } else {
            Err(DocumentError::DocumentConversion("Could not convert value to list".to_string()))
        }
    }

    #[inline]
    fn try_into_map(self: Box<Self>) -> Result<IndexMap<Cow<'static, str>, Box<dyn Document>>, DocumentError> {
        if let Value::Map(m) = self.value {
            Ok(m)
        } else {
            Err(DocumentError::DocumentConversion("Could not convert value to map".to_string()))
        }
    }

    #[inline]
    fn box_clone(&self) -> Box<dyn Document> {
        Box::new(self.clone())
    }
}

// === FROM IMPLS ===
impl From<StaticDocument> for Box<dyn Document> {
    #[inline]
    fn from(value: StaticDocument) -> Self {
        Box::new(value)
    }
}

// ============================================================================
// Conversions INTO Default Document
// ============================================================================

macro_rules! doc_from_primitive {
    ($src_type:ty, $schema:expr, $value_expr:expr) => {
        impl From<$src_type> for Box<dyn Document> {
            #[inline]
            fn from(value: $src_type) -> Self {
                StaticDocument {
                    schema: &$schema,
                    value: $value_expr(value),
                }
                .into()
            }
        }
    };
}
doc_from_primitive!(bool, BOOLEAN, Value::Boolean);

// Numbers
doc_from_primitive!(i8, BYTE, |v| Value::Number(Number::Byte(v)));
doc_from_primitive!(i16, SHORT, |v| Value::Number(Number::Short(v)));
doc_from_primitive!(i32, INTEGER, |v| Value::Number(Number::Integer(v)));
doc_from_primitive!(i64, LONG, |v| Value::Number(Number::Long(v)));
doc_from_primitive!(f32, FLOAT, |v| Value::Number(Number::Float(v)));
doc_from_primitive!(f64, DOUBLE, |v| Value::Number(Number::Double(v)));
doc_from_primitive!(&str, STRING, |v: &str| Value::String(v.to_string()));
doc_from_primitive!(BigInt, BIG_INTEGER, |v| Value::Number(Number::BigInt(v)));
doc_from_primitive!(BigDecimal, BIG_DECIMAL, |v| Value::Number(Number::BigDecimal(v)));
doc_from_primitive!(ByteBuffer, BLOB, Value::Blob);
doc_from_primitive!(String, STRING, Value::String);
doc_from_primitive!(Instant, TIMESTAMP, Value::Timestamp);
doc_from_primitive!(&Instant, TIMESTAMP, |v: &Instant| Value::Timestamp(*v));

mod list {
    use crate::{prelude::DOCUMENT as PRELUDE_DOCUMENT, smithy};

    smithy!(
        #[doc(hidden)]
        list smithy::api::Document {
            member: PRELUDE_DOCUMENT
        }
    );
}

impl<T: Into<Box<dyn Document>>> From<Vec<T>> for Box<dyn Document> {
    fn from(value: Vec<T>) -> Self {
        let mut result = Vec::with_capacity(value.len());
        for v in value {
            result.push(v.into());
        }
        StaticDocument {
            schema: &list::DOCUMENT,
            value: Value::List(result),
        }
        .into()
    }
}

mod map {
    use crate::{
        prelude::{DOCUMENT as PRELUDE_DOCUMENT, STRING},
        smithy,
    };

    smithy!(
        #[doc(hidden)]
        map smithy::api::Document {
            key: STRING
            value: PRELUDE_DOCUMENT
        }
    );
}

impl<T: Into<Box<dyn Document>>> From<IndexMap<String, T>> for Box<dyn Document> {
    fn from(value: IndexMap<String, T>) -> Self {
        let mut result = IndexMap::with_capacity(value.len());
        for (key, value) in value {
            result.insert(Cow::Owned(key), value.into());
        }
        StaticDocument {
            schema: &map::DOCUMENT,
            value: Value::Map(result),
        }
        .into()
    }
}

macro_rules! option_conversion {
    ($ty:ty, $schema:ident) => {
        impl From<Option<$ty>> for Box<dyn Document> {
            fn from(value: Option<$ty>) -> Self {
                value.map_or_else(
                    || {
                        StaticDocument {
                            schema: &$schema,
                            value: Value::Null,
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

// ============================================================================
// Conversions FROM Document into Shape
// ============================================================================

/// Attempt to create a typed shape from an untyped document.
///
/// NOTE: Implemented as a separate trait to avoid conflict with
/// blanket `TryFrom` impls.
///
/// This trait is reflexive in that any types that implement this trait
/// will also support `try_into` from a dynamic document.
pub trait TryFromDocument: Sized {
    /// Attempt to create a typed shape from an untyped document.
    ///
    /// ## Errors
    /// Returns [`DocumentError`] if the type could not be converted to a
    /// document value.
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError>;
}

// === Impls for basic types ===

impl TryFromDocument for bool {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.as_variant().to_bool()
    }
}

impl TryFromDocument for i8 {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.as_variant().to_byte()
    }
}

impl TryFromDocument for i16 {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.as_variant().to_short()
    }
}

impl TryFromDocument for i32 {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.as_variant().to_integer()
    }
}

impl TryFromDocument for i64 {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.as_variant().to_long()
    }
}

impl TryFromDocument for f32 {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.as_variant().to_float()
    }
}

impl TryFromDocument for f64 {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.as_variant().to_double()
    }
}

impl TryFromDocument for BigInt {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        let cow_data = document.as_variant().to_big_integer()?;
        Ok(cow_data.into_owned())
    }
}

impl TryFromDocument for BigDecimal {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        let cow_data = document.as_variant().to_big_decimal()?;
        Ok(cow_data.into_owned())
    }
}

impl TryFromDocument for ByteBuffer {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        let borrow = document.as_variant().to_blob()?;
        Ok(borrow.clone())
    }
}

impl TryFromDocument for String {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        let str = document.as_variant().to_str()?;
        Ok(str.to_string())
    }
}

impl TryFromDocument for Instant {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.as_variant().to_timestamp()
    }
}

impl<T: TryFromDocument> TryFromDocument for Vec<T> {
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        let list = document.try_into_list()?;
        let mut result = Vec::with_capacity(list.len());
        for item in list {
            result.push(item.try_into()?);
        }
        Ok(result)
    }
}

impl<T: TryFromDocument> TryFromDocument for IndexMap<String, T> {
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        let map = document.try_into_map()?;
        let mut result: IndexMap<String, T> = IndexMap::with_capacity(map.len());
        for (key, value) in map {
            result.insert(key.to_string(), value.try_into()?);
        }
        Ok(result)
    }
}

impl<T: TryFromDocument> TryFromDocument for Option<T> {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        if matches!(document.as_variant(), DocumentVariant::Null) {
            Ok(None)
        } else {
            Ok(Some(document.try_into()?))
        }
    }
}

// === Blanket impl for Typed Shapes ===
impl dyn Document {
    /// Attempt to convert a document into a shape by deserialization.
    ///
    /// ## Errors
    /// Returns [`DocumentError`] if the type could not be converted to a
    /// document value.
    #[inline]
    pub fn try_into<S: TryFromDocument>(self: Box<Self>) -> Result<S, DocumentError> {
        <S as TryFromDocument>::try_from(self)
    }
}

// =========================================================================
// Null Document
// =========================================================================
#[doc(hidden)]
pub static NULL: LazyLock<Box<dyn Document>> = LazyLock::new(|| {
    StaticDocument {
        schema: &DOCUMENT,
        value: Value::Null,
    }
    .into()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_document_value() {
        let document_str: Box<dyn Document> = "MyStr".into();
        let output_str: String = document_str.as_variant().to_str().unwrap().to_string();
        assert_eq!(output_str, "MyStr".to_string());

        let document_string: Box<dyn Document> = "MyString".into();
        let output_string: String = document_string.as_variant().to_str().unwrap().to_string();
        assert_eq!(&output_string, &"MyString");
    }

    #[test]
    fn list_document_value() {
        let vec = vec!["a", "b", "c"];
        let document_list: Box<dyn Document> = vec.into();
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
        let map_doc: Box<dyn Document> = map_in.into();
        assert_eq!(map_doc.size(), 1);

        let map_out: IndexMap<String, String> = map_doc.try_into().unwrap();
        assert_eq!(map_out.len(), 1);
        assert_eq!(map_out["a"], "b");
    }

    // TODO(numeric comparisons): Add comparison checks

    #[test]
    fn integer_document_values() {
        // TODO: Update
    }

    #[test]
    fn float_document_values() {
        // TODO: Update
        // let float: Box<dyn Document> = 1f32.into();
        // let double: Box<dyn Document> = 1f64.into();
        //
        // let float_value: f32 = float.try_into().unwrap();
        // assert_eq!(float_value, 1f32);
        // let double_value: f64 = double.try_into().unwrap();
        // assert_eq!(double_value, 1f64);
    }
}
