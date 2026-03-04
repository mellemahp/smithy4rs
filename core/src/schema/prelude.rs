//! # Prelude
//! [`Schema`] definitions for the [Smithy prelude](https://github.com/smithy-lang/smithy/blob/65b473ddb94f9edda933f00bab988d465b2bd2fe/smithy-model/src/main/resources/software/amazon/smithy/model/loader/prelude.smithy)
//!
//! The prelude consists of public, built-in shapes like `STRING`, `INTEGER`, etc. that
//! are available to all models. Prelude shapes and traits are all in the `smithy.api` namespace
//! and must be hard-coded as they are used by generate shapes.

use bigdecimal::Zero;
use regex::Regex;
use smithy4rs_core_derive::{SmithyShape, SmithyTraitImpl, smithy_enum};

use crate::{
    BigDecimal, LazyLock, annotation_trait,
    schema::{Document, DocumentError, Schema, TryFromDocument},
    serde::{
        Buildable, ShapeBuilder,
        de::{DeserializeWithSchema, Deserializer},
        deserializers::Error,
        se::{SerializeWithSchema, Serializer},
        validation::ValidationErrors,
    },
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
    SensitiveTrait: SENSITIVE = "smithy.api#sensitive"
);
annotation_trait!(
    /// Indicates that the data represented by the shape needs to be streamed.
    ///
    /// *See* - [`@streaming`](https://smithy.io/2.0/spec/streaming.html#smithy-api-streaming-trait)
    StreamingTrait: STREAMING = "smithy.api#streaming"
);
annotation_trait!(
    /// Indicates that lists and maps MAY contain null values
    ///
    /// *See* - [`@sparse`](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-sparse-trait)
    SparseTrait: SPARSE = "smithy.api#sparse");
annotation_trait!(
    /// Indicates that a member value MUST be set for the shape to be valid.
    ///
    /// *See* - [`@required`](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-required-trait)
    RequiredTrait: REQUIRED = "smithy.api#required"
);
annotation_trait!(
    /// Indicates that a member should be serialized as an event header when sent through an event stream.
    ///
    /// *See* - [`@eventHeader`](https://smithy.io/2.0/spec/streaming.html#smithy-api-eventheader-trait)
    EventHeaderTrait: EVENT_HEADER = "smithy.api#eventheader"
);
annotation_trait!(
    /// Indicates that a member should be serialized as the payload of an event when sent through an event stream.
    ///
    /// *See* - [`@eventPayload`](https://smithy.io/2.0/spec/streaming.html#eventpayload-trait)
    EventPayloadTrait: EVENT_PAYLOAD = "smithy.api#eventPayload"
);
annotation_trait!(
    /// Indicates that a member is used by the server to identify and discard replayed requests.
    ///
    /// *See* - [`@idempotencyToken`](https://smithy.io/2.0/spec/behavior-traits.html#smithy-api-idempotencytoken-trait)
    IdempotencyTokenTrait: IDEMPOTENCY_TOKEN = "smithy.api#IdempotencyToken"
);
annotation_trait!(
    /// Binds a member so that it is used as part of an HTTP request URI.
    ///
    /// *See* - [`@httpLabel`](https://smithy.io/2.0/spec/http-bindings.html#smithy-api-httplabel-trait)
    HttpLabelTrait: HTTP_LABEL = "smithy.api#httpLabel"
);
annotation_trait!(
    /// Binds a member to the body of an HTTP message.
    ///
    /// *See* - [`@httpPayload`](https://smithy.io/2.0/spec/http-bindings.html#httppayload-trait)
    HttpPayloadTrait: HTTP_PAYLOAD = "smithy.api#httpPayload"
);
annotation_trait!(
    /// Binds a map of key-value pairs to query string parameters.
    ///
    /// *See* - [`@httpQueryParams`](https://smithy.io/2.0/spec/http-bindings.html#httpqueryparams-trait)
    HTTPQueryParamsTrait: HTTP_QUERY_PARAMS = "smithy.api#httpQueryParams"
);
annotation_trait!(
    /// Binds a member to the HTTP response status code so that an HTTP response status code can be set dynamically.
    ///
    /// *See* - [`@httpResponseCode`](https://smithy.io/2.0/spec/http-bindings.html#httpresponsecode-trait)
    HTTPResponseCodeTrait: HTTP_RESPONSE = "smithy.api#httpResponseCode"
);
annotation_trait!(
    /// Indicates that an operation requires a checksum in HTTP requests.
    ///
    /// *See* - [`@httpChecksumRequired`](https://smithy.io/2.0/spec/http-bindings.html#httpchecksumrequired-trait)
    HTTPChecksumRequiredTrait: HTTP_CHECKSUM_REQUIRED = "smithy.api#httpChecksumRequired"
);
annotation_trait!(
    /// Binds a top-level operation input structure member to a label in the hostPrefix of an endpoint trait.
    ///
    /// *See* - [`@eventPayload`](https://smithy.io/2.0/spec/endpoint-traits.html#smithy-api-hostlabel-trait)
    HostLabelTrait: HOST_LABEL = "smithy.api#hostLabel"
);
annotation_trait!(
    /// Serializes an object property as an XML attribute rather than a nested XML element.
    ///
    /// *See* - [`@eventPayload`](https://smithy.io/2.0/spec/protocol-traits.html#xmlattribute-trait)
    XmlAttributeTrait: XML_ATTRIBUTE = "smithy.api#xmlAttribute"
);

