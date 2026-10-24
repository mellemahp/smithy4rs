//! Document

use std::{borrow::Cow, error::Error};

use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;
use temporal_rs::Instant;
use thiserror::Error;

use crate::{
    schema::{Schema, ShapeId},
    serde::validation::ValidationErrors,
};

// ============================================================================
// Modules
// ============================================================================

mod conversion;
mod deserialization;
mod impls;
mod serialization;
#[cfg(test)]
mod tests;

pub use conversion::TryFromDocument;

// ============================================================================
// Base trait definitions
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
    fn as_string(&self) -> Option<&str> {
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
    DocumentError::CastFailed(format!("expected {expected}."))
}

// ================================================================
// Accessor Traits
// ================================================================

/// Provides Access to List Document items
pub trait ListAccessor {
    /// Get an element by index if possible
    ///
    /// Note: Named `get_index` to avoid clash with `vec.get`
    fn get_index(&self, idx: usize) -> Option<&(dyn Document + '_)>;

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
    /// Could not convert document to requested type
    ///
    /// Indicates that a "smoothing" procedure such as
    /// integer -> long conversion failed
    #[error("Failed to convert document to type {0}")]
    ConversionFailed(String),
    /// Could not _take_ document as type
    ///
    /// Indicates that the underlying document type is incompatible
    /// with requested cast (i.e. `.take_$type()` method)
    #[error("Failed to `take` document as type {0}")]
    CastFailed(String),
    /// A call to get element or entry at index was out of bounds
    #[error("Index {0} out of bounds")]
    InvalidIndex(usize),
    /// Null document was read as a non-nullable (option) type
    ///
    /// Note: This is likely a bug
    #[error("Attempted to read null document as type")]
    NullRead,
    /// Encountered validation error while deserializing document
    #[error("Invalid Document")]
    Invalid(#[from] ValidationErrors),
    /// An unknown error
    #[error("Encountered unknown error")]
    Unknown(#[from] Box<dyn Error>),
    /// A custom error
    #[error("Encountered error: {0}")]
    CustomError(String),
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
                // TODO: Still not sure these comparisons are quite correct, but they
                // are workable for now.
                Number::Byte => self.as_byte() == other.as_byte(),
                Number::Short => self.as_short() == other.as_short(),
                Number::Integer => self.as_integer() == other.as_integer(),
                Number::Long => self.as_long() == other.as_long(),
                Number::BigInt => self.as_big_integer() == other.as_big_integer(),
                Number::Float => self.as_float() == other.as_float(),
                Number::Double => self.as_double() == other.as_double(),
                Number::BigDecimal => self.as_big_decimal() == other.as_big_decimal(),
            },
            DocumentType::String => self.as_string() == other.as_string(),
            DocumentType::Blob => self.as_blob() == other.as_blob(),
            DocumentType::List => {
                let (Some(tc), Some(oc)) = (self.as_list(), other.as_list()) else {
                    return false;
                };
                if tc.len() != oc.len() {
                    return false;
                }
                for i in 0..tc.len() {
                    if tc.get_index(i) != oc.get_index(i) {
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
// Implementation Macros
// ================================================================

/// Automatically implements [`Document`] for wrapper types like MyWrapper(String).
///
/// Note: Mostly used by generated code to simplify proc macro calls
#[macro_export]
macro_rules! wrapped_document {
    ($ty:ident) => {
        impl $crate::features::documents::Document for $ty {
            #[inline]
            fn discriminant(&self) -> Option<&$crate::schema::ShapeId> {
                Some(<Self as $crate::schema::StaticSchemaShape>::schema().id())
            }

            #[inline]
            fn document_type(&self) -> $crate::features::documents::DocumentType {
                self.0.document_type()
            }

            #[inline]
            fn as_boolean(&self) -> Option<bool> {
                self.0.as_boolean()
            }

            #[inline]
            fn as_blob(&self) -> Option<&$crate::ByteBuffer> {
                self.0.as_blob()
            }

            #[inline]
            fn as_string(&self) -> Option<&str> {
                self.0.as_string()
            }

            #[inline]
            fn as_timestamp(&self) -> Option<$crate::Instant> {
                self.0.as_timestamp()
            }

            #[inline]
            fn as_byte(&self) -> Option<i8> {
                self.0.as_byte()
            }

            #[inline]
            fn as_short(&self) -> Option<i16> {
                self.0.as_short()
            }

            #[inline]
            fn as_integer(&self) -> Option<i32> {
                self.0.as_integer()
            }

            #[inline]
            fn as_long(&self) -> Option<i64> {
                self.0.as_long()
            }

            #[inline]
            fn as_big_integer(&self) -> Option<std::borrow::Cow<'_, $crate::BigInt>> {
                self.0.as_big_integer()
            }

            #[inline]
            fn as_float(&self) -> Option<f32> {
                self.0.as_float()
            }

            #[inline]
            fn as_double(&self) -> Option<f64> {
                self.0.as_double()
            }

            #[inline]
            fn as_big_decimal(&self) -> Option<std::borrow::Cow<'_, $crate::BigDecimal>> {
                self.0.as_big_decimal()
            }

            #[inline]
            fn as_list(&self) -> Option<&dyn $crate::features::documents::ListAccessor> {
                self.0.as_list()
            }

            #[inline]
            fn as_map(&self) -> Option<&dyn $crate::features::documents::MapAccessor> {
                self.0.as_map()
            }

            #[inline]
            fn take_bool(
                &mut self,
                schema: &$crate::schema::Schema,
            ) -> Result<bool, $crate::features::documents::DocumentError> {
                self.0.take_bool(schema)
            }

            #[inline]
            fn take_blob(
                &mut self,
                schema: &$crate::schema::Schema,
            ) -> Result<$crate::ByteBuffer, $crate::features::documents::DocumentError> {
                self.0.take_blob(schema)
            }

            #[inline]
            fn take_string(
                &mut self,
                schema: &$crate::schema::Schema,
            ) -> Result<String, $crate::features::documents::DocumentError> {
                self.0.take_string(schema)
            }

            #[inline]
            fn take_byte(
                &mut self,
                schema: &$crate::schema::Schema,
            ) -> Result<i8, $crate::features::documents::DocumentError> {
                self.0.take_byte(schema)
            }

            #[inline]
            fn take_short(
                &mut self,
                schema: &$crate::schema::Schema,
            ) -> Result<i16, $crate::features::documents::DocumentError> {
                self.0.take_short(schema)
            }

            #[inline]
            fn take_integer(
                &mut self,
                schema: &$crate::schema::Schema,
            ) -> Result<i32, $crate::features::documents::DocumentError> {
                self.0.take_integer(schema)
            }

            #[inline]
            fn take_long(
                &mut self,
                schema: &$crate::schema::Schema,
            ) -> Result<i64, $crate::features::documents::DocumentError> {
                self.0.take_long(schema)
            }

            #[inline]
            fn take_big_integer(
                &mut self,
                schema: &$crate::schema::Schema,
            ) -> Result<$crate::BigInt, $crate::features::documents::DocumentError> {
                self.0.take_big_integer(schema)
            }

            #[inline]
            fn take_float(
                &mut self,
                schema: &$crate::schema::Schema,
            ) -> Result<f32, $crate::features::documents::DocumentError> {
                self.0.take_float(schema)
            }

            #[inline]
            fn take_double(
                &mut self,
                schema: &$crate::schema::Schema,
            ) -> Result<f64, $crate::features::documents::DocumentError> {
                self.0.take_double(schema)
            }

            #[inline]
            fn take_big_decimal(
                &mut self,
                schema: &$crate::schema::Schema,
            ) -> Result<$crate::BigDecimal, $crate::features::documents::DocumentError> {
                self.0.take_big_decimal(schema)
            }

            #[inline]
            fn take_list(
                &mut self,
            ) -> Result<
                &mut dyn $crate::features::documents::ListAccessor,
                $crate::features::documents::DocumentError,
            > {
                self.0.take_list()
            }

            #[inline]
            fn take_map(
                &mut self,
            ) -> Result<
                &mut dyn $crate::features::documents::MapAccessor,
                $crate::features::documents::DocumentError,
            > {
                self.0.take_map()
            }
        }
    };
}
