use crate::{
    BigDecimal,
    IndexMap,
    derive::{
        SmithyShape,
        SmithyTrait,
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

smithy!(
    /// Schema for [`AddedDefaultTrait`]
    structure smithy::api::addedDefault {
    }
);

/// Indicates that the default trait was added to a member.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = ADDED_DEFAULT)]
pub struct AddedDefaultTrait {
}

smithy!(
    #[doc(hidden)]
    list smithy::api::TraitShapeIdList {
        member: TRAIT_SHAPE_ID
    }
);

smithy!(
    /// Schema for [`AuthDefinitionTrait`]
    structure smithy::api::authDefinition {
        traits: TRAIT_SHAPE_ID_LIST
    }
);

/// Marks a trait as an auth scheme defining trait.
///
/// The targeted trait must only be applied to service shapes or operation
/// shapes, must be a structure, and must have the `trait` trait.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = AUTH_DEFINITION)]
pub struct AuthDefinitionTrait {
    /// The list of traits that auth implementations must understand in order
    /// to successfully use the scheme.
    pub traits: Option<Vec<String>>,
}

smithy!(
    /// Defines the ordered list of supported authentication schemes.
    @UniqueItemsTrait::builder().build();
    list smithy::api::auth {
        member: AUTH_TRAIT_REFERENCE
    }
);
/// Defines the ordered list of supported authentication schemes.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = AUTH)]
#[repr(transparent)]
pub struct AuthTrait(Vec<String>);

smithy!(
    /// Schema for [`BoxTrait`]
    structure smithy::api::r#box {
    }
);

/// Used only in Smithy 1.0 to indicate that a shape is boxed.
///
/// This trait cannot be used in Smithy 2.0 models. When a boxed shape is the
/// target of a member, the member may or may not contain a value, and the
/// member has no default value.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = BOX)]
pub struct BoxTrait {
}

smithy!(
    /// Schema for [`ClientOptionalTrait`]
    structure smithy::api::clientOptional {
    }
);

/// Requires that non-authoritative generators like clients treat a structure
/// member as nullable regardless of if the member is also marked with the
/// required trait.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = CLIENT_OPTIONAL)]
pub struct ClientOptionalTrait {
}

smithy!(
    #[doc(hidden)]
    list smithy::api::NonEmptyStringList {
        member: NON_EMPTY_STRING
    }
);

smithy!(
    /// Schema for [`CorsTrait`]
    structure smithy::api::cors {
        origin: NON_EMPTY_STRING
        maxAge: INTEGER
        additionalAllowedHeaders: NON_EMPTY_STRING_LIST
        additionalExposedHeaders: NON_EMPTY_STRING_LIST
    }
);

/// Defines how a service supports cross-origin resource sharing.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = CORS)]
pub struct CorsTrait {
    /// The origin from which browser script-originating requests will be
    /// allowed.
    #[schema(default = "*".to_string())]
    pub origin: String,
    /// The maximum number of seconds for which browsers are allowed to cache
    /// the results of a preflight OPTIONS request.
    ///
    /// Defaults to 600, the maximum age permitted by several browsers.
    /// Set to -1 to disable caching entirely.
    #[schema(default = 600i32)]
    pub max_age: i32,
    /// The names of headers that should be included in the
    /// Access-Control-Allow-Headers header in responses to preflight OPTIONS
    /// requests. This list will be used in addition to the names of all
    /// request headers bound to an input data member via the httpHeader, as
    /// well as any headers required by the protocol or authentication scheme.
    pub additional_allowed_headers: Option<Vec<String>>,
    /// The names of headers that should be included in the
    /// Access-Control-Expose-Headers header in all responses sent by the
    /// service. This list will be used in addition to the names of all
    /// request headers bound to an output data member via the httpHeader,
    /// as well as any headers required by the protocol or authentication
    /// scheme.
    pub additional_exposed_headers: Option<Vec<String>>,
}

smithy!(
    /// Schema for [`DeprecatedTrait`]
    structure smithy::api::deprecated {
        message: STRING
        since: STRING
    }
);

/// Marks a shape or member as deprecated.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = DEPRECATED)]
pub struct DeprecatedTrait {
    /// The reason for deprecation.
    pub message: Option<String>,
    /// A description of when the shape was deprecated (e.g., a date or
    /// version).
    pub since: Option<String>,
}

smithy!(
    /// Schema for [`EndpointTrait`]
    structure smithy::api::endpoint {
        @RequiredTrait::builder().build();
        hostPrefix: NON_EMPTY_STRING
    }
);

/// Configures a custom operation endpoint.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = ENDPOINT)]
pub struct EndpointTrait {
    /// A host prefix pattern for the operation.
    ///
    /// Labels defined in the host pattern are used to bind top-level
    /// operation input members to the host.
    pub host_prefix: String,
}

smithy!(
    /// Schema for [`EnumDefinition`]
    structure smithy::api::EnumDefinition {
        @RequiredTrait::builder().build();
        value: NON_EMPTY_STRING
        name: ENUM_CONSTANT_BODY_NAME
        documentation: STRING
        tags: NON_EMPTY_STRING_LIST
        deprecated: BOOLEAN
    }
);

/// An enum definition for the enum trait.
#[doc(hidden)]
#[derive(SmithyShape, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = ENUM_DEFINITION)]
pub struct EnumDefinition {
    /// Defines the enum value that is sent over the wire.
    pub value: String,
    /// Defines the name that is used in code to represent this variant.
    pub name: Option<String>,
    /// Provides optional documentation about the enum constant value.
    pub documentation: Option<String>,
    /// Applies a list of tags to the enum constant.
    pub tags: Option<Vec<String>>,
    /// Whether the enum value should be considered deprecated.
    pub deprecated: Option<bool>,
}

smithy!(
    /// Schema for [`ErrorTrait`]
    enum smithy::api::error {
        Client = "client"
        Server = "server"
    }
);

/// Indicates that a structure shape represents an error.
///
/// All shapes referenced by the errors list of an operation MUST be targeted
/// with this trait.
#[smithy_enum]
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = ERROR)]
pub enum ErrorTrait {
    Client = "client",
    Server = "server",
}

