//! # Prelude
//! [`crate::schema::Schema`] definitions for the [Smithy prelude](https://github.com/smithy-lang/smithy/blob/65b473ddb94f9edda933f00bab988d465b2bd2fe/smithy-model/src/main/resources/software/amazon/smithy/model/loader/prelude.smithy)
//!
//! The prelude consists of public, built-in shapes like `STRING`, `INTEGER`, etc. that
//! are available to all models. Prelude shapes and traits are all in the `smithy.api` namespace
//! and must be hard-coded as they are used by generate shapes.

use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use bigdecimal::Zero;
use regex::Regex;

use crate::{
    BigDecimal, LazyLock, annotation_trait,
    schema::{Document, ShapeId, SmithyTrait, StaticTraitId},
    smithy, static_trait_id, string_trait,
};

// ============================================================================
// Prelude Shape Schemas
// ---------------------
// These are the base shapes of the smithy data model.
// For more information see: https://smithy.io/2.0/spec/index.html
// ============================================================================

// === Simple types ===
smithy!("smithy.api#Blob": {
    /// Smithy [blob](https://smithy.io/2.0/spec/simple-types.html#blob) shape
    blob BLOB
});
smithy!("smithy.api#Boolean": {
    /// Smithy [boolean](https://smithy.io/2.0/spec/simple-types.html#boolean) shape
    boolean BOOLEAN
});
smithy!("smithy.api#String": {
    /// Smithy [string](https://smithy.io/2.0/spec/simple-types.html#string) shape
    string STRING
});
smithy!("smithy.api#Timestamp": {
    /// Smithy [timestamp](https://smithy.io/2.0/spec/simple-types.html#timestamp) shape
    timestamp TIMESTAMP
});
smithy!("smithy.api#Byte": {
    /// Smithy [byte](https://smithy.io/2.0/spec/simple-types.html#byte) shape
    byte BYTE
});
smithy!("smithy.api#Short": {
    /// Smithy [short](https://smithy.io/2.0/spec/simple-types.html#short) shape
    short SHORT
});
smithy!("smithy.api#Integer": {
    /// Smithy [integer](https://smithy.io/2.0/spec/simple-types.html#integer) shape
    integer INTEGER
});
smithy!("smithy.api#Long": {
    /// Smithy [long](https://smithy.io/2.0/spec/simple-types.html#long) shape
    long LONG
});
smithy!("smithy.api#Float": {
    /// Smithy [float](https://smithy.io/2.0/spec/simple-types.html#float) shape
    float FLOAT
});
smithy!("smithy.api#Double": {
    /// Smithy [double](https://smithy.io/2.0/spec/simple-types.html#double) shape
    double DOUBLE
});
smithy!("smithy.api#BigInteger": {
    /// Smithy [bigInteger](https://smithy.io/2.0/spec/simple-types.html#bigInteger) shape
    bigInteger BIG_INTEGER
});
smithy!("smithy.api#BigDecimal": {
    /// Smithy [bigDecimal](https://smithy.io/2.0/spec/simple-types.html#bigDecimal) shape
    bigDecimal BIG_DECIMAL
});
smithy!("smithy.api#Document": {
    /// Smithy [document](https://smithy.io/2.0/spec/simple-types.html#document) shape
    document DOCUMENT
});

// === Primitive types ===
smithy!("smithy.api#PrimitiveBoolean": {
    #[doc(hidden)]
    @DefaultTrait::new(false);
    boolean PRIMITIVE_BOOLEAN
});
smithy!("smithy.api#PrimitiveByte": {
    #[doc(hidden)]
    @DefaultTrait::new(0i8);
    byte PRIMITIVE_BYTE
});
smithy!("smithy.api#PrimitiveShort": {
    #[doc(hidden)]
    @DefaultTrait::new(0i16);
    short PRIMITIVE_SHORT
});
smithy!("smithy.api#PrimitiveInteger": {
    #[doc(hidden)]
    @DefaultTrait::new(0i32);
    integer PRIMITIVE_INTEGER
});
smithy!("smithy.api#PrimitiveLong": {
    #[doc(hidden)]
    @DefaultTrait::new(0i64);
    boolean PRIMITIVE_LONG
});
smithy!("smithy.api#PrimitiveFloat": {
    #[doc(hidden)]
    @DefaultTrait::new(0f32);
    float PRIMITIVE_FLOAT
});
smithy!("smithy.api#PrimitiveDouble": {
    #[doc(hidden)]
    @DefaultTrait::new(0f64);
    double PRIMITIVE_DOUBLE
});

