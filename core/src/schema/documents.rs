use core::fmt;
use std::{error::Error, fmt::Debug, sync::LazyLock};
use std::borrow::Cow;
use std::fmt::Formatter;
use bigdecimal::{num_traits, FromPrimitive, ToPrimitive};
use num_bigint::ToBigInt;
use thiserror::Error;

use crate::{BigDecimal, BigInt, ByteBuffer, IndexMap, Instant, schema::{
    ShapeId,
    prelude::*,
}, serde::validation::ValidationErrors};
use crate::serde::de::Deserializer;
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
pub trait Document: Send + Sync + BoxClone + DocumentReader {
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
    fn value(&self) -> DocumentValue<'_>;

    /// Into converters

}

// ===== Clone support =====

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

// ==== As-Ers =====
impl dyn Document {
    /// Get the `boolean` value of the Document if it is a `boolean` or can be converted into one.
    ///
    /// # Error
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

// Converters

// TODO

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

    /// Number of elements in map
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

    #[inline]
    fn is_null(&self) -> bool {
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

// ================================================================
// Owned Document Data View
// ================================================================
impl Deserializer<'_> for DocumentData {
    type Error = DocumentError;
    type StructReader = ();
    type ListReader = ();
    type MapReader = ();



}

// ============================================================================
// Conversions INTO Default Document
// ============================================================================

macro_rules! doc_impl_primitive {
    ($ty:ty, $value_expr:expr, $data_expr:expr) => {
       impl Document for $ty {
            #[inline]
            fn value(&self) -> DocumentValue<'_> {
                $value_expr(*self)
            }

            #[inline]
            fn data(self) -> DocumentData {
                $data_expr(self)
            }
        }
    };
    (ref $ty:ty, $value_expr:expr, $data_expr:expr) => {
       impl Document for $ty {
            #[inline]
            fn value(&self) -> DocumentValue<'_> {
                $value_expr(self)
            }

            #[inline]
            fn data(self) -> DocumentData {
                $data_expr(self)
            }
        }
    };
}
doc_impl_primitive!(bool, DocumentValue::Boolean, DocumentData::Boolean);
doc_impl_primitive!(ref ByteBuffer, DocumentValue::Blob, DocumentData::Blob);
doc_impl_primitive!(ref String, DocumentValue::String, DocumentData::String);
doc_impl_primitive!(Instant, DocumentValue::Timestamp, DocumentData::Timestamp);
doc_impl_primitive!(
    i8,
    |v| DocumentValue::Number(NumberVariant::Byte(v)),
    |v| DocumentData::Number(NumberData::Byte(v))
);
doc_impl_primitive!(
    i16,
    |v| DocumentValue::Number(NumberVariant::Short(v)),
    |v| DocumentData::Number(NumberData::Short(v))
);
doc_impl_primitive!(
    i32,
    |v| DocumentValue::Number(NumberVariant::Integer(v)),
    |v| DocumentData::Number(NumberData::Integer(v))
);
doc_impl_primitive!(i64,
    |v| DocumentValue::Number(NumberVariant::Long(v)),
    |v| DocumentData::Number(NumberData::Long(v))
);
doc_impl_primitive!(f32,
    |v| DocumentValue::Number(NumberVariant::Float(v)),
    |v| DocumentData::Number(NumberData::Float(v))
);
doc_impl_primitive!(f64,
    |v| DocumentValue::Number(NumberVariant::Double(v)),
    |v| DocumentData::Number(NumberData::Double(v))
);
doc_impl_primitive!(ref BigInt,
    |v| DocumentValue::Number(NumberVariant::BigInt(v)),
    |v| DocumentData::Number(NumberData::BigInt(v))
);
doc_impl_primitive!(ref BigDecimal,
    |v| DocumentValue::Number(NumberVariant::BigDecimal(v)),
    |v| DocumentData::Number(NumberData::BigDecimal(v))
);

// Note: necessary for trait instantiators!
impl Document for &'static str {
    fn value(&self) -> DocumentValue<'_> {
        DocumentValue::String(self)
    }

    fn data(self) -> DocumentData {
        DocumentData::String(self.to_string())
    }
}

// Collections
impl <T: Document + Clone + 'static> Document for Vec<T> {
    fn value(&self) -> DocumentValue<'_> {
        DocumentValue::List(self)
    }

    fn data(self) -> DocumentData {
        todo!()
    }
}

impl <T: Document> ListCursor for Vec<T> {
    #[inline]
    #[allow(trivial_casts)]
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

    fn data(self) -> DocumentData {
        todo!()
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
impl <T: Document + Clone + 'static> Document for Option<T> {
    fn value(&self) -> DocumentValue<'_> {
        match self {
            Some(v) => v.value(),
            None => DocumentValue::Null
        }
    }

    fn data(self) -> DocumentData {
        todo!()
    }
}

// Blanket conver
impl <T: Document + 'static> From<T> for Box<dyn Document> {
    #[inline]
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

// ============================================================================
// Conversions FROM Document into Shape
// ============================================================================

/// Owned view of the document datat
///
/// Used for deserialization / conversion
pub enum DocumentData {
    /// An empty (null) document
    Null,
    /// A `boolean` document
    Boolean(bool),
    /// A number document
    Number(NumberData),
    /// A `string` document
    String(String),
    /// A `blob` document
    Blob(ByteBuffer),
    /// A `timestamp` document
    Timestamp(Instant),
    /// A `list` document
    /// TODO
    List(String),
    /// A `map` document
    /// TODO
    Map(String),
}

/// A numeric document type in the Smithy data model
pub enum NumberData {
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
        if matches!(document.as_variant(), DocumentValue::Null) {
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
