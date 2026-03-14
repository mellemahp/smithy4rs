use crate::{
    BigDecimal,
    IndexMap,
    derive::{
        SmithyShape,
        SmithyTraitImpl,
        smithy_enum,
    },
    doc_map,
    schema::{
        Document,
        DynamicTrait,
        RegexWrapper,
    },
    smithy,
};

smithy!("smithy.api#addedDefault": {
    /// Schema for [`AddedDefaultTrait`]
    structure ADDED_DEFAULT_SCHEMA {
    }
});

/// Indicates that the default trait was added to a member.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(ADDED_DEFAULT_SCHEMA)]
pub struct AddedDefaultTrait {
}

smithy!("smithy.api#TraitShapeIdList": {
    #[doc(hidden)]
    list TRAIT_SHAPE_ID_LIST {
        member: TRAIT_SHAPE_ID
    }
});

smithy!("smithy.api#authDefinition": {
    /// Schema for [`AuthDefinitionTrait`]
    structure AUTH_DEFINITION_SCHEMA {
        TRAITS: TRAIT_SHAPE_ID_LIST = "traits"
    }
});

/// Marks a trait as an auth scheme defining trait.
///
/// The targeted trait must only be applied to service shapes or operation
/// shapes, must be a structure, and must have the `trait` trait.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(AUTH_DEFINITION_SCHEMA)]
pub struct AuthDefinitionTrait {
    /// The list of traits that auth implementations must understand in order
    /// to successfully use the scheme.
    #[smithy_schema(TRAITS)]
    pub traits: Option<Vec<String>>,
}

smithy!("smithy.api#auth": {
    /// Defines the ordered list of supported authentication schemes.
    @UniqueItemsTrait::builder().build();
    list AUTH {
        member: AUTH_TRAIT_REFERENCE
    }
});
/// Defines the ordered list of supported authentication schemes.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(AUTH)]
#[repr(transparent)]
pub struct AuthTrait(Vec<String>);

smithy!("smithy.api#box": {
    /// Schema for [`BoxTrait`]
    structure BOX_SCHEMA {
    }
});

/// Used only in Smithy 1.0 to indicate that a shape is boxed.
///
/// This trait cannot be used in Smithy 2.0 models. When a boxed shape is the
/// target of a member, the member may or may not contain a value, and the
/// member has no default value.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(BOX_SCHEMA)]
pub struct BoxTrait {
}

smithy!("smithy.api#clientOptional": {
    /// Schema for [`ClientOptionalTrait`]
    structure CLIENT_OPTIONAL_SCHEMA {
    }
});

/// Requires that non-authoritative generators like clients treat a structure
/// member as nullable regardless of if the member is also marked with the
/// required trait.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(CLIENT_OPTIONAL_SCHEMA)]
pub struct ClientOptionalTrait {
}

smithy!("smithy.api#NonEmptyStringList": {
    #[doc(hidden)]
    list NON_EMPTY_STRING_LIST {
        member: NON_EMPTY_STRING
    }
});

smithy!("smithy.api#cors": {
    /// Schema for [`CorsTrait`]
    structure CORS_SCHEMA {
        ORIGIN: NON_EMPTY_STRING = "origin"
        MAX_AGE: INTEGER = "maxAge"
        ADDITIONAL_ALLOWED_HEADERS: NON_EMPTY_STRING_LIST = "additionalAllowedHeaders"
        ADDITIONAL_EXPOSED_HEADERS: NON_EMPTY_STRING_LIST = "additionalExposedHeaders"
    }
});

/// Defines how a service supports cross-origin resource sharing.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(CORS_SCHEMA)]
pub struct CorsTrait {
    /// The origin from which browser script-originating requests will be
    /// allowed.
    #[smithy_schema(ORIGIN)]
    pub origin: Option<String>,
    /// The maximum number of seconds for which browsers are allowed to cache
    /// the results of a preflight OPTIONS request.
    ///
    /// Defaults to 600, the maximum age permitted by several browsers.
    /// Set to -1 to disable caching entirely.
    #[smithy_schema(MAX_AGE)]
    pub max_age: Option<i32>,
    /// The names of headers that should be included in the
    /// Access-Control-Allow-Headers header in responses to preflight OPTIONS
    /// requests. This list will be used in addition to the names of all
    /// request headers bound to an input data member via the httpHeader, as
    /// well as any headers required by the protocol or authentication scheme.
    #[smithy_schema(ADDITIONAL_ALLOWED_HEADERS)]
    pub additional_allowed_headers: Option<Vec<String>>,
    /// The names of headers that should be included in the
    /// Access-Control-Expose-Headers header in all responses sent by the
    /// service. This list will be used in addition to the names of all
    /// request headers bound to an output data member via the httpHeader,
    /// as well as any headers required by the protocol or authentication
    /// scheme.
    #[smithy_schema(ADDITIONAL_EXPOSED_HEADERS)]
    pub additional_exposed_headers: Option<Vec<String>>,
}

smithy!("smithy.api#deprecated": {
    /// Schema for [`DeprecatedTrait`]
    structure DEPRECATED_SCHEMA {
        MESSAGE: STRING = "message"
        SINCE: STRING = "since"
    }
});

/// Marks a shape or member as deprecated.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(DEPRECATED_SCHEMA)]
pub struct DeprecatedTrait {
    /// The reason for deprecation.
    #[smithy_schema(MESSAGE)]
    pub message: Option<String>,
    /// A description of when the shape was deprecated (e.g., a date or
    /// version).
    #[smithy_schema(SINCE)]
    pub since: Option<String>,
}

smithy!("smithy.api#endpoint": {
    /// Schema for [`EndpointTrait`]
    structure ENDPOINT_SCHEMA {
        @RequiredTrait::builder().build();
        HOST_PREFIX: NON_EMPTY_STRING = "hostPrefix"
    }
});