// ============================================================================
// Prelude Traits
// ============================================================================

// ==== Annotation traits ====
annotation_trait!(
    /// Indicates that data is sensitive and must be handled with care.
    ///
    /// *See* - [`@sensitive`](https://smithy.io/2.0/spec/documentation-traits.html#smithy-api-sensitive-trait)
    SensitiveTrait = "smithy.api#sensitive"
);
annotation_trait!(
    /// Indicates that the data represented by the shape needs to be streamed.
    ///
    /// *See* - [`@streaming`](https://smithy.io/2.0/spec/streaming.html#smithy-api-streaming-trait)
    StreamingTrait = "smithy.api#streaming"
);
annotation_trait!(
    /// Indicates that lists and maps MAY contain null values
    ///
    /// *See* - [`@sparse`](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-sparse-trait)
    SparseTrait = "smithy.api#sparse");
annotation_trait!(
    /// Indicates that a member value MUST be set for the shape to be valid.
    ///
    /// *See* - [`@required`](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-required-trait)
    RequiredTrait = "smithy.api#required"
);
annotation_trait!(
    /// Indicates that a member should be serialized as an event header when sent through an event stream.
    ///
    /// *See* - [`@eventHeader`](https://smithy.io/2.0/spec/streaming.html#smithy-api-eventheader-trait)
    EventHeaderTrait = "smithy.api#eventheader"
);
annotation_trait!(
    /// Indicates that a member should be serialized as the payload of an event when sent through an event stream.
    ///
    /// *See* - [`@eventPayload`](https://smithy.io/2.0/spec/streaming.html#eventpayload-trait)
    EventPayloadTrait = "smithy.api#eventPayload"
);
annotation_trait!(
    /// Indicates that a member is used by the server to identify and discard replayed requests.
    ///
    /// *See* - [`@idempotencyToken`](https://smithy.io/2.0/spec/behavior-traits.html#smithy-api-idempotencytoken-trait)
    IdempotencyTokenTrait = "smithy.api#IdempotencyToken"
);
annotation_trait!(
    /// Binds a member so that it is used as part of an HTTP request URI.
    ///
    /// *See* - [`@httpLabel`](https://smithy.io/2.0/spec/http-bindings.html#smithy-api-httplabel-trait)
    HttpLabelTrait = "smithy.api#httpLabel"
);
annotation_trait!(
    /// Binds a member to the body of an HTTP message.
    ///
    /// *See* - [`@httpPayload`](https://smithy.io/2.0/spec/http-bindings.html#httppayload-trait)
    HttpPayloadTrait = "smithy.api#httpPayload"
);
annotation_trait!(
    /// Binds a map of key-value pairs to query string parameters.
    ///
    /// *See* - [`@httpQueryParams`](https://smithy.io/2.0/spec/http-bindings.html#httpqueryparams-trait)
    HTTPQueryParamsTrait = "smithy.api#httpQueryParams"
);
annotation_trait!(
    /// Binds a member to the HTTP response status code so that an HTTP response status code can be set dynamically.
    ///
    /// *See* - [`@httpResponseCode`](https://smithy.io/2.0/spec/http-bindings.html#httpresponsecode-trait)
    HTTPResponseCodeTrait = "smithy.api#httpResponseCode"
);
annotation_trait!(
    /// Indicates that an operation requires a checksum in HTTP requests.
    ///
    /// *See* - [`@httpChecksumRequired`](https://smithy.io/2.0/spec/http-bindings.html#httpchecksumrequired-trait)
    HTTPChecksumRequiredTrait = "smithy.api#httpChecksumRequired"
);
annotation_trait!(
    /// Binds a top-level operation input structure member to a label in the hostPrefix of an endpoint trait.
    ///
    /// *See* - [`@eventPayload`](https://smithy.io/2.0/spec/endpoint-traits.html#smithy-api-hostlabel-trait)
    HostLabelTrait = "smithy.api#hostLabel"
);
annotation_trait!(
    /// Serializes an object property as an XML attribute rather than a nested XML element.
    ///
    /// *See* - [`@eventPayload`](https://smithy.io/2.0/spec/protocol-traits.html#xmlattribute-trait)
    XmlAttributeTrait = "smithy.api#xmlAttribute"
);

