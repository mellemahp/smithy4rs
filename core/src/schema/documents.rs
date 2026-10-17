use core::fmt;
use std::{error::Error, fmt::Debug};
use std::borrow::Cow;
use bigdecimal::{num_traits, FromPrimitive, ToPrimitive};
use indexmap::map::MutableKeys;
use num_bigint::ToBigInt;
use thiserror::Error;

use crate::{BigDecimal, BigInt, ByteBuffer, IndexMap, Instant, schema::{
    ShapeId,
    prelude::*,
}, serde::validation::ValidationErrors};
use crate::schema::Schema;

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
pub trait Document: Send + Sync + BoxClone {
    /// Get the discriminator (type ID) of a type document
    ///
    /// The discriminator is primarily used to implement polymorphism using documents in deserialization.
    ///
    /// <div class="warning">
    /// It is expected that protocols set the discriminator on deserialization if applicable
    /// </div>
    ///
    /// Default implementation is `None`
    #[must_use]
    fn discriminator(&self) -> Option<&ShapeId> {
        None
    }

    /// Get a read-only view of the document type in the smithy data model
    ///
    /// Note: This indirection makes it straightforward to handle `PartialEq` and common
    /// behaviors
    fn value(&self) -> DocumentValue<'_>;

    // === Consumers for deserialization ====

    /// Read the document as a `boolean` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `boolean`
    fn take_bool(&mut self, _schema: &Schema) -> Result<bool, DocumentError> {
        Err(DocumentError::DocumentConversion("failed to read as boolean".to_string()))
    }

    /// Read the document as a `blob` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `blob`
    fn take_blob(&mut self, _schema: &Schema) -> Result<ByteBuffer, DocumentError> {
        Err(DocumentError::DocumentConversion("failed to read as blob".to_string()))
    }

    /// Read the document as a `string` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `string`
    fn take_string(&mut self, _schema: &Schema) -> Result<String, DocumentError> {
        Err(DocumentError::DocumentConversion("failed to read as string".to_string()))
    }

    /// Read the document as a `timestamp` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `timestamp`
    fn take_timestamp(&mut self, _schema: &Schema) -> Result<Instant, DocumentError> {
        Err(DocumentError::DocumentConversion("failed to read as timestamp".to_string()))
    }

    /// Read the document as a `byte` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `byte`
    fn take_byte(&mut self, _schema: &Schema) -> Result<i8, DocumentError> {
        Err(DocumentError::DocumentConversion("failed to read as byte".to_string()))
    }

    /// Read the document as a `short` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `short`
    fn take_short(&mut self, _schema: &Schema) -> Result<i16, DocumentError> {
        Err(DocumentError::DocumentConversion("failed to read as short".to_string()))
    }

    /// Read the document as a `integer` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as an `integer`
    fn take_integer(&mut self, _schema: &Schema) -> Result<i32, DocumentError> {
        Err(DocumentError::DocumentConversion("failed to read as integer".to_string()))
    }

    /// Read the document as a `long` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `long`
    fn take_long(&mut self, _schema: &Schema) -> Result<i64, DocumentError> {
        Err(DocumentError::DocumentConversion("failed to read as long".to_string()))
    }

    /// Read the document as a `big_integer` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `big_integer`
    fn take_big_integer(&mut self, _schema: &Schema) -> Result<BigInt, DocumentError> {
        Err(DocumentError::DocumentConversion("failed to read as big integer".to_string()))
    }

    /// Read the document as a `float` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `float`
    fn take_float(&mut self, _schema: &Schema) -> Result<f32, DocumentError> {
        Err(DocumentError::DocumentConversion("failed to read as float".to_string()))
    }

    /// Read the document as a `double` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `double`
    fn take_double(&mut self, _schema: &Schema) -> Result<f64, DocumentError> {
        Err(DocumentError::DocumentConversion("failed to read as double".to_string()))
    }

