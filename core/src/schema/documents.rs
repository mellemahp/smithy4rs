use std::{error::Error, fmt::Debug, sync::LazyLock};
use bigdecimal::{num_traits, FromPrimitive, ToPrimitive};
use num_bigint::ToBigInt;
use thiserror::Error;

use crate::{
    BigDecimal, BigInt, ByteBuffer, IndexMap, Instant,
    schema::{
        SchemaShape, ShapeId, ShapeType,
        prelude::*,
    },
    serde::validation::ValidationErrors,
};
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

    // TODO: Does it need to be optional?
    /// Get the Shape Type for the underlying contents of the document.
    ///
    /// The type returned from this method will differ from the type of the document (which will
    /// always be [`ShapeType::Document`]).
    ///
    /// NOTE: It is important that `self.get_type()` NEVER return `Document` as the type as that
    /// will result in infinite loops in serializers.
    ///
    /// ### Enums
    /// - `enum` shapes: Enum shapes are treated as a `string`, and variants can be found in
    ///   the corresponding schema for the document.
    ///  - `intEnum` shapes: Enum shapes are treated as an `integer`, and variants can be found in
    ///    the corresponding schema for the document.
    #[must_use]
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
    /// # Errors
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `blob` ([`ByteBuffer`]) value.
    fn into_blob(self: Box<Self>) -> Result<ByteBuffer, DocumentError>;

    /// Convert the [`Document`] to a `boolean`.
    ///
    /// # Errors
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `boolean` value.
    fn into_bool(self: Box<Self>) -> Result<bool, DocumentError>;

    /// Convert the [`Document`] to a `string`.
    ///
    /// # Errors
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `string` value.
    fn into_string(self: Box<Self>) -> Result<String, DocumentError>;

    /// Convert the [`Document`] to a `timestamp`.
    ///
    /// # Errors
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `timestamp` ([`Instant`]) value.
    fn into_timestamp(self: Box<Self>) -> Result<Instant, DocumentError>;

    /// Convert the [`Document`] to a `byte`.
    ///
    /// # Errors
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `byte` (`i8`) value.
    fn into_byte(self: Box<Self>) -> Result<i8, DocumentError>;

    /// Convert the [`Document`] to a `short`.
    ///
    /// # Errors
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `short` (`i16`) value.
    fn into_short(self: Box<Self>) -> Result<i16, DocumentError>;

    /// Convert the [`Document`] to an `integer`.
    ///
    /// # Errors
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `integer` (`i32`) value.
    fn into_integer(self: Box<Self>) -> Result<i32, DocumentError>;

    /// Convert the [`Document`] to a `long`.
    ///
    /// # Errors
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `long` (`i64`) value.
    fn into_long(self: Box<Self>) -> Result<i64, DocumentError>;

    /// Convert the [`Document`] to a `float`.
    ///
    /// # Errors
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `float` (`f32`) value.
    fn into_float(self: Box<Self>) -> Result<f32, DocumentError>;

    /// Convert the [`Document`] to a `double`.
    ///
    /// # Errors
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `double` (`f64`) value.
    fn into_double(self: Box<Self>) -> Result<f64, DocumentError>;

    /// Convert the [`Document`] to a `bigInteger`.
    ///
    /// # Errors
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `bigInteger` ([`BigInt`]) value.
    fn into_big_integer(self: Box<Self>) -> Result<BigInt, DocumentError>;

    /// Convert the [`Document`] to a `bigDecimal`.
    ///
    /// # Errors
    /// Returns a [`DocumentError`] if the document could not be converted to a
    /// `bigDecimal` ([`BigDecimal`]) value.
    fn into_big_decimal(self: Box<Self>) -> Result<BigDecimal, DocumentError>;

    /// Convert the [`Document`] to a `list`.
    ///
    /// # Errors
    /// Returns a [`DocumentError`] if the document is not a list.
    fn into_list(self: Box<Self>) -> Result<Vec<Box<dyn Document>>, DocumentError>;

    /// Convert the [`Document`] to a `map`.
    ///
    /// # Errors
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
            (Some(ShapeType::List), Some(ShapeType::List)) => self.as_list() == other.as_list(),
            (Some(ShapeType::Map), Some(ShapeType::Map)) => self.as_map() == other.as_map(),
            (None, None) => true,
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
#[derive(Clone, PartialEq, Debug)]
pub struct StaticDocument {
    /// Reference to the static schema for the shape this was generated from
    pub schema: &'static Schema,
    /// Document value
    pub value: Value
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
    Map(IndexMap<String, Box<dyn Document>>),
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

macro_rules! as_variant {
    ($variant:ident, $value:expr) => {
        if let Value::$variant(value) = $value {
            Some(value)
        } else {
            None
        }
    };

    ($variant:ident, ref $value:expr) => {
        if let Value::$variant(ref value) = $value {
            Some(value)
        } else {
            None
        }
    };
}

macro_rules! into_variant {
    ($variant:ident, $value:expr, $msg:expr) => {
        match $value {
            Value::$variant(v) => Ok(v),
            _ => Err(DocumentError::DocumentConversion($msg.to_string()))
        }
    };
}

macro_rules! as_int {
    ($to_method:ident, $value:expr) => {
        match $value {
            Value::Number(Number::Byte(n)) => n.try_into().ok(),
            Value::Number(Number::Short(n)) => n.try_into().ok(),
            Value::Number(Number::Integer(n)) => n.try_into().ok(),
            Value::Number(Number::Long(n)) => n.try_into().ok(),
            Value::Number(Number::BigInt(ref b)) => b.$to_method(),
            _ => None,
        }
    };
}

macro_rules! as_float {
    ($to_method:ident, $value:expr) => {
        match $value {
            Value::Number(Number::Float(f)) => num_traits::cast(f),
            Value::Number(Number::Double(d)) => num_traits::cast(d),
            Value::Number(Number::BigDecimal(ref b)) => b.$to_method(),
            _ => None,
        }
    };
}

macro_rules! into_int {
    ($target_type:ident, $to_method:ident, $value:expr) => {{
        let err = || DocumentError::DocumentConversion(
            concat!("Invalid conversion to ", stringify!($target_type)).to_string()
        );

        match $value {
            Value::Number(Number::Byte(n)) => n.try_into().map_err(|_| err()),
            Value::Number(Number::Short(n)) => n.try_into().map_err(|_| err()),
            Value::Number(Number::Integer(n)) => n.try_into().map_err(|_| err()),
            Value::Number(Number::Long(n)) => n.try_into().map_err(|_| err()),
            Value::Number(Number::BigInt(b)) => b.$to_method().ok_or_else(err),
            _ => Err(err()),
        }
    }};
}

macro_rules! into_float {
    ($target_type:ident, $to_method:ident, $value:expr) => {{
        let err = || DocumentError::DocumentConversion(
            concat!("Invalid conversion to ", stringify!($target_type)).to_string()
        );

        match $value {
            Value::Number(Number::Float(f)) => num_traits::cast(f).ok_or_else(err),
            Value::Number(Number::Double(d)) => num_traits::cast(d).ok_or_else(err),
            Value::Number(Number::BigDecimal(b)) => b.$to_method().ok_or_else(err),
            _ => Err(err()),
        }
    }};
}

impl Document for StaticDocument {
    #[inline]
    fn discriminator(&self) -> Option<&ShapeId> {
        // A static document ALWAYS has a discriminator
        // as it was generated from a shape with a concrete ID
        Some(self.schema.id())
    }