/// Configures a custom operation endpoint.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(ENDPOINT_SCHEMA)]
pub struct EndpointTrait {
    /// A host prefix pattern for the operation.
    ///
    /// Labels defined in the host pattern are used to bind top-level
    /// operation input members to the host.
    #[smithy_schema(HOST_PREFIX)]
    pub host_prefix: String,
}

smithy!("smithy.api#EnumDefinition": {
    /// Schema for [`EnumDefinition`]
    structure ENUM_DEFINITION_SCHEMA {
        @RequiredTrait::builder().build();
        VALUE: NON_EMPTY_STRING = "value"
        NAME: ENUM_CONSTANT_BODY_NAME = "name"
        DOCUMENTATION: STRING = "documentation"
        TAGS: NON_EMPTY_STRING_LIST = "tags"
        DEPRECATED: BOOLEAN = "deprecated"
    }
});

/// An enum definition for the enum trait.
#[doc(hidden)]
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(ENUM_DEFINITION_SCHEMA)]
pub struct EnumDefinition {
    /// Defines the enum value that is sent over the wire.
    #[smithy_schema(VALUE)]
    pub value: String,
    /// Defines the name that is used in code to represent this variant.
    #[smithy_schema(NAME)]
    pub name: Option<String>,
    /// Provides optional documentation about the enum constant value.
    #[smithy_schema(DOCUMENTATION)]
    pub documentation: Option<String>,
    /// Applies a list of tags to the enum constant.
    #[smithy_schema(TAGS)]
    pub tags: Option<Vec<String>>,
    /// Whether the enum value should be considered deprecated.
    #[smithy_schema(DEPRECATED)]
    pub deprecated: Option<bool>,
}

smithy!("smithy.api#error": {
    /// Schema for [`ErrorTrait`]
    enum ERROR_SCHEMA {
        Client = "client"
        Server = "server"
    }
});

/// Indicates that a structure shape represents an error.
///
/// All shapes referenced by the errors list of an operation MUST be targeted
/// with this trait.
#[smithy_enum]
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(ERROR_SCHEMA)]
pub enum ErrorTrait {
    Client = "client",
    Server = "server",
}

smithy!("smithy.api#eventHeader": {
    /// Schema for [`EventHeaderTrait`]
    structure EVENT_HEADER_SCHEMA {
    }
});

/// Marks a member as a header of an event.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(EVENT_HEADER_SCHEMA)]
pub struct EventHeaderTrait {
}

smithy!("smithy.api#eventPayload": {
    /// Schema for [`EventPayloadTrait`]
    structure EVENT_PAYLOAD_SCHEMA {
    }
});

/// Marks a member as the payload of an event.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(EVENT_PAYLOAD_SCHEMA)]
pub struct EventPayloadTrait {
}

smithy!("smithy.api#externalDocumentation": {
    /// Provides a link to additional documentation.
    @LengthTrait::builder().min(1i64).build();
    map EXTERNAL_DOCUMENTATION {
        key: NON_EMPTY_STRING
        value: NON_EMPTY_STRING
    }
});
/// Provides a link to additional documentation.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(EXTERNAL_DOCUMENTATION)]
#[repr(transparent)]
pub struct ExternalDocumentationTrait(IndexMap<String, String>);

smithy!("smithy.api#hostLabel": {
    /// Schema for [`HostLabelTrait`]
    structure HOST_LABEL_SCHEMA {
    }
});

/// Binds a top-level operation input structure member to a label
/// in the hostPrefix of an endpoint trait.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(HOST_LABEL_SCHEMA)]
pub struct HostLabelTrait {
}

smithy!("smithy.api#HttpApiKeyLocations": {
    /// Schema for [`HttpApiKeyLocations`]
    enum HTTP_API_KEY_LOCATIONS_SCHEMA {
        Header = "header"
        Query = "query"
    }
});

#[doc(hidden)]
#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(HTTP_API_KEY_LOCATIONS_SCHEMA)]
pub enum HttpApiKeyLocations {
    Header = "header",
    Query = "query",
}

smithy!("smithy.api#httpBasicAuth": {
    /// Schema for [`HttpBasicAuthTrait`]
    @DynamicTrait::from("smithy.api#authDefinition", doc_map![]);
    structure HTTP_BASIC_AUTH_SCHEMA {
    }
});

/// HTTP Basic Authentication as defined in [RFC
/// 2617](https://tools.ietf.org/html/rfc2617.html).
/// ## References
/// - [**RFC 2617**]("https://tools.ietf.org/html/rfc2617.html")
///
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(HTTP_BASIC_AUTH_SCHEMA)]
pub struct HttpBasicAuthTrait {
}

smithy!("smithy.api#httpBearerAuth": {
    /// Schema for [`HttpBearerAuthTrait`]
    @DynamicTrait::from("smithy.api#authDefinition", doc_map![]);
    structure HTTP_BEARER_AUTH_SCHEMA {
    }
});

/// HTTP Bearer Authentication as defined in [RFC
/// 6750](https://tools.ietf.org/html/rfc6750.html).
/// ## References
/// - [**RFC 6750**]("https://tools.ietf.org/html/rfc6750.html")
///
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(HTTP_BEARER_AUTH_SCHEMA)]
pub struct HttpBearerAuthTrait {
}

smithy!("smithy.api#httpChecksumRequired": {
    /// Schema for [`HttpChecksumRequiredTrait`]
    structure HTTP_CHECKSUM_REQUIRED_SCHEMA {
    }
});

/// Marks an operation as requiring checksum in its HTTP request.
/// By default, the checksum used for a service is a MD5 checksum
/// passed in the Content-MD5 header.
/// <div class="warning">
///
/// **WARNING**: Unstable feature
///
/// </div>
///
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(HTTP_CHECKSUM_REQUIRED_SCHEMA)]
pub struct HttpChecksumRequiredTrait {
}

smithy!("smithy.api#httpDigestAuth": {
    /// Schema for [`HttpDigestAuthTrait`]
    @DynamicTrait::from("smithy.api#authDefinition", doc_map![]);
    structure HTTP_DIGEST_AUTH_SCHEMA {
    }
});

