use smithy4rs_core::{
    IndexMap,
    derive::SmithyShape,
    schema::{
        OperationShape, Schema, StaticSchemaShape,
        prelude::{
            HTTPHeaderTrait, HTTPPrefixHeadersTrait, HTTPQueryParamsTrait, HTTPQueryTrait,
            HTTPResponseCodeTrait, HttpLabelTrait, HttpTrait, INTEGER, STRING,
        },
    },
    smithy,
};
pub use smithy4rs_http::binding::HttpBinding;
pub use smithy4rs_json_codec::JsonCodec;
pub use smithy4rs_test_utils::{STRING_LIST_SCHEMA, STRING_MAP_SCHEMA};

// --- Roundtrip helpers ---

/// Create a test HTTP binding with JSON codec.
pub fn binding() -> HttpBinding<'static, JsonCodec> {
    HttpBinding::new(JsonCodec)
}

/// Serialize then deserialize an HTTP request, returning the reconstructed input.
pub fn request_roundtrip<Op: smithy4rs_core::schema::Operation>(
    binding: &HttpBinding<'static, JsonCodec>,
    op: &'static Op,
    input: &Op::Input,
) -> Op::Input
where
    Op::Input: std::fmt::Debug + PartialEq,
{
    let request = binding.serialize_request(op, input).unwrap();
    let body = request.body().clone();
    let request = http::Request::builder()
        .method(request.method().clone())
        .uri(request.uri().clone())
        .body(body.as_slice())
        .unwrap();
    // Copy headers
    // Note: headers aren't copied by the builder; use a full rebuild
    let (mut parts, body) = request.into_parts();
    let orig_headers = binding
        .serialize_request(op, input)
        .unwrap()
        .into_parts()
        .0
        .headers;
    parts.headers = orig_headers;
    let request = http::Request::from_parts(parts, &body[..]);
    binding.deserialize_request(op, &request).unwrap()
}

/// Serialize then deserialize an HTTP response, returning the reconstructed output.
pub fn response_roundtrip<Op: smithy4rs_core::schema::Operation>(
    binding: &HttpBinding<'static, JsonCodec>,
    op: &'static Op,
    output: &Op::Output,
) -> Op::Output
where
    Op::Output: std::fmt::Debug + PartialEq,
{
    let response = binding.serialize_response(op, output).unwrap();
    let (parts, body) = response.into_parts();
    let response = http::Response::from_parts(parts, body.as_slice());
    binding.deserialize_response(op, &response).unwrap()
}

// --- GetCity — label input, body output ---

smithy!("test.http#GetCityInput": {
    structure GET_CITY_INPUT_SCHEMA {
        @HttpLabelTrait;
        CITY_ID: STRING = "city_id"
    }
});

smithy!("test.http#GetCityOutput": {
    structure GET_CITY_OUTPUT_SCHEMA {
        NAME: STRING = "name"
        POPULATION: INTEGER = "population"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(GET_CITY_INPUT_SCHEMA)]
pub struct GetCityInput {
    #[smithy_schema(CITY_ID)]
    pub city_id: String,
}

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(GET_CITY_OUTPUT_SCHEMA)]
pub struct GetCityOutput {
    #[smithy_schema(NAME)]
    pub name: String,
    #[smithy_schema(POPULATION)]
    pub population: i32,
}

smithy!("test.http#GetCity": {
    @HttpTrait::new("GET", "/cities/{city_id}", 200);
    operation GetCityOp {
        input: GET_CITY_INPUT_SCHEMA
        output: GET_CITY_OUTPUT_SCHEMA
    }
});

pub struct GetCityOp;
impl StaticSchemaShape for GetCityOp {
    fn schema() -> &'static Schema {
        &GET_CITY_OP_SCHEMA
    }
}
impl OperationShape for GetCityOp {
    type Input = GetCityInput;
    type Output = GetCityOutput;
}

// --- ListCities — query input, header + body output ---

smithy!("test.http#ListCitiesInput": {
    structure LIST_CITIES_INPUT_SCHEMA {
        @HTTPQueryTrait::new("pageSize");
        PAGE_SIZE: INTEGER = "page_size"
        @HTTPQueryTrait::new("nextToken");
        NEXT_TOKEN: STRING = "next_token"
    }
});

