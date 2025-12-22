#![allow(dead_code)]
//! # Prelude
//! [`crate::schema::Schema`] definitions for the [Smithy prelude](https://github.com/smithy-lang/smithy/blob/65b473ddb94f9edda933f00bab988d465b2bd2fe/smithy-model/src/main/resources/software/amazon/smithy/model/loader/prelude.smithy)
//!
//! The prelude consists of public, built-in shapes like [`STRING`], [`INTEGER`], etc. that
//! are available to all models. Prelude shapes and traits are all in the `smithy.api` namespace
//! and must be hard-coded as they are used by generate shapes.

use std::fmt::Display;

use bigdecimal::Zero;
use regex::Regex;

use crate::{BigDecimal, IndexMap, LazyLock, annotation_trait, schema::{
    NumberFloat, NumberInteger, NumberValue, ShapeId, SmithyTrait, StaticTraitId,
}, smithy, static_trait_id, string_trait};
use crate::schema::{DefaultDocumentValue, DocumentImpl};
// ============================================================================
// Prelude Shape Schemas
// ---------------------
// These are the base shapes of the smithy data model.
// For more information see: https://smithy.io/2.0/spec/index.html
// ============================================================================

// === Simple types ===
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

// === Primitive types ===
smithy!("smithy.api#PrimitiveBoolean": {
    @DefaultTrait(DefaultDocumentValue::Boolean(false).into());
    boolean PRIMITIVE_BOOLEAN
});
smithy!("smithy.api#PrimitiveByte": {
    @DefaultTrait(DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::Byte(0i8))).into());
    byte PRIMITIVE_BYTE
});
smithy!("smithy.api#PrimitiveShort": {
    @DefaultTrait(DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::Short(0i16))).into());
    short PRIMITIVE_SHORT
});
smithy!("smithy.api#PrimitiveInteger": {
    @DefaultTrait(DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::Integer(0i32))).into());
    integer PRIMITIVE_INTEGER
});
smithy!("smithy.api#PrimitiveLong": {
    @DefaultTrait(DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::Long(0i64))).into());
    boolean PRIMITIVE_LONG
});
smithy!("smithy.api#PrimitiveFloat": {
    @DefaultTrait(DefaultDocumentValue::Number(NumberValue::Float(NumberFloat::Float(0f32))).into());
    float PRIMITIVE_FLOAT
});
smithy!("smithy.api#PrimitiveDouble": {
    @DefaultTrait(DefaultDocumentValue::Number(NumberValue::Float(NumberFloat::Double(0f64))).into());
    double PRIMITIVE_DOUBLE
});

// ============================================================================
// Unit Type
// ---------
// Unit types is used by unions and operations to indicate input/output
// or variants that have no meaningful value
// ============================================================================
annotation_trait!(UnitTypeTrait, "smithy.api#UnitTypeTrait");

smithy!("smithy.api#Unit": {
    @UnitTypeTrait;
    structure UNIT {}
});

// ============================================================================
// Prelude Traits
// ============================================================================

// ==== Annotation traits ====
annotation_trait!(SensitiveTrait, "smithy.api#sensitive");
annotation_trait!(StreamingTrait, "smithy.api#streaming");
annotation_trait!(SparseTrait, "smithy.api#sparse");
annotation_trait!(RequiredTrait, "smithy.api#required");
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
pub struct DefaultTrait(pub DocumentImpl);
static_trait_id!(DefaultTrait, "smithy.api#default");
impl SmithyTrait for DefaultTrait {
    fn id(&self) -> &ShapeId {
        DefaultTrait::trait_id()
    }

    fn value(&self) -> &DocumentImpl {
        &self.0
    }
}