    /// Read the document as a `big_decimal` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `big_decimal`
    fn take_big_decimal(&mut self, _schema: &Schema) -> Result<BigDecimal, DocumentError> {
        Err(DocumentError::DocumentConversion("failed to read as big decimal".to_string()))
    }

    /// Get the length of the document
    ///
    /// Defaults to 0. List and Map documents should override this
    fn len(&self) -> usize {
        0
    }

    /// Get an element of a list document
    ///
    /// # Errors
    /// Raises a `DocumentError` if the document is not a list
    fn get_element_mut(&mut self, _index: usize) -> Result<Option<&mut dyn Document>, DocumentError> {
        Err(DocumentError::DocumentConversion("Not a list".to_string()))
    }

    /// Get a map document entry
    ///
    /// TODO: Should we update to split from key?
    /// Note: Should only really be used by deserializers.
    ///
    /// # Errors
    /// Raises a `DocumentError` if the document is not a map
    fn get_entry_mut(&mut self, _index: usize) -> Result<(&str, &mut dyn Document), DocumentError> {
        Err(DocumentError::DocumentConversion("Not a map".to_string()))
    }

    /// Get the map key
    ///
    /// TODO: Support non-string maps
    /// # Errors
    /// Raises a `DocumentError` if the document is not a map
    fn get_key_mut(&mut self, _index: usize) -> Result<Option<&mut dyn Document>, DocumentError> {
        Err(DocumentError::DocumentConversion("Not a map".to_string()))
    }

    /// Check if this document is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}


// ===== Clone support =====

/// Bridge trait to allow us to clone boxed Document
pub trait BoxClone {
    /// Clone implementation for dyn trait to allow us to support clones of
    /// `Box<dyn Document>`
    fn box_clone(&self) -> Box<dyn Document>;
}

impl<T: Document + Clone + 'static> BoxClone for T {
    fn box_clone(&self) -> Box<dyn Document> {
        Box::new(self.clone())
    }
}

// ================================================================
// Blanket impl for dyn Documents
// ================================================================

impl PartialEq for dyn Document {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl Clone for Box<dyn Document> {
    #[inline]
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

// ================================================================
// Accessors
// ================================================================

// ==== As-Ers Convenience Methods =====
impl dyn Document + '_ {
    /// Get the `boolean` value of the Document if it is a `boolean` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `boolean`.
    #[inline]
    pub fn as_boolean(&self) -> Result<bool, DocumentError> {
        self.value().as_bool()
    }

    /// Get the `blob` value of the Document if it is a `blob` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `blob`.
    #[inline]
    pub fn as_blob(&self) -> Result<&ByteBuffer, DocumentError>  {
        self.value().as_blob()
    }

    /// Get the `string` value of the Document if it is a `string` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `string`.
    #[inline]
    pub fn as_str(&self) -> Result<&str, DocumentError> {
        self.value().as_str()
    }

    /// Get the `timestamp` value of the Document if it is a `timestamp` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `timestamp`.
    #[inline]
    pub fn as_timestamp(&self) -> Result<Instant, DocumentError> {
        self.value().as_timestamp()
    }

    /// Get the `byte` value of the Document if it is a `byte` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `byte`.
    #[inline]
    pub fn as_byte(&self) -> Result<i8, DocumentError> {
        self.value().as_byte()
    }

    /// Get the `short` value of the Document if it is a `short` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `short`.
    #[inline]
    pub fn as_short(&self) -> Result<i16, DocumentError> {
        self.value().as_short()
    }

    /// Get the `integer` value of the Document if it is a `integer` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `integer`.
    #[inline]
    pub fn as_integer(&self) -> Result<i32, DocumentError> {
        self.value().as_integer()
    }

    /// Get the `long` value of the Document if it is a `long` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `long`.
    #[inline]
    pub fn as_long(&self) -> Result<i64, DocumentError> {
        self.value().as_long()
    }

