#![allow(dead_code)]

use std::{fmt::Display, sync::LazyLock};

use crate::{
    annotation_trait, lazy_shape_id,
    schema::{
        DocumentValue, NumberInteger, NumberValue, Schema, SchemaRef, ShapeId, SmithyTrait,
        StaticTraitId,
    },
    static_trait_id, traits,
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
annotation_trait!(InternalTrait, INTERNAL_TRAIT_ID, "smithy.api#internal");
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

// ==== Traits with values ====

/// Provides a structure member with a default value.
///
/// *See* - [Default Trait](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-default-trait)
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

macro_rules! smithy_trait_impl {
    ($t:ident) => {
        impl SmithyTrait for $t {
            fn id(&self) -> &ShapeId {
                $t::trait_id()
            }

            fn value(&self) -> &DocumentValue {
                &self.value
            }
        }
    };
}

/// Indicates that a structure shape represents an error.
///
/// *See* - [Error Trait](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-error-trait)
pub struct ErrorTrait {
    pub error: ErrorFault,
    value: DocumentValue,
}
impl ErrorTrait {
    #[must_use]
    pub fn new(error: ErrorFault) -> Self {
        ErrorTrait {
            value: DocumentValue::String(error.to_string()),
            error,
        }
    }
}
static_trait_id!(ErrorTrait, ERROR_TRAIT_ID, "smithy.api#error");
smithy_trait_impl!(ErrorTrait);

/// Indicates if the client or server is at fault for a given error.
pub enum ErrorFault {
    Client,
    Server,
}
impl Display for ErrorFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ErrorFault::Client => String::from("client"),
            ErrorFault::Server => String::from("server"),
        };
        write!(f, "{str}")
    }
}

/// Describes the contents of a blob or string shape using a design-time media type as
/// defined by [RFC 6838](https://datatracker.ietf.org/doc/html/rfc6838.html).
///
/// *See* - [MediaType Trait](https://smithy.io/2.0/spec/protocol-traits.html#smithy-api-mediatype-trait)
pub struct MediaTypeTrait {
    pub media_type: String,
    value: DocumentValue,
}
impl MediaTypeTrait {
    #[must_use]
    pub fn new(media_type: &str) -> Self {
        MediaTypeTrait {
            media_type: media_type.to_string(),
            value: DocumentValue::String(media_type.to_string()),
        }
    }
}
static_trait_id!(MediaTypeTrait, MEDIA_TYPE_TRAIT_ID, "smithy.api#mediaType");
smithy_trait_impl!(MediaTypeTrait);

/// Allows a serialized object property name in a JSON document to differ from a structure or union member name.
///
/// *See* - [JsonName Trait](https://smithy.io/2.0/spec/protocol-traits.html#smithy-api-jsonname-trait)
pub struct JsonNameTrait {
    pub name: String,
    value: DocumentValue,
}
impl JsonNameTrait {
    #[must_use]
    pub fn new(name: &str) -> Self {
        JsonNameTrait {
            name: name.to_string(),
            value: DocumentValue::String(name.to_string()),
        }
    }
}
static_trait_id!(JsonNameTrait, JSON_NAME_TRAIT_ID, "smithy.api#jsonName");
smithy_trait_impl!(JsonNameTrait);