smithy!(
    /// Schema for [`EventHeaderTrait`]
    structure smithy::api::eventHeader {
    }
);

/// Marks a member as a header of an event.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = EVENT_HEADER)]
pub struct EventHeaderTrait {
}

smithy!(
    /// Schema for [`EventPayloadTrait`]
    structure smithy::api::eventPayload {
    }
);

/// Marks a member as the payload of an event.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = EVENT_PAYLOAD)]
pub struct EventPayloadTrait {
}

smithy!(
    /// Provides a link to additional documentation.
    @LengthTrait::builder().min(1i64).build();
    map smithy::api::externalDocumentation {
        key: NON_EMPTY_STRING
        value: NON_EMPTY_STRING
    }
);
/// Provides a link to additional documentation.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = EXTERNAL_DOCUMENTATION)]
#[repr(transparent)]
pub struct ExternalDocumentationTrait(IndexMap<String, String>);

smithy!(
    /// Schema for [`HostLabelTrait`]
    structure smithy::api::hostLabel {
    }
);

/// Binds a top-level operation input structure member to a label
/// in the hostPrefix of an endpoint trait.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = HOST_LABEL)]
pub struct HostLabelTrait {
}

smithy!(
    /// Schema for [`HttpApiKeyLocations`]
    enum smithy::api::HttpApiKeyLocations {
        Header = "header"
        Query = "query"
    }
);

#[doc(hidden)]
#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = HTTP_API_KEY_LOCATIONS)]
pub enum HttpApiKeyLocations {
    Header = "header",
    Query = "query",
}

smithy!(
    /// Schema for [`HttpBasicAuthTrait`]
    @DynamicTrait::from("smithy.api#authDefinition", doc_map![]);
    structure smithy::api::httpBasicAuth {
    }
);

/// HTTP Basic Authentication as defined in [RFC
/// 2617](https://tools.ietf.org/html/rfc2617.html).
/// ## References
/// - [**RFC 2617**]("https://tools.ietf.org/html/rfc2617.html")
///
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = HTTP_BASIC_AUTH)]
pub struct HttpBasicAuthTrait {
}

smithy!(
    /// Schema for [`HttpBearerAuthTrait`]
    @DynamicTrait::from("smithy.api#authDefinition", doc_map![]);
    structure smithy::api::httpBearerAuth {
    }
);

/// HTTP Bearer Authentication as defined in [RFC
/// 6750](https://tools.ietf.org/html/rfc6750.html).
/// ## References
/// - [**RFC 6750**]("https://tools.ietf.org/html/rfc6750.html")
///
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = HTTP_BEARER_AUTH)]
pub struct HttpBearerAuthTrait {
}

smithy!(
    /// Schema for [`HttpChecksumRequiredTrait`]
    structure smithy::api::httpChecksumRequired {
    }
);

/// Marks an operation as requiring checksum in its HTTP request.
/// By default, the checksum used for a service is a MD5 checksum
/// passed in the Content-MD5 header.
/// <div class="warning">
///
/// **WARNING**: Unstable feature
///
/// </div>
///
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = HTTP_CHECKSUM_REQUIRED)]
pub struct HttpChecksumRequiredTrait {
}

smithy!(
    /// Schema for [`HttpDigestAuthTrait`]
    @DynamicTrait::from("smithy.api#authDefinition", doc_map![]);
    structure smithy::api::httpDigestAuth {
    }
);