    /// Get the `big_integer` value of the Document if it is a `big_integer` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `big_integer`.
    pub fn as_big_integer(&self) -> Result<Cow<'_, BigInt>, DocumentError> {
        self.value().as_big_integer()
    }

    /// Get the `float` value of the Document if it is a `float` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `float`.
    #[inline]
    pub fn as_float(&self) -> Result<f32, DocumentError> {
        self.value().as_float()
    }

    /// Get the `double` value of the Document if it is a `double` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `double`.
    #[inline]
    pub fn as_double(&self) -> Result<f64, DocumentError> {
        self.value().as_double()
    }

    /// Get the `big_decimal` value of the Document if it is a `big_decimal` or can be converted into one.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `big_decimal`.
    pub fn as_big_decimal(&self) -> Result<Cow<'_, BigDecimal>, DocumentError> {
        self.value().as_big_decimal()
    }

    /// Get the `list` value of the Document if it is a `list`.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `list`.
    #[inline]
    pub fn as_list(&self) -> Result<&dyn ListCursor, DocumentError> {
        self.value().as_list()
    }

    /// Get the `big_decimal` value of the Document if it is a `map` or `structure`.
    ///
    /// # Errors
    /// Returns a `DocumentError` if the document could not be converted to a `map`.
    #[inline]
    pub fn as_map(&self) -> Result<&dyn MapCursor, DocumentError> {
        self.value().as_map()
    }

    /// Check if the Document is Null (empty)
    #[inline]
    pub fn is_null(&self) -> bool {
        self.value().is_null()
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
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DocumentError::CustomError(msg.to_string())
    }
}


// ================================================================
// Read-Only Document View
// ================================================================

/// Borrow-only view of a document type
///
/// This is effectively a read-only view of the document data in the Smithy
/// data model and represents the primary access point for serializers into
/// the document data.
///
/// This allows us to use document values quickly with a single vtable
/// lookup rather than requiring multiple vtable lookups.
pub enum DocumentValue<'document> {
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
    List(&'document dyn ListCursor),
    /// A `map` document
    Map(&'document dyn MapCursor),
}

impl PartialEq for DocumentValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DocumentValue::Null, DocumentValue::Null) => true,
            (DocumentValue::Boolean(t), DocumentValue::Boolean(o)) => t == o,
            (DocumentValue::Number(t), DocumentValue::Number(o)) => t == o,
            (DocumentValue::String(t), DocumentValue::String(o)) => t == o,
            (DocumentValue::List(t), DocumentValue::List(o)) => {
                if t.len() != o.len() {
                    return false;
                }
                for i in 0..t.len() {
                    match (t.get(i).map(|d| d.value()), o.get(i).map(|d| d.value())) {
                        (Some(a), Some(b)) => {
                            if a != b {
                                return false;
                            }
                        }
                        (None, None) => {}
                        _ => return false,
                    }
                }
                true
            },
            (DocumentValue::Blob(b1), DocumentValue::Blob(b2)) => b1 == b2,
            (DocumentValue::Timestamp(t), DocumentValue::Timestamp(o)) => t == o,
            (DocumentValue::Map(t), DocumentValue::Map(o)) => {
                if t.len() != o.len() {
                    return false;
                }
                for i in 0..t.len() {
                    if let Some(key) = t.get_key(i) {
                        let left_variant = t.get(key).map(|d| d.value());
                        let right_variant = o.get(key).map(|d| d.value());
                        match (left_variant, right_variant) {
                            (Some(a), Some(b)) => {
                                if a != b {
                                    return false;
                                }
                            }
                            (None, None) => {}
                            _ => return false,
                        }
                    } else {
                        return false;
                    }
                }
                true
            },
            _ => false
        }
    }
}

/// Immutable view of document list
///
/// Permits easy access to list document
#[allow(clippy::len_without_is_empty)]
pub trait ListCursor {
    /// Get an element by index if possible
    fn get(&self, idx: usize) -> Option<&dyn Document>;

