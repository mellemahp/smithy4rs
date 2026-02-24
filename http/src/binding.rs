use std::{collections::HashMap, sync::Arc};

use http::header::HeaderName;
use parking_lot::RwLock;
use smithy4rs_core::{
    schema::{
        Operation, Schema, ShapeId,
        prelude::{
            HTTPHeaderTrait, HTTPPrefixHeadersTrait, HTTPQueryParamsTrait, HTTPQueryTrait,
            HTTPResponseCodeTrait, HttpLabelTrait, HttpPayloadTrait, HttpTrait,
        },
    },
    serde::{BuildWithSchema, BuildableShape, codec::Codec, correction::ErrorCorrection},
};

use crate::{
    deserialize::{HttpRequestDeserializer, HttpResponseDeserializer},
    error::HttpBindingError,
    serialize::{HttpRequestSerializer, HttpResponseSerializer},
    uri::{UriPattern, UriSegment},
};

/// HTTP binding for a single struct member.
///
/// Each variant carries a reference to the member's schema and any
/// type-specific routing information (header name, query key, etc.).
#[derive(Debug)]
pub enum MemberBinding<'a> {
    /// URI label segment.
    Label {
        /// Position in the URI pattern (for positional access).
        pattern_index: usize,
        /// Whether this is a greedy label (`{key+}`).
        greedy: bool,
        /// The member schema.
        schema: &'a Schema,
    },
    /// HTTP header.
    Header {
        /// Pre-parsed HTTP header name (from `@httpHeader` trait).
        name: HeaderName,
        /// The member schema.
        schema: &'a Schema,
    },
    /// Query string parameter.
    QueryParam {
        /// The query parameter key (from `@httpQuery` trait).
        key: String,
        /// The member schema.
        schema: &'a Schema,
    },
    /// Map bound to all query parameters (`@httpQueryParams`).
    QueryParams {
        /// The member schema.
        schema: &'a Schema,
    },
    /// HTTP header prefix for a map (`@httpPrefixHeaders`).
    PrefixHeaders {
        /// The header prefix string.
        prefix: String,
        /// The member schema.
        schema: &'a Schema,
    },
    /// Explicit body payload (`@httpPayload`).
    Payload {
        /// The member schema.
        schema: &'a Schema,
    },
    /// Implicit body member (no HTTP binding trait).
    Body {
        /// The member schema.
        schema: &'a Schema,
    },
    /// HTTP response status code (`@httpResponseCode`).
    StatusCode {
        /// The member schema.
        schema: &'a Schema,
    },
}

