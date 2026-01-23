use std::{error::Error, fmt::Debug, sync::LazyLock};

use thiserror::Error;

use crate::{
    BigDecimal, BigInt, ByteBuffer, IndexMap, Instant,
    schema::{
        SchemaShape, ShapeId, ShapeType,
        default::{Number, Value},
        prelude::*,
    },
    smithy,
};

// ============================================================================
// Base Document Wrapper and trait
// ============================================================================

/// A Smithy [document](https://smithy.io/2.0/spec/simple-types.html#document) type,
/// representing untyped data in the Smithy data model.
///
/// TODO(doc): Update with dyn doc model
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
/// 2. Lossless Serialization: Documents created from a shape should serialize exactly the same as the shape they are created from. I.e.
///
/// ```rust,ignore
/// // These two should be equivalent
/// my_smithy_shape.serialize(serializer)?;
/// my_smithy_shape.as_document().serialize(serializer)?;
/// ```
///
/// 3. Lossless Deserialization: Deserializing to a document then converting to a shape should be the same as deserializing to that shape. I.e. :
///
/// ```rust,ignore
/// // These two should be equivalent
/// let my_shape = deserializer::deserialize::<MyShape>::();
/// let my_shape = deserializer::deserialize::<Document>::().into();
/// ```
/// This is particularly important for dealing with errors, events, and over-the-wire polymorphism.
///
/// 4. Protocol Smoothing: Documents should try to smooth over any protocol-specific incompatibilities with the smithy data model.
/// 5. Discriminators - Documents with an encloded type should be able to be (de)serialized with data that identifies their
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
// Convert Document into types
// ============================================================================

impl TryFrom<Box<dyn Document>> for ByteBuffer {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        value.into_blob()
    }
}

impl TryFrom<Box<dyn Document>> for bool {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        value.into_bool()
    }
}

impl TryFrom<Box<dyn Document>> for String {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        value.into_string()
    }
}

impl TryFrom<Box<dyn Document>> for Instant {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        value.into_timestamp()
    }
}

impl TryFrom<Box<dyn Document>> for i8 {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        value.into_byte()
    }
}

impl TryFrom<Box<dyn Document>> for i16 {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        value.into_short()
    }
}

impl TryFrom<Box<dyn Document>> for i32 {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        value.into_integer()
    }
}

impl TryFrom<Box<dyn Document>> for i64 {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        value.into_long()
    }
}

impl TryFrom<Box<dyn Document>> for f32 {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        value.into_float()
    }
}

impl TryFrom<Box<dyn Document>> for f64 {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        value.into_double()
    }
}

impl TryFrom<Box<dyn Document>> for BigInt {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        value.into_big_integer()
    }
}

impl TryFrom<Box<dyn Document>> for BigDecimal {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        value.into_big_decimal()
    }
}

impl TryFrom<Box<dyn Document>> for Vec<Box<dyn Document>> {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        value.into_list()
    }
}

impl TryFrom<Box<dyn Document>> for IndexMap<String, Box<dyn Document>> {
    type Error = DocumentError;

    #[inline]
    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        value.into_map()
    }
}

impl<T: TryFrom<Box<dyn Document>, Error = DocumentError>> TryFrom<Box<dyn Document>> for Vec<T> {
    type Error = DocumentError;

    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
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

impl<T: TryFrom<Box<dyn Document>, Error = DocumentError>> TryFrom<Box<dyn Document>>
    for IndexMap<String, T>
{
    type Error = DocumentError;

    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        let map = value.into_map()?;
        let mut result: IndexMap<String, T> = IndexMap::new();
        for (key, val) in map {
            let _ = match T::try_from(val.clone()) {
                Ok(val) => result.insert(key.clone(), val),
                Err(e) => return Err(e),
            };
        }
        Ok(result)
    }
}

impl<T: TryFrom<Box<dyn Document>, Error = DocumentError>> TryFrom<Box<dyn Document>>
    for Option<T>
{
    type Error = DocumentError;

    fn try_from(value: Box<dyn Document>) -> Result<Self, Self::Error> {
        if value.is_null() {
            return Ok(None);
        }
        Ok(Some(value.try_into()?))
    }
}

