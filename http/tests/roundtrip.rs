mod common;

use common::*;
use smithy4rs_core::IndexMap;

#[test]
fn get_city_request_roundtrip() {
    let b = binding();
    let input = GetCityInput {
        city_id: "seattle".into(),
    };
    assert_eq!(request_roundtrip(&b, &GetCityOp, &input), input);
}

#[test]
fn get_city_response_roundtrip() {
    let b = binding();
    let output = GetCityOutput {
        name: "Seattle".into(),
        population: 750_000,
    };
    assert_eq!(response_roundtrip(&b, &GetCityOp, &output), output);
}

#[test]
fn get_city_request_uri_has_label() {
    let b = binding();
    let input = GetCityInput {
        city_id: "new-york".into(),
    };
    let request = b.serialize_request(&GetCityOp, &input).unwrap();
    assert_eq!(request.uri().path(), "/cities/new-york");
    assert_eq!(request.method(), http::Method::GET);
}

#[test]
fn get_city_label_percent_encoding() {
    let b = binding();
    let input = GetCityInput {
        city_id: "san francisco".into(),
    };
    let request = b.serialize_request(&GetCityOp, &input).unwrap();
    assert_eq!(request.uri().path(), "/cities/san%20francisco");

    // Roundtrip: percent-encoded label should decode back
    assert_eq!(request_roundtrip(&b, &GetCityOp, &input), input);
}

#[test]
fn get_city_label_utf8_roundtrip() {
    let b = binding();
    let input = GetCityInput {
        city_id: "caf\u{00e9}".into(), // café
    };
    assert_eq!(request_roundtrip(&b, &GetCityOp, &input), input);
}

#[test]
fn list_cities_request_roundtrip() {
    let b = binding();
    let input = ListCitiesInput {
        page_size: Some(10),
        next_token: Some("abc123".into()),
    };
    assert_eq!(request_roundtrip(&b, &ListCitiesOp, &input), input);
}

#[test]
fn list_cities_optional_query_absent() {
    let b = binding();
    let input = ListCitiesInput {
        page_size: None,
        next_token: None,
    };
    let request = b.serialize_request(&ListCitiesOp, &input).unwrap();
    assert_eq!(request.uri().query(), None);
    assert_eq!(request_roundtrip(&b, &ListCitiesOp, &input), input);
}

#[test]
fn list_cities_response_roundtrip() {
    let b = binding();
    let output = ListCitiesOutput {
        next_token: Some("page2".into()),
        items: vec!["Seattle".into(), "Portland".into()],
    };
    assert_eq!(response_roundtrip(&b, &ListCitiesOp, &output), output);
}

#[test]
fn list_cities_response_header_absent() {
    let b = binding();
    let output = ListCitiesOutput {
        next_token: None,
        items: vec!["Seattle".into()],
    };
    let response = b.serialize_response(&ListCitiesOp, &output).unwrap();
    assert!(!response.headers().contains_key("x-next-token"));
}

#[test]
fn create_city_request_roundtrip() {
    let b = binding();
    let input = CreateCityInput {
        name: "Austin".into(),
        population: 1_000_000,
    };
    assert_eq!(request_roundtrip(&b, &CreateCityOp, &input), input);
}

#[test]
fn create_city_request_has_body() {
    let b = binding();
    let input = CreateCityInput {
        name: "Austin".into(),
        population: 1_000_000,
    };
    let request = b.serialize_request(&CreateCityOp, &input).unwrap();
    assert_eq!(request.method(), http::Method::POST);
    assert!(!request.body().is_empty());
    assert_eq!(
        request.headers().get("content-type").unwrap(),
        "application/json"
    );
}

#[test]
fn create_city_response_roundtrip() {
    let b = binding();
    let output = CreateCityOutput {
        city_id: Some("city-123".into()),
        status_code: Some(201),
    };
    assert_eq!(response_roundtrip(&b, &CreateCityOp, &output), output);
}

#[test]
fn create_city_default_status_code() {
    let b = binding();
    let output = CreateCityOutput {
        city_id: Some("city-123".into()),
        status_code: None,
    };
    let response = b.serialize_response(&CreateCityOp, &output).unwrap();
    // Default from @http trait is 201
    assert_eq!(response.status(), http::StatusCode::CREATED);
}

#[test]
fn prefix_headers_request_roundtrip() {
    let b = binding();
    let input = PrefixHeadersInput {
        metadata: IndexMap::from_iter([
            ("key1".into(), "value1".into()),
            ("key2".into(), "value2".into()),
        ]),
    };
    assert_eq!(request_roundtrip(&b, &PrefixHeadersOp, &input), input);
}