/// HTTP Digest Authentication as defined in [RFC
/// 2617](https://tools.ietf.org/html/rfc2617.html).
/// ## References
/// - [**RFC 2617**]("https://tools.ietf.org/html/rfc2617.html")
///
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = HTTP_DIGEST_AUTH)]
pub struct HttpDigestAuthTrait {
}

smithy!(
    /// Schema for [`HttpTrait`]
    structure smithy::api::http {
        @RequiredTrait::builder().build();
        method: NON_EMPTY_STRING
        @RequiredTrait::builder().build();
        uri: NON_EMPTY_STRING
        @RangeTrait::builder().min(100).max(999).build();
        code: INTEGER
    }
);

/// Configures the HTTP bindings of an operation.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = HTTP)]
pub struct HttpTrait {
    /// The HTTP method of the operation.
    pub method: String,
    /// The URI pattern of the operation.
    ///
    /// Labels defined in the URI pattern are used to bind operation input
    /// members to the URI.
    pub uri: String,
    #[schema(default = 200i32)]
    pub code: i32,
}

smithy!(
    /// Schema for [`HttpLabelTrait`]
    structure smithy::api::httpLabel {
    }
);

/// Binds an operation input structure member to an HTTP label.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = HTTP_LABEL)]
pub struct HttpLabelTrait {
}

smithy!(
    /// Schema for [`HttpPayloadTrait`]
    structure smithy::api::httpPayload {
    }
);

/// Binds a single structure member to the body of an HTTP request.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = HTTP_PAYLOAD)]
pub struct HttpPayloadTrait {
}

smithy!(
    /// Schema for [`HttpQueryParamsTrait`]
    structure smithy::api::httpQueryParams {
    }
);

/// Binds an operation input structure member to the HTTP query string.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = HTTP_QUERY_PARAMS)]
pub struct HttpQueryParamsTrait {
}

smithy!(
    /// Schema for [`HttpResponseCodeTrait`]
    structure smithy::api::httpResponseCode {
    }
);

/// Indicates that the structure member represents the HTTP response
/// status code. The value MAY differ from the HTTP status code provided
/// on the response.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = HTTP_RESPONSE_CODE)]
pub struct HttpResponseCodeTrait {
}

smithy!(
    /// Schema for [`IdempotencyTokenTrait`]
    @DynamicTrait::from("smithy.api#notProperty", doc_map![]);
    structure smithy::api::idempotencyToken {
    }
);

/// Defines the input member of an operation that is used by the server to
/// identify and discard replayed requests.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = IDEMPOTENCY_TOKEN)]
pub struct IdempotencyTokenTrait {
}

smithy!(
    /// Schema for [`IdempotentTrait`]
    structure smithy::api::idempotent {
    }
);

/// Indicates that the intended effect on the server of multiple identical
/// requests with an operation is the same as the effect for a single such
/// request.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = IDEMPOTENT)]
pub struct IdempotentTrait {
}

smithy!(
    /// Schema for [`IdRefTrait`]
    structure smithy::api::idRef {
        selector: STRING
        failWhenMissing: BOOLEAN
        errorMessage: STRING
    }
);

/// Indicates that a string value MUST contain a valid shape ID.
///
/// The provided shape ID MAY be absolute or relative to the shape to which
/// the trait is applied. A relative shape ID that does not resolve to a
/// shape defined in the same namespace resolves to a shape defined in the
/// prelude if the prelude shape is not marked with the private trait.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = ID_REF)]
pub struct IdRefTrait {
    /// Defines the selector that the resolved shape, if found, MUST match.
    #[schema(default = "*".to_string())]
    pub selector: String,
    /// When set to `true`, the shape ID MUST target a shape that can be
    /// found in the model.
    pub fail_when_missing: Option<bool>,
    /// Defines a custom error message to use when the shape ID cannot be
    /// found or does not match the selector.
    ///
    /// A default message is generated when errorMessage is not defined.
    pub error_message: Option<String>,
}

smithy!(
    /// Schema for [`InputTrait`]
    structure smithy::api::input {
    }
);

/// Specializes a structure for use only as the input of a single operation.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = INPUT)]
pub struct InputTrait {
}

smithy!(
    /// Schema for [`InternalTrait`]
    structure smithy::api::internal {
    }
);

/// Shapes marked with the internal trait are meant only for internal use and
/// must not be exposed to customers.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = INTERNAL)]
pub struct InternalTrait {
}

smithy!(
    /// Schema for [`LengthTrait`]
    structure smithy::api::length {
        min: LONG
        max: LONG
    }
);

/// Constrains a shape to minimum and maximum number of elements or size.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = LENGTH)]
pub struct LengthTrait {
    /// Integer value that represents the minimum inclusive length of a shape.
    pub min: Option<i64>,
    /// Integer value that represents the maximum inclusive length of a shape.
    pub max: Option<i64>,
}

smithy!(
    #[doc(hidden)]
    list smithy::api::LocalMixinTraitList {
        member: LOCAL_MIXIN_TRAIT
    }
);