// ====  Traits that take just a string value ====
string_trait!(
    /// Describes the contents of a blob or string shape using a design-time media type.
    ///
    /// *See* - [`@mediaType`](https://smithy.io/2.0/spec/protocol-traits.html#smithy-api-mediatype-trait)
    MediaTypeTrait = "smithy.api#mediaType"
);
string_trait!(
    /// Allows a serialized object property name in a JSON document to differ from a structure or union
    /// member name used in the model.
    ///
    /// *See* - [`@jsonName`](https://smithy.io/2.0/spec/protocol-traits.html#smithy-api-jsonname-trait)
    JsonNameTrait = "smithy.api#jsonName"
);
string_trait!(
    /// Allows a serialized object property name in an XML document to differ from name used in model.
    ///
    /// *See* - [`@xmlName`](https://smithy.io/2.0/spec/protocol-traits.html#xmlname-trait)
    XmlNameTrait = "smithy.api#xmlName"
);
string_trait!(
    /// Binds a structure member to an HTTP header.
    ///
    /// *See* - [`@httpHeader`](https://smithy.io/2.0/spec/http-bindings.html#smithy-api-httpheader-trait)
    HTTPHeaderTrait = "smithy.api#httpHeader"
);
string_trait!(
    /// Binds a map of key-value pairs to prefixed HTTP headers.
    ///
    /// *See* - [`@httpPrefixHeaders`](https://smithy.io/2.0/spec/http-bindings.html#httpprefixheaders-trait)
    HTTPPrefixHeadersTrait = "smithy.api#httpPrefixHeaders"
);
string_trait!(
    /// Binds an operation input structure member to a query string parameter.
    ///
    /// *See* - [`@httpQuery`](https://smithy.io/2.0/spec/http-bindings.html#httpquery-trait)
    HTTPQueryTrait = "smithy.api#httpQuery"
);
string_trait!(
    /// Configures a custom operation endpoint.
    ///
    /// *See* - [`@endpoint`](https://smithy.io/2.0/spec/endpoint-traits.html#smithy-api-endpoint-trait)
    EndpointTrait = "smithy.api#endpoint"
);

// ==== Traits with other values ====

/// Provides a structure member with a default value.
///
/// *See* - [Default Trait](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-default-trait)
#[derive(Debug)]
pub struct DefaultTrait(pub Box<dyn Document>);
static_trait_id!(DefaultTrait, "smithy.api#default");

impl SmithyTrait for DefaultTrait {
    fn id(&self) -> &ShapeId {
        DefaultTrait::trait_id()
    }

    fn from_document(document: Box<dyn Document>) -> Option<Self>
    where
        Self: Sized,
    {
        Some(DefaultTrait(document))
    }
}
impl DefaultTrait {
    /// Create a new [`DefaultTrait`] instance
    pub fn new<D: Into<Box<dyn Document>>>(doc: D) -> Self {
        DefaultTrait(doc.into())
    }
}