// ====  Traits that take just a string value ====
string_trait!(
    /// Describes the contents of a blob or string shape using a design-time media type.
    ///
    /// *See* - [`@mediaType`](https://smithy.io/2.0/spec/protocol-traits.html#smithy-api-mediatype-trait)
    MediaTypeTrait: MEDIA_TYPE = "smithy.api#mediaType"
);
string_trait!(
    /// Allows a serialized object property name in a JSON document to differ from a structure or union
    /// member name used in the model.
    ///
    /// *See* - [`@jsonName`](https://smithy.io/2.0/spec/protocol-traits.html#smithy-api-jsonname-trait)
    JsonNameTrait: JSON_NAME = "smithy.api#jsonName"
);
string_trait!(
    /// Allows a serialized object property name in an XML document to differ from name used in model.
    ///
    /// *See* - [`@xmlName`](https://smithy.io/2.0/spec/protocol-traits.html#xmlname-trait)
    XmlNameTrait: XML_NAME = "smithy.api#xmlName"
);
string_trait!(
    /// Binds a structure member to an HTTP header.
    ///
    /// *See* - [`@httpHeader`](https://smithy.io/2.0/spec/http-bindings.html#smithy-api-httpheader-trait)
    HTTPHeaderTrait: HTTP_HEADER = "smithy.api#httpHeader"
);
string_trait!(
    /// Binds a map of key-value pairs to prefixed HTTP headers.
    ///
    /// *See* - [`@httpPrefixHeaders`](https://smithy.io/2.0/spec/http-bindings.html#httpprefixheaders-trait)
    HTTPPrefixHeadersTrait: HTTP_PREFIX_HEADERS = "smithy.api#httpPrefixHeaders"
);
string_trait!(
    /// Binds an operation input structure member to a query string parameter.
    ///
    /// *See* - [`@httpQuery`](https://smithy.io/2.0/spec/http-bindings.html#httpquery-trait)
    HTTPQueryTrait: HTTP_QUERY = "smithy.api#httpQuery"
);
string_trait!(
    /// Configures a custom operation endpoint.
    ///
    /// *See* - [`@endpoint`](https://smithy.io/2.0/spec/endpoint-traits.html#smithy-api-endpoint-trait)
    EndpointTrait: ENDPOINT = "smithy.api#endpoint"
);

// ==== Traits with other values ====

smithy!("smithy.api#default": {
    /// Schema for [`DefaultTrait`]
    document DEFAULT
});

/// Provides a structure member with a default value.
///
/// *See* - [Default Trait](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-default-trait)
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(DEFAULT)]
pub struct DefaultTrait(pub Box<dyn Document>);

smithy!("smithy.api#error": {
    /// Schema for [`ErrorTrait`]
    enum ERROR {
        CLIENT = "client"
        SERVER = "server"
    }
});

/// Indicates that a structure shape represents an error.
///
/// *See* - [Error Trait](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-error-trait)
#[smithy_enum]
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(ERROR)]
pub enum ErrorTrait {
    /// Error originates from client
    Client = "client",
    /// Error originates from server
    Server = "server",
}

smithy!("smithy.api#httpError": {
    /// Schema for [`HttpErrorTrait`]
    integer HTTP_ERROR
});

/// Defines an HTTP response code for an operation error.
///
/// *See* - [HttpError Trait](https://smithy.io/2.0/spec/http-bindings.html#smithy-api-httperror-trait)
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(HTTP_ERROR)]
pub struct HttpErrorTrait(pub i32);

// ============================================================================
// Constraint Traits
// ============================================================================

smithy!("smithy.api#range": {
    /// Schema for [`RangeTrait`]
    structure RANGE {
        MIN: BIG_DECIMAL = "min"
        MAX: BIG_DECIMAL = "max"
    }
});

/// Defines a range constraint for numeric values.
///
/// *See* - [Range Trait](https://smithy.io/2.0/spec/constraint-traits.html#range-trait)
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(RANGE)]
pub struct RangeTrait {
    /// Minimum value
    #[smithy_schema(MIN)]
    min: Option<BigDecimal>,
    /// Maximum value
    #[smithy_schema(MAX)]
    max: Option<BigDecimal>,
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
}

smithy!("smithy.api#length": {
    /// Schema for [`LengthTrait`]
    structure LENGTH {
        MIN: LONG = "min"
        MAX: LONG = "max"
    }
});