smithy!(
    /// Schema for [`MixinTrait`]
    structure smithy::api::mixin {
        localTraits: LOCAL_MIXIN_TRAIT_LIST
    }
);

/// Makes a structure or union a mixin.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = MIXIN)]
pub struct MixinTrait {
    pub local_traits: Option<Vec<String>>,
}

smithy!(
    /// Schema for [`NestedPropertiesTrait`]
    @DynamicTrait::from("smithy.api#notProperty", doc_map![]);
    structure smithy::api::nestedProperties {
    }
);

/// Adjusts the resource property mapping of a lifecycle operation to the
/// targeted member.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = NESTED_PROPERTIES)]
pub struct NestedPropertiesTrait {
}

smithy!(
    #[doc(hidden)]
    map smithy::api::NonEmptyStringMap {
        key: NON_EMPTY_STRING
        value: NON_EMPTY_STRING
    }
);

smithy!(
    /// Schema for [`NoReplaceTrait`]
    structure smithy::api::noReplace {
    }
);

/// Indicates that the put lifecycle operation of a resource can only be used
/// to create a resource and cannot replace an existing resource.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = NO_REPLACE)]
pub struct NoReplaceTrait {
}

smithy!(
    /// Schema for [`NotPropertyTrait`]
    @DynamicTrait::from("smithy.api#notProperty", doc_map![]);
    structure smithy::api::notProperty {
    }
);

/// Explicitly excludes a member from resource property mapping or enables
/// another trait to carry the same implied meaning.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = NOT_PROPERTY)]
pub struct NotPropertyTrait {
}

smithy!(
    /// Schema for [`OptionalAuthTrait`]
    structure smithy::api::optionalAuth {
    }
);

/// Indicates that an operation can be called without authentication.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = OPTIONAL_AUTH)]
pub struct OptionalAuthTrait {
}

smithy!(
    /// Schema for [`OutputTrait`]
    structure smithy::api::output {
    }
);

/// Specializes a structure for use only as the output of a single operation.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = OUTPUT)]
pub struct OutputTrait {
}

smithy!(
    /// Schema for [`PaginatedTrait`]
    structure smithy::api::paginated {
        inputToken: NON_EMPTY_STRING
        outputToken: NON_EMPTY_STRING
        items: NON_EMPTY_STRING
        pageSize: NON_EMPTY_STRING
    }
);

/// The paginated trait indicates that an operation intentionally limits the
/// number of results returned in a single response and that multiple
/// invocations might be necessary to retrieve all results.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = PAGINATED)]
pub struct PaginatedTrait {
    /// The name of the operation input member that represents the continuation
    /// token.
    ///
    /// When this value is provided as operation input, the service returns
    /// results from where the previous response left off. This input member
    /// MUST NOT be required and MUST target a string shape.
    pub input_token: Option<String>,
    /// The name of the operation output member that represents the
    /// continuation token.
    ///
    /// When this value is present in operation output, it indicates that there
    /// are more results to retrieve. To get the next page of results, the
    /// client uses the output token as the input token of the next request.
    /// This output member MUST NOT be required and MUST target a string shape.
    pub output_token: Option<String>,
    /// The name of a top-level output member of the operation that is the data
    /// that is being paginated across many responses.
    ///
    /// The named output member, if specified, MUST target a list or map.
    pub items: Option<String>,
    /// The name of an operation input member that limits the maximum number of
    /// results to include in the operation output. This input member MUST NOT
    /// be required and MUST target an integer shape.
    pub page_size: Option<String>,
}

smithy!(
    /// Schema for [`PrivateTrait`]
    structure smithy::api::private {
    }
);

/// Prevents models defined in a different namespace from referencing the
/// targeted shape.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = PRIVATE)]
pub struct PrivateTrait {
}

smithy!(
    /// Schema for [`PropertyTrait`]
    structure smithy::api::property {
        name: STRING
    }
);

/// Configures a structure member's resource property mapping behavior.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = PROPERTY)]
pub struct PropertyTrait {
    pub name: Option<String>,
}

smithy!(
    /// Schema for [`ProtocolDefinitionTrait`]
    structure smithy::api::protocolDefinition {
        traits: TRAIT_SHAPE_ID_LIST
        noInlineDocumentSupport: BOOLEAN
    }
);

/// Marks a trait as a protocol defining trait.
///
/// The targeted trait must only be applied to service shapes, must be a
/// structure, and must have the `trait` trait.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = PROTOCOL_DEFINITION)]
pub struct ProtocolDefinitionTrait {
    /// The list of traits that protocol implementations must understand in
    /// order to successfully use the protocol.
    pub traits: Option<Vec<String>>,
    /// Set to true if inline documents are not supported by this protocol.
    #[deprecated]
    pub no_inline_document_support: Option<bool>,
}