/// Defines an HTTP response code for an operation error.
///
/// *See* - [HttpError Trait](https://smithy.io/2.0/spec/http-bindings.html#smithy-api-httperror-trait)
pub struct HTTPErrorTrait {
    pub code: i32,
    value: DocumentValue,
}
impl HTTPErrorTrait {
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
smithy_trait_impl!(HTTPErrorTrait);

/// Binds a structure member to an HTTP header.
///
/// *See* - [HttpHeader Trait](https://smithy.io/2.0/spec/http-bindings.html#smithy-api-httpheader-trait)
struct HTTPHeaderTrait {
    pub name: String,
    value: DocumentValue,
}
static_trait_id!(
    HTTPHeaderTrait,
    HTTP_HEADER_TRAIT_ID,
    "smithy.api#httpHeader"
);
smithy_trait_impl!(HTTPHeaderTrait);

impl HTTPHeaderTrait {
    #[must_use]
    pub fn new(name: &str) -> Self {
        HTTPHeaderTrait {
            name: name.to_string(),
            value: DocumentValue::String(name.to_string()),
        }
    }
}

/// Binds a map of key-value pairs to prefixed HTTP headers.
///
/// *See* - [HttpPrefixHeaders Trait](https://smithy.io/2.0/spec/http-bindings.html#smithy-api-httpprefixheaders-trait)
struct HTTPPrefixHeadersTrait {
    pub prefix: String,
    value: DocumentValue,
}
static_trait_id!(
    HTTPPrefixHeadersTrait,
    HTTP_PREFIX_HEADERS_TRAIT_ID,
    "smithy.api#httpPrefixHeaders"
);
smithy_trait_impl!(HTTPPrefixHeadersTrait);

impl HTTPPrefixHeadersTrait {
    #[must_use]
    pub fn new(prefix: &str) -> Self {
        HTTPPrefixHeadersTrait {
            prefix: prefix.to_string(),
            value: DocumentValue::String(prefix.to_string()),
        }
    }
}

/// Binds an operation input structure member to a query string parameter.
///
/// *See* - [HttpQuery Trait](https://smithy.io/2.0/spec/http-bindings.html#httpquery-trait)
struct HTTPQueryTrait {
    pub key: String,
    value: DocumentValue,
}
static_trait_id!(HTTPQueryTrait, HTTP_QUERY_TRAIT_ID, "smithy.api#httpQuery");
smithy_trait_impl!(HTTPQueryTrait);

impl HTTPQueryTrait {
    #[must_use]
    pub fn new(key: &str) -> Self {
        HTTPQueryTrait {
            key: key.to_string(),
            value: DocumentValue::String(key.to_string()),
        }
    }
}

/// Configures a custom operation endpoint.
///
/// *See* - [Endpoint Trait](https://smithy.io/2.0/spec/endpoint-traits.html#smithy-api-endpoint-trait)
pub struct EndpointTrait {
    pub host_prefix: String,
    value: DocumentValue,
}
static_trait_id!(EndpointTrait, ENDPOINT_TRAIT_ID, "smithy.api#endpoint");
smithy_trait_impl!(EndpointTrait);

impl EndpointTrait {
    #[must_use]
    pub fn new(host_prefix: &str) -> Self {
        EndpointTrait {
            host_prefix: host_prefix.to_string(),
            value: DocumentValue::String(host_prefix.to_string()),
        }
    }
}

/////////////////////////////////////////////////
// Constraint Traits
/////////////////////////////////////////////////
#[derive(Debug)]
pub struct RangeTrait {
    pub min: Option<usize>,
    pub max: Option<usize>,
}
static_trait_id!(RangeTrait, RANGE_TRAIT_ID, "smithy.api#range");

impl SmithyTrait for RangeTrait {
    fn id(&self) -> &ShapeId {
        RangeTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        todo!()
    }
}
// Builder
impl RangeTrait {
    #[must_use]
    pub const fn builder() -> RangeTraitBuilder {
        RangeTraitBuilder::new()
    }
}

pub struct RangeTraitBuilder {
    pub min: Option<usize>,
    pub max: Option<usize>,
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
        RangeTrait {
            min: self.min,
            max: self.max,
        }
    }
}

#[derive(Debug)]
pub struct LengthTrait {
    pub min: Option<usize>,
    pub max: Option<usize>,
}
static_trait_id!(LengthTrait, LENGTH_TRAIT_ID, "smithy.api#length");

impl SmithyTrait for LengthTrait {
    fn id(&self) -> &ShapeId {
        LengthTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        todo!()
    }
}

// Builder
impl LengthTrait {
    #[must_use]
    pub const fn builder() -> LengthTraitBuilder {
        LengthTraitBuilder::new()
    }
}

pub struct LengthTraitBuilder {
    pub min: Option<usize>,
    pub max: Option<usize>,
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
        LengthTrait {
            min: self.min,
            max: self.max,
        }
    }
}