/// Member bindings for a single schema (input or output).
///
/// Contains bindings in member declaration order.
#[derive(Debug)]
pub struct MemberBindings<'a> {
    /// Bindings in member declaration order.
    members: Box<[MemberBinding<'a>]>,
    /// Schemas of URI label members.
    label_members: Box<[&'a Schema]>,
    /// Schemas of implicit body members (members with no HTTP binding trait).
    body_members: Box<[&'a Schema]>,
    /// Schema of the explicit `@httpPayload` member, if any.
    payload_member: Option<&'a Schema>,
    /// Keys of `@httpQuery`-bound members, used to filter `@httpQueryParams` (per Smithy spec).
    query_keys: Box<[&'a str]>,
}

impl<'a> MemberBindings<'a> {
    /// Build member bindings for a request schema (with URI pattern for label indices).
    pub fn for_request(
        schema: &'a Schema,
        uri_pattern: &UriPattern,
    ) -> Result<Self, HttpBindingError> {
        let label_info: Vec<(&str, bool, usize)> = uri_pattern
            .segments()
            .iter()
            .filter_map({
                let mut idx = 0;
                move |seg| match seg {
                    UriSegment::Label(name) => {
                        let i = idx;
                        idx += 1;
                        Some((name.as_str(), false, i))
                    }
                    UriSegment::GreedyLabel(name) => {
                        let i = idx;
                        idx += 1;
                        Some((name.as_str(), true, i))
                    }
                    _ => None,
                }
            })
            .collect();

        Self::build(schema, &label_info)
    }

    /// Build member bindings for a response schema (no labels).
    pub fn for_response(schema: &'a Schema) -> Result<Self, HttpBindingError> {
        Self::build(schema, &[])
    }

    fn build(
        schema: &'a Schema,
        label_info: &[(&str, bool, usize)],
    ) -> Result<Self, HttpBindingError> {
        let mut label_members = Vec::with_capacity(4);
        let mut body_members = Vec::with_capacity(8);
        let mut query_keys = Vec::with_capacity(4);
        let mut payload_member = None;

        let mut members = Vec::with_capacity(schema.members().len());

        for (_name, member_schema) in schema.members().iter() {
            let binding = if member_schema.contains_type::<HttpLabelTrait>() {
                let member = member_schema
                    .as_member()
                    .expect("member schema should be a member");
                let member_name = member.name();
                let (_, greedy, pattern_index) = label_info
                    .iter()
                    .find(|(name, _, _)| *name == member_name)
                    .copied()
                    .ok_or_else(|| {
                        HttpBindingError::new(format!(
                            "@httpLabel member '{member_name}' has no matching URI pattern label"
                        ))
                    })?;
                label_members.push(member_schema);
                MemberBinding::Label {
                    pattern_index,
                    greedy,
                    schema: member_schema,
                }
            } else if let Some(header) = member_schema.get_trait_as::<HTTPHeaderTrait>() {
                MemberBinding::Header {
                    name: HeaderName::from_bytes(header.name().as_bytes()).map_err(|_| {
                        HttpBindingError::new(format!(
                            "@httpHeader trait contains invalid header name: '{}'",
                            header.name()
                        ))
                    })?,
                    schema: member_schema,
                }
            } else if let Some(query) = member_schema.get_trait_as::<HTTPQueryTrait>() {
                query_keys.push(query.key());
                MemberBinding::QueryParam {
                    key: query.key().into(),
                    schema: member_schema,
                }
            } else if member_schema.contains_type::<HttpPayloadTrait>() {
                payload_member = Some(member_schema);
                MemberBinding::Payload {
                    schema: member_schema,
                }
            } else if member_schema.contains_type::<HTTPQueryParamsTrait>() {
                MemberBinding::QueryParams {
                    schema: member_schema,
                }
            } else if let Some(prefix) = member_schema.get_trait_as::<HTTPPrefixHeadersTrait>() {
                MemberBinding::PrefixHeaders {
                    prefix: prefix.prefix().to_ascii_lowercase(),
                    schema: member_schema,
                }
            } else if member_schema.contains_type::<HTTPResponseCodeTrait>() {
                MemberBinding::StatusCode {
                    schema: member_schema,
                }
            } else {
                body_members.push(member_schema);
                MemberBinding::Body {
                    schema: member_schema,
                }
            };
            members.push(binding);
        }

        Ok(Self {
            members: members.into_boxed_slice(),
            label_members: label_members.into_boxed_slice(),
            body_members: body_members.into_boxed_slice(),
            payload_member,
            query_keys: query_keys.into_boxed_slice(),
        })
    }

    /// Bindings as a slice in member declaration order.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[MemberBinding<'a>] {
        &self.members
    }

    /// Number of members.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Whether there are no members.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Schemas of URI label members.
    #[inline]
    #[must_use]
    pub fn label_members(&self) -> &[&'a Schema] {
        &self.label_members
    }

    /// Schemas of implicit body members (members with no HTTP binding trait).
    #[inline]
    #[must_use]
    pub fn body_members(&self) -> &[&'a Schema] {
        &self.body_members
    }

    /// Schema of the explicit `@httpPayload` member, if any.
    #[inline]
    #[must_use]
    pub fn payload_member(&self) -> Option<&'a Schema> {
        self.payload_member
    }

    /// Keys of `@httpQuery`-bound members (for filtering `@httpQueryParams`).
    #[inline]
    #[must_use]
    pub fn query_keys(&self) -> &[&'a str] {
        &self.query_keys
    }
}

/// All HTTP binding information for an operation.
///
/// Combines the HTTP method, URI pattern, default status code, and member
/// bindings for both request (input) and response (output) schemas.
/// Built once per operation and cached.
#[derive(Debug)]
pub struct OperationBinding<'a> {
    /// HTTP method (GET, POST, etc.).
    pub method: http::Method,
    /// Parsed URI pattern with label and literal segments.
    pub uri_pattern: UriPattern,
    /// Default HTTP response status code.
    pub default_status: u16,
    /// Member bindings for the input (request) schema.
    pub request: MemberBindings<'a>,
    /// Member bindings for the output (response) schema.
    pub response: MemberBindings<'a>,
}

