use core::fmt;
use std::{borrow::Cow, error::Error, fmt::Debug, mem};

use bigdecimal::{FromPrimitive};
use indexmap::map::MutableKeys;
use thiserror::Error;

use crate::{
    BigDecimal, BigInt, ByteBuffer, IndexMap, Instant,
    schema::{Schema, ShapeId, prelude::*},
    serde::validation::ValidationErrors,
};
use crate::schema::StaticSchemaShape;
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
/// - `map`: map of int|long|string keys to Document values (also represents `structure` and `union`)
///
/// This data model is represented by the [`DocumentType`] enum
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
    /// Get the discriminant (type ID) of a type document
    ///
    /// The discriminant is primarily used to implement polymorphism using documents in deserialization.
    ///
    /// <div class="warning">
    /// ### Protocol Support
    ///
    /// It is expected that protocols set the discriminant on deserialization if applicable
    /// </div>
    ///
    /// Default implementation is `None`
    #[must_use]
    fn discriminant(&self) -> Option<&ShapeId> {
        None
    }

    /// Get what data type the document represents in the Smithy data model
    fn document_type(&self) -> DocumentType;

    // === As-ers methods for read-only views of document data
    // Note: All as-ers default to `None`

    /// Get the `boolean` value of the Document if it is a `boolean` or can be converted into one.
    ///
    /// Returns `None` if Document could not be represented as a `boolean`.
    fn as_boolean(&self) -> Option<bool> {
        None
    }

    /// Get the `blob` value of the Document if it is a `blob` or can be converted into one.
    ///
    /// Returns `None` if Document could not be represented as a `blob`.
    fn as_blob(&self) -> Option<&ByteBuffer> {
        None
    }

    /// Get the `string` value of the Document if it is a `string` or can be converted into one.
    ///
    /// Returns `None` if Document could not be represented as a `string`.
    fn as_str(&self) -> Option<&str> {
        None
    }

    /// Get the `timestamp` value of the Document if it is a `timestamp` or can be converted into one.
    ///
    /// Returns `None` if Document could not be represented as a `timestamp`.
    fn as_timestamp(&self) -> Option<Instant> {
        None
    }

    /// Get the `byte` value of the Document if it is a `byte` or can be converted into one.
    ///
    /// Returns `None` if Document could not be represented as a `byte`.
    fn as_byte(&self) -> Option<i8> {
        None
    }

    /// Get the `short` value of the Document if it is a `short` or can be converted into one.
    ///
    /// Returns `None` if Document could not be represented as a `short`.
    fn as_short(&self) -> Option<i16> {
        None
    }

    /// Get the `integer` value of the Document if it is an `integer` or can be converted into one.
    ///
    /// Returns `None` if Document could not be represented as an `integer`.
    fn as_integer(&self) -> Option<i32> {
        None
    }

    /// Get the `long` value of the Document if it is a `long` or can be converted into one.
    ///
    /// Returns `None` if Document could not be represented as a `long`.
    fn as_long(&self) -> Option<i64> {
        None
    }

    /// Get the `big_integer` value of the Document if it is a `big_integer` or can be converted into one.
    ///
    /// Returns `None` if Document could not be represented as a `big_integer`.
    fn as_big_integer(&self) -> Option<Cow<'_, BigInt>> {
        None
    }

    /// Get the `float` value of the Document if it is a `float` or can be converted into one.
    ///
    /// Returns `None` if Document could not be represented as a `float`.
    fn as_float(&self) -> Option<f32> {
        None
    }

    /// Get the `double` value of the Document if it is a `double` or can be converted into one.
    ///
    /// Returns `None` if Document could not be represented as a `double`.
    fn as_double(&self) -> Option<f64> {
        None
    }

    /// Get the `big_decimal` value of the Document if it is a `big_decimal` or can be converted into one.
    ///
    /// Returns `None` if Document could not be represented as a `big_decimal`.
    fn as_big_decimal(&self) -> Option<Cow<'_, BigDecimal>> {
        None
    }

    /// Get a `list` cursor for the Document if it is a `list`.
    ///
    /// Returns `None` if document is not a list
    fn as_list(&self) -> Option<&dyn ListAccessor> {
        None
    }

    /// Get a `map` cursor for the Document if it is a map or can be represented as one.
    ///
    /// Returns `None` if document is not representable as a map.
    fn as_map(&self) -> Option<&dyn MapAccessor> {
        None
    }

    /// Check if the Document is Null (empty)
    fn null(&self) -> bool {
        matches!(self.document_type(), DocumentType::Null)
    }

    // === Consumers for deserialization ====

    /// Read the document as a `boolean` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `boolean`
    fn take_bool(&mut self, _schema: &Schema) -> Result<bool, DocumentError> {
        Err(take_error("boolean"))
    }

    /// Read the document as a `blob` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `blob`
    fn take_blob(&mut self, _schema: &Schema) -> Result<ByteBuffer, DocumentError> {
        Err(take_error("blob"))
    }

    /// Read the document as a `string` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `string`
    fn take_string(&mut self, _schema: &Schema) -> Result<String, DocumentError> {
        Err(take_error("string"))
    }

    /// Read the document as a `byte` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `byte`
    fn take_byte(&mut self, _schema: &Schema) -> Result<i8, DocumentError> {
        Err(take_error("byte"))
    }

    /// Read the document as a `short` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `short`
    fn take_short(&mut self, _schema: &Schema) -> Result<i16, DocumentError> {
        Err(take_error("short"))
    }

    /// Read the document as a `integer` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as an `integer`
    fn take_integer(&mut self, _schema: &Schema) -> Result<i32, DocumentError> {
        Err(take_error("integer"))
    }

    /// Read the document as a `long` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `long`
    fn take_long(&mut self, _schema: &Schema) -> Result<i64, DocumentError> {
        Err(take_error("long"))
    }

    /// Read the document as a `big_integer` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `big_integer`
    fn take_big_integer(&mut self, _schema: &Schema) -> Result<BigInt, DocumentError> {
        Err(take_error("big_integer"))
    }

    /// Read the document as a `float` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `float`
    fn take_float(&mut self, _schema: &Schema) -> Result<f32, DocumentError> {
        Err(take_error("float"))
    }

    /// Read the document as a `double` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `double`
    fn take_double(&mut self, _schema: &Schema) -> Result<f64, DocumentError> {
        Err(take_error("double"))
    }

    /// Read the document as a `big_decimal` if possible
    ///
    /// Default implementation errors
    ///
    /// # Errors
    /// Raises `DocumentError` if the Document could not be read as a `big_decimal`
    fn take_big_decimal(&mut self, _schema: &Schema) -> Result<BigDecimal, DocumentError> {
        Err(take_error("big_decimal"))
    }

    /// Get an accessor for a list document
    ///
    /// # Errors
    /// Raises a `DocumentError` if the document is not a list
    fn take_list(&mut self) -> Result<&mut dyn ListAccessor, DocumentError> {
        Err(take_error("list"))
    }

    /// Get an accessor for a map document
    ///
    /// # Errors
    /// Raises a `DocumentError` if the document is not a list
    fn take_map(&mut self) -> Result<&mut dyn MapAccessor, DocumentError> {
        Err(take_error("map"))
    }
}