smithy!(
    /// Schema for [`RangeTrait`]
    structure smithy::api::range {
        min: BIG_DECIMAL
        max: BIG_DECIMAL
    }
);

/// Restricts allowed values of byte, short, integer, long, float, double,
/// bigDecimal, and bigInteger shapes within an acceptable lower and upper
/// bound.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = RANGE)]
pub struct RangeTrait {
    /// Specifies the allowed inclusive minimum value.
    pub min: Option<BigDecimal>,
    /// Specifies the allowed inclusive maximum value.
    pub max: Option<BigDecimal>,
}

smithy!(
    /// Schema for [`ReadonlyTrait`]
    structure smithy::api::readonly {
    }
);

/// Indicates that an operation is effectively read-only.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = READONLY)]
pub struct ReadonlyTrait {
}

smithy!(
    /// Schema for [`RecommendedTrait`]
    structure smithy::api::recommended {
        reason: STRING
    }
);

/// Indicates that a structure member SHOULD be set.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = RECOMMENDED)]
pub struct RecommendedTrait {
    /// Provides a reason why the member is recommended.
    pub reason: Option<String>,
}

smithy!(
    /// Schema for [`Reference`]
    structure smithy::api::Reference {
        @RequiredTrait::builder().build();
        resource: NON_EMPTY_STRING
        ids: NON_EMPTY_STRING_MAP
        service: NON_EMPTY_STRING
        rel: NON_EMPTY_STRING
    }
);

#[doc(hidden)]
#[derive(SmithyShape, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = REFERENCE)]
pub struct Reference {
    /// The shape ID of the referenced resource.
    pub resource: String,
    /// Defines a mapping of each resource identifier name to a structure member
    /// name that provides its value. Each key in the map MUST refer to one of the
    /// identifier names in the identifiers property of the resource, and each
    /// value in the map MUST refer to a valid structure member name that targets
    /// a string shape.
    pub ids: Option<IndexMap<String, String>>,
    /// Providing a service makes the reference specific to a particular binding
    /// of the resource to a service. When omitted, the reference is late-bound to
    /// a service, meaning the reference is assumed to be a reference to the
    /// resource bound to the service currently in use by the client or server.
    pub service: Option<String>,
    /// Defines the semantics of the relationship. The rel property SHOULD
    /// contain a link relation as defined in RFC 5988#section-4.
    pub rel: Option<String>,
}

smithy!(
    /// Defines the priority-ordered list of compression algorithms supported by
    /// the service operation.
    #[doc(hidden)]
    list smithy::api::RequestCompressionEncodingsList {
        member: STRING
    }
);

smithy!(
    /// Schema for [`RequestCompressionTrait`]
    structure smithy::api::requestCompression {
        @RequiredTrait::builder().build();
        encodings: REQUEST_COMPRESSION_ENCODINGS_LIST
    }
);

/// Indicates that an operation supports compressing requests from clients to
/// services.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = REQUEST_COMPRESSION)]
pub struct RequestCompressionTrait {
    pub encodings: Vec<String>,
}

smithy!(
    /// Schema for [`RequiredTrait`]
    structure smithy::api::required {
    }
);

/// Marks a structure member as required, meaning a value for the member MUST
/// be present.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = REQUIRED)]
pub struct RequiredTrait {
}

smithy!(
    /// Schema for [`RequiresLengthTrait`]
    structure smithy::api::requiresLength {
    }
);

/// Indicates that the streaming blob must be finite and has a known size.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = REQUIRES_LENGTH)]
pub struct RequiresLengthTrait {
}

smithy!(
    /// Schema for [`RetryableTrait`]
    structure smithy::api::retryable {
        throttling: BOOLEAN
    }
);

/// Indicates that an error MAY be retried by the client.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = RETRYABLE)]
pub struct RetryableTrait {
    /// Classifies the retry as throttling.
    pub throttling: Option<bool>,
}

smithy!(
    /// Schema for [`SensitiveTrait`]
    structure smithy::api::sensitive {
    }
);

/// Indicates that the data stored in the shape is sensitive and MUST be
/// handled with care.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = SENSITIVE)]
pub struct SensitiveTrait {
}

smithy!(
    /// Schema for [`Severity`]
    enum smithy::api::Severity {
        Note = "NOTE"
        Warning = "WARNING"
        Danger = "DANGER"
        Error = "ERROR"
    }
);

#[doc(hidden)]
#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = SEVERITY)]
pub enum Severity {
    Note = "NOTE",
    Warning = "WARNING",
    Danger = "DANGER",
    Error = "ERROR",
}

smithy!(
    /// Schema for [`SparseTrait`]
    structure smithy::api::sparse {
    }
);

/// Marks a list or map as sparse.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = SPARSE)]
pub struct SparseTrait {
}

smithy!(
    /// Schema for [`StreamingTrait`]
    structure smithy::api::streaming {
    }
);