    /// Number of list elements
    fn len(&self) -> usize;
}

/// Immutable view of document map
///
/// Permits easy access to map document
#[allow(clippy::len_without_is_empty)]
pub trait MapCursor {
    /// Used to iterate through the keys of the first map sequentially
    fn get_key(&self, idx: usize) -> Option<&str>;

    /// Used to query for a specific key
    fn get(&self, key: &str) -> Option<&dyn Document>;

    /// Get entry for a specific idx
    fn get_entry(&self, idx: usize) -> Option<(&str, &dyn Document)>;

    /// Number of entries map document;
    fn len(&self) -> usize;
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
            DocumentValue::$variant(v) => Ok(v),
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
            DocumentValue::Number(NumberVariant::Byte(n)) => n.try_into().map_err(|_| err()),
            DocumentValue::Number(NumberVariant::Short(n)) => n.try_into().map_err(|_| err()),
            DocumentValue::Number(NumberVariant::Integer(n)) => n.try_into().map_err(|_| err()),
            DocumentValue::Number(NumberVariant::Long(n)) => n.try_into().map_err(|_| err()),
            DocumentValue::Number(NumberVariant::BigInt(ref b)) => b.$to_method().ok_or_else(err),
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
            DocumentValue::Number(NumberVariant::Float(f)) => num_traits::cast(f).ok_or_else(err),
            DocumentValue::Number(NumberVariant::Double(d)) => num_traits::cast(d).ok_or_else(err),
            DocumentValue::Number(NumberVariant::BigDecimal(b)) => b.$to_method().ok_or_else(err),
            _ => Err(err()),
        }
    }};
}


impl <'document> DocumentValue<'document> {
    #[inline]
    fn as_bool(&self) -> Result<bool, DocumentError> {
        match self {
            &DocumentValue::Boolean(b) => Ok(b),
            _ => Err(DocumentError::DocumentConversion("Expected Boolean Document".to_string()))
        }
    }