smithy!("test.http#ListCitiesOutput": {
    structure LIST_CITIES_OUTPUT_SCHEMA {
        @HTTPHeaderTrait::new("x-next-token");
        NEXT_TOKEN: STRING = "next_token"
        ITEMS: STRING_LIST_SCHEMA = "items"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(LIST_CITIES_INPUT_SCHEMA)]
pub struct ListCitiesInput {
    #[smithy_schema(PAGE_SIZE)]
    pub page_size: Option<i32>,
    #[smithy_schema(NEXT_TOKEN)]
    pub next_token: Option<String>,
}

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(LIST_CITIES_OUTPUT_SCHEMA)]
pub struct ListCitiesOutput {
    #[smithy_schema(NEXT_TOKEN)]
    pub next_token: Option<String>,
    #[smithy_schema(ITEMS)]
    pub items: Vec<String>,
}

smithy!("test.http#ListCities": {
    @HttpTrait::new("GET", "/cities", 200);
    operation ListCitiesOp {
        input: LIST_CITIES_INPUT_SCHEMA
        output: LIST_CITIES_OUTPUT_SCHEMA
    }
});

pub struct ListCitiesOp;
impl StaticSchemaShape for ListCitiesOp {
    fn schema() -> &'static Schema {
        &LIST_CITIES_OP_SCHEMA
    }
}
impl OperationShape for ListCitiesOp {
    type Input = ListCitiesInput;
    type Output = ListCitiesOutput;
}

// --- CreateCity — body input, header + status code output ---

smithy!("test.http#CreateCityInput": {
    structure CREATE_CITY_INPUT_SCHEMA {
        NAME: STRING = "name"
        POPULATION: INTEGER = "population"
    }
});

smithy!("test.http#CreateCityOutput": {
    structure CREATE_CITY_OUTPUT_SCHEMA {
        @HTTPHeaderTrait::new("x-city-id");
        CITY_ID: STRING = "city_id"
        @HTTPResponseCodeTrait;
        STATUS_CODE: INTEGER = "status_code"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(CREATE_CITY_INPUT_SCHEMA)]
pub struct CreateCityInput {
    #[smithy_schema(NAME)]
    pub name: String,
    #[smithy_schema(POPULATION)]
    pub population: i32,
}

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(CREATE_CITY_OUTPUT_SCHEMA)]
pub struct CreateCityOutput {
    #[smithy_schema(CITY_ID)]
    pub city_id: Option<String>,
    #[smithy_schema(STATUS_CODE)]
    pub status_code: Option<i32>,
}

smithy!("test.http#CreateCity": {
    @HttpTrait::new("POST", "/cities", 201);
    operation CreateCityOp {
        input: CREATE_CITY_INPUT_SCHEMA
        output: CREATE_CITY_OUTPUT_SCHEMA
    }
});

pub struct CreateCityOp;
impl StaticSchemaShape for CreateCityOp {
    fn schema() -> &'static Schema {
        &CREATE_CITY_OP_SCHEMA
    }
}
impl OperationShape for CreateCityOp {
    type Input = CreateCityInput;
    type Output = CreateCityOutput;
}

// --- HeaderList — list-valued header ---

smithy!("test.http#HeaderListInput": {
    structure HEADER_LIST_INPUT_SCHEMA {
        @HTTPHeaderTrait::new("x-tags");
        TAGS: STRING_LIST_SCHEMA = "tags"
    }
});

smithy!("test.http#HeaderListOutput": {
    structure HEADER_LIST_OUTPUT_SCHEMA {
        @HTTPHeaderTrait::new("x-tags");
        TAGS: STRING_LIST_SCHEMA = "tags"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(HEADER_LIST_INPUT_SCHEMA)]
pub struct HeaderListInput {
    #[smithy_schema(TAGS)]
    pub tags: Vec<String>,
}

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(HEADER_LIST_OUTPUT_SCHEMA)]
pub struct HeaderListOutput {
    #[smithy_schema(TAGS)]
    pub tags: Vec<String>,
}

smithy!("test.http#HeaderList": {
    @HttpTrait::new("GET", "/header-list", 200);
    operation HeaderListOp {
        input: HEADER_LIST_INPUT_SCHEMA
        output: HEADER_LIST_OUTPUT_SCHEMA
    }
});

pub struct HeaderListOp;
impl StaticSchemaShape for HeaderListOp {
    fn schema() -> &'static Schema {
        &HEADER_LIST_OP_SCHEMA
    }
}
impl OperationShape for HeaderListOp {
    type Input = HeaderListInput;
    type Output = HeaderListOutput;
}

// --- PrefixHeaders — prefix headers ---

smithy!("test.http#PrefixHeadersInput": {
    structure PREFIX_HEADERS_INPUT_SCHEMA {
        @HTTPPrefixHeadersTrait::new("x-meta-");
        METADATA: STRING_MAP_SCHEMA = "metadata"
    }
});