impl<'a> OperationBinding<'a> {
    /// Build an [`OperationBinding`] from an operation by inspecting the `@http`
    /// trait and HTTP binding traits on input/output members.
    pub fn from_operation<Op: Operation>(operation: &'a Op) -> Result<Self, HttpBindingError> {
        let op_schema = operation.schema();
        let http_trait = op_schema
            .get_trait_as::<HttpTrait>()
            .ok_or_else(|| HttpBindingError::new("operation missing @http trait"))?;

        let method: http::Method = http_trait.method().parse().map_err(|_| {
            HttpBindingError::new(format!("invalid HTTP method: {}", http_trait.method()))
        })?;

        let uri_pattern = UriPattern::parse(http_trait.uri())?;

        let default_status = u16::try_from(http_trait.code()).map_err(|_| {
            HttpBindingError::new(format!(
                "HTTP status code out of range: {}",
                http_trait.code()
            ))
        })?;

        let input_schema = operation.input_schema();
        let output_schema = operation.output_schema();

        let request = MemberBindings::for_request(input_schema, &uri_pattern)?;
        let response = MemberBindings::for_response(output_schema)?;

        Ok(Self {
            method,
            uri_pattern,
            default_status,
            request,
            response,
        })
    }
}

/// Central factory for HTTP binding operations.
///
/// Provides serialization and deserialization of HTTP requests and responses,
/// with caching of operation bindings for improved performance.
///
/// # Example
///
/// ```ignore
/// let binding = HttpBinding::new(JsonCodec::new());
///
/// // Client: serialize request, deserialize response
/// let request = binding.serialize_request(&GetCityOp, &input)?;
/// let output = binding.deserialize_response(&GetCityOp, &response)?;
///
/// // Server: deserialize request, serialize response
/// let input = binding.deserialize_request(&GetCityOp, &request)?;
/// let response = binding.serialize_response(&GetCityOp, &output)?;
/// ```
pub struct HttpBinding<'a, C: Codec> {
    codec: C,
    /// Cache of operation bindings, keyed by operation shape ID.
    binding_cache: Arc<RwLock<HashMap<ShapeId, Arc<OperationBinding<'a>>>>>,
}

impl<C: Codec + Clone> Clone for HttpBinding<'_, C> {
    fn clone(&self) -> Self {
        Self {
            codec: self.codec.clone(),
            binding_cache: Arc::clone(&self.binding_cache),
        }
    }
}

