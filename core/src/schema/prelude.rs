#![allow(dead_code)]

use std::{fmt::Display, sync::LazyLock};

use indexmap::IndexMap;
use regex::Regex;

use crate::{
    annotation_trait, lazy_shape_id,
    schema::{
        DocumentValue, NumberInteger, NumberValue, Schema, SchemaRef, ShapeId, SmithyTrait,
        StaticTraitId,
    },
    static_trait_id, string_trait, traits,
};

////////////////////////////////////////////////////////////////////////////////////
// Prelude Shape Schemas
////////////////////////////////////////////////////////////////////////////////////
macro_rules! prelude_schema {
    ($ident:ident, $factory:expr, $id:literal) => {
        pub static $ident: LazyLock<SchemaRef> = LazyLock::new(|| $factory($id, traits![]));
    };
}

prelude_schema!(BLOB, Schema::create_blob, "smithy.api#Blob");
prelude_schema!(BOOLEAN, Schema::create_boolean, "smithy.api#Boolean");
prelude_schema!(STRING, Schema::create_string, "smithy.api#String");
prelude_schema!(TIMESTAMP, Schema::create_timestamp, "smithy.api#Timestamp");
prelude_schema!(BYTE, Schema::create_byte, "smithy.api#Byte");
prelude_schema!(SHORT, Schema::create_short, "smithy.api#Short");
prelude_schema!(INTEGER, Schema::create_integer, "smithy.api#Integer");
prelude_schema!(LONG, Schema::create_long, "smithy.api#Long");
prelude_schema!(FLOAT, Schema::create_float, "smithy.api#Float");
prelude_schema!(DOUBLE, Schema::create_double, "smithy.api#Double");
prelude_schema!(
    BIG_INTEGER,
    Schema::create_big_integer,
    "smithy.api#BigInteger"
);
prelude_schema!(
    BIG_DECIMAL,
    Schema::create_big_decimal,
    "smithy.api#BigDecimal"
);
prelude_schema!(DOCUMENT, Schema::create_document, "smithy.api#Document");

// TODO:
// - Primitive types

///////////////////////////////////////////////////////////////////////
// Prelude Traits
///////////////////////////////////////////////////////////////////////

// ==== Annotation traits ====
annotation_trait!(SensitiveTrait, SENSITIVE_TRAIT_ID, "smithy.api#sensitive");
annotation_trait!(StreamingTrait, STREAMING_TRAIT_ID, "smithy.api#streaming");
annotation_trait!(SparseTrait, SPARSE_TRAIT_ID, "smithy.api#sparse");
annotation_trait!(RequiredTrait, REQUIRED_TRAIT_ID, "smithy.api#required");
annotation_trait!(
    UnitTypeTrait,
    UNIT_TYPE_TRAIT_ID,
    "smithy.api#UnitTypeTrait"
);
annotation_trait!(
    EventHeaderTrait,
    EVENT_HEADER_TRAIT_ID,
    "smithy.api#eventheader"
);
annotation_trait!(
    EventPayloadTrait,
    EVENT_PAYLOAD_TRAIT_ID,
    "smithy.api#eventPayload"
);
annotation_trait!(
    IdempotencyTokenTrait,
    IDEMPOTENCY_TOKEN_TRAIT_ID,
    "smithy.api#IdempotencyToken"
);
annotation_trait!(HttpLabelTrait, HTTP_LABEL_TRAIT_ID, "smithy.api#httpLabel");
annotation_trait!(
    HttpPayloadTrait,
    HTTP_PAYLOAD_TRAIT_ID,
    "smithy.api#httpPayload"
);
annotation_trait!(
    HTTPQueryParamsTrait,
    HTTP_QUERY_PARAMS_TRAIT_ID,
    "smithy.api#httpQueryParams"
);
annotation_trait!(
    HTTPResponseCodeTrait,
    HTTP_RESPONSE_CODE_TRAIT_ID,
    "smithy.api#httpResponseCode"
);
annotation_trait!(
    HTTPChecksumRequiredTrait,
    HTTP_CHECKSUM_REQUIRED_TRAIT_ID,
    "smithy.api#httpChecksumRequired"
);
annotation_trait!(
    HostLabelTrait,
    HTTP_HOST_LABEL_TRAIT_ID,
    "smithy.api#hostLabel"
);