/// Indicates that the data stored in the shape is very large and should not
/// be stored in memory, or that the size of the data stored in the shape is
/// unknown at the start of a request. If the target is a union then the shape
/// represents a stream of events.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = STREAMING)]
pub struct StreamingTrait {
}

smithy!(
    /// Schema for [`StructurallyExclusive`]
    enum smithy::api::StructurallyExclusive {
        Member = "member"
        Target = "target"
    }
);

#[doc(hidden)]
#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = STRUCTURALLY_EXCLUSIVE)]
pub enum StructurallyExclusive {
    Member = "member",
    Target = "target",
}

smithy!(
    /// Suppresses validation events by ID for a given shape.
    list smithy::api::suppress {
        @LengthTrait::builder().min(1i64).build();
        member: STRING
    }
);
/// Suppresses validation events by ID for a given shape.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = SUPPRESS)]
#[repr(transparent)]
pub struct SuppressTrait(Vec<String>);

smithy!(
    /// Tags a shape with arbitrary tag names that can be used to filter and
    /// group shapes in the model.
    list smithy::api::tags {
        member: STRING
    }
);
/// Tags a shape with arbitrary tag names that can be used to filter and
/// group shapes in the model.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = TAGS)]
#[repr(transparent)]
pub struct TagsTrait(Vec<String>);

smithy!(
    /// Schema for [`TimestampFormatTrait`]
    enum smithy::api::timestampFormat {
        DateTime = "date-time"
        EpochSeconds = "epoch-seconds"
        HttpDate = "http-date"
    }
);

#[smithy_enum]
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = TIMESTAMP_FORMAT)]
pub enum TimestampFormatTrait {
    DateTime = "date-time",
    EpochSeconds = "epoch-seconds",
    HttpDate = "http-date",
}

smithy!(
    /// Schema for [`TraitChangeType`]
    enum smithy::api::TraitChangeType {
        Update = "update"
        Add = "add"
        Remove = "remove"
        Presence = "presence"
        Any = "any"
    }
);

#[doc(hidden)]
#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = TRAIT_CHANGE_TYPE)]
pub enum TraitChangeType {
    Update = "update",
    Add = "add",
    Remove = "remove",
    Presence = "presence",
    Any = "any",
}

smithy!(
    /// Schema for [`TraitDiffRule`]
    structure smithy::api::TraitDiffRule {
        path: STRING
        @RequiredTrait::builder().build();
        change: TRAIT_CHANGE_TYPE
        severity: SEVERITY
        message: STRING
    }
);

#[doc(hidden)]
#[derive(SmithyShape, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = TRAIT_DIFF_RULE)]
pub struct TraitDiffRule {
    /// Defines a JSON Pointer to the value to evaluate.
    pub path: Option<String>,
    /// Defines the type of change that is not allowed.
    #[schema(no_builder)]
    pub change: TraitChangeType,
    /// Defines the severity of the change. Defaults to ERROR if not defined.
    #[schema(default = Severity::Error, no_builder)]
    pub severity: Severity,
    /// Provides a reason why the change is potentially backward incompatible.
    pub message: Option<String>,
}

smithy!(
    #[doc(hidden)]
    @LengthTrait::builder().min(1i64).build();
    list smithy::api::TraitDiffRules {
        member: TRAIT_DIFF_RULE
    }
);

smithy!(
    /// Schema for [`TraitTrait`]
    structure smithy::api::r#trait {
        selector: STRING
        structurallyExclusive: STRUCTURALLY_EXCLUSIVE
        conflicts: NON_EMPTY_STRING_LIST
        breakingChanges: TRAIT_DIFF_RULES
    }
);

/// Makes a shape a trait.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = TRAIT)]
pub struct TraitTrait {
    /// The valid places in a model that the trait can be applied.
    pub selector: Option<String>,
    /// Whether or not only a single member in a shape can have this trait.
    /// This only has an effect on members of structure shapes.
    #[schema(no_builder)]
    pub structurally_exclusive: Option<StructurallyExclusive>,
    /// The traits that this trait conflicts with.
    pub conflicts: Option<Vec<String>>,
    /// Defines the backward compatibility rules of the trait.
    pub breaking_changes: Option<Vec<TraitDiffRule>>,
}

smithy!(
    /// Schema for [`TraitValidator`]
    structure smithy::api::TraitValidator {
        @RequiredTrait::builder().build();
        selector: STRING
        message: STRING
        severity: SEVERITY
    }
);

#[doc(hidden)]
#[derive(SmithyShape, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = TRAIT_VALIDATOR)]
pub struct TraitValidator {
    /// A Smithy selector that receives only the shape to which the `traitValidators` trait is
    /// applied.
    /// Any shape yielded by the selector is considered incompatible with the trait.
    pub selector: String,
    /// A message to use when a matching shape is found.
    pub message: Option<String>,
    /// The severity to use when a matching shape is found.
    #[schema(default = Severity::Error, no_builder)]
    pub severity: Severity,
}