#[cold]
#[inline(never)]
fn take_error(expected: &'static str) -> DocumentError {
    DocumentError::DocumentConversion(format!("expected {expected}."))
}

// ================================================================
// Data model
// ================================================================

/// Document data model representation of untyped data in the Smithy data model
///
/// The Smithy data model consists of:
/// - Numbers: `byte`, `short`, `integer`, `long`, `float`, `double`, `bigInteger`, `bigDecimal`.
///   `IntEnum` shapes are represented as integers in the Smithy data model.
/// - `boolean`
/// - `blob`
/// - `string`: `enum` shapes are represented as strings in the Smithy data model
/// - `timestamp`: Represented as an [`Instant`]
/// - `list`: list of Documents
/// - `map`: map of int|long|string keys to Document values (also represents `structure` and `union`)
pub enum DocumentType {
    /// An empty (null) document
    Null,
    /// A `boolean` document
    Boolean,
    /// A number document
    Number(Number),
    /// A `string` document
    String,
    /// A `blob` document
    Blob,
    /// A `list` document
    List,
    /// A `map` document
    ///
    /// Also used to represent `structure`'s and `union`'s
    Map,
}

/// A numeric document type in the Smithy data model
pub enum Number {
    /// Smithy `byte` type
    Byte,
    /// Smithy `short` type
    Short,
    /// Smithy `integer` type
    Integer,
    /// Smithy `long` type
    Long,
    /// Smithy `big_integer` type
    BigInt,
    /// Smithy `float` type
    Float,
    /// Smithy `double` type
    Double,
    /// Smithy `big_decimal` type
    BigDecimal,
}

// ================================================================
// Accessor Traits
// ================================================================