// ====  Traits that take just a string value ====
string_trait!(
    MediaTypeTrait,
    MEDIA_TYPE_TRAIT_ID,
    media_type,
    "smithy.api#mediaType"
);
string_trait!(
    JsonNameTrait,
    JSON_NAME_TRAIT_ID,
    name,
    "smithy.api#jsonName"
);
string_trait!(
    HTTPHeaderTrait,
    HTTP_HEADER_TRAIT_ID,
    name,
    "smithy.api#httpHeader"
);
string_trait!(
    HTTPPrefixHeadersTrait,
    HTTP_PREFIX_HEADERS_TRAIT_ID,
    prefix,
    "smithy.api#httpPrefixHeaders"
);
string_trait!(
    HTTPQueryTrait,
    HTTP_QUERY_TRAIT_ID,
    key,
    "smithy.api#httpQuery"
);
string_trait!(
    EndpointTrait,
    ENDPOINT_TRAIT_ID,
    host_prefix,
    "smithy.api#endpoint"
);

// ==== Traits with other values ====

/// Provides a structure member with a default value.
///
/// *See* - [Default Trait](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-default-trait)
#[derive(Debug)]
pub struct DefaultTrait(DocumentValue);
static_trait_id!(DefaultTrait, DEFAULT_TRAIT_ID, "smithy.api#default");
impl SmithyTrait for DefaultTrait {
    fn id(&self) -> &ShapeId {
        DefaultTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.0
    }
}

/// Indicates that a structure shape represents an error.
///
/// *See* - [Error Trait](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-error-trait)
#[derive(Debug)]
pub struct ErrorTrait {
    error: ErrorFault,
    value: DocumentValue,
}
impl ErrorTrait {
    /// Get whether the Error was the fault of the client or server.
    pub fn error(&self) -> &ErrorFault {
        &self.error
    }

