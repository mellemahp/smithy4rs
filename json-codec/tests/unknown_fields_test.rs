use smithy4rs_core::serde::deserializers::DeserializeWithSchema;
use smithy4rs_json_codec::JsonDeserializer;
use smithy4rs_test_utils::*;

#[test]
fn test_unknown_field_skipped() {
    // SimpleStruct only has field_a and field_b, but JSON has extra fields
    let json = br#"{"fieldA": "test", "unknown_field": "should_be_skipped", "fieldB": 42}"#;
    let mut de = JsonDeserializer::new(json);
    let result = SimpleStructBuilder::deserialize_with_schema(&SIMPLE_STRUCT_SCHEMA, &mut de)
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(result.field_a, "test");
    assert_eq!(result.field_b, 42);
}

#[test]
fn test_multiple_unknown_fields_skipped() {
    let json = br#"{
        "extra1": "ignored",
        "fieldA": "hello",
        "extra2": 999,
        "extra3": true,
        "fieldB": 123,
        "extra4": null
    }"#;
    let mut de = JsonDeserializer::new(json);
    let result = SimpleStructBuilder::deserialize_with_schema(&SIMPLE_STRUCT_SCHEMA, &mut de)
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(result.field_a, "hello");
    assert_eq!(result.field_b, 123);
}

#[test]
fn test_unknown_nested_object_skipped() {
    let json = br#"{
        "fieldA": "test",
        "nested_unknown": {"deep": {"deeper": [1, 2, 3]}},
        "fieldB": 42
    }"#;
    let mut de = JsonDeserializer::new(json);
    let result = SimpleStructBuilder::deserialize_with_schema(&SIMPLE_STRUCT_SCHEMA, &mut de)
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(result.field_a, "test");
    assert_eq!(result.field_b, 42);
}

#[test]
fn test_unknown_array_skipped() {
    let json = br#"{
        "fieldA": "test",
        "unknown_array": [1, 2, {"nested": true}, [3, 4, 5]],
        "fieldB": 42
    }"#;
    let mut de = JsonDeserializer::new(json);
    let result = SimpleStructBuilder::deserialize_with_schema(&SIMPLE_STRUCT_SCHEMA, &mut de)
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(result.field_a, "test");
    assert_eq!(result.field_b, 42);
}

#[test]
fn test_unknown_fields_in_nested_struct() {
    let json = r#"{
        "name": "outer",
        "count": 1,
        "singleNested": {
            "fieldA": "a",
            "unknown_inner": "skipped",
            "fieldB": "b",
            "another_unknown": 123,
            "fieldC": "c"
        },
        "optionalNested": null,
        "listNested": [],
        "mapNested": {}
    }"#;

    let mut de = JsonDeserializer::new(json.as_bytes());
    let result = NestedCollectionsStructBuilder::deserialize_with_schema(
        &NESTED_COLLECTIONS_STRUCT_SCHEMA,
        &mut de,
    )
    .unwrap()
    .build()
    .unwrap();

    assert_eq!(result.single_nested.field_a, "a");
    assert_eq!(result.single_nested.field_b, "b");
    assert_eq!(result.single_nested.field_c, "c");
}

#[test]
fn test_unknown_fields_at_multiple_nesting_levels() {
    let json = r#"{
        "name": "test",
        "extra_top": "ignored",
        "count": 5,
        "singleNested": {
            "fieldA": "alpha",
            "extra_inner": {"deep": true},
            "fieldB": "beta",
            "fieldC": "gamma"
        },
        "optionalNested": null,
        "listNested": [
            {
                "fieldA": "list_a",
                "extra_in_list": [1, 2, 3],
                "fieldB": "list_b",
                "fieldC": "list_c"
            }
        ],
        "extra_before_map": false,
        "mapNested": {
            "key1": {
                "fieldA": "map_a",
                "extra_in_map": null,
                "fieldB": "map_b",
                "fieldC": "map_c"
            }
        },
        "extra_end": 42
    }"#;

    let mut de = JsonDeserializer::new(json.as_bytes());
    let result = NestedCollectionsStructBuilder::deserialize_with_schema(
        &NESTED_COLLECTIONS_STRUCT_SCHEMA,
        &mut de,
    )
    .unwrap()
    .build()
    .unwrap();

    assert_eq!(result.name, "test");
    assert_eq!(result.count, 5);
    assert_eq!(result.single_nested.field_a, "alpha");
    assert_eq!(result.list_nested.len(), 1);
    assert_eq!(result.list_nested[0].field_a, "list_a");
    assert_eq!(result.map_nested.len(), 1);
    assert_eq!(result.map_nested.get("key1").unwrap().field_a, "map_a");
}
