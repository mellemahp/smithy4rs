#![allow(dead_code)]

use crate::schema::{ShapeId, SchemaRef, SmithyTrait, StaticTraitId, Schema};
use crate::schema::{DocumentValue, NumberInteger, NumberValue};
use crate::{lazy_shape_id, annotation_trait, static_trait_id, traits};
use std::sync::LazyLock;
use std::fmt::Display;

// =============================
// Prelude Shape Schemas
// =============================

/// Schema for Smithy [Blob](https://smithy.io/2.0/spec/simple-types.html#blob) Type
pub static BLOB: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_blob("smithy.api#Blob", traits![]));

/// Schema for Smithy [Boolean](https://smithy.io/2.0/spec/simple-types.html#boolean) Type
pub static BOOLEAN: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_boolean("smithy.api#Boolean", traits![]));

/// Schema for Smithy [String](https://smithy.io/2.0/spec/simple-types.html#string) Type
pub static STRING: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_string("smithy.api#String", traits![]));

/// Schema for Smithy [Timestamp](https://smithy.io/2.0/spec/simple-types.html#timestamp) type
pub static TIMESTAMP: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_timestamp("smithy.api#Timestamp", traits![]));

/// Schema for Smithy [Byte](https://smithy.io/2.0/spec/simple-types.html#byte) type
pub static BYTE: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_byte("smithy.api#Byte", traits![]));

/// Schema for Smithy [short](https://smithy.io/2.0/spec/simple-types.html#short) type
pub static SHORT: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_short("smithy.api#Short", traits![]));

/// Schema for Smithy [integer](https://smithy.io/2.0/spec/simple-types.html#integer) type
pub static INTEGER: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_integer("smithy.api#Integer", traits![]));

/// Schema for Smithy [long](https://smithy.io/2.0/spec/simple-types.html#long) type
pub static LONG: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_long("smithy.api#Long", traits![]));

/// Schema for Smithy [float](https://smithy.io/2.0/spec/simple-types.html#float) type
pub static FLOAT: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_float("smithy.api#Float", traits![]));

/// Schema for Smithy [double](https://smithy.io/2.0/spec/simple-types.html#double) type
pub static DOUBLE: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_double("smithy.api#Double", traits![]));

/// Schema for Smithy [bigInteger](https://smithy.io/2.0/spec/simple-types.html#biginteger) type
pub static BIG_INTEGER: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_big_integer("smithy.api#BigInteger", traits![]));

/// Schema for Smithy [bigDecimal](https://smithy.io/2.0/spec/simple-types.html#bigdecimal) type
pub static BIG_DECIMAL: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_big_decimal("smithy.api#BigDecimal", traits![]));

/// Schema for Smithy [document](https://smithy.io/2.0/spec/simple-types.html#document) type
pub static DOCUMENT: LazyLock<SchemaRef> =
    LazyLock::new(|| Schema::create_document("smithy.api#Document", traits![]));

// TODO:
// - Primitive types

///////////////////////////////////////////////////////////////////////
// Prelude Traits
///////////////////////////////////////////////////////////////////////

// ==== Annotation traits ====
/// Indicates that the data stored in the shape is sensitive and MUST be handled with care.
///
/// *See* - [Sensitive Trait](https://smithy.io/2.0/spec/documentation-traits.html#smithy-api-sensitive-trait)
annotation_trait!(SensitiveTrait, SENSITIVE_TRAIT_ID, "smithy.api#sensitive");

/// Indicates that the data represented by the shape needs to be streamed.
/// *Note*: Should only be applied to `Blob` or `Union` schemas.
/// *See* - [Streaming Trait](https://smithy.io/2.0/spec/streaming.html#streaming-trait)
annotation_trait!(StreamingTrait, STREAMING_TRAIT_ID, "smithy.api#streaming");