    // TODO: Is this even needed? Couldnt we just use the schema type?
    #[inline]
    fn get_type(&self) -> Option<&ShapeType> {
        Some(self.schema.shape_type())
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
    fn as_blob(&self) -> Option<&ByteBuffer> {
        as_variant!(Blob, ref self.value)
    }

    #[inline]
    fn as_bool(&self) -> Option<bool> {
        as_variant!(Boolean, self.value)
    }

    #[inline]
    fn as_string(&self) -> Option<&str> {
        as_variant!(String, ref self.value)
    }

    #[inline]
    fn as_timestamp(&self) -> Option<&Instant> {
        as_variant!(Timestamp, ref self.value)
    }

    #[inline]
    fn as_byte(&self) -> Option<i8> {
        as_int!(to_i8, self.value)
    }

    #[inline]
    fn as_short(&self) -> Option<i16> {
        as_int!(to_i16, self.value)
    }

    #[inline]
    fn as_integer(&self) -> Option<i32> {
        as_int!(to_i32, self.value)
    }

    #[inline]
    fn as_long(&self) -> Option<i64> {
        as_int!(to_i64, self.value)
    }

    #[inline]
    fn as_float(&self) -> Option<f32> {
        as_float!(to_f32, self.value)
    }

    #[inline]
    fn as_double(&self) -> Option<f64> {
        as_float!(to_f64, self.value)
    }

    #[inline]
    fn as_big_integer(&self) -> Option<&BigInt> {
        if let Value::Number(Number::BigInt(ref b)) = self.value {
            Some(b)
        } else {
            None
        }
    }

    #[inline]
    fn as_big_decimal(&self) -> Option<&BigDecimal> {
        if let Value::Number(Number::BigDecimal(ref b)) = self.value {
            Some(b)
        } else {
            None
        }
    }

    #[inline]
    fn as_list(&self) -> Option<&Vec<Box<dyn Document>>> {
        as_variant!(List, ref self.value)
    }

    #[inline]
    fn as_map(&self) -> Option<&IndexMap<String, Box<dyn Document>>> {
        as_variant!(Map, ref self.value)
    }

    #[inline]
    fn is_null(&self) -> bool {
        matches!(&self.value, Value::Null)
    }

    #[inline]
    fn into_blob(self: Box<Self>) -> Result<ByteBuffer, DocumentError> {
        into_variant!(Blob, self.value, "Expected blob document")
    }

    #[inline]
    fn into_bool(self: Box<Self>) -> Result<bool, DocumentError> {
        into_variant!(Boolean, self.value, "Expected boolean document")
    }

    #[inline]
    fn into_string(self: Box<Self>) -> Result<String, DocumentError> {
        into_variant!(String, self.value, "Expected string document")
    }

    #[inline]
    fn into_timestamp(self: Box<Self>) -> Result<Instant, DocumentError> {
        into_variant!(Timestamp, self.value, "Expected timestamp document")
    }

    #[inline]
    fn into_byte(self: Box<Self>) -> Result<i8, DocumentError> {
        into_int!(i8, to_i8, self.value)
    }

    #[inline]
    fn into_short(self: Box<Self>) -> Result<i16, DocumentError> {
        into_int!(i16, to_i16, self.value)
    }

    #[inline]
    fn into_integer(self: Box<Self>) -> Result<i32, DocumentError> {
        into_int!(i32, to_i32, self.value)
    }

    #[inline]
    fn into_long(self: Box<Self>) -> Result<i64, DocumentError> {
        into_int!(i64, to_i64, self.value)
    }

    fn into_float(self: Box<Self>) -> Result<f32, DocumentError> {
        into_float!(f32, to_f32, self.value)
    }

    fn into_double(self: Box<Self>) -> Result<f64, DocumentError> {
        into_float!(f64, to_f64, self.value)
    }

    #[inline]
    fn into_big_integer(self: Box<Self>) -> Result<BigInt, DocumentError> {
        let err = || DocumentError::DocumentConversion(
            "Invalid conversion to BigInt".to_string()
        );

        match self.value {
            Value::Number(Number::BigInt(b)) => Ok(b),
            Value::Number(Number::BigDecimal(b)) => b.to_bigint().ok_or_else(err),
            Value::Number(Number::Byte(n)) => Ok(BigInt::from(n)),
            Value::Number(Number::Short(n)) => Ok(BigInt::from(n)),
            Value::Number(Number::Integer(n)) => Ok(BigInt::from(n)),
            Value::Number(Number::Long(n)) => Ok(BigInt::from(n)),
            Value::Number(Number::Float(f)) => BigInt::from_f32(f).ok_or_else(err),
            Value::Number(Number::Double(d)) => BigInt::from_f64(d).ok_or_else(err),
            _ => Err(err()),
        }
    }

    #[inline]
    fn into_big_decimal(self: Box<Self>) -> Result<BigDecimal, DocumentError> {
        let err = || DocumentError::DocumentConversion(
            "Invalid conversion to BigDecimal".to_string()
        );

        match self.value {
            Value::Number(Number::BigDecimal(b)) => Ok(b),
            Value::Number(Number::BigInt(b)) => Ok(BigDecimal::from_bigint(b, 0)),
            Value::Number(Number::Byte(n)) => Ok(BigDecimal::from(n)),
            Value::Number(Number::Short(n)) => Ok(BigDecimal::from(n)),
            Value::Number(Number::Integer(n)) => Ok(BigDecimal::from(n)),
            Value::Number(Number::Long(n)) => Ok(BigDecimal::from(n)),
            Value::Number(Number::Float(f)) => BigDecimal::from_f32(f).ok_or_else(err),
            Value::Number(Number::Double(d)) => BigDecimal::from_f64(d).ok_or_else(err),
            _ => Err(err()),
        }
    }

    #[inline]
    fn into_list(self: Box<Self>) -> Result<Vec<Box<dyn Document>>, DocumentError> {
        into_variant!(List, self.value, "Expected list document")
    }

    #[inline]
    fn into_map(self: Box<Self>) -> Result<IndexMap<String, Box<dyn Document>>, DocumentError> {
        into_variant!(Map, self.value, "Expected map document")
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
            result.insert(key, value.into());
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
        document.into_bool()
    }
}

impl TryFromDocument for i8 {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.into_byte()
    }
}

impl TryFromDocument for i16 {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.into_short()
    }
}