smithy!(
    /// Schema for [`UniqueItemsTrait`]
    structure smithy::api::uniqueItems {
    }
);

/// Indicates that the items in a list MUST be unique.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = UNIQUE_ITEMS)]
pub struct UniqueItemsTrait {
}

smithy!(
    /// Schema for [`Unit`]
    @DynamicTrait::from("smithy.api#unitType", doc_map![]);
    structure smithy::api::Unit {
    }
);

#[derive(SmithyShape, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = UNIT)]
pub struct Unit {
}

smithy!(
    /// Schema for [`UnitTypeTrait`]
    structure smithy::api::unitType {
    }
);

/// Specializes a structure as a unit type that has no meaningful value.
/// This trait can only be applied to smithy.api#Unit, which ensures that
/// only a single Unit shape can be created.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = UNIT_TYPE)]
pub struct UnitTypeTrait {
}

smithy!(
    /// Schema for [`UnstableTrait`]
    structure smithy::api::unstable {
    }
);

/// Indicates that the shape is unstable and could change in the future.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = UNSTABLE)]
pub struct UnstableTrait {
}

smithy!(
    /// Schema for [`XmlAttributeTrait`]
    structure smithy::api::xmlAttribute {
    }
);

/// Serializes an object property as an XML attribute rather than a nested XML
/// element.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = XML_ATTRIBUTE)]
pub struct XmlAttributeTrait {
}

smithy!(
    /// Schema for [`XmlFlattenedTrait`]
    structure smithy::api::xmlFlattened {
    }
);

/// Unwraps the values of a list, set, or map into the containing
/// structure/union.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = XML_FLATTENED)]
pub struct XmlFlattenedTrait {
}

smithy!(
    /// Schema for [`XmlNamespaceTrait`]
    structure smithy::api::xmlNamespace {
        @RequiredTrait::builder().build();
        uri: NON_EMPTY_STRING
        @PatternTrait::new("^[a-zA-Z_][a-zA-Z_0-9-]*$");
        prefix: NON_EMPTY_STRING
    }
);

/// Adds an xmlns namespace definition URI to an XML element.
#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = XML_NAMESPACE)]
pub struct XmlNamespaceTrait {
    /// The namespace URI for scoping this XML element.
    pub uri: String,
    /// The prefix for the given namespace.
    pub prefix: Option<String>,
}

smithy!(
    /// Changes the serialized element or attribute name of a structure, union,
    /// or member.
    @PatternTrait::new("^[a-zA-Z_][a-zA-Z_0-9-]*(:[a-zA-Z_][a-zA-Z_0-9-]*)?$");
    string smithy::api::xmlName
);

/// Changes the serialized element or attribute name of a structure, union,
/// or member.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = XML_NAME)]
#[repr(transparent)]
pub struct XmlNameTrait(String);

smithy!(
    string smithy::api::String
);

smithy!(
    /// A string that must target an auth trait.
    #[doc(hidden)]
    @DynamicTrait::from("smithy.api#idRef", doc_map!["selector" => "[trait|authDefinition]"]);
    string smithy::api::AuthTraitReference
);

smithy!(
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    double smithy::api::Double
);

smithy!(
    byte smithy::api::PrimitiveByte
);

smithy!(
    double smithy::api::PrimitiveDouble
);

smithy!(
    boolean smithy::api::PrimitiveBoolean
);

smithy!(
    /// Describes the contents of a blob shape using a media type as defined by
    /// RFC 6838 (e.g., "video/quicktime").
    string smithy::api::mediaType
);

/// Describes the contents of a blob shape using a media type as defined by
/// RFC 6838 (e.g., "video/quicktime").
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = MEDIA_TYPE)]
#[repr(transparent)]
pub struct MediaTypeTrait(String);

smithy!(
    integer smithy::api::PrimitiveInteger
);

smithy!(
    /// Defines an HTTP response code for an operation error.
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    integer smithy::api::httpError
);

/// Defines an HTTP response code for an operation error.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = HTTP_ERROR)]
#[repr(transparent)]
pub struct HttpErrorTrait(i32);

smithy!(
    blob smithy::api::Blob
);

smithy!(
    float smithy::api::PrimitiveFloat
);

smithy!(
    short smithy::api::PrimitiveShort
);

smithy!(
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    integer smithy::api::Integer
);

smithy!(
    /// Defines the version or date in which a shape or member was added to the
    /// model.
    string smithy::api::since
);

/// Defines the version or date in which a shape or member was added to the
/// model.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = SINCE)]
#[repr(transparent)]
pub struct SinceTrait(String);

smithy!(
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    byte smithy::api::Byte
);

smithy!(
    /// Binds a structure member to an HTTP header.
    @LengthTrait::builder().min(1i64).build();
    string smithy::api::httpHeader
);