/// Provides Access to List Document items
pub trait ListAccessor {
    /// Get an element by index if possible
    fn get(&self, idx: usize) -> Option<&(dyn Document + '_)>;

    /// Get an element of a list document
    ///
    /// Returns `None` if index out of bounds
    fn get_element_mut(&mut self, index: usize) -> Option<&mut dyn Document>;

    /// Number of list elements
    fn len(&self) -> usize;

    /// If the list is empty
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Provides Access to Map Document entries
pub trait MapAccessor {
    /// Used to query for a specific key
    fn get(&self, key: &str) -> Option<&(dyn Document + '_)>;

    /// Get entry for a specific idx
    fn get_entry(&self, idx: usize) -> Option<(&str, &dyn Document)>;

    /// Get a map document entry
    ///
    /// TODO: Should we update to split from key?
    /// Note: Should only really be used by deserializers.
    /// # Errors
    /// Returns a document error if no entry is available for the index
    fn get_entry_mut(&mut self, index: usize) -> Result<(&str, &mut dyn Document), DocumentError>;

    /// Get the map key for the given index
    ///
    /// Returns `None` if index is out of bounds
    fn get_key_mut(&mut self, index: usize) -> Option<Cow<'_, str>>;

    /// Number of entries map document;
    fn len(&self) -> usize;

    /// If the Map is empty
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ================================================================
// Clone Support
// ================================================================

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
// Blanket partial eq for dyn Documents
// ================================================================

impl PartialEq for dyn Document + '_ {
    fn eq(&self, other: &Self) -> bool {
        match self.document_type() {
            DocumentType::Null => other.null(),
            DocumentType::Boolean => self.as_boolean() == other.as_boolean(),
            DocumentType::Number(number) => match number {
                Number::Byte => self.as_byte() == other.as_byte(),
                Number::Short => self.as_short() == other.as_short(),
                Number::Integer => self.as_integer() == other.as_integer(),
                Number::Long => self.as_long() == other.as_long(),
                Number::BigInt => self.as_big_integer() == other.as_big_integer(),
                Number::Float => self.as_float() == other.as_float(),
                Number::Double => self.as_double() == other.as_double(),
                Number::BigDecimal => self.as_big_decimal() == other.as_big_decimal(),
            },
            DocumentType::String => self.as_str() == other.as_str(),
            DocumentType::Blob => self.as_blob() == other.as_blob(),
            DocumentType::List => {
                let (Some(tc), Some(oc)) = (self.as_list(), other.as_list()) else {
                    return false;
                };
                if tc.len() != oc.len() {
                    return false;
                }
                for i in 0..tc.len() {
                    if tc.get(i) != oc.get(i) {
                        return false;
                    }
                }
                true
            }
            DocumentType::Map => {
                let (Some(tc), Some(oc)) = (self.as_map(), other.as_map()) else {
                    return false;
                };
                if tc.len() != oc.len() {
                    return false;
                }
                for i in 0..tc.len() {
                    if tc.get_entry(i) != oc.get_entry(i) {
                        return false;
                    }
                }
                true
            }
        }
    }
}

// ================================================================
// Blanket convenience impls for generated shapes
// ================================================================

impl <T: StaticSchemaShape + MapAccessor + Send + Sync + Clone + 'static> Document for T {
    #[inline]
    fn discriminant(&self) -> Option<&ShapeId> {
        Some(Self::schema().id())
    }

    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Map
    }

    #[inline]
    fn as_map(&self) -> Option<&dyn MapAccessor> {
        Some(self)
    }

    #[inline]
    fn take_map(&mut self) -> Result<&mut dyn MapAccessor, DocumentError> {
        Ok(self)
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

macro_rules! to_int {
    ($target_type:ident, $to_method:ident, $value:expr) => {{
        match *$value {
            DocumentValue::Number(NumberVariant::Byte(n)) => n.try_into().ok(),
            DocumentValue::Number(NumberVariant::Short(n)) => n.try_into().ok(),
            DocumentValue::Number(NumberVariant::Integer(n)) => n.try_into().ok(),
            DocumentValue::Number(NumberVariant::Long(n)) => n.try_into().ok(),
            DocumentValue::Number(NumberVariant::BigInt(ref b)) => b.$to_method(),
            _ => None,
        }
    }};
}

macro_rules! to_float {
    ($target_type:ident, $to_method:ident, $value:expr) => {{
        match *$value {
            DocumentValue::Number(NumberVariant::Float(f)) => num_traits::cast(f),
            DocumentValue::Number(NumberVariant::Double(d)) => num_traits::cast(d),
            DocumentValue::Number(NumberVariant::BigDecimal(b)) => b.$to_method(),
            _ => None,
        }
    }};
}

// ============================================================================
// Conversions INTO Default Document
// ============================================================================

impl Document for bool {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Boolean
    }

    fn as_boolean(&self) -> Option<bool> {
        Some(*self)
    }

    #[inline]
    fn take_bool(&mut self, _schema: &Schema) -> Result<bool, DocumentError> {
        Ok(*self)
    }
}

impl Document for ByteBuffer {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Blob
    }

    #[inline]
    fn as_blob(&self) -> Option<&ByteBuffer> {
        Some(self)
    }

    #[inline]
    fn take_blob(&mut self, _schema: &Schema) -> Result<ByteBuffer, DocumentError> {
        Ok(mem::take(self))
    }
}

impl Document for String {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::String
    }