/// Indicates that a structure shape represents an error.
///
/// *See* - [Error Trait](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-error-trait)
#[derive(Debug)]
pub struct ErrorTrait(pub ErrorFault);
impl ErrorTrait {
    /// Create a new [`ErrorTrait`] instance
    #[must_use]
    pub fn new(error: ErrorFault) -> Self {
        ErrorTrait(error)
    }
}
static_trait_id!(ErrorTrait, "smithy.api#error");
impl SmithyTrait for ErrorTrait {
    fn id(&self) -> &ShapeId {
        <ErrorTrait as StaticTraitId>::trait_id()
    }

    fn from_document(document: Box<dyn Document>) -> Option<Self>
    where
        Self: Sized,
    {
        document.as_string().and_then(|str| match str {
            "client" => Some(Self(ErrorFault::Client)),
            "server" => Some(Self(ErrorFault::Server)),
            _ => None,
        })
    }
}

/// Indicates if the client or server is at fault for a given error.
#[derive(Debug)]
#[doc(hidden)]
pub enum ErrorFault {
    Client,
    Server,
}
impl Display for ErrorFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorFault::Client => write!(f, "client"),
            ErrorFault::Server => write!(f, "server"),
        }
    }
}

/// Defines an HTTP response code for an operation error.
///
/// *See* - [HttpError Trait](https://smithy.io/2.0/spec/http-bindings.html#smithy-api-httperror-trait)
#[derive(Debug)]
pub struct HttpErrorTrait(pub i32);
impl HttpErrorTrait {
    /// Create a new [`HttpErrorTrait`] instance
    ///
    /// # Panics
    /// Http error codes should be between 200 and 600. This
    /// constructor panics when an error code is outside of
    /// this range is provided.
    ///
    /// Smithy validation will check this constraint in models.
    #[must_use]
    pub fn new(code: i32) -> Self {
        assert!(
            200 < code && code < 599,
            "HTTPErrorTrait code out of range: {code}"
        );
        Self(code)
    }
}
static_trait_id!(HttpErrorTrait, "smithy.api#httpError");
impl SmithyTrait for HttpErrorTrait {
    fn id(&self) -> &ShapeId {
        <HttpErrorTrait as StaticTraitId>::trait_id()
    }

    fn from_document(document: Box<dyn Document>) -> Option<Self>
    where
        Self: Sized,
    {
        document.as_integer().map(HttpErrorTrait::new)
    }
}

// ============================================================================
// Constraint Traits
// ============================================================================

/// Defines a range constraint for numeric values.
///
/// *See* - [Range Trait](https://smithy.io/2.0/spec/constraint-traits.html#range-trait)
#[derive(Debug)]
pub struct RangeTrait {
    min: Option<BigDecimal>,
    max: Option<BigDecimal>,
}
static_trait_id!(RangeTrait, "smithy.api#range");
impl SmithyTrait for RangeTrait {
    fn id(&self) -> &ShapeId {
        <RangeTrait as StaticTraitId>::trait_id()
    }

    fn from_document(_document: Box<dyn Document>) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}

static ZERO: LazyLock<BigDecimal> = LazyLock::new(BigDecimal::zero);
static MAX: LazyLock<BigDecimal> = LazyLock::new(|| BigDecimal::from(u64::MAX));

impl RangeTrait {
    /// Get the minimum value allowed by this range constraint.
    ///
    /// Defaults to zero
    #[inline]
    #[must_use]
    pub fn min(&self) -> &BigDecimal {
        self.min.as_ref().unwrap_or_else(|| &ZERO)
    }

    /// Get the maximum value allowed by this range constraint.
    ///
    /// Defaults to `u64::MAX`
    #[inline]
    #[must_use]
    pub fn max(&self) -> &BigDecimal {
        self.max.as_ref().unwrap_or_else(|| &MAX)
    }

    /// Get a new builder instance for this trait.
    #[must_use]
    pub const fn builder() -> RangeTraitBuilder {
        RangeTraitBuilder::new()
    }
}

/// Builder for the [`RangeTrait`]
#[derive(Debug)]
pub struct RangeTraitBuilder {
    min: Option<BigDecimal>,
    max: Option<BigDecimal>,
}
impl RangeTraitBuilder {
    pub(super) const fn new() -> Self {
        RangeTraitBuilder {
            min: None,
            max: None,
        }
    }