impl TryFromDocument for i32 {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.into_integer()
    }
}

impl TryFromDocument for i64 {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.into_long()
    }
}

impl TryFromDocument for f32 {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.into_float()
    }
}

impl TryFromDocument for f64 {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.into_double()
    }
}

impl TryFromDocument for BigInt {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.into_big_integer()
    }
}

impl TryFromDocument for BigDecimal {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.into_big_decimal()
    }
}

impl TryFromDocument for ByteBuffer {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.into_blob()
    }
}

impl TryFromDocument for String {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.into_string()
    }
}

impl TryFromDocument for Instant {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        document.into_timestamp()
    }
}

impl<T: TryFromDocument> TryFromDocument for Vec<T> {
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        let list = document.into_list()?;
        let mut result = Vec::with_capacity(list.len());
        for item in list {
            result.push(item.try_into()?);
        }
        Ok(result)
    }
}

impl<T: TryFromDocument> TryFromDocument for IndexMap<String, T> {
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        let map = document.into_map()?;
        let mut result = IndexMap::with_capacity(map.len());
        for (key, value) in map {
            result.insert(key, value.try_into()?);
        }
        Ok(result)
    }
}

impl<T: TryFromDocument> TryFromDocument for Option<T> {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        if document.is_null() {
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
    use crate::schema::Schema;