impl<'a, C: Codec> HttpBinding<'a, C> {
    /// Create a new HTTP binding factory with the given codec.
    pub fn new(codec: C) -> Self {
        Self {
            codec,
            binding_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a cached operation binding.
    pub fn get_operation_binding<Op: Operation>(
        &self,
        operation: &'a Op,
    ) -> Result<Arc<OperationBinding<'a>>, HttpBindingError> {
        let id = operation.schema().id();

        // TODO(performance): Might be worth looking into dashmap for
        // less contention... maybe not worth since number of operations
        // is usually on scale of tens, not thousands
        // Fast path: read lock
        {
            let cache = self.binding_cache.read();
            if let Some(binding) = cache.get(id) {
                return Ok(Arc::clone(binding));
            }
        }

        // Slow path: write lock, double-check
        let mut cache = self.binding_cache.write();
        if let Some(binding) = cache.get(id) {
            return Ok(Arc::clone(binding));
        }

        let binding = Arc::new(OperationBinding::from_operation(operation)?);
        cache.insert(id.clone(), Arc::clone(&binding));
        Ok(binding)
    }

    /// Get the underlying codec.
    pub fn codec(&self) -> &C {
        &self.codec
    }

    /// Serialize an operation input into an HTTP request.
    pub fn serialize_request<Op: Operation>(
        &self,
        operation: &'a Op,
        input: &Op::Input,
    ) -> Result<http::Request<Vec<u8>>, HttpBindingError> {
        let op_binding = self.get_operation_binding(operation)?;
        let input_schema = operation.input_schema();

        HttpRequestSerializer::new(&self.codec).serialize(input, input_schema, &op_binding)
    }

    /// Serialize an operation output into an HTTP response.
    pub fn serialize_response<Op: Operation>(
        &self,
        operation: &'a Op,
        output: &Op::Output,
    ) -> Result<http::Response<Vec<u8>>, HttpBindingError> {
        let op_binding = self.get_operation_binding(operation)?;
        let output_schema = operation.output_schema();

        HttpResponseSerializer::new(&self.codec).serialize(output, output_schema, &op_binding)
    }

    /// Deserialize an HTTP request into an operation input.
    ///
    /// Deserializes into the input's builder, then builds the shape with validation.
    pub fn deserialize_request<'de, Op: Operation>(
        &self,
        operation: &'a Op,
        request: &'de http::Request<&'de [u8]>,
    ) -> Result<Op::Input, HttpBindingError> {
        let op_binding = self.get_operation_binding(operation)?;
        let input_schema = operation.input_schema();

        let builder: <Op::Input as BuildableShape>::Builder = HttpRequestDeserializer::new(
            &self.codec,
        )
        .deserialize(request, input_schema, &op_binding)?;
        builder
            .build_with_schema(input_schema)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    /// Deserialize an HTTP response into an operation output using error correction.
    ///
    /// Deserializes into the output's builder, then applies error correction
    /// (filling zero-values for missing required fields) without validation.
    /// This is appropriate for clients deserializing server responses.
    pub fn deserialize_response<'de, Op: Operation>(
        &self,
        operation: &'a Op,
        response: &'de http::Response<&'de [u8]>,
    ) -> Result<Op::Output, HttpBindingError> {
        let op_binding = self.get_operation_binding(operation)?;
        let output_schema = operation.output_schema();

        let builder: <Op::Output as BuildableShape>::Builder = HttpResponseDeserializer::new(
            &self.codec,
        )
        .deserialize(response, output_schema, &op_binding)?;
        Ok(builder.correct())
    }
}

#[cfg(test)]
mod tests {
    use smithy4rs_core::{
        derive::SmithyShape,
        schema::{
            OperationShape, StaticSchemaShape,
            prelude::{
                HTTPHeaderTrait, HTTPPrefixHeadersTrait, HTTPQueryParamsTrait, HTTPQueryTrait,
                HTTPResponseCodeTrait, HttpLabelTrait, HttpPayloadTrait, INTEGER, STRING,
            },
        },
        smithy,
    };
    use smithy4rs_json_codec::JsonCodec;
    use smithy4rs_test_utils::STRING_MAP_SCHEMA;

    use super::*;

    // --- MemberBindings::for_request ---

    #[test]
    fn request_bindings_label_members() {
        smithy!("test.binding#LabelInput": {
            structure LABEL_INPUT_SCHEMA {
                @HttpLabelTrait;
                BUCKET: STRING = "bucket"
                @HttpLabelTrait;
                KEY: STRING = "key"
            }
        });

        let pattern = UriPattern::parse("/buckets/{bucket}/keys/{key}").unwrap();
        let bindings = MemberBindings::for_request(&LABEL_INPUT_SCHEMA, &pattern).unwrap();

        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings.label_members().len(), 2);
        assert!(bindings.body_members().is_empty());