    #[inline]
    fn as_blob(&self) -> Result<&'document ByteBuffer, DocumentError>  {
        into_variant_d!(Blob, self, "Expected Blob Document")
    }

    #[inline]
    fn as_str(&self) -> Result<&'document str, DocumentError> {
        into_variant_d!(String, self, "Expected String Document")
    }

    #[inline]
    fn as_timestamp(&self) -> Result<Instant, DocumentError> {
        match self {
            &DocumentValue::Timestamp(t) => Ok(t),
            _ => Err(DocumentError::DocumentConversion("Expected timestamp Document".to_string()))
        }
    }

    #[inline]
    fn as_byte(&self) -> Result<i8, DocumentError> {
        to_int_d!(i8, to_i8, self)
    }

    #[inline]
    fn as_short(&self) -> Result<i16, DocumentError> {
        to_int_d!(i16, to_i16, self)
    }

    #[inline]
    fn as_integer(&self) -> Result<i32, DocumentError> {
        to_int_d!(i32, to_i32, self)
    }

    #[inline]
    fn as_long(&self) -> Result<i64, DocumentError> {
        to_int_d!(i64, to_i64, self)
    }

    fn as_big_integer(&self) -> Result<Cow<'document, BigInt>, DocumentError> {
        let err = || DocumentError::DocumentConversion(
            "Invalid conversion to BigInt".to_string()
        );

        match *self {
            DocumentValue::Number(NumberVariant::BigInt(b)) => Ok(Cow::Borrowed(b)),
            DocumentValue::Number(NumberVariant::Byte(b)) => Ok(Cow::Owned(BigInt::from(b))),
            DocumentValue::Number(NumberVariant::Short(s)) => Ok(Cow::Owned(BigInt::from(s))),
            DocumentValue::Number(NumberVariant::Integer(i)) => Ok(Cow::Owned(BigInt::from(i))),
            DocumentValue::Number(NumberVariant::Long(l)) => Ok(Cow::Owned(BigInt::from(l))),
            DocumentValue::Number(NumberVariant::Float(f)) => {
                let b = BigInt::from_f32(f).ok_or_else(err)?;
                Ok(Cow::Owned(b))
            },
            DocumentValue::Number(NumberVariant::Double(d)) => {
                let b = BigInt::from_f64(d).ok_or_else(err)?;
                Ok(Cow::Owned(b))
            },
            DocumentValue::Number(NumberVariant::BigDecimal(b)) => {
                let b = b.to_bigint().ok_or_else(err)?;
                Ok(Cow::Owned(b))
            },
            _ => Err(err()),
        }
    }

    #[inline]
    fn as_float(&self) -> Result<f32, DocumentError> {
        to_float_d!(f32, to_f32, self)
    }

    #[inline]
    fn as_double(&self) -> Result<f64, DocumentError> {
        to_float_d!(f64, to_f64, self)
    }

    fn as_big_decimal(&self) -> Result<Cow<'document, BigDecimal>, DocumentError> {
        let err = || DocumentError::DocumentConversion(
            "Invalid conversion to BigDecimal".to_string()
        );

        match *self {
            DocumentValue::Number(NumberVariant::BigDecimal(b)) => Ok(Cow::Borrowed(b)),
            DocumentValue::Number(NumberVariant::BigInt(b)) => {
                let bd = BigDecimal::from_bigint(b.clone(), 0);
                Ok(Cow::Owned(bd))
            },
            DocumentValue::Number(NumberVariant::Byte(n)) => Ok(Cow::Owned(BigDecimal::from(n))),
            DocumentValue::Number(NumberVariant::Short(n)) => Ok(Cow::Owned(BigDecimal::from(n))),
            DocumentValue::Number(NumberVariant::Integer(n)) => Ok(Cow::Owned(BigDecimal::from(n))),
            DocumentValue::Number(NumberVariant::Long(n)) => Ok(Cow::Owned(BigDecimal::from(n))),
            DocumentValue::Number(NumberVariant::Float(f)) => {
                let d = BigDecimal::from_f32(f).ok_or_else(err)?;
                Ok(Cow::Owned(d))
            },
            DocumentValue::Number(NumberVariant::Double(d)) => {
                let d = BigDecimal::from_f64(d).ok_or_else(err)?;
                Ok(Cow::Owned(d))
            },
            _ => Err(err()),
        }
    }

    #[inline]
    fn as_list(&self) -> Result<&'document dyn ListCursor, DocumentError> {
        match self {
            &DocumentValue::List(v) => Ok(v),
            _ => Err(DocumentError::DocumentConversion("Expected List".to_string()))
        }
    }

    #[inline]
    fn as_map(&self) -> Result<&'document dyn MapCursor, DocumentError> {
        match self {
            &DocumentValue::Map(v) => Ok(v),
            _ => Err(DocumentError::DocumentConversion("Expected Map".to_string()))
        }
    }

    /// TODO
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, DocumentValue::Null)
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
// Conversions INTO Default Document
// ============================================================================

impl Document for bool {
    #[inline]
    fn value(&self) -> DocumentValue<'_> {
        DocumentValue::Boolean(*self)
    }

    #[inline]
    fn take_bool(&mut self, _schema: &Schema) -> Result<bool, DocumentError> {
        Ok(*self)
    }
}

impl Document for ByteBuffer {
    #[inline]
    fn value(&self) -> DocumentValue<'_> {
        DocumentValue::Blob(self)
    }

    #[inline]
    fn take_blob(&mut self, _schema: &Schema) -> Result<ByteBuffer, DocumentError> {
        Ok(std::mem::take(self))
    }
}