/// HTTP Digest Authentication as defined in [RFC
/// 2617](https://tools.ietf.org/html/rfc2617.html).
/// ## References
/// - [**RFC 2617**]("https://tools.ietf.org/html/rfc2617.html")
///
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(HTTP_DIGEST_AUTH_SCHEMA)]
pub struct HttpDigestAuthTrait {
}

smithy!("smithy.api#http": {
    /// Schema for [`HttpTrait`]
    structure HTTP_SCHEMA {
        @RequiredTrait::builder().build();
        METHOD: NON_EMPTY_STRING = "method"
        @RequiredTrait::builder().build();
        URI: NON_EMPTY_STRING = "uri"
        @RangeTrait::builder().min(100).max(999).build();
        CODE: INTEGER = "code"
    }
});

/// Configures the HTTP bindings of an operation.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(HTTP_SCHEMA)]
pub struct HttpTrait {
    /// The HTTP method of the operation.
    #[smithy_schema(METHOD)]
    pub method: String,
    /// The URI pattern of the operation.
    ///
    /// Labels defined in the URI pattern are used to bind operation input
    /// members to the URI.
    #[smithy_schema(URI)]
    pub uri: String,
    #[smithy_schema(CODE)]
    pub code: Option<i32>,
}

smithy!("smithy.api#httpLabel": {
    /// Schema for [`HttpLabelTrait`]
    structure HTTP_LABEL_SCHEMA {
    }
});

/// Binds an operation input structure member to an HTTP label.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(HTTP_LABEL_SCHEMA)]
pub struct HttpLabelTrait {
}

smithy!("smithy.api#httpPayload": {
    /// Schema for [`HttpPayloadTrait`]
    structure HTTP_PAYLOAD_SCHEMA {
    }
});

/// Binds a single structure member to the body of an HTTP request.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(HTTP_PAYLOAD_SCHEMA)]
pub struct HttpPayloadTrait {
}

smithy!("smithy.api#httpQueryParams": {
    /// Schema for [`HttpQueryParamsTrait`]
    structure HTTP_QUERY_PARAMS_SCHEMA {
    }
});

/// Binds an operation input structure member to the HTTP query string.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(HTTP_QUERY_PARAMS_SCHEMA)]
pub struct HttpQueryParamsTrait {
}

smithy!("smithy.api#httpResponseCode": {
    /// Schema for [`HttpResponseCodeTrait`]
    structure HTTP_RESPONSE_CODE_SCHEMA {
    }
});

/// Indicates that the structure member represents the HTTP response
/// status code. The value MAY differ from the HTTP status code provided
/// on the response.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(HTTP_RESPONSE_CODE_SCHEMA)]
pub struct HttpResponseCodeTrait {
}

smithy!("smithy.api#idempotencyToken": {
    /// Schema for [`IdempotencyTokenTrait`]
    @DynamicTrait::from("smithy.api#notProperty", doc_map![]);
    structure IDEMPOTENCY_TOKEN_SCHEMA {
    }
});

/// Defines the input member of an operation that is used by the server to
/// identify and discard replayed requests.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(IDEMPOTENCY_TOKEN_SCHEMA)]
pub struct IdempotencyTokenTrait {
}

smithy!("smithy.api#idempotent": {
    /// Schema for [`IdempotentTrait`]
    structure IDEMPOTENT_SCHEMA {
    }
});

/// Indicates that the intended effect on the server of multiple identical
/// requests with an operation is the same as the effect for a single such
/// request.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(IDEMPOTENT_SCHEMA)]
pub struct IdempotentTrait {
}

smithy!("smithy.api#idRef": {
    /// Schema for [`IdRefTrait`]
    structure ID_REF_SCHEMA {
        SELECTOR: STRING = "selector"
        FAIL_WHEN_MISSING: BOOLEAN = "failWhenMissing"
        ERROR_MESSAGE: STRING = "errorMessage"
    }
});

/// Indicates that a string value MUST contain a valid shape ID.
///
/// The provided shape ID MAY be absolute or relative to the shape to which
/// the trait is applied. A relative shape ID that does not resolve to a
/// shape defined in the same namespace resolves to a shape defined in the
/// prelude if the prelude shape is not marked with the private trait.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(ID_REF_SCHEMA)]
pub struct IdRefTrait {
    /// Defines the selector that the resolved shape, if found, MUST match.
    #[smithy_schema(SELECTOR)]
    pub selector: Option<String>,
    /// When set to `true`, the shape ID MUST target a shape that can be
    /// found in the model.
    #[smithy_schema(FAIL_WHEN_MISSING)]
    pub fail_when_missing: Option<bool>,
    /// Defines a custom error message to use when the shape ID cannot be
    /// found or does not match the selector.
    ///
    /// A default message is generated when errorMessage is not defined.
    #[smithy_schema(ERROR_MESSAGE)]
    pub error_message: Option<String>,
}

smithy!("smithy.api#input": {
    /// Schema for [`InputTrait`]
    structure INPUT_SCHEMA {
    }
});

/// Specializes a structure for use only as the input of a single operation.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(INPUT_SCHEMA)]
pub struct InputTrait {
}

smithy!("smithy.api#internal": {
    /// Schema for [`InternalTrait`]
    structure INTERNAL_SCHEMA {
    }
});

/// Shapes marked with the internal trait are meant only for internal use and
/// must not be exposed to customers.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(INTERNAL_SCHEMA)]
pub struct InternalTrait {
}

smithy!("smithy.api#length": {
    /// Schema for [`LengthTrait`]
    structure LENGTH_SCHEMA {
        MIN: LONG = "min"
        MAX: LONG = "max"
    }
});

/// Constrains a shape to minimum and maximum number of elements or size.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(LENGTH_SCHEMA)]
pub struct LengthTrait {
    /// Integer value that represents the minimum inclusive length of a shape.
    #[smithy_schema(MIN)]
    pub min: Option<i64>,
    /// Integer value that represents the maximum inclusive length of a shape.
    #[smithy_schema(MAX)]
    pub max: Option<i64>,
}