/// Indicates that lists and maps MAY contain null values.
/// *Note*: Should only be applied to `List` or `Map` schemas.
/// *See* - [Sparse Trait](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-sparse-trait)
annotation_trait!(SparseTrait, SPARSE_TRAIT_ID, "smithy.api#sparse");

/// Marks a structure member as required, meaning a value for the member MUST be present.
///
/// *See* - [Required Trait](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-required-trait)
annotation_trait!(RequiredTrait, REQUIRED_TRAIT_ID, "smithy.api#required");

/// Shapes marked with the internal trait are meant only for internal use.
///
/// *See* - [Internal Trait](https://smithy.io/2.0/spec/documentation-traits.html#smithy-api-internal-trait)
annotation_trait!(InternalTrait, INTERNAL_TRAIT_ID, "smithy.api#internal");

/// Marks a value an empty "Unit" shape.
///
/// Used for marker values in operations and Unions.
annotation_trait!(
    UnitTypeTrait,
    UNIT_TYPE_TRAIT_ID,
    "smithy.api#UnitTypeTrait"
);

/// Binds a member of a structure to be serialized as an event header when sent through an event stream.
///
/// *See* - [EventHeader Trait](https://smithy.io/2.0/spec/streaming.html#smithy-api-eventheader-trait)
annotation_trait!(
    EventHeaderTrait,
    EVENT_HEADER_TRAIT_ID,
    "smithy.api#eventheader"
);

/// Binds a member of a structure to be serialized as the payload of an event sent through an event stream.
///
/// *See* - [EventPayload Trait](https://smithy.io/2.0/spec/streaming.html#smithy-api-eventpayload-trait)
annotation_trait!(
    EventPayloadTrait,
    EVENT_PAYLOAD_TRAIT_ID,
    "smithy.api#eventPayload"
);

/// Defines the input member of an operation that is used by the server to identify and discard replayed requests.
///
/// *See* - [IdempotencyToken Trait](https://smithy.io/2.0/spec/behavior-traits.html#smithy-api-idempotencytoken-trait)
annotation_trait!(
    IdempotencyTokenTrait,
    IDEMPOTENCY_TOKEN_TRAIT_ID,
    "smithy.api#IdempotencyToken"
);

/// Binds an operation input structure member to an HTTP label.
///
/// *See* - [HttpLabel Trait](https://smithy.io/2.0/spec/http-bindings.html#smithy-api-httplabel-trait)
annotation_trait!(HttpLabelTrait, HTTP_LABEL_TRAIT_ID, "smithy.api#httpLabel");

/// Binds a single structure member to the body of an HTTP message.
///
/// *See* - [HttpPayload Trait](https://smithy.io/2.0/spec/http-bindings.html#httppayload-trait)
annotation_trait!(
    HttpPayloadTrait,
    HTTP_PAYLOAD_TRAIT_ID,
    "smithy.api#httpPayload"
);

/// Binds a map of key-value pairs to query string parameters.
///
/// *See* - [HttpQueryParams Trait](https://smithy.io/2.0/spec/http-bindings.html#httpqueryparams-trait)
annotation_trait!(
    HTTPQueryParamsTrait,
    HTTP_QUERY_PARAMS_TRAIT_ID,
    "smithy.api#httpQueryParams"
);

/// Binds a structure member to the HTTP response status code.
///
/// *See* - [HttpResponseCode Trait](https://smithy.io/2.0/spec/http-bindings.html#httpresponsecode-trait)
annotation_trait!(
    HTTPResponseCodeTrait,
    HTTP_RESPONSE_CODE_TRAIT_ID,
    "smithy.api#httpResponseCode"
);

/// Indicates that an operation requires a checksum in its HTTP request.
///
/// *See* - [HttpChecksumRequired Trait](https://smithy.io/2.0/spec/http-bindings.html#httpchecksumrequired-trait)
annotation_trait!(
    HTTPChecksumRequiredTrait,
    HTTP_CHECKSUM_REQUIRED_TRAIT_ID,
    "smithy.api#httpChecksumRequired"
);

