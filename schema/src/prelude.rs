#![allow(dead_code)]

use crate::{ShapeId, SchemaRef, SmithyTrait, StaticTraitId, Schema, traits};
use std::sync::LazyLock;
use crate::lazy_shape_id;
use crate::documents::{DocumentValue, NumberInteger, NumberValue};
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

// =============================
// Prelude Traits
// =============================
macro_rules! static_id {
    ($trait_struct:ident, $id_var:ident, $id_name:literal) => {
        lazy_shape_id!($id_var, $id_name);
        impl StaticTraitId for $trait_struct {
            fn trait_id() -> &'static ShapeId {
                &$id_var
            }
        }
    };
}

macro_rules! annotation_trait {
    ($trait_struct:ident, $id_var:ident, $id_name:literal) => {
        pub struct $trait_struct {}
        impl $trait_struct {
            pub fn new() -> Self {
                Self {}
            }
        }
        static_id!($trait_struct, $id_var, $id_name);
        impl SmithyTrait for $trait_struct {
            fn id(&self) -> &ShapeId {
                &$id_var
            }

            fn value(&self) -> &DocumentValue {
                &DocumentValue::Null
            }
        }
    };
}

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
pub struct DefaultTrait(DocumentValue);
static_id!(DefaultTrait, DEFAULT_TRAIT_ID, "smithy.api#default");
impl SmithyTrait for DefaultTrait {
    fn id(&self) -> &ShapeId {
        DefaultTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.0
    }
}

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
static_id!(ErrorTrait, ERROR_TRAIT_ID, "smithy.api#error");
impl SmithyTrait for ErrorTrait {
    fn id(&self) -> &ShapeId {
        ErrorTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

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
static_id!(MediaTypeTrait, MEDIA_TYPE_TRAIT_ID, "smithy.api#mediaType");
impl SmithyTrait for MediaTypeTrait {
    fn id(&self) -> &ShapeId {
        MediaTypeTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

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
static_id!(JsonNameTrait, JSON_NAME_TRAIT_ID, "smithy.api#jsonName");
impl SmithyTrait for JsonNameTrait {
    fn id(&self) -> &ShapeId {
        JsonNameTrait::trait_id()
    }

    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

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
static_id!(HTTPErrorTrait, HTTP_ERROR_TRAIT_ID, "smithy.api#httpError");
impl SmithyTrait for HTTPErrorTrait {
    fn id(&self) -> &ShapeId {
        HTTPErrorTrait::trait_id()
    }
    fn value(&self) -> &DocumentValue {
        &self.value
    }
}

struct HTTPHeaderTrait {
    pub name: String,
    value: DocumentValue,
}
static_id!(
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

struct HTTPPrefixHeadersTrait {
    pub prefix: String,
    value: DocumentValue,
}
static_id!(
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

struct HTTPQueryTrait {
    pub key: String,
    value: DocumentValue,
}
static_id!(HTTPQueryTrait, HTTP_QUERY_TRAIT_ID, "smithy.api#httpQuery");
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

pub struct EndpointTrait {
    pub host_prefix: String,
    value: DocumentValue,
}
static_id!(EndpointTrait, ENDPOINT_TRAIT_ID, "smithy.api#endpoint");
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