smithy!("smithy.api#LocalMixinTraitList": {
    #[doc(hidden)]
    list LOCAL_MIXIN_TRAIT_LIST {
        member: LOCAL_MIXIN_TRAIT
    }
});

smithy!("smithy.api#mixin": {
    /// Schema for [`MixinTrait`]
    structure MIXIN_SCHEMA {
        LOCAL_TRAITS: LOCAL_MIXIN_TRAIT_LIST = "localTraits"
    }
});

/// Makes a structure or union a mixin.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(MIXIN_SCHEMA)]
pub struct MixinTrait {
    #[smithy_schema(LOCAL_TRAITS)]
    pub local_traits: Option<Vec<String>>,
}

smithy!("smithy.api#nestedProperties": {
    /// Schema for [`NestedPropertiesTrait`]
    @DynamicTrait::from("smithy.api#notProperty", doc_map![]);
    structure NESTED_PROPERTIES_SCHEMA {
    }
});

/// Adjusts the resource property mapping of a lifecycle operation to the
/// targeted member.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(NESTED_PROPERTIES_SCHEMA)]
pub struct NestedPropertiesTrait {
}

smithy!("smithy.api#NonEmptyStringMap": {
    #[doc(hidden)]
    map NON_EMPTY_STRING_MAP {
        key: NON_EMPTY_STRING
        value: NON_EMPTY_STRING
    }
});

smithy!("smithy.api#noReplace": {
    /// Schema for [`NoReplaceTrait`]
    structure NO_REPLACE_SCHEMA {
    }
});

/// Indicates that the put lifecycle operation of a resource can only be used
/// to create a resource and cannot replace an existing resource.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(NO_REPLACE_SCHEMA)]
pub struct NoReplaceTrait {
}

smithy!("smithy.api#notProperty": {
    /// Schema for [`NotPropertyTrait`]
    @DynamicTrait::from("smithy.api#notProperty", doc_map![]);
    structure NOT_PROPERTY_SCHEMA {
    }
});

/// Explicitly excludes a member from resource property mapping or enables
/// another trait to carry the same implied meaning.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(NOT_PROPERTY_SCHEMA)]
pub struct NotPropertyTrait {
}

smithy!("smithy.api#optionalAuth": {
    /// Schema for [`OptionalAuthTrait`]
    structure OPTIONAL_AUTH_SCHEMA {
    }
});

/// Indicates that an operation can be called without authentication.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(OPTIONAL_AUTH_SCHEMA)]
pub struct OptionalAuthTrait {
}

smithy!("smithy.api#output": {
    /// Schema for [`OutputTrait`]
    structure OUTPUT_SCHEMA {
    }
});

/// Specializes a structure for use only as the output of a single operation.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(OUTPUT_SCHEMA)]
pub struct OutputTrait {
}

smithy!("smithy.api#paginated": {
    /// Schema for [`PaginatedTrait`]
    structure PAGINATED_SCHEMA {
        INPUT_TOKEN: NON_EMPTY_STRING = "inputToken"
        OUTPUT_TOKEN: NON_EMPTY_STRING = "outputToken"
        ITEMS: NON_EMPTY_STRING = "items"
        PAGE_SIZE: NON_EMPTY_STRING = "pageSize"
    }
});

/// The paginated trait indicates that an operation intentionally limits the
/// number of results returned in a single response and that multiple
/// invocations might be necessary to retrieve all results.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(PAGINATED_SCHEMA)]
pub struct PaginatedTrait {
    /// The name of the operation input member that represents the continuation
    /// token.
    ///
    /// When this value is provided as operation input, the service returns
    /// results from where the previous response left off. This input member
    /// MUST NOT be required and MUST target a string shape.
    #[smithy_schema(INPUT_TOKEN)]
    pub input_token: Option<String>,
    /// The name of the operation output member that represents the
    /// continuation token.
    ///
    /// When this value is present in operation output, it indicates that there
    /// are more results to retrieve. To get the next page of results, the
    /// client uses the output token as the input token of the next request.
    /// This output member MUST NOT be required and MUST target a string shape.
    #[smithy_schema(OUTPUT_TOKEN)]
    pub output_token: Option<String>,
    /// The name of a top-level output member of the operation that is the data
    /// that is being paginated across many responses.
    ///
    /// The named output member, if specified, MUST target a list or map.
    #[smithy_schema(ITEMS)]
    pub items: Option<String>,
    /// The name of an operation input member that limits the maximum number of
    /// results to include in the operation output. This input member MUST NOT
    /// be required and MUST target an integer shape.
    #[smithy_schema(PAGE_SIZE)]
    pub page_size: Option<String>,
}

smithy!("smithy.api#private": {
    /// Schema for [`PrivateTrait`]
    structure PRIVATE_SCHEMA {
    }
});

/// Prevents models defined in a different namespace from referencing the
/// targeted shape.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(PRIVATE_SCHEMA)]
pub struct PrivateTrait {
}

smithy!("smithy.api#property": {
    /// Schema for [`PropertyTrait`]
    structure PROPERTY_SCHEMA {
        NAME: STRING = "name"
    }
});

/// Configures a structure member's resource property mapping behavior.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(PROPERTY_SCHEMA)]
pub struct PropertyTrait {
    #[smithy_schema(NAME)]
    pub name: Option<String>,
}

smithy!("smithy.api#protocolDefinition": {
    /// Schema for [`ProtocolDefinitionTrait`]
    structure PROTOCOL_DEFINITION_SCHEMA {
        TRAITS: TRAIT_SHAPE_ID_LIST = "traits"
        NO_INLINE_DOCUMENT_SUPPORT: BOOLEAN = "noInlineDocumentSupport"
    }
});