impl Document for String {
    #[inline]
    fn value(&self) -> DocumentValue<'_> {
        DocumentValue::String(self)
    }

    #[inline]
    fn take_string(&mut self, _schema: &Schema) -> Result<String, DocumentError> {
        Ok(std::mem::take(self))
    }
}

impl Document for Instant {
    #[inline]
    fn value(&self) -> DocumentValue<'_> {
        DocumentValue::Timestamp(*self)
    }

    #[inline]
    fn take_timestamp(&mut self, _schema: &Schema) -> Result<Instant, DocumentError> {
        Ok(*self)
    }
}

macro_rules! integer_doc {
    ($ty:ty, $variant:ident) => {
        #[allow(trivial_casts)]
        impl Document for $ty {
            #[inline]
            fn value(&self) -> DocumentValue<'_> {
                DocumentValue::Number(NumberVariant::$variant(*self))
            }

            #[inline]
            #[allow(trivial_numeric_casts)]
            #[allow(clippy::cast_lossless)]
            fn take_byte(&mut self, _schema: &Schema) -> Result<i8, DocumentError> {
                Ok(*self as i8)
            }

            #[inline]
            #[allow(trivial_numeric_casts)]
            #[allow(clippy::cast_lossless)]
            fn take_short(&mut self, _schema: &Schema) -> Result<i16, DocumentError> {
                Ok(*self as i16)
            }

            #[inline]
            #[allow(trivial_numeric_casts)]
            #[allow(clippy::cast_lossless)]
            fn take_integer(&mut self, _schema: &Schema) -> Result<i32, DocumentError> {
                Ok(*self as i32)
            }

            #[inline]
            #[allow(trivial_numeric_casts)]
            #[allow(clippy::cast_lossless)]
            fn take_long(&mut self, _schema: &Schema) -> Result<i64, DocumentError> {
                Ok(*self as i64)
            }

            #[inline]
            fn take_big_integer(&mut self, _schema: &Schema) -> Result<BigInt, DocumentError> {
                Ok(BigInt::from(*self))
            }
        }
    };
}
integer_doc!(i8, Byte);
integer_doc!(i16, Short);
integer_doc!(i32, Integer);
integer_doc!(i64, Long);

// TODO: Should we allow integer conversions between float/int?
impl Document for f32 {
    fn value(&self) -> DocumentValue<'_> {
        DocumentValue::Number(NumberVariant::Float(*self))
    }

    fn take_float(&mut self, _schema: &Schema) -> Result<f32, DocumentError> {
        Ok(*self)
    }

    #[allow(clippy::cast_lossless)]
    fn take_double(&mut self, _schema: &Schema) -> Result<f64, DocumentError> {
        Ok(f64::from(*self))
    }

    fn take_big_decimal(&mut self, _schema: &Schema) -> Result<BigDecimal, DocumentError> {
        BigDecimal::from_f32(*self)
               .ok_or_else(|| DocumentError::DocumentConversion("Could not convert float".to_string()))
    }
}

impl Document for f64 {
    fn value(&self) -> DocumentValue<'_> {
        DocumentValue::Number(NumberVariant::Double(*self))
    }

    fn take_float(&mut self, _schema: &Schema) -> Result<f32, DocumentError> {
        Ok(*self as f32)
    }

    fn take_double(&mut self, _schema: &Schema) -> Result<f64, DocumentError> {
        Ok(*self)
    }

    fn take_big_decimal(&mut self, _schema: &Schema) -> Result<BigDecimal, DocumentError> {
        BigDecimal::from_f64(*self)
            .ok_or_else(|| DocumentError::DocumentConversion("Could not convert double".to_string()))
    }
}

impl Document for BigDecimal {
    fn value(&self) -> DocumentValue<'_> {
        DocumentValue::Number(NumberVariant::BigDecimal(self))
    }

    fn take_float(&mut self, _schema: &Schema) -> Result<f32, DocumentError> {
        todo!()
    }

    fn take_double(&mut self, _schema: &Schema) -> Result<f64, DocumentError> {
        todo!()
    }

    fn take_big_decimal(&mut self, _schema: &Schema) -> Result<BigDecimal, DocumentError> {
        Ok(std::mem::take(self))
    }
}