/// Binds a top-level operation input structure member to a label in the hostPrefix of an endpoint trait.
///
/// *See* - [HostLabel Trait](https://smithy.io/2.0/spec/endpoint-traits.html#smithy-api-hostlabel-trait)
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

/// Indicates that a structure shape represents an error.
///
/// *See* - [Error Trait](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-error-trait)
pub struct ErrorTrait {
    pub error: ErrorFault,
    value: DocumentValue,
}
impl ErrorTrait {
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
        write!(f, "{}", str)
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
    pub fn new(media_type: &str) -> Self {
        MediaTypeTrait {
            media_type: media_type.to_string(),
            value: DocumentValue::String(media_type.to_string()),
        }
    }
}
static_trait_id!(MediaTypeTrait, MEDIA_TYPE_TRAIT_ID, "smithy.api#mediaType");
impl SmithyTrait for MediaTypeTrait {
    fn id(&self) -> &ShapeId {
        MediaTypeTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

/// Allows a serialized object property name in a JSON document to differ from a structure or union member name.
///
/// *See* - [JsonName Trait](https://smithy.io/2.0/spec/protocol-traits.html#smithy-api-jsonname-trait)
pub struct JsonNameTrait {
    pub name: String,
    value: DocumentValue,
}
impl JsonNameTrait {
    pub fn new(name: &str) -> Self {
        JsonNameTrait {
            name: name.to_string(),
            value: DocumentValue::String(name.to_string()),
        }
    }
}
static_trait_id!(JsonNameTrait, JSON_NAME_TRAIT_ID, "smithy.api#jsonName");
impl SmithyTrait for JsonNameTrait {
    fn id(&self) -> &ShapeId {
        JsonNameTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

/// Defines an HTTP response code for an operation error.
///
/// *See* - [HttpError Trait](https://smithy.io/2.0/spec/http-bindings.html#smithy-api-httperror-trait)
pub struct HTTPErrorTrait {
    pub code: i32,
    value: DocumentValue,
}
impl HTTPErrorTrait {
    pub fn new(code: i32) -> Self {
        if !(200 < code && code < 599) {
            panic!("HTTPErrorTrait code out of range: {}", code);
        }
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
impl HTTPHeaderTrait {
    pub fn new(name: &str) -> Self {
        HTTPHeaderTrait {
            name: name.to_string(),
            value: DocumentValue::String(name.to_string()),
        }
    }
}
impl SmithyTrait for HTTPHeaderTrait {
    fn id(&self) -> &ShapeId {
        HTTPHeaderTrait::trait_id()
    }
    fn value(&self) -> &DocumentValue {
        &self.value
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
impl HTTPPrefixHeadersTrait {
    pub fn new(prefix: &str) -> Self {
        HTTPPrefixHeadersTrait {
            prefix: prefix.to_string(),
            value: DocumentValue::String(prefix.to_string()),
        }
    }
}
impl SmithyTrait for HTTPPrefixHeadersTrait {
    fn id(&self) -> &ShapeId {
        HTTPPrefixHeadersTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.value
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
impl HTTPQueryTrait {
    pub fn new(key: &str) -> Self {
        HTTPQueryTrait {
            key: key.to_string(),
            value: DocumentValue::String(key.to_string()),
        }
    }
}
impl SmithyTrait for HTTPQueryTrait {
    fn id(&self) -> &ShapeId {
        HTTPQueryTrait::trait_id()
    }
    fn value(&self) -> &DocumentValue {
        &self.value
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
impl EndpointTrait {
    pub fn new(host_prefix: &str) -> Self {
        EndpointTrait {
            host_prefix: host_prefix.to_string(),
            value: DocumentValue::String(host_prefix.to_string()),
        }
    }
}
impl SmithyTrait for EndpointTrait {
    fn id(&self) -> &ShapeId {
        EndpointTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}