    #[inline]
    fn as_str(&self) -> Option<&str> {
        Some(self)
    }

    #[inline]
    fn take_string(&mut self, _schema: &Schema) -> Result<String, DocumentError> {
        Ok(mem::take(self))
    }
}

// TODO: How to handle timestamps? They can be both?
impl Document for Instant {
    #[inline]
    fn document_type(&self) -> DocumentType {
        todo!()
    }
}

macro_rules! integer_doc {
    ($ty:ty, $variant:ident) => {
        #[allow(trivial_casts)]
        impl Document for $ty {
            #[inline]
            fn document_type(&self) -> DocumentType {
                DocumentType::Number(Number::$variant)
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
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Number(Number::Float)
    }

    #[inline]
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
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Number(Number::Double)
    }

    #[inline]
    fn take_float(&mut self, _schema: &Schema) -> Result<f32, DocumentError> {
        Ok(*self as f32)
    }

    #[inline]
    fn take_double(&mut self, _schema: &Schema) -> Result<f64, DocumentError> {
        Ok(*self)
    }

    #[inline]
    fn take_big_decimal(&mut self, _schema: &Schema) -> Result<BigDecimal, DocumentError> {
        BigDecimal::from_f64(*self).ok_or_else(|| {
            DocumentError::DocumentConversion("Could not convert double".to_string())
        })
    }
}

impl Document for BigDecimal {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Number(Number::BigDecimal)
    }

    fn take_float(&mut self, _schema: &Schema) -> Result<f32, DocumentError> {
        todo!()
    }

    fn take_double(&mut self, _schema: &Schema) -> Result<f64, DocumentError> {
        todo!()
    }

    fn take_big_decimal(&mut self, _schema: &Schema) -> Result<BigDecimal, DocumentError> {
        Ok(mem::take(self))
    }
}

// TODO: Implement
impl Document for BigInt {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Number(Number::BigInt)
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
        Ok(mem::take(self))
    }
}

// Note: necessary for trait instantiators!
impl Document for &'static str {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::String
    }

    #[inline]
    fn as_str(&self) -> Option<&str> {
        Some(self)
    }

    #[inline]
    fn take_string(&mut self, _schema: &Schema) -> Result<String, DocumentError> {
        Ok(self.to_string())
    }
}

// Collections
impl<T: Document + Clone + 'static> Document for Vec<T> {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::List
    }

    #[inline]
    fn as_list(&self) -> Option<&dyn ListAccessor> {
        Some(self)
    }

    #[inline]
    fn take_list(&mut self) -> Result<&mut dyn ListAccessor, DocumentError> {
        Ok(self)
    }
}

impl<T: Document> ListAccessor for Vec<T> {
    #[inline]
    #[allow(trivial_casts)]
    #[allow(clippy::unnecessary_cast)]
    fn get(&self, idx: usize) -> Option<&(dyn Document + '_)> {
        self[..].get(idx).map(|doc| doc as &dyn Document)
    }

    #[inline]
    #[allow(trivial_casts)]
    fn get_element_mut(&mut self, index: usize) -> Option<&mut dyn Document> {
        self.get_mut(index).map(|item| item as &mut dyn Document)
    }

    #[inline]
    fn len(&self) -> usize {
        self[..].len()
    }
}

impl<T: Document + Clone + 'static> Document for IndexMap<String, T> {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Map
    }

    #[inline]
    fn as_map(&self) -> Option<&dyn MapAccessor> {
        Some(self)
    }

    #[inline]
    #[allow(trivial_casts)]
    fn take_map(&mut self) -> Result<&mut dyn MapAccessor, DocumentError> {
        Ok(&mut *self as &mut dyn MapAccessor)
    }
}