/// Binds a structure member to an HTTP header.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = HTTP_HEADER)]
#[repr(transparent)]
pub struct HttpHeaderTrait(String);

smithy!(
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    float smithy::api::Float
);

smithy!(
    /// Provides a structure member with a default value. When added to root
    /// level shapes, requires that every targeting structure member defines the
    /// same default value on the member or sets a default of null.
    ///
    /// This trait can currently only be used in Smithy 2.0 models.
    document smithy::api::default
);

/// Provides a structure member with a default value. When added to root
/// level shapes, requires that every targeting structure member defines the
/// same default value on the member or sets a default of null.
///
/// This trait can currently only be used in Smithy 2.0 models.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = DEFAULT)]
#[repr(transparent)]
pub struct DefaultTrait(Box<dyn Document>);

smithy!(
    bigInteger smithy::api::BigInteger
);

smithy!(
    /// Indicates that the targeted structure member provides an identifier for
    /// a resource.
    @DynamicTrait::from("smithy.api#notProperty", doc_map![]);
    @LengthTrait::builder().min(1i64).build();
    string smithy::api::resourceIdentifier
);

/// Indicates that the targeted structure member provides an identifier for
/// a resource.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = RESOURCE_IDENTIFIER)]
#[repr(transparent)]
pub struct ResourceIdentifierTrait(String);

smithy!(
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    short smithy::api::Short
);

smithy!(
    #[doc(hidden)]
    @DynamicTrait::from("smithy.api#idRef", doc_map!["selector" => "[trait|trait]", "failWhenMissing" => true, "errorMessage" => "Strings provided to the localTraits property of a mixin trait\nmust target a valid trait."]);
    string smithy::api::LocalMixinTrait
);

smithy!(
    /// The optional name or label of the enum constant value.
    ///
    /// This property is used in code generation to provide a label for
    /// each enum value. No two enums can have the same 'name' value.
    #[doc(hidden)]
    @PatternTrait::new("^[a-zA-Z_]+[a-zA-Z_0-9]*$");
    string smithy::api::EnumConstantBodyName
);

smithy!(
    #[doc(hidden)]
    @DynamicTrait::from("smithy.api#idRef", doc_map!["failWhenMissing" => true, "selector" => "[trait|trait]"]);
    string smithy::api::TraitShapeId
);

smithy!(
    bigDecimal smithy::api::BigDecimal
);

smithy!(
    /// Binds a map of key-value pairs to prefixed HTTP headers.
    string smithy::api::httpPrefixHeaders
);

/// Binds a map of key-value pairs to prefixed HTTP headers.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = HTTP_PREFIX_HEADERS)]
#[repr(transparent)]
pub struct HttpPrefixHeadersTrait(String);

smithy!(
    /// Adds documentation to a shape or member using CommonMark syntax.
    string smithy::api::documentation
);

/// Adds documentation to a shape or member using CommonMark syntax.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = DOCUMENTATION)]
#[repr(transparent)]
pub struct DocumentationTrait(String);

smithy!(
    #[doc(hidden)]
    @LengthTrait::builder().min(1i64).build();
    string smithy::api::NonEmptyString
);

smithy!(
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    long smithy::api::Long
);

smithy!(
    @DynamicTrait::from("smithy.api#box", doc_map![]);
    boolean smithy::api::Boolean
);

smithy!(
    timestamp smithy::api::Timestamp
);

smithy!(
    /// Defines a proper name for a shape.
    ///
    /// This title can be used in automatically generated documentation
    /// and other contexts to provide a user-friendly for shapes.
    string smithy::api::title
);

/// Defines a proper name for a shape.
///
/// This title can be used in automatically generated documentation
/// and other contexts to provide a user-friendly for shapes.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = TITLE)]
#[repr(transparent)]
pub struct TitleTrait(String);

smithy!(
    /// Allows a serialized object property name to differ from a structure member
    /// name used in the model.
    string smithy::api::jsonName
);

/// Allows a serialized object property name to differ from a structure member
/// name used in the model.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = JSON_NAME)]
#[repr(transparent)]
pub struct JsonNameTrait(String);

smithy!(
    /// Restricts string shape values to a specified regular expression.
    string smithy::api::pattern
);

/// Restricts string shape values to a specified regular expression.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = PATTERN)]
#[repr(transparent)]
pub struct PatternTrait(RegexWrapper);

smithy!(
    document smithy::api::Document
);

smithy!(
    long smithy::api::PrimitiveLong
);

smithy!(
    /// Binds an operation input structure member to a query string parameter.
    @LengthTrait::builder().min(1i64).build();
    string smithy::api::httpQuery
);

/// Binds an operation input structure member to a query string parameter.
#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = HTTP_QUERY)]
#[repr(transparent)]
pub struct HttpQueryTrait(String);