    /// Set a minimum value for this constraint.
    ///
    /// # Panics
    /// If the string is not a valid bigDecimal. This is validated
    /// by the Smithy build system.
    #[must_use]
    pub fn min(mut self, min: &str) -> Self {
        self.min = Some(BigDecimal::from_str(min).expect("invalid min"));
        self
    }

    /// Set a maximum value for this constraint.
    ///
    /// # Panics
    /// If the string is not a valid bigDecimal. This is validated
    /// by the Smithy build system.
    #[must_use]
    pub fn max(mut self, max: &str) -> Self {
        self.max = Some(BigDecimal::from_str(max).expect("invalid max"));
        self
    }

    /// Construct a new [`RangeTrait`] instance.
    #[must_use]
    pub fn build(self) -> RangeTrait {
        RangeTrait {
            min: self.min,
            max: self.max,
        }
    }
}

/// Length constraint for lists, maps, and strings.
///
/// *See* - [LengthTrait](https://smithy.io/2.0/spec/constraint-traits.html#length-trait)
#[derive(Debug)]
pub struct LengthTrait {
    min: Option<usize>,
    max: Option<usize>,
}
static_trait_id!(LengthTrait, "smithy.api#length");
impl SmithyTrait for LengthTrait {
    fn id(&self) -> &ShapeId {
        <LengthTrait as StaticTraitId>::trait_id()
    }

    fn from_document(_document: Box<dyn Document>) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl LengthTrait {
    /// Get the minimum length for this constraint.
    ///
    /// Defaults to 0 if not set.
    #[must_use]
    pub fn min(&self) -> usize {
        self.min.unwrap_or(0)
    }

    /// Get the maximum length for this constraint.
    ///
    /// Defaults to `usize::MAX` if not set.
    #[must_use]
    pub fn max(&self) -> usize {
        self.max.unwrap_or(usize::MAX)
    }

    /// Get a new [`LengthTraitBuilder`] instance.
    #[must_use]
    pub const fn builder() -> LengthTraitBuilder {
        LengthTraitBuilder::new()
    }
}

/// Builder for creating a [`LengthTrait`] instance
#[derive(Debug)]
pub struct LengthTraitBuilder {
    min: Option<usize>,
    max: Option<usize>,
}
impl LengthTraitBuilder {
    pub(super) const fn new() -> Self {
        LengthTraitBuilder {
            min: None,
            max: None,
        }
    }

    /// Set a minimum length for this constraint.
    #[must_use]
    pub fn min(mut self, min: usize) -> Self {
        self.min = Some(min);
        self
    }

    /// Set a maximum length for this constraint.
    #[must_use]
    pub fn max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    /// Create a new [`LengthTrait`] instance.
    #[must_use]
    pub fn build(self) -> LengthTrait {
        LengthTrait {
            min: self.min,
            max: self.max,
        }
    }
}

annotation_trait!(
    /// Requires that items in a list are unique.
    ///
    /// *See* - [`@uniqueItems`](https://smithy.io/2.0/spec/constraint-traits.html#smithy-api-uniqueitems-trait)
    UniqueItemsTrait = "smithy.api#uniqueItems"
);

/// Regex pattern used to constrian string values
///
/// *See* - [LengthTrait](https://smithy.io/2.0/spec/constraint-traits.html#pattern-trait)
#[derive(Debug)]
pub struct PatternTrait(pub Regex);
static_trait_id!(PatternTrait, "smithy.api#pattern");
impl SmithyTrait for PatternTrait {
    fn id(&self) -> &ShapeId {
        <PatternTrait as StaticTraitId>::trait_id()
    }