    #[test]
    fn string_document_value() {
        let document_str: Box<dyn Document> = "MyStr".into();
        let val: &Schema = &STRING;
        assert_eq!(document_str.schema(), val);
        let output_str: String = document_str.as_string().unwrap().to_string();
        assert_eq!(output_str, "MyStr".to_string());

        let document_string: Box<dyn Document> = "MyString".into();
        assert_eq!(document_string.schema(), val);
        let output_string: String = document_string.as_string().unwrap().to_string();
        assert_eq!(&output_string, &"MyString");
    }

    #[test]
    fn list_document_value() {
        let vec = vec!["a", "b", "c"];
        let document_list: Box<dyn Document> = vec.into();
        let val: &Schema = &list::DOCUMENT;
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
        let map_doc: Box<dyn Document> = map_in.into();
        let val: &Schema = &map::DOCUMENT;
        assert_eq!(map_doc.schema(), val);
        assert_eq!(map_doc.size(), 1);

        let map_out: IndexMap<String, String> = map_doc.try_into().unwrap();
        assert_eq!(map_out.len(), 1);
        assert_eq!(map_out["a"], "b");
    }

    #[test]
    fn integer_document_values() {
        let byte: Box<dyn Document> = 1i8.into();
        let byte_val: &Schema = &BYTE;
        assert_eq!(byte.schema(), byte_val);

        let short: Box<dyn Document> = 1i16.into();
        let short_val: &Schema = &SHORT;

        assert_eq!(short.schema(), short_val);

        let integer: Box<dyn Document> = 1i32.into();
        let integer_val: &Schema = &INTEGER;

        assert_eq!(integer.schema(), integer_val);

        let long: Box<dyn Document> = 1i64.into();
        let long_val: &Schema = &LONG;

        assert_eq!(long.schema(), long_val);
    }

    // TODO(numeric comparisons): Add comparison checks

    #[test]
    fn float_document_values() {
        let float: Box<dyn Document> = 1f32.into();
        let float_val: &Schema = &FLOAT;
        assert_eq!(float.schema(), float_val);

        let double: Box<dyn Document> = 1f64.into();
        let double_val: &Schema = &DOUBLE;
        assert_eq!(double.schema(), double_val);

        let float_value: f32 = float.try_into().unwrap();
        assert_eq!(float_value, 1f32);
        let double_value: f64 = double.try_into().unwrap();
        assert_eq!(double_value, 1f64);
    }
}