#[test]
fn prefix_headers_response_roundtrip() {
    let b = binding();
    let output = PrefixHeadersOutput {
        metadata: IndexMap::from_iter([("foo".into(), "bar".into())]),
    };
    assert_eq!(response_roundtrip(&b, &PrefixHeadersOp, &output), output);
}

#[test]
fn prefix_headers_serialized_as_prefixed() {
    let b = binding();
    let input = PrefixHeadersInput {
        metadata: IndexMap::from_iter([("region".into(), "us-west-2".into())]),
    };
    let request = b.serialize_request(&PrefixHeadersOp, &input).unwrap();
    assert_eq!(request.headers().get("x-meta-region").unwrap(), "us-west-2");
}

#[test]
fn query_params_request_roundtrip() {
    let b = binding();
    let input = QueryParamsInput {
        status: Some("active".into()),
        extra_params: IndexMap::from_iter([("color".into(), "blue".into())]),
    };
    assert_eq!(request_roundtrip(&b, &QueryParamsOp, &input), input);
}

#[test]
fn query_params_filtering() {
    let b = binding();

    // Build a request with "status" appearing in both @httpQuery and extra params
    let request = http::Request::builder()
        .uri("/query-params?status=active&color=blue")
        .body(b"".as_slice())
        .unwrap();
    let decoded: QueryParamsInput = b.deserialize_request(&QueryParamsOp, &request).unwrap();
    // @httpQuery "status" should be "active"
    assert_eq!(decoded.status, Some("active".into()));
    // @httpQueryParams should NOT include "status" keys
    assert!(!decoded.extra_params.contains_key("status"));
    assert_eq!(decoded.extra_params.get("color").unwrap(), "blue");
}

#[test]
fn all_bindings_request_roundtrip() {
    let b = binding();
    let input = AllBindingsInput {
        bucket: "my-bucket".into(),
        item_id: "item-42".into(),
        auth: Some("Bearer token123".into()),
        limit: Some(100),
        extras: IndexMap::from_iter([("format".into(), "json".into())]),
        body_field: "hello world".into(),
    };
    assert_eq!(request_roundtrip(&b, &AllBindingsOp, &input), input);
}

#[test]
fn all_bindings_response_roundtrip() {
    let b = binding();
    let output = AllBindingsOutput {
        etag: Some("\"abc123\"".into()),
        status_code: Some(200),
        metadata: IndexMap::from_iter([("version".into(), "2".into())]),
        data: "response data".into(),
    };
    assert_eq!(response_roundtrip(&b, &AllBindingsOp, &output), output);
}

#[test]
fn all_bindings_request_structure() {
    let b = binding();
    let input = AllBindingsInput {
        bucket: "b1".into(),
        item_id: "i2".into(),
        auth: Some("Bearer tok".into()),
        limit: Some(50),
        extras: IndexMap::from_iter([("x".into(), "y".into())]),
        body_field: "data".into(),
    };
    let request = b.serialize_request(&AllBindingsOp, &input).unwrap();
    assert_eq!(request.method(), http::Method::PUT);
    assert_eq!(request.uri().path(), "/buckets/b1/items/i2");
    assert_eq!(
        request.headers().get("authorization").unwrap(),
        "Bearer tok"
    );
    let query = request.uri().query().unwrap();
    assert!(query.contains("limit=50"));
    assert!(query.contains("x=y"));
    assert!(!request.body().is_empty());
}

#[test]
fn header_list_request_roundtrip() {
    let b = binding();
    let input = HeaderListInput {
        tags: vec!["alpha".into(), "beta".into(), "gamma".into()],
    };
    assert_eq!(request_roundtrip(&b, &HeaderListOp, &input), input);
}

#[test]
fn header_list_response_roundtrip() {
    let b = binding();
    let output = HeaderListOutput {
        tags: vec!["one".into(), "two".into()],
    };
    assert_eq!(response_roundtrip(&b, &HeaderListOp, &output), output);
}

#[test]
fn header_list_comma_in_value() {
    let b = binding();
    let input = HeaderListInput {
        tags: vec!["a, b".into(), "c".into()],
    };
    let request = b.serialize_request(&HeaderListOp, &input).unwrap();
    // Value with comma should be quoted
    let header_val = request.headers().get("x-tags").unwrap().to_str().unwrap();
    assert!(header_val.contains("\"a, b\""));

    // Roundtrip should preserve the values
    assert_eq!(request_roundtrip(&b, &HeaderListOp, &input), input);
}