// ============================================================================
// Default Document Implementation
// ============================================================================

pub(crate) mod default {
    use bigdecimal::ToPrimitive;

    use crate::{
        BigDecimal, BigInt, ByteBuffer, IndexMap, Instant,
        schema::{DocumentError, SchemaRef, SchemaShape, ShapeId, ShapeType},
    };

    #[derive(Clone, PartialEq, Debug)]
    pub struct Document {
        pub schema: SchemaRef,
        pub value: Value,
        pub discriminator: Option<ShapeId>,
    }

    impl SchemaShape for Document {
        fn schema(&self) -> &SchemaRef {
            &self.schema
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    pub enum Value {
        Null,
        Number(Number),
        Boolean(bool),
        Blob(ByteBuffer),
        String(String),
        Timestamp(Instant),
        List(Vec<Box<dyn super::Document>>),
        Map(IndexMap<String, Box<dyn super::Document>>),
    }

    impl super::Document for Document {
        fn discriminator(&self) -> Option<&ShapeId> {
            self.discriminator.as_ref()
        }

        fn get_type(&self) -> Option<&ShapeType> {
            match &self.value {
                Value::Number(n) => match n {
                    Number::Byte(_) => Some(&ShapeType::Byte),
                    Number::Short(_) => Some(&ShapeType::Short),
                    Number::Integer(_) => Some(&ShapeType::Integer),
                    Number::Long(_) => Some(&ShapeType::Long),
                    Number::BigInt(_) => Some(&ShapeType::BigInteger),
                    Number::Float(_) => Some(&ShapeType::Float),
                    Number::Double(_) => Some(&ShapeType::Double),
                    Number::BigDecimal(_) => Some(&ShapeType::BigDecimal),
                },
                Value::Boolean(_) => Some(&ShapeType::Boolean),
                Value::Blob(_) => Some(&ShapeType::Blob),
                Value::String(_) => Some(&ShapeType::String),
                Value::Timestamp(_) => Some(&ShapeType::Timestamp),
                Value::List(_) => Some(&ShapeType::List),
                Value::Map(_) => {
                    if self.schema.shape_type() == &ShapeType::Map {
                        Some(&ShapeType::Map)
                    } else {
                        // If we created the document from a structure schema, treat it as a struct
                        Some(&ShapeType::Structure)
                    }
                }
                // Null is not representable as a shape type
                Value::Null => None,
            }
        }

        fn size(&self) -> usize {
            match &self.value {
                Value::List(v) => v.len(),
                Value::Map(v) => v.len(),
                _ => 0,
            }
        }

        fn as_blob(&self) -> Option<&ByteBuffer> {
            if let Value::Blob(b) = &self.value {
                Some(b)
            } else {
                None
            }
        }

        fn as_bool(&self) -> Option<bool> {
            if let Value::Boolean(b) = &self.value {
                Some(*b)
            } else {
                None
            }
        }

        fn as_string(&self) -> Option<&str> {
            if let Value::String(s) = &self.value {
                Some(s.as_str())
            } else {
                None
            }
        }

        fn as_timestamp(&self) -> Option<&Instant> {
            if let Value::Timestamp(t) = &self.value {
                Some(t)
            } else {
                None
            }
        }

        fn as_byte(&self) -> Option<i8> {
            match &self.value {
                Value::Number(n) => match n {
                    &Number::Byte(b) => Some(b),
                    &Number::Short(s) => s.try_into().ok(),
                    &Number::Integer(i) => i.try_into().ok(),
                    &Number::Long(l) => l.try_into().ok(),
                    Number::BigInt(b) => b.to_i8(),
                    _ => None,
                },
                _ => None,
            }
        }

        fn as_short(&self) -> Option<i16> {
            match &self.value {
                Value::Number(n) => match n {
                    &Number::Byte(b) => Some(b.into()),
                    &Number::Short(s) => Some(s),
                    &Number::Integer(i) => i.try_into().ok(),
                    &Number::Long(l) => l.try_into().ok(),
                    Number::BigInt(b) => b.to_i16(),
                    _ => None,
                },
                _ => None,
            }
        }

        fn as_integer(&self) -> Option<i32> {
            match &self.value {
                Value::Number(n) => match n {
                    &Number::Byte(b) => Some(b.into()),
                    &Number::Short(s) => Some(s.into()),
                    &Number::Integer(i) => Some(i),
                    &Number::Long(l) => l.try_into().ok(),
                    Number::BigInt(b) => b.to_i32(),
                    _ => None,
                },
                _ => None,
            }
        }

        fn as_long(&self) -> Option<i64> {
            match &self.value {
                Value::Number(n) => match n {
                    &Number::Byte(b) => Some(b.into()),
                    &Number::Short(s) => Some(s.into()),
                    &Number::Integer(i) => Some(i.into()),
                    &Number::Long(l) => Some(l),
                    Number::BigInt(b) => b.to_i64(),
                    _ => None,
                },
                _ => None,
            }
        }

        fn as_float(&self) -> Option<f32> {
            match &self.value {
                Value::Number(n) => match n {
                    &Number::Float(f) => Some(f),
                    &Number::Double(d) => Some(d as f32),
                    Number::BigDecimal(b) => b.to_f32(),
                    _ => None,
                },
                _ => None,
            }
        }

        fn as_double(&self) -> Option<f64> {
            match &self.value {
                Value::Number(n) => match n {
                    &Number::Float(f) => Some(f.into()),
                    &Number::Double(d) => Some(d),
                    Number::BigDecimal(b) => b.to_f64(),
                    _ => None,
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

        fn as_list(&self) -> Option<&Vec<Box<dyn super::Document>>> {
            if let Value::List(document_list) = &self.value {
                Some(document_list)
            } else {
                None
            }
        }

        fn as_map(&self) -> Option<&IndexMap<String, Box<dyn super::Document>>> {
            if let Value::Map(document_map) = &self.value {
                Some(document_map)
            } else {
                None
            }
        }

        fn is_null(&self) -> bool {
            matches!(self.value, Value::Null)
        }

        fn into_blob(self: Box<Self>) -> Result<ByteBuffer, DocumentError> {
            if let Value::Blob(value) = self.value {
                Ok(value)
            } else {
                Err(DocumentError::DocumentConversion(
                    "Expected blob document".to_string(),
                ))
            }
        }

        fn into_bool(self: Box<Self>) -> Result<bool, DocumentError> {
            if let Value::Boolean(value) = self.value {
                Ok(value)
            } else {
                Err(DocumentError::DocumentConversion(
                    "Expected boolean document".to_string(),
                ))
            }
        }

        fn into_string(self: Box<Self>) -> Result<String, DocumentError> {
            if let Value::String(value) = self.value {
                Ok(value)
            } else {
                Err(DocumentError::DocumentConversion(
                    "Expected string document".to_string(),
                ))
            }
        }

        fn into_timestamp(self: Box<Self>) -> Result<Instant, DocumentError> {
            if let Value::Timestamp(value) = self.value {
                Ok(value)
            } else {
                Err(DocumentError::DocumentConversion(
                    "Expected timestamp document".to_string(),
                ))
            }
        }

        // TODO(numeric conversion): Review these numeric conversions.
        fn into_byte(self: Box<Self>) -> Result<i8, DocumentError> {
            self.as_byte().ok_or_else(|| {
                DocumentError::DocumentConversion("Expected byte document".to_string())
            })
        }

        fn into_short(self: Box<Self>) -> Result<i16, DocumentError> {
            self.as_short().ok_or_else(|| {
                DocumentError::DocumentConversion("Expected short document".to_string())
            })
        }

        fn into_integer(self: Box<Self>) -> Result<i32, DocumentError> {
            self.as_integer().ok_or_else(|| {
                DocumentError::DocumentConversion("Expected integer document".to_string())
            })
        }

        fn into_long(self: Box<Self>) -> Result<i64, DocumentError> {
            self.as_long().ok_or_else(|| {
                DocumentError::DocumentConversion("Expected long document".to_string())
            })
        }

        fn into_float(self: Box<Self>) -> Result<f32, DocumentError> {
            self.as_float().ok_or_else(|| {
                DocumentError::DocumentConversion("Expected float document".to_string())
            })
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

        fn into_list(self: Box<Self>) -> Result<Vec<Box<dyn super::Document>>, DocumentError> {
            if let Value::List(value) = self.value {
                Ok(value)
            } else {
                Err(DocumentError::DocumentConversion(
                    "Expected list document".to_string(),
                ))
            }
        }

        fn into_map(
            self: Box<Self>,
        ) -> Result<IndexMap<String, Box<dyn super::Document>>, DocumentError> {
            if let Value::Map(value) = self.value {
                Ok(value)
            } else {
                Err(DocumentError::DocumentConversion(
                    "Expected map document".to_string(),
                ))
            }
        }

        fn box_clone(&self) -> Box<dyn super::Document> {
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
    pub enum Number {
        Byte(i8),
        Short(i16),
        Integer(i32),
        Long(i64),
        BigInt(BigInt),
        Float(f32),
        Double(f64),
        BigDecimal(BigDecimal),
    }
}

// ============================================================================
// Conversions INTO Default Document
// ============================================================================

impl From<default::Document> for Box<dyn Document> {
    #[inline]
    fn from(value: default::Document) -> Self {
        Box::new(value)
    }
}

impl From<bool> for Box<dyn Document> {
    #[inline]
    fn from(value: bool) -> Self {
        default::Document {
            schema: BOOLEAN.clone(),
            value: Value::Boolean(value),
            discriminator: None,
        }
        .into()
    }
}

impl From<i8> for Box<dyn Document> {
    #[inline]
    fn from(value: i8) -> Self {
        default::Document {
            schema: BYTE.clone(),
            value: Value::Number(Number::Byte(value)),
            discriminator: None,
        }
        .into()
    }
}

impl From<i16> for Box<dyn Document> {
    #[inline]
    fn from(value: i16) -> Self {
        default::Document {
            schema: SHORT.clone(),
            value: Value::Number(Number::Short(value)),
            discriminator: None,
        }
        .into()
    }
}

impl From<i32> for Box<dyn Document> {
    #[inline]
    fn from(value: i32) -> Self {
        default::Document {
            schema: INTEGER.clone(),
            value: Value::Number(Number::Integer(value)),
            discriminator: None,
        }
        .into()
    }
}

impl From<i64> for Box<dyn Document> {
    #[inline]
    fn from(value: i64) -> Self {
        default::Document {
            schema: LONG.clone(),
            value: Value::Number(Number::Long(value)),
            discriminator: None,
        }
        .into()
    }
}

impl From<f32> for Box<dyn Document> {
    #[inline]
    fn from(value: f32) -> Self {
        default::Document {
            schema: FLOAT.clone(),
            value: Value::Number(Number::Float(value)),
            discriminator: None,
        }
        .into()
    }
}

impl From<f64> for Box<dyn Document> {
    #[inline]
    fn from(value: f64) -> Self {
        default::Document {
            schema: DOUBLE.clone(),
            value: Value::Number(Number::Double(value)),
            discriminator: None,
        }
        .into()
    }
}

impl From<&str> for Box<dyn Document> {
    #[inline]
    fn from(value: &str) -> Self {
        default::Document {
            schema: STRING.clone(),
            value: Value::String(value.to_string()),
            discriminator: None,
        }
        .into()
    }
}

impl From<BigInt> for Box<dyn Document> {
    #[inline]
    fn from(value: BigInt) -> Self {
        default::Document {
            schema: BIG_INTEGER.clone(),
            value: Value::Number(Number::BigInt(value)),
            discriminator: None,
        }
        .into()
    }
}

impl From<BigDecimal> for Box<dyn Document> {
    #[inline]
    fn from(value: BigDecimal) -> Self {
        default::Document {
            schema: BIG_DECIMAL.clone(),
            value: Value::Number(Number::BigDecimal(value)),
            discriminator: None,
        }
        .into()
    }
}

impl From<ByteBuffer> for Box<dyn Document> {
    #[inline]
    fn from(value: ByteBuffer) -> Self {
        default::Document {
            schema: BLOB.clone(),
            value: Value::Blob(value),
            discriminator: None,
        }
        .into()
    }
}

impl From<String> for Box<dyn Document> {
    #[inline]
    fn from(value: String) -> Self {
        default::Document {
            schema: STRING.clone(),
            value: Value::String(value),
            discriminator: None,
        }
        .into()
    }
}

impl From<Instant> for Box<dyn Document> {
    #[inline]
    fn from(value: Instant) -> Self {
        default::Document {
            schema: TIMESTAMP.clone(),
            value: Value::Timestamp(value),
            discriminator: None,
        }
        .into()
    }
}
impl From<&Instant> for Box<dyn Document> {
    #[inline]
    fn from(value: &Instant) -> Self {
        default::Document {
            schema: TIMESTAMP.clone(),
            value: Value::Timestamp(*value),
            discriminator: None,
        }
        .into()
    }
}

smithy!("smithy.api#Document": {
    #[doc(hidden)]
    list LIST_DOCUMENT_SCHEMA {
        member: DOCUMENT
    }
});

impl<T: Into<Box<dyn Document>>> From<Vec<T>> for Box<dyn Document> {
    fn from(value: Vec<T>) -> Self {
        let mut result = Vec::with_capacity(value.len());
        for v in value.into_iter() {
            result.push(v.into());
        }
        default::Document {
            schema: LIST_DOCUMENT_SCHEMA.clone(),
            value: Value::List(result),
            discriminator: None,
        }
        .into()
    }
}

smithy!("smithy.api#Document": {
    #[doc(hidden)]
    map MAP_DOCUMENT_SCHEMA {
        key: STRING
        value: DOCUMENT
    }
});

impl<T: Into<Box<dyn Document>>> From<IndexMap<String, T>> for Box<dyn Document> {
    fn from(value: IndexMap<String, T>) -> Self {
        let mut result = IndexMap::with_capacity(value.len());
        for (key, value) in value {
            result.insert(key, value.into());
        }
        default::Document {
            schema: MAP_DOCUMENT_SCHEMA.clone(),
            value: Value::Map(result),
            discriminator: None,
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
                        default::Document {
                            schema: $schema.clone(),
                            value: Value::Null,
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
#[doc(hidden)]
pub static NULL: LazyLock<Box<dyn Document>> = LazyLock::new(|| {
    default::Document {
        schema: DOCUMENT.clone(),
        value: Value::Null,
        discriminator: None,
    }
    .into()
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::SchemaRef;

    #[test]
    fn string_document_value() {
        let document_str: Box<dyn Document> = "MyStr".into();
        let val: &SchemaRef = &STRING;
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
        let map_doc: Box<dyn Document> = map_in.into();
        let val: &SchemaRef = &MAP_DOCUMENT_SCHEMA;
        assert_eq!(map_doc.schema(), val);
        assert_eq!(map_doc.size(), 1);

        let map_out: IndexMap<String, String> = map_doc.try_into().unwrap();
        assert_eq!(map_out.len(), 1);
        assert_eq!(map_out["a"], "b");
    }

    #[test]
    fn integer_document_values() {
        let byte: Box<dyn Document> = 1i8.into();
        let byte_val: &SchemaRef = &BYTE;
        assert_eq!(byte.schema(), byte_val);

        let short: Box<dyn Document> = 1i16.into();
        let short_val: &SchemaRef = &SHORT;

        assert_eq!(short.schema(), short_val);

        let integer: Box<dyn Document> = 1i32.into();
        let integer_val: &SchemaRef = &INTEGER;

        assert_eq!(integer.schema(), integer_val);

        let long: Box<dyn Document> = 1i64.into();
        let long_val: &SchemaRef = &LONG;

        assert_eq!(long.schema(), long_val);
    }

    // TODO(numeric comparisons): Add comparison checks

    #[test]
    fn float_document_values() {
        let float: Box<dyn Document> = 1f32.into();
        let float_val: &SchemaRef = &FLOAT;
        assert_eq!(float.schema(), float_val);

        let double: Box<dyn Document> = 1f64.into();
        let double_val: &SchemaRef = &DOUBLE;
        assert_eq!(double.schema(), double_val);

        let float_value: f32 = float.try_into().unwrap();
        assert_eq!(float_value, 1f32);
        let double_value: f64 = double.try_into().unwrap();
        assert_eq!(double_value, 1f64);
    }
}