/// Marks a trait as a protocol defining trait.
///
/// The targeted trait must only be applied to service shapes, must be a
/// structure, and must have the `trait` trait.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(PROTOCOL_DEFINITION_SCHEMA)]
pub struct ProtocolDefinitionTrait {
    /// The list of traits that protocol implementations must understand in
    /// order to successfully use the protocol.
    #[smithy_schema(TRAITS)]
    pub traits: Option<Vec<String>>,
    /// Set to true if inline documents are not supported by this protocol.
    #[deprecated]
    #[smithy_schema(NO_INLINE_DOCUMENT_SUPPORT)]
    pub no_inline_document_support: Option<bool>,
}

smithy!("smithy.api#range": {
    /// Schema for [`RangeTrait`]
    structure RANGE_SCHEMA {
        MIN: BIG_DECIMAL = "min"
        MAX: BIG_DECIMAL = "max"
    }
});

/// Restricts allowed values of byte, short, integer, long, float, double,
/// bigDecimal, and bigInteger shapes within an acceptable lower and upper
/// bound.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(RANGE_SCHEMA)]
pub struct RangeTrait {
    /// Specifies the allowed inclusive minimum value.
    #[smithy_schema(MIN)]
    pub min: Option<BigDecimal>,
    /// Specifies the allowed inclusive maximum value.
    #[smithy_schema(MAX)]
    pub max: Option<BigDecimal>,
}

smithy!("smithy.api#readonly": {
    /// Schema for [`ReadonlyTrait`]
    structure READONLY_SCHEMA {
    }
});

/// Indicates that an operation is effectively read-only.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(READONLY_SCHEMA)]
pub struct ReadonlyTrait {
}

smithy!("smithy.api#recommended": {
    /// Schema for [`RecommendedTrait`]
    structure RECOMMENDED_SCHEMA {
        REASON: STRING = "reason"
    }
});

/// Indicates that a structure member SHOULD be set.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(RECOMMENDED_SCHEMA)]
pub struct RecommendedTrait {
    /// Provides a reason why the member is recommended.
    #[smithy_schema(REASON)]
    pub reason: Option<String>,
}

smithy!("smithy.api#Reference": {
    /// Schema for [`Reference`]
    structure REFERENCE_SCHEMA {
        @RequiredTrait::builder().build();
        RESOURCE: NON_EMPTY_STRING = "resource"
        IDS: NON_EMPTY_STRING_MAP = "ids"
        SERVICE: NON_EMPTY_STRING = "service"
        REL: NON_EMPTY_STRING = "rel"
    }
});

#[doc(hidden)]
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(REFERENCE_SCHEMA)]
pub struct Reference {
    /// The shape ID of the referenced resource.
    #[smithy_schema(RESOURCE)]
    pub resource: String,
    /// Defines a mapping of each resource identifier name to a structure member
    /// name that provides its value. Each key in the map MUST refer to one of the
    /// identifier names in the identifiers property of the resource, and each
    /// value in the map MUST refer to a valid structure member name that targets
    /// a string shape.
    #[smithy_schema(IDS)]
    pub ids: Option<IndexMap<String, String>>,
    /// Providing a service makes the reference specific to a particular binding
    /// of the resource to a service. When omitted, the reference is late-bound to
    /// a service, meaning the reference is assumed to be a reference to the
    /// resource bound to the service currently in use by the client or server.
    #[smithy_schema(SERVICE)]
    pub service: Option<String>,
    /// Defines the semantics of the relationship. The rel property SHOULD
    /// contain a link relation as defined in RFC 5988#section-4.
    #[smithy_schema(REL)]
    pub rel: Option<String>,
}

smithy!("smithy.api#RequestCompressionEncodingsList": {
    /// Defines the priority-ordered list of compression algorithms supported by
    /// the service operation.
    #[doc(hidden)]
    list REQUEST_COMPRESSION_ENCODINGS_LIST {
        member: STRING
    }
});

smithy!("smithy.api#requestCompression": {
    /// Schema for [`RequestCompressionTrait`]
    structure REQUEST_COMPRESSION_SCHEMA {
        @RequiredTrait::builder().build();
        ENCODINGS: REQUEST_COMPRESSION_ENCODINGS_LIST = "encodings"
    }
});

/// Indicates that an operation supports compressing requests from clients to
/// services.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(REQUEST_COMPRESSION_SCHEMA)]
pub struct RequestCompressionTrait {
    #[smithy_schema(ENCODINGS)]
    pub encodings: Vec<String>,
}

smithy!("smithy.api#required": {
    /// Schema for [`RequiredTrait`]
    structure REQUIRED_SCHEMA {
    }
});

/// Marks a structure member as required, meaning a value for the member MUST
/// be present.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(REQUIRED_SCHEMA)]
pub struct RequiredTrait {
}

smithy!("smithy.api#requiresLength": {
    /// Schema for [`RequiresLengthTrait`]
    structure REQUIRES_LENGTH_SCHEMA {
    }
});

/// Indicates that the streaming blob must be finite and has a known size.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(REQUIRES_LENGTH_SCHEMA)]
pub struct RequiresLengthTrait {
}

smithy!("smithy.api#retryable": {
    /// Schema for [`RetryableTrait`]
    structure RETRYABLE_SCHEMA {
        THROTTLING: BOOLEAN = "throttling"
    }
});

/// Indicates that an error MAY be retried by the client.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(RETRYABLE_SCHEMA)]
pub struct RetryableTrait {
    /// Classifies the retry as throttling.
    #[smithy_schema(THROTTLING)]
    pub throttling: Option<bool>,
}

smithy!("smithy.api#sensitive": {
    /// Schema for [`SensitiveTrait`]
    structure SENSITIVE_SCHEMA {
    }
});

/// Indicates that the data stored in the shape is sensitive and MUST be
/// handled with care.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(SENSITIVE_SCHEMA)]
pub struct SensitiveTrait {
}

smithy!("smithy.api#Severity": {
    /// Schema for [`Severity`]
    enum SEVERITY_SCHEMA {
        Note = "NOTE"
        Warning = "WARNING"
        Danger = "DANGER"
        Error = "ERROR"
    }
});