/// Length constraint for lists, maps, and strings.
///
/// *See* - [LengthTrait](https://smithy.io/2.0/spec/constraint-traits.html#length-trait)
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(LENGTH)]
pub struct LengthTrait {
    #[smithy_schema(MIN)]
    min: Option<i64>,
    #[smithy_schema(MAX)]
    max: Option<i64>,
}
static_trait_id!(LengthTrait, "smithy.api#length");
impl TryFromDocument for LengthTrait {
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        Ok(<LengthTraitBuilder as TryFromDocument>::try_from(document)?.build()?)
    }
}

impl LengthTrait {
    /// Get the minimum length for this constraint.
    ///
    /// Defaults to 0 if not set.
    #[must_use]
    pub fn min(&self) -> usize {
        self.min.unwrap_or(0) as usize
    }

    /// Get the maximum length for this constraint.
    ///
    /// Defaults to `usize::MAX` if not set.
    #[must_use]
    pub fn max(&self) -> usize {
        self.max.unwrap_or(i64::MAX) as usize
    }
}

annotation_trait!(
    /// Requires that items in a list are unique.
    ///
    /// *See* - [`@uniqueItems`](https://smithy.io/2.0/spec/constraint-traits.html#smithy-api-uniqueitems-trait)
    UniqueItemsTrait: UNIQUE_ITEMS = "smithy.api#uniqueItems"
);

smithy!("smithy.api#pattern": {
    /// Schema for [`PatternTrait`]
    string PATTERN
});

/// Regex pattern used to constrian string values
///
/// *See* - [LengthTrait](https://smithy.io/2.0/spec/constraint-traits.html#pattern-trait)
#[derive(SmithyShape, Clone)]
#[smithy_schema(PATTERN)]
pub struct PatternTrait(pub Regex);
static_trait_id!(PatternTrait, "smithy.api#pattern");
impl PatternTrait {
    /// Create a new `PatternTrait` instance
    pub fn new(pattern: &str) -> Result<PatternTrait, ValidationErrors> {
        let pat =
            Regex::new(pattern).map_err(<ValidationErrors as crate::serde::se::Error>::custom)?;
        Ok(Self(pat))
    }
}
impl SerializeWithSchema for Regex {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.write_string(schema, self.as_str())
    }
}
impl<'de> DeserializeWithSchema<'de> for Regex {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = deserializer.read_string(schema)?;
        Regex::new(&s).map_err(D::Error::custom)
    }
}

// ============================================================================
// Auth Traits
// ============================================================================

annotation_trait!(
    /// Indicates that a service supports HTTP Basic Authentication.
    ///
    /// *See* - [`@httpBasicAuth`](https://smithy.io/2.0/spec/authentication-traits.html#smithy-api-httpbasicauth-trait)
    HttpBasicAuthTrait: HTTP_BASIC_AUTH = "smithy.api#httpBasicAuth"
);
annotation_trait!(
    /// Indicates that a service supports HTTP Digest Authentication.
    ///
    /// *See* - [`@httpDigestAuth`](https://smithy.io/2.0/spec/authentication-traits.html#httpdigestauth-trait)
    HttpDigestAuthTrait: HTTP_DIGEST_AUTH = "smithy.api#httpDigestAuth"
);
annotation_trait!(
    /// Indicates that a service supports HTTP Bearer Authentication .
    ///
    /// *See* - [`@httpBearerAuth`](https://smithy.io/2.0/spec/authentication-traits.html#httpbearerauth-trait)
    HttpBearerAuthTrait: HTTP_BEARER_AUTH = "smithy.api#httpBearerAuth"
);

smithy!("smithy.api#NonEmptyString": {
    #[doc(hidden)]
    @LengthTrait::builder().min(1).build();
    string NON_EMPTY_STRING
});

smithy!("smithy.api#httpApiKeyLocations": {
    /// Schema for [`HttpApiKeyLocations`]
    enum HTTP_API_KEY_LOCATIONS {
        Header = "header"
        Query = "query"
    }
});

/// Options for API key locations
#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(HTTP_API_KEY_LOCATIONS)]
pub enum HttpApiKeyLocations {
    /// Key is in Http Headers
    Header = "header",
    /// Key is in Query params
    Query = "query",
}
smithy!("smithy.api#httpApiKeyAuth": {
    /// Schema for [`HttpApiKeyAuthTrait`]
    structure HTTP_API_KEY_AUTH {
        @RequiredTrait;
        NAME: NON_EMPTY_STRING = "name"
        @RequiredTrait;
        IN: HTTP_API_KEY_LOCATIONS = "in"
        SCHEME: NON_EMPTY_STRING = "scheme"
    }
});

/// Trait indicating that a service supports HTTP-specific authentication
/// using an API key sent in a header or query string parameter.
///
/// *See* - [`@httpApiKeyAuth`](https://smithy.io/2.0/spec/authentication-traits.html#httpapikeyauth-trait)
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(HTTP_API_KEY_AUTH)]
pub struct HttpApiKeyAuthTrait {
    #[smithy_schema(NAME)]
    name: String,
    // TODO: Can we find a way to determine if builder should be used or not?
    #[smithy_schema(IN)]
    #[no_builder]
    in_location: HttpApiKeyLocations,
    #[smithy_schema(SCHEME)]
    scheme: Option<String>,
}