        assert!(matches!(
            &bindings.as_slice()[0],
            MemberBinding::Label {
                pattern_index: 0,
                greedy: false,
                ..
            }
        ));
        assert!(matches!(
            &bindings.as_slice()[1],
            MemberBinding::Label {
                pattern_index: 1,
                greedy: false,
                ..
            }
        ));
    }

    #[test]
    fn request_bindings_greedy_label() {
        smithy!("test.binding#GreedyInput": {
            structure GREEDY_INPUT_SCHEMA {
                @HttpLabelTrait;
                PATH: STRING = "path"
            }
        });

        let pattern = UriPattern::parse("/{path+}").unwrap();
        let bindings = MemberBindings::for_request(&GREEDY_INPUT_SCHEMA, &pattern).unwrap();

        assert!(matches!(
            &bindings.as_slice()[0],
            MemberBinding::Label {
                greedy: true,
                pattern_index: 0,
                ..
            }
        ));
    }

    #[test]
    fn request_bindings_query_param() {
        smithy!("test.binding#QueryInput": {
            structure QUERY_INPUT_SCHEMA {
                @HTTPQueryTrait::new("limit");
                LIMIT: INTEGER = "limit"
            }
        });

        let pattern = UriPattern::parse("/items").unwrap();
        let bindings = MemberBindings::for_request(&QUERY_INPUT_SCHEMA, &pattern).unwrap();

        assert!(matches!(
            &bindings.as_slice()[0],
            MemberBinding::QueryParam { key, .. } if key == "limit"
        ));
        assert_eq!(bindings.query_keys(), &["limit"]);
    }

    #[test]
    fn request_bindings_query_params_map() {
        smithy!("test.binding#QueryParamsInput": {
            structure QUERY_PARAMS_INPUT_SCHEMA {
                @HTTPQueryParamsTrait;
                EXTRAS: STRING_MAP_SCHEMA = "extras"
            }
        });

        let pattern = UriPattern::parse("/items").unwrap();
        let bindings = MemberBindings::for_request(&QUERY_PARAMS_INPUT_SCHEMA, &pattern).unwrap();

        assert!(matches!(
            &bindings.as_slice()[0],
            MemberBinding::QueryParams { .. }
        ));
    }

    #[test]
    fn request_bindings_header() {
        smithy!("test.binding#HeaderInput": {
            structure HEADER_INPUT_SCHEMA {
                @HTTPHeaderTrait::new("x-api-key");
                API_KEY: STRING = "api_key"
            }
        });

        let pattern = UriPattern::parse("/").unwrap();
        let bindings = MemberBindings::for_request(&HEADER_INPUT_SCHEMA, &pattern).unwrap();

        assert!(matches!(
            &bindings.as_slice()[0],
            MemberBinding::Header { name, .. } if name.as_str() == "x-api-key"
        ));
    }

    #[test]
    fn request_bindings_payload() {
        smithy!("test.binding#PayloadInput": {
            structure PAYLOAD_INPUT_SCHEMA {
                @HttpPayloadTrait;
                DATA: STRING = "data"
            }
        });

        let pattern = UriPattern::parse("/").unwrap();
        let bindings = MemberBindings::for_request(&PAYLOAD_INPUT_SCHEMA, &pattern).unwrap();

        assert!(bindings.payload_member().is_some());
        assert!(matches!(
            &bindings.as_slice()[0],
            MemberBinding::Payload { .. }
        ));
    }

    #[test]
    fn request_bindings_body_default() {
        smithy!("test.binding#BodyInput": {
            structure BODY_INPUT_SCHEMA {
                NAME: STRING = "name"
                AGE: INTEGER = "age"
            }
        });

        let pattern = UriPattern::parse("/").unwrap();
        let bindings = MemberBindings::for_request(&BODY_INPUT_SCHEMA, &pattern).unwrap();

        assert_eq!(bindings.body_members().len(), 2);
        assert!(bindings.payload_member().is_none());
        assert!(matches!(
            &bindings.as_slice()[0],
            MemberBinding::Body { .. }
        ));
    }

    #[test]
    fn request_bindings_missing_label_in_pattern_is_error() {
        smithy!("test.binding#MissingLabelInput": {
            structure MISSING_LABEL_INPUT_SCHEMA {
                @HttpLabelTrait;
                ID: STRING = "id"
            }
        });

        // Pattern has no labels, but schema member has @httpLabel
        let pattern = UriPattern::parse("/items").unwrap();
        assert!(MemberBindings::for_request(&MISSING_LABEL_INPUT_SCHEMA, &pattern).is_err());
    }

    #[test]
    fn request_bindings_empty_schema() {
        smithy!("test.binding#EmptyInput": {
            structure EMPTY_INPUT_SCHEMA {}
        });

        let pattern = UriPattern::parse("/").unwrap();
        let bindings = MemberBindings::for_request(&EMPTY_INPUT_SCHEMA, &pattern).unwrap();

        assert!(bindings.is_empty());
        assert_eq!(bindings.len(), 0);
    }

    // --- MemberBindings::for_response ---

    #[test]
    fn response_bindings_status_code() {
        smithy!("test.binding#StatusOutput": {
            structure STATUS_OUTPUT_SCHEMA {
                @HTTPResponseCodeTrait;
                CODE: INTEGER = "code"
            }
        });

        let bindings = MemberBindings::for_response(&STATUS_OUTPUT_SCHEMA).unwrap();

        assert!(matches!(
            &bindings.as_slice()[0],
            MemberBinding::StatusCode { .. }
        ));
    }

    #[test]
    fn response_bindings_header_and_body() {
        smithy!("test.binding#MixedOutput": {
            structure MIXED_OUTPUT_SCHEMA {
                @HTTPHeaderTrait::new("x-request-id");
                REQUEST_ID: STRING = "request_id"
                DATA: STRING = "data"
            }
        });

        let bindings = MemberBindings::for_response(&MIXED_OUTPUT_SCHEMA).unwrap();

        assert_eq!(bindings.len(), 2);
        assert!(matches!(
            &bindings.as_slice()[0],
            MemberBinding::Header { name, .. } if name.as_str() == "x-request-id"
        ));
        assert!(matches!(
            &bindings.as_slice()[1],
            MemberBinding::Body { .. }
        ));
        assert_eq!(bindings.body_members().len(), 1);
    }

    #[test]
    fn response_bindings_prefix_headers_lowercased() {
        smithy!("test.binding#PrefixOutput": {
            structure PREFIX_OUTPUT_SCHEMA {
                @HTTPPrefixHeadersTrait::new("X-Meta-");
                META: STRING_MAP_SCHEMA = "meta"
            }
        });

        let bindings = MemberBindings::for_response(&PREFIX_OUTPUT_SCHEMA).unwrap();

        assert!(matches!(
            &bindings.as_slice()[0],
            MemberBinding::PrefixHeaders { prefix, .. } if prefix == "x-meta-"
        ));
    }

    #[test]
    fn bindings_preserve_declaration_order() {
        smithy!("test.binding#OrderOutput": {
            structure ORDER_OUTPUT_SCHEMA {
                @HTTPHeaderTrait::new("x-first");
                FIRST: STRING = "first"
                @HTTPResponseCodeTrait;
                CODE: INTEGER = "code"
                DATA: STRING = "data"
            }
        });

        let bindings = MemberBindings::for_response(&ORDER_OUTPUT_SCHEMA).unwrap();

        assert_eq!(bindings.len(), 3);
        assert!(matches!(
            &bindings.as_slice()[0],
            MemberBinding::Header { .. }
        ));
        assert!(matches!(
            &bindings.as_slice()[1],
            MemberBinding::StatusCode { .. }
        ));
        assert!(matches!(
            &bindings.as_slice()[2],
            MemberBinding::Body { .. }
        ));
    }

    #[test]
    fn body_members_excludes_all_bound_types() {
        smithy!("test.binding#ManyBindingsOutput": {
            structure MANY_BINDINGS_OUTPUT_SCHEMA {
                @HTTPHeaderTrait::new("etag");
                ETAG: STRING = "etag"
                @HTTPResponseCodeTrait;
                STATUS_CODE: INTEGER = "status_code"
                @HTTPPrefixHeadersTrait::new("x-meta-");
                METADATA: STRING_MAP_SCHEMA = "metadata"
                DATA: STRING = "data"
            }
        });

        let bindings = MemberBindings::for_response(&MANY_BINDINGS_OUTPUT_SCHEMA).unwrap();

        assert_eq!(bindings.len(), 4);
        // Only the unbound "data" member should be in body_members
        assert_eq!(bindings.body_members().len(), 1);
        assert!(bindings.payload_member().is_none());
    }

    // --- Stub types for OperationBinding tests ---

    smithy!("test.binding.ops#TestEmpty": {
        structure TEST_EMPTY_SCHEMA {}
    });

    #[derive(SmithyShape, Clone, PartialEq)]
    #[smithy_schema(TEST_EMPTY_SCHEMA)]
    pub struct TestEmpty {}
    smithy!("test.binding.ops#GetOpInput": {
        structure GET_OP_INPUT_SCHEMA {
            @HttpLabelTrait;
            ID: STRING = "id"
        }
    });

    smithy!("test.binding.ops#GetOp": {
        @HttpTrait::new("GET", "/items/{id}", 200);
        operation GetOp {
            input: GET_OP_INPUT_SCHEMA
            output: TEST_EMPTY_SCHEMA
        }
    });

    struct GetOp;
    impl StaticSchemaShape for GetOp {
        fn schema() -> &'static Schema {
            &GET_OP_SCHEMA
        }
    }
    impl OperationShape for GetOp {
        type Input = TestEmpty;
        type Output = TestEmpty;
    }
    smithy!("test.binding.ops#PostOp": {
        @HttpTrait::new("POST", "/items", 201);
        operation PostOp {
            input: TEST_EMPTY_SCHEMA
            output: TEST_EMPTY_SCHEMA
        }
    });

    struct PostOp;
    impl StaticSchemaShape for PostOp {
        fn schema() -> &'static Schema {
            &POST_OP_SCHEMA
        }
    }
    impl OperationShape for PostOp {
        type Input = TestEmpty;
        type Output = TestEmpty;
    }
    smithy!("test.binding.ops#PutOp": {
        @HttpTrait::new("PUT", "/items", 200);
        operation PutOp {
            input: TEST_EMPTY_SCHEMA
            output: TEST_EMPTY_SCHEMA
        }
    });

    struct PutOp;
    impl StaticSchemaShape for PutOp {
        fn schema() -> &'static Schema {
            &PUT_OP_SCHEMA
        }
    }
    impl OperationShape for PutOp {
        type Input = TestEmpty;
        type Output = TestEmpty;
    }
    smithy!("test.binding.ops#MultiInput": {
        structure MULTI_INPUT_SCHEMA {
            @HttpLabelTrait;
            BUCKET: STRING = "bucket"
            @HttpLabelTrait;
            ITEM_ID: STRING = "item_id"
            @HTTPHeaderTrait::new("authorization");
            AUTH: STRING = "auth"
            @HTTPQueryTrait::new("limit");
            LIMIT: INTEGER = "limit"
            @HTTPQueryParamsTrait;
            EXTRAS: STRING_MAP_SCHEMA = "extras"
            BODY_FIELD: STRING = "body_field"
        }
    });

    smithy!("test.binding.ops#MultiOutput": {
        structure MULTI_OUTPUT_SCHEMA {
            @HTTPHeaderTrait::new("etag");
            ETAG: STRING = "etag"
            @HTTPResponseCodeTrait;
            STATUS_CODE: INTEGER = "status_code"
            @HTTPPrefixHeadersTrait::new("x-meta-");
            METADATA: STRING_MAP_SCHEMA = "metadata"
            DATA: STRING = "data"
        }
    });

    smithy!("test.binding.ops#MultiBindingsOp": {
        @HttpTrait::new("PUT", "/buckets/{bucket}/items/{item_id}", 200);
        operation MultiBindingsOp {
            input: MULTI_INPUT_SCHEMA
            output: MULTI_OUTPUT_SCHEMA
        }
    });

    struct MultiBindingsOp;
    impl StaticSchemaShape for MultiBindingsOp {
        fn schema() -> &'static Schema {
            &MULTI_BINDINGS_OP_SCHEMA
        }
    }
    impl OperationShape for MultiBindingsOp {
        type Input = TestEmpty;
        type Output = TestEmpty;
    }
    smithy!("test.binding.ops#QueryResInput": {
        structure QUERY_RES_INPUT_SCHEMA {
            @HTTPQueryTrait::new("pageSize");
            PAGE_SIZE: INTEGER = "page_size"
            @HTTPQueryTrait::new("nextToken");
            NEXT_TOKEN: STRING = "next_token"
        }
    });

    smithy!("test.binding.ops#QueryResOutput": {
        structure QUERY_RES_OUTPUT_SCHEMA {
            @HTTPHeaderTrait::new("x-next-token");
            NEXT_TOKEN: STRING = "next_token"
            ITEMS: STRING = "items"
        }
    });

    smithy!("test.binding.ops#QueryResOp": {
        @HttpTrait::new("GET", "/items", 200);
        operation QueryResOp {
            input: QUERY_RES_INPUT_SCHEMA
            output: QUERY_RES_OUTPUT_SCHEMA
        }
    });

    struct QueryResOp;
    impl StaticSchemaShape for QueryResOp {
        fn schema() -> &'static Schema {
            &QUERY_RES_OP_SCHEMA
        }
    }
    impl OperationShape for QueryResOp {
        type Input = TestEmpty;
        type Output = TestEmpty;
    }
    smithy!("test.binding.ops#NoHttpOp": {
        operation NoHttpOp {
            input: TEST_EMPTY_SCHEMA
            output: TEST_EMPTY_SCHEMA
        }
    });

    struct NoHttpOp;
    impl StaticSchemaShape for NoHttpOp {
        fn schema() -> &'static Schema {
            &NO_HTTP_OP_SCHEMA
        }
    }
    impl OperationShape for NoHttpOp {
        type Input = TestEmpty;
        type Output = TestEmpty;
    }

    // --- OperationBinding::from_operation ---

    #[test]
    fn operation_binding_extracts_method_and_status() {
        let binding = OperationBinding::from_operation(&GetOp).unwrap();
        assert_eq!(binding.method, http::Method::GET);
        assert_eq!(binding.default_status, 200);
    }

    #[test]
    fn operation_binding_post_method() {
        let binding = OperationBinding::from_operation(&PostOp).unwrap();
        assert_eq!(binding.method, http::Method::POST);
        assert_eq!(binding.default_status, 201);
    }

    #[test]
    fn operation_binding_put_method() {
        let binding = OperationBinding::from_operation(&PutOp).unwrap();
        assert_eq!(binding.method, http::Method::PUT);
    }

    #[test]
    fn operation_binding_parses_uri_pattern() {
        let binding = OperationBinding::from_operation(&MultiBindingsOp).unwrap();
        assert_eq!(binding.uri_pattern.segments().len(), 4);
    }

    #[test]
    fn operation_binding_request_response_schemas_resolved() {
        let binding = OperationBinding::from_operation(&QueryResOp).unwrap();

        // Request: page_size (query), next_token (query)
        assert_eq!(binding.request.len(), 2);
        assert_eq!(binding.request.query_keys().len(), 2);

        // Response: next_token (header), items (body)
        assert_eq!(binding.response.len(), 2);
        assert_eq!(binding.response.body_members().len(), 1);
    }

    #[test]
    fn operation_binding_request_all_types() {
        let binding = OperationBinding::from_operation(&MultiBindingsOp).unwrap();
        let req = &binding.request;

        assert_eq!(req.len(), 6);
        assert_eq!(req.label_members().len(), 2);
        assert_eq!(req.body_members().len(), 1);
        assert!(req.payload_member().is_none());
        assert_eq!(req.query_keys(), &["limit"]);
    }

    #[test]
    fn operation_binding_response_all_types() {
        let binding = OperationBinding::from_operation(&MultiBindingsOp).unwrap();
        let resp = &binding.response;

        assert_eq!(resp.len(), 4);
        assert_eq!(resp.body_members().len(), 1);
    }

    #[test]
    fn operation_missing_http_trait_is_error() {
        let result = OperationBinding::from_operation(&NoHttpOp);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("@http"),
            "error should mention missing @http trait: {err}"
        );
    }

    // --- HttpBinding caching ---

    fn http_binding() -> HttpBinding<'static, JsonCodec> {
        HttpBinding::new(JsonCodec)
    }

    #[test]
    fn binding_cache_returns_same_arc() {
        let b = http_binding();
        let first = b.get_operation_binding(&GetOp).unwrap();
        let second = b.get_operation_binding(&GetOp).unwrap();

        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn binding_cache_separate_operations() {
        let b = http_binding();
        let get = b.get_operation_binding(&GetOp).unwrap();
        let post = b.get_operation_binding(&PostOp).unwrap();

        assert!(!Arc::ptr_eq(&get, &post));
    }

    #[test]
    fn cloned_binding_shares_cache() {
        let b = http_binding();
        let b2 = b.clone();

        // Populate cache via first binding
        let _ = b.get_operation_binding(&GetOp).unwrap();

        // Second binding should see the cached entry
        let from_clone = b2.get_operation_binding(&GetOp).unwrap();
        let from_original = b.get_operation_binding(&GetOp).unwrap();

        assert!(Arc::ptr_eq(&from_clone, &from_original));
    }
}