impl Document for BigInt {
    fn value(&self) -> DocumentValue<'_> {
        DocumentValue::Number(NumberVariant::BigInt(self))
    }

    fn take_byte(&mut self, _schema: &Schema) -> Result<i8, DocumentError> {
        todo!()
    }

    fn take_short(&mut self, _schema: &Schema) -> Result<i16, DocumentError> {
        todo!()
    }

    fn take_integer(&mut self, _schema: &Schema) -> Result<i32, DocumentError> {
        todo!()
    }

    fn take_long(&mut self, _schema: &Schema) -> Result<i64, DocumentError> {
        todo!()
    }

    fn take_big_integer(&mut self, _schema: &Schema) -> Result<BigInt, DocumentError> {
        Ok(std::mem::take(self))
    }
}

// Note: necessary for trait instantiators!
impl Document for &'static str {
    fn value(&self) -> DocumentValue<'_> {
        DocumentValue::String(self)
    }

    fn take_string(&mut self, _schema: &Schema) -> Result<String, DocumentError> {
        Ok(self.to_string())
    }
}

// Collections
impl <T: Document + Clone + 'static> Document for Vec<T> {
    #[inline]
    fn value(&self) -> DocumentValue<'_> {
        DocumentValue::List(self)
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    #[allow(trivial_casts)]
    fn get_element_mut(&mut self, index: usize) -> Result<Option<&mut dyn Document>, DocumentError> {
        Ok(self.get_mut(index).map(|item| item as &mut dyn Document))
    }
}

impl <T: Document> ListCursor for Vec<T> {
    #[inline]
    #[allow(trivial_casts)]
    #[allow(clippy::unnecessary_cast)]
    fn get(&self, idx: usize) -> Option<&dyn Document> {
        self[..].get(idx).map(|doc| doc as &dyn Document)
    }

    #[inline]
    fn len(&self) -> usize {
        self[..].len()
    }
}

impl <T: Document + Clone + 'static> Document for IndexMap<String, T> {
    fn value(&self) -> DocumentValue<'_> {
        DocumentValue::Map(self)
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    #[allow(trivial_casts)]
    fn get_entry_mut(&mut self, index: usize) -> Result<(&str, &mut dyn Document), DocumentError> {
        let Some((k, v)) = self.get_index_mut(index) else {
            return Err(DocumentError::DocumentConversion("Could not get entry".to_string()));
        };
        Ok((k.as_str(), v as &mut dyn Document))
    }

    #[inline]
    #[allow(trivial_casts)]
    fn get_key_mut(&mut self, index: usize) -> Result<Option<&mut dyn Document>, DocumentError> {
        Ok(self.get_index_mut2(index).map(|(k, _)| k as &mut dyn Document))
    }
}

impl <T: Document + Clone> MapCursor for IndexMap<String, T> {
    #[inline]
    fn get_key(&self, idx: usize) -> Option<&str> {
        self.get_index(idx).map(|(k, _)| k.as_str())
    }

    #[inline]
    #[allow(trivial_casts)]
    fn get(&self, key: &str) -> Option<&dyn Document> {
        self.get(key).map(|doc| doc as &dyn Document)
    }

    #[inline]
    #[allow(trivial_casts)]
    fn get_entry(&self, idx: usize) -> Option<(&str, &dyn Document)> {
        self.get_index(idx).map(|(k, v)| (k.as_str(), v as &dyn Document))
    }

    fn len(&self) -> usize {
        self.len()
    }
}

// TODO: Is this right?
impl <T: Document + Clone + 'static> Document for Option<T> {
    fn value(&self) -> DocumentValue<'_> {
        self.as_ref().map_or(DocumentValue::Null, |v| v.value())
    }
}

