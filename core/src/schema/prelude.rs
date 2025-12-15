#![allow(dead_code)]

use std::fmt::Display;

use indexmap::IndexMap;
use regex::Regex;

use crate::{
    annotation_trait,
    schema::{DocumentValue, NumberInteger, NumberValue, ShapeId, SmithyTrait, StaticTraitId},
    smithy, static_trait_id, string_trait,
};

////////////////////////////////////////////////////////////////////////////////////
// Prelude Shape Schemas
////////////////////////////////////////////////////////////////////////////////////

smithy!("smithy.api#Blob": { blob BLOB });
smithy!("smithy.api#Boolean": { boolean BOOLEAN });
smithy!("smithy.api#String": { string STRING });
smithy!("smithy.api#Timestamp": { timestamp TIMESTAMP });
smithy!("smithy.api#Byte": { byte BYTE });
smithy!("smithy.api#Short": { short SHORT });
smithy!("smithy.api#Integer": { integer INTEGER });
smithy!("smithy.api#Long": { long LONG });
smithy!("smithy.api#Float": { float FLOAT });
smithy!("smithy.api#Double": { double DOUBLE });
smithy!("smithy.api#BigInteger": { bigInteger BIG_INTEGER });
smithy!("smithy.api#BigDecimal": { bigDecimal BIG_DECIMAL });
smithy!("smithy.api#Document": { document DOCUMENT });

// TODO:
// - Primitive types

///////////////////////////////////////////////////////////////////////
// Prelude Traits
///////////////////////////////////////////////////////////////////////

// ==== Annotation traits ====
annotation_trait!(SensitiveTrait, "smithy.api#sensitive");
annotation_trait!(StreamingTrait, "smithy.api#streaming");
annotation_trait!(SparseTrait, "smithy.api#sparse");
annotation_trait!(RequiredTrait, "smithy.api#required");
annotation_trait!(UnitTypeTrait, "smithy.api#UnitTypeTrait");
annotation_trait!(EventHeaderTrait, "smithy.api#eventheader");
annotation_trait!(EventPayloadTrait, "smithy.api#eventPayload");
annotation_trait!(IdempotencyTokenTrait, "smithy.api#IdempotencyToken");
annotation_trait!(HttpLabelTrait, "smithy.api#httpLabel");
annotation_trait!(HttpPayloadTrait, "smithy.api#httpPayload");
annotation_trait!(HTTPQueryParamsTrait, "smithy.api#httpQueryParams");
annotation_trait!(HTTPResponseCodeTrait, "smithy.api#httpResponseCode");
annotation_trait!(HTTPChecksumRequiredTrait, "smithy.api#httpChecksumRequired");
annotation_trait!(HostLabelTrait, "smithy.api#hostLabel");

// ====  Traits that take just a string value ====
string_trait!(MediaTypeTrait, media_type, "smithy.api#mediaType");
string_trait!(JsonNameTrait, name, "smithy.api#jsonName");
string_trait!(HTTPHeaderTrait, name, "smithy.api#httpHeader");
string_trait!(
    HTTPPrefixHeadersTrait,
    prefix,
    "smithy.api#httpPrefixHeaders"
);
string_trait!(HTTPQueryTrait, key, "smithy.api#httpQuery");
string_trait!(EndpointTrait, host_prefix, "smithy.api#endpoint");

// ==== Traits with other values ====

/// Provides a structure member with a default value.
///
/// *See* - [Default Trait](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-default-trait)
pub struct DefaultTrait(pub DocumentValue);
static_trait_id!(DefaultTrait, "smithy.api#default");
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
static_trait_id!(ErrorTrait, "smithy.api#error");
smithy_trait_impl!(ErrorTrait);

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
static_trait_id!(HTTPErrorTrait, "smithy.api#httpError");
smithy_trait_impl!(HTTPErrorTrait);

/////////////////////////////////////////////////
// Constraint Traits
/////////////////////////////////////////////////
#[derive(Debug)]
pub struct RangeTrait {
    min: Option<usize>,
    max: Option<usize>,
    value: DocumentValue,
}
static_trait_id!(RangeTrait, "smithy.api#range");
smithy_trait_impl!(RangeTrait);

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
    pub(super) const fn new() -> Self {
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
static_trait_id!(LengthTrait, "smithy.api#length");
smithy_trait_impl!(LengthTrait);

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
    pub(super) const fn new() -> Self {
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

annotation_trait!(UniqueItemsTrait, "smithy.api#uniqueItems");

#[derive(Debug)]
pub struct PatternTrait {
    pattern: Regex,
    value: DocumentValue,
}
static_trait_id!(PatternTrait, "smithy.api#pattern");
smithy_trait_impl!(PatternTrait);

impl PatternTrait {
    pub fn pattern(&self) -> &Regex {
        &self.pattern
    }

    #[must_use]
    /// Create a new [`PatternTrait`]
    ///
    /// *NOTE*: Will panic if the pattern is invalid.
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

annotation_trait!(HttpBasicAuthTrait, "smithy.api#httpBasicAuth");
annotation_trait!(HttpDigestAuthTrait, "smithy.api#httpDigestAuth");
annotation_trait!(HttpBearerAuthTrait, "smithy.api#httpBearerAuth");

#[derive(Debug)]
pub struct HttpApiKeyAuthTrait {
    name: String,
    in_location: String,
    scheme: Option<String>,
    value: DocumentValue,
}
static_trait_id!(HttpApiKeyAuthTrait, "smithy.api#httpApiKeyAuth");
smithy_trait_impl!(HttpApiKeyAuthTrait);

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