impl<T: Document> MapAccessor for IndexMap<String, T> {
    #[inline]
    #[allow(trivial_casts)]
    fn get(&self, key: &str) -> Option<&dyn Document> {
        self.get(key).map(|doc| doc as &dyn Document)
    }

    #[inline]
    #[allow(trivial_casts)]
    fn get_entry(&self, idx: usize) -> Option<(&str, &dyn Document)> {
        self.get_index(idx)
            .map(|(k, v)| (k.as_str(), v as &dyn Document))
    }

    #[inline]
    #[allow(trivial_casts)]
    fn get_entry_mut(&mut self, index: usize) -> Result<(&str, &mut dyn Document), DocumentError> {
        let Some((k, v)) = self.get_index_mut(index) else {
            return Err(DocumentError::DocumentConversion(
                "Could not get entry".to_string(),
            ));
        };
        Ok((k.as_str(), v as &mut dyn Document))
    }

    #[inline]
    #[allow(trivial_casts)]
    fn get_key_mut(&mut self, index: usize) -> Option<Cow<'_, str>> {
        self.get_index_mut2(index)
            .map(|(k, _)| Cow::Owned(mem::take(k)))
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T: Document + Clone + 'static> Document for Option<T> {
    #[inline]
    fn document_type(&self) -> DocumentType {
        self.as_ref().map_or(DocumentType::Null, Document::document_type)
    }

    // ==== Delegation of As-ers to inner document type ====

    #[inline]
    fn as_boolean(&self) -> Option<bool> {
        self.as_ref().and_then(Document::as_boolean)
    }

    #[inline]
    fn as_blob(&self) -> Option<&ByteBuffer> {
        self.as_ref().and_then(Document::as_blob)
    }

    #[inline]
    fn as_str(&self) -> Option<&str> {
        self.as_ref().and_then(Document::as_str)
    }

    #[inline]
    fn as_byte(&self) -> Option<i8> {
        self.as_ref().and_then(Document::as_byte)
    }

    #[inline]
    fn as_short(&self) -> Option<i16> {
        self.as_ref().and_then(Document::as_short)
    }

    #[inline]
    fn as_integer(&self) -> Option<i32> {
        self.as_ref().and_then(Document::as_integer)
    }

    #[inline]
    fn as_long(&self) -> Option<i64> {
        self.as_ref().and_then(Document::as_long)
    }

    #[inline]
    fn as_big_integer(&self) -> Option<Cow<'_, BigInt>> {
        self.as_ref().and_then(Document::as_big_integer)
    }

    #[inline]
    fn as_float(&self) -> Option<f32> {
        self.as_ref().and_then(Document::as_float)
    }

    #[inline]
    fn as_double(&self) -> Option<f64> {
        self.as_ref().and_then(Document::as_double)
    }

    #[inline]
    fn as_big_decimal(&self) -> Option<Cow<'_, BigDecimal>> {
        self.as_ref().and_then(Document::as_big_decimal)
    }

    #[inline]
    fn as_list(&self) -> Option<&dyn ListAccessor> {
        self.as_ref().and_then(Document::as_list)
    }

    #[inline]
    fn as_map(&self) -> Option<&dyn MapAccessor> {
        self.as_ref().and_then(Document::as_map)
    }

    #[inline]
    fn null(&self) -> bool {
        self.as_ref().is_none_or(Document::null)
    }

    // ==== Delegation of takers to inner document type ====

    // TODO: Errors could be clearer I think?

    #[inline]
    fn take_bool(&mut self, schema: &Schema) -> Result<bool, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::DocumentConversion("null entry read".to_string()))?
            .take_bool(schema)
    }

    #[inline]
    fn take_blob(&mut self, schema: &Schema) -> Result<ByteBuffer, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::DocumentConversion("null entry read".to_string()))?
            .take_blob(schema)
    }

    #[inline]
    fn take_string(&mut self, schema: &Schema) -> Result<String, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::DocumentConversion("null entry read".to_string()))?
            .take_string(schema)
    }

    #[inline]
    fn take_byte(&mut self, schema: &Schema) -> Result<i8, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::DocumentConversion("null entry read".to_string()))?
            .take_byte(schema)
    }

    #[inline]
    fn take_short(&mut self, schema: &Schema) -> Result<i16, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::DocumentConversion("null entry read".to_string()))?
            .take_short(schema)
    }

    #[inline]
    fn take_integer(&mut self, schema: &Schema) -> Result<i32, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::DocumentConversion("null entry read".to_string()))?
            .take_integer(schema)
    }

    #[inline]
    fn take_long(&mut self, schema: &Schema) -> Result<i64, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::DocumentConversion("null entry read".to_string()))?
            .take_long(schema)
    }

    #[inline]
    fn take_big_integer(&mut self, schema: &Schema) -> Result<BigInt, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::DocumentConversion("null entry read".to_string()))?
            .take_big_integer(schema)
    }

    #[inline]
    fn take_float(&mut self, schema: &Schema) -> Result<f32, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::DocumentConversion("null entry read".to_string()))?
            .take_float(schema)
    }

    #[inline]
    fn take_double(&mut self, schema: &Schema) -> Result<f64, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::DocumentConversion("null entry read".to_string()))?
            .take_double(schema)
    }

    #[inline]
    fn take_big_decimal(&mut self, schema: &Schema) -> Result<BigDecimal, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::DocumentConversion("null entry read".to_string()))?
            .take_big_decimal(schema)
    }

    #[inline]
    fn take_list(&mut self) -> Result<&mut dyn ListAccessor, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::DocumentConversion("null entry read".to_string()))?
            .take_list()
    }

    #[inline]
    fn take_map(&mut self) -> Result<&mut dyn MapAccessor, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::DocumentConversion("null entry read".to_string()))?
            .take_map()
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
    fn try_from<'a>(_document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        todo!()
    }
}

