use smithy4rs_core::serde::deserializers::DeserializeWithSchema;
use smithy4rs_json_codec::JsonDeserializer;
use smithy4rs_test_utils::*;

#[test]
fn test_empty_list_at_top_level() {
    let json = b"[]";
    let mut de = JsonDeserializer::new(json);
    let result: Vec<i32> = Vec::deserialize_with_schema(&INTEGER_LIST_SCHEMA, &mut de).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_empty_struct_fields() {
    // Struct with empty list and empty map
    let json = r#"{
        "name": "empty_test",
        "count": 0,
        "singleNested": {
            "fieldA": "a",
            "fieldB": "b",
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

    assert_eq!(result.name, "empty_test");
    assert!(result.list_nested.is_empty());
    assert!(result.map_nested.is_empty());
}

#[test]
fn test_empty_nested_struct_in_list() {
    // List containing structs (but list is empty)
    let json = r#"{
        "name": "test",
        "count": 1,
        "singleNested": {"fieldA": "a", "fieldB": "b", "fieldC": "c"},
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

    assert!(result.list_nested.is_empty());
}