// Blanket conversion into a boxed document type
impl <T: Document + 'static> From<T> for Box<dyn Document> {
    #[inline]
    #[allow(trivial_casts)]
    fn from(value: T) -> Self {
        Box::new(value) as Box<dyn Document>
    }
}

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
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError>;
}

// === Impls for basic types ===

// TODO: MOVE
impl Document for IndexMap<String, Box<dyn Document>> {
    fn value(&self) -> DocumentValue<'_> {
        todo!()
    }
}

impl Document for Vec<Box<dyn Document>> {
    fn value(&self) -> DocumentValue<'_> {
        todo!()
    }
}

impl TryFromDocument for bool {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_bool(&BOOLEAN)
    }
}

impl TryFromDocument for i8 {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_byte(&BYTE)
    }
}

impl TryFromDocument for i16 {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_short(&SHORT)
    }
}

impl TryFromDocument for i32 {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_integer(&INTEGER)
    }
}

impl TryFromDocument for i64 {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_long(&LONG)
    }
}

impl TryFromDocument for f32 {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_float(&FLOAT)
    }
}

impl TryFromDocument for f64 {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_double(&DOUBLE)
    }
}

impl TryFromDocument for BigInt {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_big_integer(&BIG_INTEGER)
    }
}

impl TryFromDocument for BigDecimal {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_big_decimal(&BIG_DECIMAL)
    }
}

impl TryFromDocument for ByteBuffer {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_blob(&BLOB)
    }
}

impl TryFromDocument for String {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_string(&STRING)
    }
}

impl TryFromDocument for Instant {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_timestamp(&TIMESTAMP)
    }
}

impl<T: TryFromDocument> TryFromDocument for Vec<T> {
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        let len = document.len();
        if len == 0 {
            return Ok(Vec::new());
        }
        let mut idx = 0;
        let mut result = Vec::with_capacity(len);
        loop {
            let Some(inner) = document.get_element_mut(idx)? else {
                break;
            };
            idx += 1;
            result.push(T::try_from(inner)?);
        }
        Ok(result)
    }
}

impl<T: TryFromDocument> TryFromDocument for IndexMap<String, T> {
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        let len = document.len();
        if len == 0 {
            return Ok(IndexMap::new());
        }
        let mut idx = 0;
        let mut result = IndexMap::with_capacity(len);
        while idx < len {
            let (k, v) = document.get_entry_mut(idx)?;
            idx += 1;
            result.insert(k.to_string(), T::try_from(v)?);
        }
        Ok(result)
    }
}

impl<T: TryFromDocument> TryFromDocument for Option<T> {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        if document.is_null() {
            Ok(None)
        } else {
            Ok(Some(T::try_from(document)?))
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
    pub fn try_into<S: TryFromDocument>(mut self: Box<Self>) -> Result<S, DocumentError> {
        <S as TryFromDocument>::try_from(&mut *self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_document_value() {
        let document_str: Box<dyn Document> = "MyStr".into();
        let output_str: String = document_str.as_str().unwrap().to_string();
        assert_eq!(output_str, "MyStr".to_string());

        let document_string: Box<dyn Document> = "MyString".into();
        let output_string: String = document_string.as_str().unwrap().to_string();
        assert_eq!(&output_string, &"MyString");
    }

    #[test]
    fn list_document_value() {
        let vec = vec!["a", "b", "c"];
        let document_list: Box<dyn Document> = vec.into();
        assert_eq!(document_list.len(), 3);
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
        assert_eq!(map_doc.len(), 1);

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
        let float: Box<dyn Document> = 1f32.into();
        let double: Box<dyn Document> = 1f64.into();

        let float_value: f32 = float.try_into().unwrap();
        assert_eq!(float_value, 1f32);
        let double_value: f64 = double.try_into().unwrap();
        assert_eq!(double_value, 1f64);
    }
}
