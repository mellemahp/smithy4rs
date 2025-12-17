// TODO(test): add some test utils for validating the serialized json outputs, perhaps snapshots would suffice
use indexmap::IndexMap;
use smithy4rs_core::serde::{builders::ShapeBuilder, serializers::SerializeWithSchema};
use smithy4rs_json_codec::JsonSerializer;
use smithy4rs_test_utils::*;

#[test]
fn test_nested_struct_serialization() {
    let single = InnerStructBuilder::new()
        .field_a("alpha".to_string())
        .field_b("beta".to_string())
        .field_c("gamma".to_string())
        .build()
        .unwrap();

    let optional = InnerStructBuilder::new()
        .field_a("delta".to_string())
        .field_b("epsilon".to_string())
        .field_c("zeta".to_string())
        .build()
        .unwrap();

    let list_nested = vec![
        InnerStructBuilder::new()
            .field_a("item1-a".to_string())
            .field_b("item1-b".to_string())
            .field_c("item1-c".to_string())
            .build()
            .unwrap(),
        InnerStructBuilder::new()
            .field_a("item2-a".to_string())
            .field_b("item2-b".to_string())
            .field_c("item2-c".to_string())
            .build()
            .unwrap(),
    ];

    let mut map_nested = IndexMap::new();
    map_nested.insert(
        "key1".to_string(),
        InnerStructBuilder::new()
            .field_a("value1-a".to_string())
            .field_b("value1-b".to_string())
            .field_c("value1-c".to_string())
            .build()
            .unwrap(),
    );
    map_nested.insert(
        "key2".to_string(),
        InnerStructBuilder::new()
            .field_a("value2-a".to_string())
            .field_b("value2-b".to_string())
            .field_c("value2-c".to_string())
            .build()
            .unwrap(),
    );

    let nested = NestedCollectionsStructBuilder::new()
        .name("test_object".to_string())
        .count(42)
        .single_nested(single)
        .optional_nested(optional)
        .list_nested(list_nested)
        .map_nested(map_nested)
        .build()
        .unwrap();

    let mut buf = Vec::new();
    let serializer = JsonSerializer::new(&mut buf);
    nested
        .serialize_with_schema(&NESTED_COLLECTIONS_STRUCT_SCHEMA, serializer)
        .unwrap();

    let json = String::from_utf8(buf).unwrap();
    println!("Serialized nested struct JSON:\n{}", json);

    assert!(json.contains("\"name\":\"test_object\""));
    assert!(json.contains("\"count\":42"));
    assert!(json.contains("\"single_nested\""));
    assert!(json.contains("\"alpha\""));
    assert!(json.contains("\"beta\""));
    assert!(json.contains("\"optional_nested\""));
    assert!(json.contains("\"delta\""));
    assert!(json.contains("\"list_nested\""));
    assert!(json.contains("\"item1-a\""));
    assert!(json.contains("\"map_nested\""));
    assert!(json.contains("\"key1\""));
    assert!(json.contains("\"value1-a\""));
}

#[test]
fn test_recursive_struct_serialization() {
    let grandparent = RecursiveShapesStructBuilder::new()
        .string_field("level_3".to_string())
        .integer_field(3)
        .list_field(vec![])
        .map_field(IndexMap::new())
        .optional_field("deepest".to_string())
        .build()
        .unwrap();

    let parent = RecursiveShapesStructBuilder::new()
        .string_field("level_2".to_string())
        .integer_field(2)
        .list_field(vec![])
        .map_field(IndexMap::new())
        .optional_field("middle".to_string())
        .next(Box::new(grandparent))
        .build()
        .unwrap();

    let child = RecursiveShapesStructBuilder::new()
        .string_field("level_1".to_string())
        .integer_field(1)
        .list_field(vec![])
        .map_field(IndexMap::new())
        .optional_field("top".to_string())
        .next(Box::new(parent))
        .build()
        .unwrap();

    let mut buf = Vec::new();
    let serializer = JsonSerializer::new(&mut buf);
    child
        .serialize_with_schema(&RECURSIVE_SHAPES_STRUCT_SCHEMA, serializer)
        .unwrap();

    let json = String::from_utf8(buf).unwrap();
    println!("Serialized recursive struct JSON:\n{}", json);

    assert!(json.contains("\"level_1\""));
    assert!(json.contains("\"top\""));
    assert!(json.contains("\"level_2\""));
    assert!(json.contains("\"middle\""));
    assert!(json.contains("\"level_3\""));
    assert!(json.contains("\"deepest\""));

    // Count the nesting levels by counting "next" occurrences
    let next_count = json.matches("\"next\"").count();
    assert_eq!(
        next_count, 2,
        "Should have 2 next references (child->parent->grandparent)"
    );
}

#[test]
fn test_deeply_nested_without_recursion() {
    let mut inner_map = IndexMap::new();
    inner_map.insert(
        "map_key".to_string(),
        InnerStructBuilder::new()
            .field_a("map_val_a".to_string())
            .field_b("map_val_b".to_string())
            .field_c("map_val_c".to_string())
            .build()
            .unwrap(),
    );

    let nested = NestedCollectionsStructBuilder::new()
        .name("complex_object".to_string())
        .count(100)
        .single_nested(
            InnerStructBuilder::new()
                .field_a("single_a".to_string())
                .field_b("single_b".to_string())
                .field_c("single_c".to_string())
                .build()
                .unwrap(),
        )
        .list_nested(vec![
            InnerStructBuilder::new()
                .field_a("list_item_0_a".to_string())
                .field_b("list_item_0_b".to_string())
                .field_c("list_item_0_c".to_string())
                .build()
                .unwrap(),
            InnerStructBuilder::new()
                .field_a("list_item_1_a".to_string())
                .field_b("list_item_1_b".to_string())
                .field_c("list_item_1_c".to_string())
                .build()
                .unwrap(),
        ])
        .map_nested(inner_map)
        .build()
        .unwrap();

    let mut buf = Vec::new();
    let serializer = JsonSerializer::new(&mut buf);
    nested
        .serialize_with_schema(&NESTED_COLLECTIONS_STRUCT_SCHEMA, serializer)
        .unwrap();

    let json = String::from_utf8(buf).unwrap();
    println!("Serialized deeply nested JSON:\n{}", json);

    // Verify all levels are present
    assert!(json.contains("\"complex_object\""));
    assert!(json.contains("\"single_a\""));
    assert!(json.contains("\"list_item_0_a\""));
    assert!(json.contains("\"list_item_1_a\""));
    assert!(json.contains("\"map_key\""));
    assert!(json.contains("\"map_val_a\""));
}