    #[inline]
    fn from_document(document: Box<dyn Document>) -> Option<Self>
    where
        Self: Sized,
    {
        document.as_string().map(PatternTrait::new)
    }
}
impl PatternTrait {
    #[must_use]
    /// Create a new [`PatternTrait`]
    ///
    /// # Panics
    /// Will panic if the pattern is invalid.
    ///
    /// Smithy validation will check this constraint in models
    pub fn new(pattern: &str) -> Self {
        PatternTrait(Regex::new(pattern).unwrap())
    }
}

// ============================================================================
// Auth Traits
// ============================================================================

annotation_trait!(
    /// Indicates that a service supports HTTP Basic Authentication.
    ///
    /// *See* - [`@httpBasicAuth`](https://smithy.io/2.0/spec/authentication-traits.html#smithy-api-httpbasicauth-trait)
    HttpBasicAuthTrait = "smithy.api#httpBasicAuth"
);
annotation_trait!(
    /// Indicates that a service supports HTTP Digest Authentication.
    ///
    /// *See* - [`@httpDigestAuth`](https://smithy.io/2.0/spec/authentication-traits.html#httpdigestauth-trait)
    HttpDigestAuthTrait = "smithy.api#httpDigestAuth"
);
annotation_trait!(
    /// Indicates that a service supports HTTP Bearer Authentication .
    ///
    /// *See* - [`@httpBearerAuth`](https://smithy.io/2.0/spec/authentication-traits.html#httpbearerauth-trait)
    HttpBearerAuthTrait = "smithy.api#httpBearerAuth"
);

/// Trait indicating that a service supports HTTP-specific authentication
/// using an API key sent in a header or query string parameter.
///
/// *See* - [`@httpApiKeyAuth`](https://smithy.io/2.0/spec/authentication-traits.html#httpapikeyauth-trait)
#[derive(Debug)]
pub struct HttpApiKeyAuthTrait {
    name: String,
    in_location: String,
    scheme: Option<String>,
}
static_trait_id!(HttpApiKeyAuthTrait, "smithy.api#httpApiKeyAuth");
impl SmithyTrait for HttpApiKeyAuthTrait {
    fn id(&self) -> &ShapeId {
        <HttpApiKeyAuthTrait as StaticTraitId>::trait_id()
    }

    #[inline]
    fn from_document(_document: Box<dyn Document>) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}
impl HttpApiKeyAuthTrait {
    /// Get the `name`
    #[must_use]
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Get the `location`
    #[must_use]
    pub fn in_location(&self) -> &String {
        &self.in_location
    }

    /// Get the `scheme`
    #[must_use]
    pub fn scheme(&self) -> &Option<String> {
        &self.scheme
    }

    /// Get a new [`HttpApiKeyAuthTraitBuilder`] instance.
    #[must_use]
    pub fn builder() -> HttpApiKeyAuthTraitBuilder {
        HttpApiKeyAuthTraitBuilder::new()
    }
}

/// Builder for [`HttpApiKeyAuthTrait`] instances
pub struct HttpApiKeyAuthTraitBuilder {
    name: Option<String>,
    in_location: Option<String>,
    scheme: Option<String>,
}
impl HttpApiKeyAuthTraitBuilder {
    fn new() -> Self {
        HttpApiKeyAuthTraitBuilder {
            name: None,
            in_location: None,
            scheme: None,
        }
    }

    /// Set the `name` to use
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Set the `location` to use
    #[must_use]
    pub fn in_location(mut self, in_location: &str) -> Self {
        self.in_location = Some(in_location.to_string());
        self
    }

    /// Set the `scheme` to use
    #[must_use]
    pub fn scheme(mut self, scheme: &str) -> Self {
        self.scheme = Some(scheme.to_string());
        self
    }

    /// Build a new [`HttpApiKeyAuthTrait`] instance
    ///
    /// # Panics
    /// If the location or name are not set.
    ///
    /// Smithy validation will check this constraint in models.
    #[must_use]
    pub fn build(self) -> HttpApiKeyAuthTrait {
        HttpApiKeyAuthTrait {
            name: self.name.unwrap(),
            in_location: self.in_location.unwrap(),
            scheme: self.scheme,
        }
    }
}