macro_rules! smithy_trait_impl {
    ($t:ident) => {
        impl SmithyTrait for $t {
            fn id(&self) -> &ShapeId {
                $t::trait_id()
            }

            fn value(&self) -> &DocumentImpl {
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
    value: DocumentImpl,
}
impl ErrorTrait {
    /// Get whether the Error was the fault of the client or server.
    pub fn error(&self) -> &ErrorFault {
        &self.error
    }

    #[must_use]
    pub fn new(error: ErrorFault) -> Self {
        ErrorTrait {
            value: DefaultDocumentValue::String(error.to_string()).into(),
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
pub struct HttpErrorTrait {
    code: i32,
    value: DocumentImpl,
}
impl HttpErrorTrait {
    /// Get the code contained by this trait.
    pub fn code(&self) -> i32 {
        self.code
    }

    /// Create a new [`HttpErrorTrait`] instance
    ///
    /// ## Errors
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
        HttpErrorTrait {
            code,
            value: DefaultDocumentValue::Number(NumberValue::Integer(NumberInteger::Integer(code))).into(),
        }
    }
}
static_trait_id!(HttpErrorTrait, "smithy.api#httpError");
smithy_trait_impl!(HttpErrorTrait);

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
    value: DocumentImpl,
}
static_trait_id!(RangeTrait, "smithy.api#range");
smithy_trait_impl!(RangeTrait);

static ZERO: LazyLock<BigDecimal> = LazyLock::new(BigDecimal::zero);
static MAX: LazyLock<BigDecimal> = LazyLock::new(|| BigDecimal::from(u64::MAX));

impl RangeTrait {
    /// Get the minimum value allowed by this range constraint.
    ///
    /// Defaults to zero
    #[inline]
    pub fn min(&self) -> &BigDecimal {
        self.min.as_ref().unwrap_or_else(|| &ZERO)
    }

    /// Get the maximum value allowed by this range constraint.
    ///
    /// Defaults to `u64::MAX`
    #[inline]
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
    pub fn min(mut self, min: BigDecimal) -> Self {
        self.min = Some(min);
        self
    }

    /// Set a maximum value for this constraint.
    pub fn max(mut self, max: BigDecimal) -> Self {
        self.max = Some(max);
        self
    }

    /// Construct a new [`RangeTrait`] instance.
    pub fn build(self) -> RangeTrait {
        let mut value_map = IndexMap::new();
        if let Some(min) = &self.min {
            value_map.insert("min".to_string(), min.clone().into());
        }
        if let Some(max) = &self.max {
            value_map.insert("min".to_string(), max.clone().into());
        }
        RangeTrait {
            min: self.min,
            max: self.max,
            value: DefaultDocumentValue::Map(value_map).into(),
        }
    }
}

#[derive(Debug)]
pub struct LengthTrait {
    min: Option<usize>,
    max: Option<usize>,
    value: DocumentImpl,
}
static_trait_id!(LengthTrait, "smithy.api#length");
smithy_trait_impl!(LengthTrait);

impl LengthTrait {
    pub fn min(&self) -> usize {
        self.min.unwrap_or(0)
    }

    pub fn max(&self) -> usize {
        self.max.unwrap_or(usize::MAX)
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
            value: DefaultDocumentValue::Map(value_map).into(),
        }
    }
}

annotation_trait!(UniqueItemsTrait, "smithy.api#uniqueItems");

#[derive(Debug)]
pub struct PatternTrait {
    pattern: Regex,
    value: DocumentImpl,
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
    /// <div class ="warning">
    /// Will panic if the pattern is invalid.
    /// </div>
    pub fn new(pattern: &str) -> Self {
        PatternTrait {
            pattern: Regex::new(pattern).unwrap(),
            value: DefaultDocumentValue::String(pattern.to_string()).into(),
        }
    }
}

// ============================================================================
// Auth Traits
// ============================================================================

annotation_trait!(HttpBasicAuthTrait, "smithy.api#httpBasicAuth");
annotation_trait!(HttpDigestAuthTrait, "smithy.api#httpDigestAuth");
annotation_trait!(HttpBearerAuthTrait, "smithy.api#httpBearerAuth");

#[derive(Debug)]
pub struct HttpApiKeyAuthTrait {
    name: String,
    in_location: String,
    scheme: Option<String>,
    value: DocumentImpl,
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
            value: DefaultDocumentValue::Map(value_map).into(),
        }
    }
}