#[doc(hidden)]
#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(SEVERITY_SCHEMA)]
pub enum Severity {
    Note = "NOTE",
    Warning = "WARNING",
    Danger = "DANGER",
    Error = "ERROR",
}

smithy!("smithy.api#sparse": {
    /// Schema for [`SparseTrait`]
    structure SPARSE_SCHEMA {
    }
});

/// Marks a list or map as sparse.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(SPARSE_SCHEMA)]
pub struct SparseTrait {
}

smithy!("smithy.api#streaming": {
    /// Schema for [`StreamingTrait`]
    structure STREAMING_SCHEMA {
    }
});

/// Indicates that the data stored in the shape is very large and should not
/// be stored in memory, or that the size of the data stored in the shape is
/// unknown at the start of a request. If the target is a union then the shape
/// represents a stream of events.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(STREAMING_SCHEMA)]
pub struct StreamingTrait {
}

smithy!("smithy.api#StructurallyExclusive": {
    /// Schema for [`StructurallyExclusive`]
    enum STRUCTURALLY_EXCLUSIVE_SCHEMA {
        Member = "member"
        Target = "target"
    }
});

#[doc(hidden)]
#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(STRUCTURALLY_EXCLUSIVE_SCHEMA)]
pub enum StructurallyExclusive {
    Member = "member",
    Target = "target",
}

smithy!("smithy.api#suppress": {
    /// Suppresses validation events by ID for a given shape.
    list SUPPRESS {
        @LengthTrait::builder().min(1i64).build();
        member: STRING
    }
});
/// Suppresses validation events by ID for a given shape.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(SUPPRESS)]
#[repr(transparent)]
pub struct SuppressTrait(Vec<String>);

smithy!("smithy.api#tags": {
    /// Tags a shape with arbitrary tag names that can be used to filter and
    /// group shapes in the model.
    list TAGS {
        member: STRING
    }
});
/// Tags a shape with arbitrary tag names that can be used to filter and
/// group shapes in the model.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(TAGS)]
#[repr(transparent)]
pub struct TagsTrait(Vec<String>);

smithy!("smithy.api#timestampFormat": {
    /// Schema for [`TimestampFormatTrait`]
    enum TIMESTAMP_FORMAT_SCHEMA {
        DateTime = "date-time"
        EpochSeconds = "epoch-seconds"
        HttpDate = "http-date"
    }
});

#[smithy_enum]
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(TIMESTAMP_FORMAT_SCHEMA)]
pub enum TimestampFormatTrait {
    DateTime = "date-time",
    EpochSeconds = "epoch-seconds",
    HttpDate = "http-date",
}

smithy!("smithy.api#TraitChangeType": {
    /// Schema for [`TraitChangeType`]
    enum TRAIT_CHANGE_TYPE_SCHEMA {
        Update = "update"
        Add = "add"
        Remove = "remove"
        Presence = "presence"
        Any = "any"
    }
});

#[doc(hidden)]
#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(TRAIT_CHANGE_TYPE_SCHEMA)]
pub enum TraitChangeType {
    Update = "update",
    Add = "add",
    Remove = "remove",
    Presence = "presence",
    Any = "any",
}

smithy!("smithy.api#TraitDiffRule": {
    /// Schema for [`TraitDiffRule`]
    structure TRAIT_DIFF_RULE_SCHEMA {
        PATH: STRING = "path"
        @RequiredTrait::builder().build();
        CHANGE: TRAIT_CHANGE_TYPE_SCHEMA = "change"
        SEVERITY: SEVERITY_SCHEMA = "severity"
        MESSAGE: STRING = "message"
    }
});

#[doc(hidden)]
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(TRAIT_DIFF_RULE_SCHEMA)]
pub struct TraitDiffRule {
    /// Defines a JSON Pointer to the value to evaluate.
    #[smithy_schema(PATH)]
    pub path: Option<String>,
    /// Defines the type of change that is not allowed.
    #[no_builder]
    #[smithy_schema(CHANGE)]
    pub change: TraitChangeType,
    /// Defines the severity of the change. Defaults to ERROR if not defined.
    #[no_builder]
    #[smithy_schema(SEVERITY)]
    pub severity: Option<Severity>,
    /// Provides a reason why the change is potentially backward incompatible.
    #[smithy_schema(MESSAGE)]
    pub message: Option<String>,
}

smithy!("smithy.api#TraitDiffRules": {
    #[doc(hidden)]
    @LengthTrait::builder().min(1i64).build();
    list TRAIT_DIFF_RULES {
        member: TRAIT_DIFF_RULE_SCHEMA
    }
});

smithy!("smithy.api#trait": {
    /// Schema for [`TraitTrait`]
    structure TRAIT_SCHEMA {
        SELECTOR: STRING = "selector"
        STRUCTURALLY_EXCLUSIVE: STRUCTURALLY_EXCLUSIVE_SCHEMA = "structurallyExclusive"
        CONFLICTS: NON_EMPTY_STRING_LIST = "conflicts"
        BREAKING_CHANGES: TRAIT_DIFF_RULES = "breakingChanges"
    }
});

/// Makes a shape a trait.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(TRAIT_SCHEMA)]
pub struct TraitTrait {
    /// The valid places in a model that the trait can be applied.
    #[smithy_schema(SELECTOR)]
    pub selector: Option<String>,
    /// Whether or not only a single member in a shape can have this trait.
    /// This only has an effect on members of structure shapes.
    #[no_builder]
    #[smithy_schema(STRUCTURALLY_EXCLUSIVE)]
    pub structurally_exclusive: Option<StructurallyExclusive>,
    /// The traits that this trait conflicts with.
    #[smithy_schema(CONFLICTS)]
    pub conflicts: Option<Vec<String>>,
    /// Defines the backward compatibility rules of the trait.
    #[smithy_schema(BREAKING_CHANGES)]
    pub breaking_changes: Option<Vec<TraitDiffRule>>,
}

