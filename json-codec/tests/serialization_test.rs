use indexmap::IndexMap;
use smithy4rs_core::{
    lazy_schema,
    prelude::*,
    schema::{Schema, ShapeId},
    serde::serializers::SerializeWithSchema,
    traits,
};
use smithy4rs_core_derive::SerializableStruct;
use smithy4rs_json_codec::JsonSerializer;

lazy_schema!(
    STRING_LIST_SCHEMA,
    Schema::list_builder(ShapeId::from("test#StringList"), traits![]),
    ("member", STRING, traits![])
);

lazy_schema!(
    STRING_MAP_SCHEMA,
    Schema::map_builder(ShapeId::from("test#StringMap"), traits![]),
    ("key", STRING, traits![]),
    ("value", STRING, traits![])
);

lazy_schema!(
    SIMPLE_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#SimpleStruct"), traits![]),
    (FIELD_A, "field_a", STRING, traits![]),
    (FIELD_B, "field_b", INTEGER, traits![]),
    (FIELD_LIST, "list_field", STRING_LIST_SCHEMA, traits![]),
    (FIELD_MAP, "map_field", STRING_MAP_SCHEMA, traits![])
);

#[derive(SerializableStruct)]
#[smithy_schema(SIMPLE_SCHEMA)]
struct SimpleStruct {
    #[smithy_schema(FIELD_A)]
    field_a: String,
    #[smithy_schema(FIELD_B)]
    field_b: i32,
    #[smithy_schema(FIELD_LIST)]
    list_field: Vec<String>,
    #[smithy_schema(FIELD_MAP)]
    map_field: IndexMap<String, String>,
}

#[test]
fn test_json_serialization() {
    let mut map = IndexMap::new();
    map.insert("key1".to_string(), "value1".to_string());
    map.insert("key2".to_string(), "value2".to_string());

    let test_struct = SimpleStruct {
        field_a: "hello".to_string(),
        field_b: 42,
        list_field: vec!["item1".to_string(), "item2".to_string()],
        map_field: map,
    };

    let mut buf = Vec::new();
    let serializer = JsonSerializer::new(&mut buf);
    test_struct
        .serialize_with_schema(&SIMPLE_SCHEMA, serializer)
        .unwrap();

    let json = String::from_utf8(buf).unwrap();
    println!("Serialized JSON: {json}");

    // Basic checks that it's valid JSON with expected fields
    assert!(json.contains("\"field_a\""));
    assert!(json.contains("\"hello\""));
    assert!(json.contains("\"field_b\""));
    assert!(json.contains("42"));
    assert!(json.contains("\"list_field\""));
    assert!(json.contains("\"item1\""));
    assert!(json.contains("\"map_field\""));
    assert!(json.contains("\"key1\""));
}
