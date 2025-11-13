use smithy4rs_core::serde::deserializers::DeserializeWithSchema;
use smithy4rs_json_codec::JsonDeserializer;
use smithy4rs_test_utils::*;

#[test]
fn test_nested_struct_deserialization() {
    // JSON matching the exact same structure as the serialization test
    let json = r#"{
        "name": "test_object",
        "count": 42,
        "single_nested": {
            "field_a": "alpha",
            "field_b": "beta",
            "field_c": "gamma"
        },
        "optional_nested": {
            "field_a": "delta",
            "field_b": "epsilon",
            "field_c": "zeta"
        },
        "list_nested": [
            {
                "field_a": "item1-a",
                "field_b": "item1-b",
                "field_c": "item1-c"
            },
            {
                "field_a": "item2-a",
                "field_b": "item2-b",
                "field_c": "item2-c"
            }
        ],
        "map_nested": {
            "key1": {
                "field_a": "value1-a",
                "field_b": "value1-b",
                "field_c": "value1-c"
            },
            "key2": {
                "field_a": "value2-a",
                "field_b": "value2-b",
                "field_c": "value2-c"
            }
        }
    }"#;

    let mut deserializer = JsonDeserializer::from_str(json);
    let nested = NestedCollectionsStruct::deserialize_with_schema(
        &NESTED_COLLECTIONS_STRUCT_SCHEMA,
        &mut deserializer,
    )
    .unwrap();

    println!("Deserialized nested struct: {:?}", nested);

    // Verify structure - matching the serialization test assertions
    assert_eq!(nested.name, "test_object");
    assert_eq!(nested.count, 42);

    // Single nested
    assert_eq!(nested.single_nested.field_a, "alpha");
    assert_eq!(nested.single_nested.field_b, "beta");
    assert_eq!(nested.single_nested.field_c, "gamma");

    // Optional nested
    assert!(nested.optional_nested.is_some());
    let optional = nested.optional_nested.as_ref().unwrap();
    assert_eq!(optional.field_a, "delta");
    assert_eq!(optional.field_b, "epsilon");
    assert_eq!(optional.field_c, "zeta");

    // List nested
    assert_eq!(nested.list_nested.len(), 2);
    assert_eq!(nested.list_nested[0].field_a, "item1-a");
    assert_eq!(nested.list_nested[0].field_b, "item1-b");
    assert_eq!(nested.list_nested[0].field_c, "item1-c");
    assert_eq!(nested.list_nested[1].field_a, "item2-a");
    assert_eq!(nested.list_nested[1].field_b, "item2-b");
    assert_eq!(nested.list_nested[1].field_c, "item2-c");

    // Map nested
    assert_eq!(nested.map_nested.len(), 2);
    let key1 = nested.map_nested.get("key1").unwrap();
    assert_eq!(key1.field_a, "value1-a");
    assert_eq!(key1.field_b, "value1-b");
    assert_eq!(key1.field_c, "value1-c");
    let key2 = nested.map_nested.get("key2").unwrap();
    assert_eq!(key2.field_a, "value2-a");
    assert_eq!(key2.field_b, "value2-b");
    assert_eq!(key2.field_c, "value2-c");
}

#[test]
fn test_recursive_struct_deserialization() {
    // JSON matching the exact recursive structure from serialization test
    let json = r#"{
        "string_field": "level_1",
        "integer_field": 1,
        "list_field": [],
        "map_field": {},
        "optional_field": "top",
        "next": {
            "string_field": "level_2",
            "integer_field": 2,
            "list_field": [],
            "map_field": {},
            "optional_field": "middle",
            "next": {
                "string_field": "level_3",
                "integer_field": 3,
                "list_field": [],
                "map_field": {},
                "optional_field": "deepest",
                "next": null
            }
        }
    }"#;

    let mut deserializer = JsonDeserializer::from_str(json);
    let child = RecursiveShapesStruct::deserialize_with_schema(
        &RECURSIVE_SHAPES_STRUCT_SCHEMA,
        &mut deserializer,
    )
    .unwrap();

    println!("Deserialized recursive struct: {:?}", child);

    // Verify recursive structure - matching the serialization test assertions
    assert_eq!(child.string_field, "level_1");
    assert_eq!(child.integer_field, 1);
    assert_eq!(child.optional_field, Some("top".to_string()));

    // Parent level
    assert!(child.next.is_some());
    let parent = child.next.as_ref().unwrap();
    assert_eq!(parent.string_field, "level_2");
    assert_eq!(parent.integer_field, 2);
    assert_eq!(parent.optional_field, Some("middle".to_string()));

    // Grandparent level
    assert!(parent.next.is_some());
    let grandparent = parent.next.as_ref().unwrap();
    assert_eq!(grandparent.string_field, "level_3");
    assert_eq!(grandparent.integer_field, 3);
    assert_eq!(grandparent.optional_field, Some("deepest".to_string()));
    assert!(grandparent.next.is_none());
}

#[test]
fn test_deeply_nested_without_recursion() {
    // JSON matching the exact deeply nested structure from serialization test
    let json = r#"{
        "name": "complex_object",
        "count": 100,
        "single_nested": {
            "field_a": "single_a",
            "field_b": "single_b",
            "field_c": "single_c"
        },
        "optional_nested": null,
        "list_nested": [
            {
                "field_a": "list_item_0_a",
                "field_b": "list_item_0_b",
                "field_c": "list_item_0_c"
            },
            {
                "field_a": "list_item_1_a",
                "field_b": "list_item_1_b",
                "field_c": "list_item_1_c"
            }
        ],
        "map_nested": {
            "map_key": {
                "field_a": "map_val_a",
                "field_b": "map_val_b",
                "field_c": "map_val_c"
            }
        }
    }"#;

    let mut deserializer = JsonDeserializer::from_str(json);
    let nested = NestedCollectionsStruct::deserialize_with_schema(
        &NESTED_COLLECTIONS_STRUCT_SCHEMA,
        &mut deserializer,
    )
    .unwrap();

    println!("Deserialized deeply nested struct: {:?}", nested);

    // Verify all levels are present - matching the serialization test assertions
    assert_eq!(nested.name, "complex_object");
    assert_eq!(nested.count, 100);
    assert_eq!(nested.single_nested.field_a, "single_a");
    assert_eq!(nested.single_nested.field_b, "single_b");
    assert_eq!(nested.single_nested.field_c, "single_c");

    assert!(nested.optional_nested.is_none());

    assert_eq!(nested.list_nested.len(), 2);
    assert_eq!(nested.list_nested[0].field_a, "list_item_0_a");
    assert_eq!(nested.list_nested[0].field_b, "list_item_0_b");
    assert_eq!(nested.list_nested[0].field_c, "list_item_0_c");
    assert_eq!(nested.list_nested[1].field_a, "list_item_1_a");
    assert_eq!(nested.list_nested[1].field_b, "list_item_1_b");
    assert_eq!(nested.list_nested[1].field_c, "list_item_1_c");

    assert_eq!(nested.map_nested.len(), 1);
    let map_val = nested.map_nested.get("map_key").unwrap();
    assert_eq!(map_val.field_a, "map_val_a");
    assert_eq!(map_val.field_b, "map_val_b");
    assert_eq!(map_val.field_c, "map_val_c");
}