smithy!("smithy.api#TraitValidator": {
    /// Schema for [`TraitValidator`]
    structure TRAIT_VALIDATOR_SCHEMA {
        @RequiredTrait::builder().build();
        SELECTOR: STRING = "selector"
        MESSAGE: STRING = "message"
        SEVERITY: SEVERITY_SCHEMA = "severity"
    }
});

#[doc(hidden)]
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(TRAIT_VALIDATOR_SCHEMA)]
pub struct TraitValidator {
    /// A Smithy selector that receives only the shape to which the `traitValidators` trait is
    /// applied.
    /// Any shape yielded by the selector is considered incompatible with the trait.
    #[smithy_schema(SELECTOR)]
    pub selector: String,
    /// A message to use when a matching shape is found.
    #[smithy_schema(MESSAGE)]
    pub message: Option<String>,
    /// The severity to use when a matching shape is found.
    #[no_builder]
    #[smithy_schema(SEVERITY)]
    pub severity: Option<Severity>,
}

smithy!("smithy.api#uniqueItems": {
    /// Schema for [`UniqueItemsTrait`]
    structure UNIQUE_ITEMS_SCHEMA {
    }
});

/// Indicates that the items in a list MUST be unique.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(UNIQUE_ITEMS_SCHEMA)]
pub struct UniqueItemsTrait {
}