impl<T: TryFromDocument> TryFromDocument for Vec<T> {
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        let list = document.take_list()?;
        if list.is_empty() {
            return Ok(Vec::new());
        }
        let mut idx = 0;
        let mut result = Vec::with_capacity(list.len());
        while let Some(inner) = list.get_element_mut(idx) {
            idx += 1;
            result.push(T::try_from(inner)?);
        }
        Ok(result)
    }
}

impl<T: TryFromDocument> TryFromDocument for IndexMap<String, T> {
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        let map = document.take_map()?;
        if map.is_empty() {
            return Ok(IndexMap::new());
        }
        let mut idx = 0;
        let mut result = IndexMap::with_capacity(map.len());
        while idx < map.len() {
            let (k, v) = map.get_entry_mut(idx)?;
            idx += 1;
            result.insert(k.to_string(), T::try_from(v)?);
        }
        Ok(result)
    }
}

impl<T: TryFromDocument> TryFromDocument for Option<T> {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        if document.null() {
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

// ============================================================================
// Boxed (owned) variant support
// ============================================================================

impl Document for Vec<Box<dyn Document>> {
    fn document_type(&self) -> DocumentType {
        DocumentType::List
    }

    fn as_list(&self) -> Option<&dyn ListAccessor> {
        todo!()
    }

    fn take_list(&mut self) -> Result<&mut dyn ListAccessor, DocumentError> {
        todo!()
    }
}

impl Document for IndexMap<String, Box<dyn Document>> {
    fn document_type(&self) -> DocumentType {
        DocumentType::Map
    }

    fn as_map(&self) -> Option<&dyn MapAccessor> {
        todo!()
    }
}

// Blanket conversion into a boxed document type
impl<T: Document + 'static> From<T> for Box<dyn Document> {
    #[inline]
    #[allow(trivial_casts)]
    fn from(value: T) -> Self {
        Box::new(value) as Box<dyn Document>
    }
}

impl Clone for Box<dyn Document> {
    #[inline]
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==== As-ers ====
    #[test]
    fn string_document_value() {
        let document_str: Box<dyn Document> = "MyStr".into();
        let output_str = document_str
            .as_str()
            .expect("Could not get document as str");
        assert_eq!(output_str, "MyStr");
    }

    #[test]
    fn list_document_value() {
        let vec = vec!["a", "b", "c"];
        let document_list: Box<dyn Document> = vec.into();
        assert_eq!(document_list.as_list().expect("list").len(), 3);
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
        assert_eq!(map_doc.as_map().expect("map").len(), 1);

        let map_out: IndexMap<String, String> =
            map_doc.try_into().expect("Failed to convert document");
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
        let float_value: f32 = float.try_into().expect("Failed to convert float");
        assert_eq!(float_value, 1f32);
    }

    #[test]
    fn double_document_value() {
        let double: Box<dyn Document> = 1f64.into();
        let double_value: f64 = double.try_into().expect("Failed to convert double");
        assert_eq!(double_value, 1f64);
    }
}