smithy!("test.http#PrefixHeadersOutput": {
    structure PREFIX_HEADERS_OUTPUT_SCHEMA {
        @HTTPPrefixHeadersTrait::new("x-meta-");
        METADATA: STRING_MAP_SCHEMA = "metadata"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(PREFIX_HEADERS_INPUT_SCHEMA)]
pub struct PrefixHeadersInput {
    #[smithy_schema(METADATA)]
    pub metadata: IndexMap<String, String>,
}

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(PREFIX_HEADERS_OUTPUT_SCHEMA)]
pub struct PrefixHeadersOutput {
    #[smithy_schema(METADATA)]
    pub metadata: IndexMap<String, String>,
}

smithy!("test.http#PrefixHeaders": {
    @HttpTrait::new("PUT", "/prefix-headers", 200);
    operation PrefixHeadersOp {
        input: PREFIX_HEADERS_INPUT_SCHEMA
        output: PREFIX_HEADERS_OUTPUT_SCHEMA
    }
});

pub struct PrefixHeadersOp;
impl StaticSchemaShape for PrefixHeadersOp {
    fn schema() -> &'static Schema {
        &PREFIX_HEADERS_OP_SCHEMA
    }
}
impl OperationShape for PrefixHeadersOp {
    type Input = PrefixHeadersInput;
    type Output = PrefixHeadersOutput;
}

// --- QueryParams — httpQuery + httpQueryParams ---

smithy!("test.http#QueryParamsInput": {
    structure QUERY_PARAMS_INPUT_SCHEMA {
        @HTTPQueryTrait::new("status");
        STATUS: STRING = "status"
        @HTTPQueryParamsTrait;
        EXTRA_PARAMS: STRING_MAP_SCHEMA = "extra_params"
    }
});

smithy!("test.http#QueryParamsOutput": {
    structure QUERY_PARAMS_OUTPUT_SCHEMA {}
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(QUERY_PARAMS_INPUT_SCHEMA)]
pub struct QueryParamsInput {
    #[smithy_schema(STATUS)]
    pub status: Option<String>,
    #[smithy_schema(EXTRA_PARAMS)]
    pub extra_params: IndexMap<String, String>,
}

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(QUERY_PARAMS_OUTPUT_SCHEMA)]
pub struct QueryParamsOutput {}

smithy!("test.http#QueryParams": {
    @HttpTrait::new("GET", "/query-params", 200);
    operation QueryParamsOp {
        input: QUERY_PARAMS_INPUT_SCHEMA
        output: QUERY_PARAMS_OUTPUT_SCHEMA
    }
});

pub struct QueryParamsOp;
impl StaticSchemaShape for QueryParamsOp {
    fn schema() -> &'static Schema {
        &QUERY_PARAMS_OP_SCHEMA
    }
}
impl OperationShape for QueryParamsOp {
    type Input = QueryParamsInput;
    type Output = QueryParamsOutput;
}

// --- AllBindings — covers every HTTP binding type ---

smithy!("test.http#AllBindingsInput": {
    structure ALL_BINDINGS_INPUT_SCHEMA {
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

smithy!("test.http#AllBindingsOutput": {
    structure ALL_BINDINGS_OUTPUT_SCHEMA {
        @HTTPHeaderTrait::new("etag");
        ETAG: STRING = "etag"
        @HTTPResponseCodeTrait;
        STATUS_CODE: INTEGER = "status_code"
        @HTTPPrefixHeadersTrait::new("x-meta-");
        METADATA: STRING_MAP_SCHEMA = "metadata"
        DATA: STRING = "data"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(ALL_BINDINGS_INPUT_SCHEMA)]
pub struct AllBindingsInput {
    #[smithy_schema(BUCKET)]
    pub bucket: String,
    #[smithy_schema(ITEM_ID)]
    pub item_id: String,
    #[smithy_schema(AUTH)]
    pub auth: Option<String>,
    #[smithy_schema(LIMIT)]
    pub limit: Option<i32>,
    #[smithy_schema(EXTRAS)]
    pub extras: IndexMap<String, String>,
    #[smithy_schema(BODY_FIELD)]
    pub body_field: String,
}

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(ALL_BINDINGS_OUTPUT_SCHEMA)]
pub struct AllBindingsOutput {
    #[smithy_schema(ETAG)]
    pub etag: Option<String>,
    #[smithy_schema(STATUS_CODE)]
    pub status_code: Option<i32>,
    #[smithy_schema(METADATA)]
    pub metadata: IndexMap<String, String>,
    #[smithy_schema(DATA)]
    pub data: String,
}

smithy!("test.http#AllBindings": {
    @HttpTrait::new("PUT", "/buckets/{bucket}/items/{item_id}", 200);
    operation AllBindingsOp {
        input: ALL_BINDINGS_INPUT_SCHEMA
        output: ALL_BINDINGS_OUTPUT_SCHEMA
    }
});

pub struct AllBindingsOp;
impl StaticSchemaShape for AllBindingsOp {
    fn schema() -> &'static Schema {
        &ALL_BINDINGS_OP_SCHEMA
    }
}
impl OperationShape for AllBindingsOp {
    type Input = AllBindingsInput;
    type Output = AllBindingsOutput;
}