    #[must_use]
    pub fn new(error: ErrorFault) -> Self {
        ErrorTrait {
            value: DocumentValue::String(error.to_string()),
            error,
        }
    }
}
static_trait_id!(ErrorTrait, ERROR_TRAIT_ID, "smithy.api#error");
impl SmithyTrait for ErrorTrait {
    fn id(&self) -> &ShapeId {
        ErrorTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

/// Indicates if the client or server is at fault for a given error.
#[derive(Debug)]
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
pub struct HTTPErrorTrait {
    code: i32,
    value: DocumentValue,
}
impl HTTPErrorTrait {
    pub fn code(&self) -> i32 {
        self.code
    }

    #[must_use]
    pub fn new(code: i32) -> Self {
        assert!(
            200 < code && code < 599,
            "HTTPErrorTrait code out of range: {code}"
        );
        HTTPErrorTrait {
            code,
            value: DocumentValue::Number(NumberValue::Integer(NumberInteger::Integer(code))),
        }
    }
}
static_trait_id!(HTTPErrorTrait, HTTP_ERROR_TRAIT_ID, "smithy.api#httpError");
impl SmithyTrait for HTTPErrorTrait {
    fn id(&self) -> &ShapeId {
        HTTPErrorTrait::trait_id()
    }
    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

/////////////////////////////////////////////////
// Constraint Traits
/////////////////////////////////////////////////
#[derive(Debug)]
pub struct RangeTrait {
    min: Option<usize>,
    max: Option<usize>,
    value: DocumentValue,
}
static_trait_id!(RangeTrait, RANGE_TRAIT_ID, "smithy.api#range");

impl SmithyTrait for RangeTrait {
    fn id(&self) -> &ShapeId {
        RangeTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

/// Builder for the [`RangeTrait`]
impl RangeTrait {
    pub fn min(&self) -> &Option<usize> {
        &self.min
    }

    pub fn max(&self) -> &Option<usize> {
        &self.max
    }

    #[must_use]
    pub const fn builder() -> RangeTraitBuilder {
        RangeTraitBuilder::new()
    }
}

#[derive(Debug)]
pub struct RangeTraitBuilder {
    min: Option<usize>,
    max: Option<usize>,
}
impl RangeTraitBuilder {
    pub(crate) const fn new() -> Self {
        RangeTraitBuilder {
            min: None,
            max: None,
        }
    }

    pub fn min(mut self, min: usize) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    pub fn build(self) -> RangeTrait {
        let mut value_map = IndexMap::new();
        if let Some(min) = self.min {
            value_map.insert("min".to_string(), (min as i32).into());
        }
        if let Some(max) = self.max {
            value_map.insert("min".to_string(), (max as i32).into());
        }
        RangeTrait {
            min: self.min,
            max: self.max,
            value: DocumentValue::Map(value_map),
        }
    }
}

#[derive(Debug)]
pub struct LengthTrait {
    min: Option<usize>,
    max: Option<usize>,
    value: DocumentValue,
}
static_trait_id!(LengthTrait, LENGTH_TRAIT_ID, "smithy.api#length");

impl SmithyTrait for LengthTrait {
    fn id(&self) -> &ShapeId {
        LengthTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

impl LengthTrait {
    pub fn min(&self) -> &Option<usize> {
        &self.min
    }

    pub fn max(&self) -> &Option<usize> {
        &self.max
    }

    #[must_use]
    pub const fn builder() -> LengthTraitBuilder {
        LengthTraitBuilder::new()
    }
}

#[derive(Debug)]
pub struct LengthTraitBuilder {
    min: Option<usize>,
    max: Option<usize>,
}
impl LengthTraitBuilder {
    pub(crate) const fn new() -> Self {
        LengthTraitBuilder {
            min: None,
            max: None,
        }
    }

    pub fn min(mut self, min: usize) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    pub fn build(self) -> LengthTrait {
        let mut value_map = IndexMap::new();
        if let Some(min) = self.min {
            value_map.insert("min".to_string(), (min as i32).into());
        }
        if let Some(max) = self.max {
            value_map.insert("min".to_string(), (max as i32).into());
        }
        LengthTrait {
            min: self.min,
            max: self.max,
            value: DocumentValue::Map(value_map),
        }
    }
}

annotation_trait!(
    UniqueItemsTrait,
    UNIQUE_ITEM_TRAIT_ID,
    "smithy.api#uniqueItems"
);

#[derive(Debug)]
pub struct PatternTrait {
    pattern: Regex,
    value: DocumentValue,
}
static_trait_id!(PatternTrait, PATTERN_TRAIT_ID, "smithy.api#pattern");

impl SmithyTrait for PatternTrait {
    fn id(&self) -> &ShapeId {
        PatternTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

impl PatternTrait {
    pub fn pattern(&self) -> &Regex {
        &self.pattern
    }

    #[must_use]
    /// Create a new [`PatternTrait`]
    ///
    /// Will panic if the pattern is invalid.
    pub fn new(pattern: &str) -> Self {
        PatternTrait {
            pattern: Regex::new(pattern).unwrap(),
            value: DocumentValue::String(pattern.to_string()),
        }
    }
}

/////////////////////////////////////////////////
// Auth Traits
/////////////////////////////////////////////////

annotation_trait!(
    HttpBasicAuthTrait,
    HTTP_BASIC_AUTH_TRAIT_ID,
    "smithy.api#httpBasicAuth"
);
annotation_trait!(
    HttpDigestAuthTrait,
    HTTP_DIGEST_AUTH_TRAIT_ID,
    "smithy.api#httpDigestAuth"
);
annotation_trait!(
    HttpBearerAuthTrait,
    HTTP_BEARER_AUTH_TRAIT_ID,
    "smithy.api#httpBearerAuth"
);

#[derive(Debug)]
pub struct HttpApiKeyAuthTrait {
    name: String,
    in_location: String,
    scheme: Option<String>,
    value: DocumentValue,
}
static_trait_id!(
    HttpApiKeyAuthTrait,
    HTTP_API_KEY_AUTH_TRAIT_ID,
    "smithy.api#httpApiKeyAuth"
);
impl SmithyTrait for HttpApiKeyAuthTrait {
    fn id(&self) -> &ShapeId {
        HttpApiKeyAuthTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}
impl HttpApiKeyAuthTrait {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn in_location(&self) -> &String {
        &self.in_location
    }

    pub fn scheme(&self) -> &Option<String> {
        &self.scheme
    }

    #[must_use]
    pub fn builder() -> HttpApiKeyAuthTraitBuilder {
        HttpApiKeyAuthTraitBuilder::new()
    }
}

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

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn in_location(mut self, in_location: &str) -> Self {
        self.in_location = Some(in_location.to_string());
        self
    }

    pub fn scheme(mut self, scheme: &str) -> Self {
        self.scheme = Some(scheme.to_string());
        self
    }

    pub fn build(self) -> HttpApiKeyAuthTrait {
        let mut value_map = IndexMap::new();
        if let Some(name) = &self.name {
            value_map.insert("name".to_string(), name.clone().into());
        }
        if let Some(location) = &self.in_location {
            value_map.insert("location".to_string(), location.clone().into());
        }
        if let Some(scheme) = &self.scheme {
            value_map.insert("scheme".to_string(), scheme.clone().into());
        }
        HttpApiKeyAuthTrait {
            name: self.name.unwrap(),
            in_location: self.in_location.unwrap(),
            scheme: self.scheme,
            value: DocumentValue::Map(value_map),
        }
    }
}