smithy!("smithy.api#Unit": {
    /// Schema for [`Unit`]
    @DynamicTrait::from("smithy.api#unitType", doc_map![]);
    structure UNIT_SCHEMA {
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(UNIT_SCHEMA)]
pub struct Unit {
}

smithy!("smithy.api#unitType": {
    /// Schema for [`UnitTypeTrait`]
    structure UNIT_TYPE_SCHEMA {
    }
});

/// Specializes a structure as a unit type that has no meaningful value.
/// This trait can only be applied to smithy.api#Unit, which ensures that
/// only a single Unit shape can be created.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(UNIT_TYPE_SCHEMA)]
pub struct UnitTypeTrait {
}

smithy!("smithy.api#unstable": {
    /// Schema for [`UnstableTrait`]
    structure UNSTABLE_SCHEMA {
    }
});

/// Indicates that the shape is unstable and could change in the future.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(UNSTABLE_SCHEMA)]
pub struct UnstableTrait {
}

smithy!("smithy.api#xmlAttribute": {
    /// Schema for [`XmlAttributeTrait`]
    structure XML_ATTRIBUTE_SCHEMA {
    }
});

/// Serializes an object property as an XML attribute rather than a nested XML
/// element.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(XML_ATTRIBUTE_SCHEMA)]
pub struct XmlAttributeTrait {
}

smithy!("smithy.api#xmlFlattened": {
    /// Schema for [`XmlFlattenedTrait`]
    structure XML_FLATTENED_SCHEMA {
    }
});

/// Unwraps the values of a list, set, or map into the containing
/// structure/union.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(XML_FLATTENED_SCHEMA)]
pub struct XmlFlattenedTrait {
}

smithy!("smithy.api#xmlNamespace": {
    /// Schema for [`XmlNamespaceTrait`]
    structure XML_NAMESPACE_SCHEMA {
        @RequiredTrait::builder().build();
        URI: NON_EMPTY_STRING = "uri"
        @PatternTrait::new("^[a-zA-Z_][a-zA-Z_0-9-]*$");
        PREFIX: NON_EMPTY_STRING = "prefix"
    }
});

/// Adds an xmlns namespace definition URI to an XML element.
#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(XML_NAMESPACE_SCHEMA)]
pub struct XmlNamespaceTrait {
    /// The namespace URI for scoping this XML element.
    #[smithy_schema(URI)]
    pub uri: String,
    /// The prefix for the given namespace.
    #[smithy_schema(PREFIX)]
    pub prefix: Option<String>,
}

/// Changes the serialized element or attribute name of a structure, union,
/// or member.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(XML_NAME)]
#[repr(transparent)]
pub struct XmlNameTrait(String);

smithy!("smithy.api#xmlName": {
    /// Changes the serialized element or attribute name of a structure, union,
    /// or member.
    @PatternTrait::new("^[a-zA-Z_][a-zA-Z_0-9-]*(:[a-zA-Z_][a-zA-Z_0-9-]*)?$");
    string XML_NAME
});

smithy!("smithy.api#String": {
    string STRING
});

smithy!("smithy.api#AuthTraitReference": {
    /// A string that must target an auth trait.
    #[doc(hidden)]
    @DynamicTrait::from("smithy.api#idRef", doc_map!["selector" => "[trait|authDefinition]"]);
    string AUTH_TRAIT_REFERENCE
});

smithy!("smithy.api#Double": {
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    double DOUBLE
});

smithy!("smithy.api#PrimitiveByte": {
    byte PRIMITIVE_BYTE
});

smithy!("smithy.api#PrimitiveDouble": {
    double PRIMITIVE_DOUBLE
});

smithy!("smithy.api#PrimitiveBoolean": {
    boolean PRIMITIVE_BOOLEAN
});

/// Describes the contents of a blob shape using a media type as defined by
/// RFC 6838 (e.g., "video/quicktime").
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(MEDIA_TYPE)]
#[repr(transparent)]
pub struct MediaTypeTrait(String);

smithy!("smithy.api#mediaType": {
    /// Describes the contents of a blob shape using a media type as defined by
    /// RFC 6838 (e.g., "video/quicktime").
    string MEDIA_TYPE
});

smithy!("smithy.api#PrimitiveInteger": {
    integer PRIMITIVE_INTEGER
});

/// Defines an HTTP response code for an operation error.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(HTTP_ERROR)]
#[repr(transparent)]
pub struct HttpErrorTrait(i32);

smithy!("smithy.api#httpError": {
    /// Defines an HTTP response code for an operation error.
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    integer HTTP_ERROR
});

smithy!("smithy.api#Blob": {
    blob BLOB
});

smithy!("smithy.api#PrimitiveFloat": {
    float PRIMITIVE_FLOAT
});

smithy!("smithy.api#PrimitiveShort": {
    short PRIMITIVE_SHORT
});

smithy!("smithy.api#Integer": {
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    integer INTEGER
});

/// Defines the version or date in which a shape or member was added to the
/// model.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(SINCE)]
#[repr(transparent)]
pub struct SinceTrait(String);

smithy!("smithy.api#since": {
    /// Defines the version or date in which a shape or member was added to the
    /// model.
    string SINCE
});

smithy!("smithy.api#Byte": {
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    byte BYTE
});

/// Binds a structure member to an HTTP header.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(HTTP_HEADER)]
#[repr(transparent)]
pub struct HttpHeaderTrait(String);

smithy!("smithy.api#httpHeader": {
    /// Binds a structure member to an HTTP header.
    @LengthTrait::builder().min(1i64).build();
    string HTTP_HEADER
});

smithy!("smithy.api#Float": {
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    float FLOAT
});

/// Provides a structure member with a default value. When added to root
/// level shapes, requires that every targeting structure member defines the
/// same default value on the member or sets a default of null.
///
/// This trait can currently only be used in Smithy 2.0 models.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(DEFAULT)]
#[repr(transparent)]
pub struct DefaultTrait(Box<dyn Document>);

smithy!("smithy.api#default": {
    /// Provides a structure member with a default value. When added to root
    /// level shapes, requires that every targeting structure member defines the
    /// same default value on the member or sets a default of null.
    ///
    /// This trait can currently only be used in Smithy 2.0 models.
    document DEFAULT
});

smithy!("smithy.api#BigInteger": {
    bigInteger BIG_INTEGER
});

/// Indicates that the targeted structure member provides an identifier for
/// a resource.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(RESOURCE_IDENTIFIER)]
#[repr(transparent)]
pub struct ResourceIdentifierTrait(String);

smithy!("smithy.api#resourceIdentifier": {
    /// Indicates that the targeted structure member provides an identifier for
    /// a resource.
    @DynamicTrait::from("smithy.api#notProperty", doc_map![]);
    @LengthTrait::builder().min(1i64).build();
    string RESOURCE_IDENTIFIER
});

smithy!("smithy.api#Short": {
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    short SHORT
});

smithy!("smithy.api#LocalMixinTrait": {
    #[doc(hidden)]
    @DynamicTrait::from("smithy.api#idRef", doc_map!["selector" => "[trait|trait]", "failWhenMissing" => true, "errorMessage" => "Strings provided to the localTraits property of a mixin trait\nmust target a valid trait."]);
    string LOCAL_MIXIN_TRAIT
});

smithy!("smithy.api#EnumConstantBodyName": {
    /// The optional name or label of the enum constant value.
    ///
    /// This property is used in code generation to provide a label for
    /// each enum value. No two enums can have the same 'name' value.
    #[doc(hidden)]
    @PatternTrait::new("^[a-zA-Z_]+[a-zA-Z_0-9]*$");
    string ENUM_CONSTANT_BODY_NAME
});

smithy!("smithy.api#TraitShapeId": {
    #[doc(hidden)]
    @DynamicTrait::from("smithy.api#idRef", doc_map!["failWhenMissing" => true, "selector" => "[trait|trait]"]);
    string TRAIT_SHAPE_ID
});

smithy!("smithy.api#BigDecimal": {
    bigDecimal BIG_DECIMAL
});

/// Binds a map of key-value pairs to prefixed HTTP headers.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(HTTP_PREFIX_HEADERS)]
#[repr(transparent)]
pub struct HttpPrefixHeadersTrait(String);

smithy!("smithy.api#httpPrefixHeaders": {
    /// Binds a map of key-value pairs to prefixed HTTP headers.
    string HTTP_PREFIX_HEADERS
});

/// Adds documentation to a shape or member using CommonMark syntax.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(DOCUMENTATION)]
#[repr(transparent)]
pub struct DocumentationTrait(String);

smithy!("smithy.api#documentation": {
    /// Adds documentation to a shape or member using CommonMark syntax.
    string DOCUMENTATION
});

smithy!("smithy.api#NonEmptyString": {
    #[doc(hidden)]
    @LengthTrait::builder().min(1i64).build();
    string NON_EMPTY_STRING
});

smithy!("smithy.api#Long": {
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    long LONG
});

smithy!("smithy.api#Boolean": {
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    boolean BOOLEAN
});

smithy!("smithy.api#Timestamp": {
    timestamp TIMESTAMP
});

/// Defines a proper name for a shape.
///
/// This title can be used in automatically generated documentation
/// and other contexts to provide a user-friendly for shapes.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(TITLE)]
#[repr(transparent)]
pub struct TitleTrait(String);

smithy!("smithy.api#title": {
    /// Defines a proper name for a shape.
    ///
    /// This title can be used in automatically generated documentation
    /// and other contexts to provide a user-friendly for shapes.
    string TITLE
});

/// Allows a serialized object property name to differ from a structure member
/// name used in the model.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(JSON_NAME)]
#[repr(transparent)]
pub struct JsonNameTrait(String);

smithy!("smithy.api#jsonName": {
    /// Allows a serialized object property name to differ from a structure member
    /// name used in the model.
    string JSON_NAME
});

/// Restricts string shape values to a specified regular expression.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(PATTERN)]
#[repr(transparent)]
pub struct PatternTrait(RegexWrapper);

smithy!("smithy.api#pattern": {
    /// Restricts string shape values to a specified regular expression.
    string PATTERN
});

smithy!("smithy.api#Document": {
    document DOCUMENT
});

smithy!("smithy.api#PrimitiveLong": {
    long PRIMITIVE_LONG
});

/// Binds an operation input structure member to a query string parameter.
#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(HTTP_QUERY)]
#[repr(transparent)]
pub struct HttpQueryTrait(String);

smithy!("smithy.api#httpQuery": {
    /// Binds an operation input structure member to a query string parameter.
    @LengthTrait::builder().min(1i64).build();
    string HTTP_QUERY
});
