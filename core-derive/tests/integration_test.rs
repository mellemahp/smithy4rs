// Integration test for struct serialization/deserialization
mod expand {
    pub mod all_shapes;
    pub mod optional_fields;
    pub mod simple_struct;
}

use expand::{
    all_shapes::{ALL_SHAPES_SCHEMA, AllShapes},
    optional_fields::{SCHEMA_WITH_OPTIONAL, StructWithOptional},
    simple_struct::{SIMPLE_SCHEMA, SimpleStruct},
};
use indexmap::IndexMap;
use smithy4rs_core::{
    ByteBuffer, Instant,
    serde::{
        deserializers::Deserialize,
        documents::{DocumentDeserializer, DocumentParser},
        serializers::SerializeWithSchema,
    },
};

// TODO: Move integration testing into its own crate at the root
#[test]
fn test_roundtrip_with_optional() {
    let original = StructWithOptional {
        required: "Alice".to_string(),
        optional: Some(42),
    };

    // Serialize to Document
    let doc = original
        .serialize_with_schema(&SCHEMA_WITH_OPTIONAL, DocumentParser)
        .unwrap();

    // Deserialize back
    let mut deserializer = DocumentDeserializer::new(&doc);
    let result = StructWithOptional::deserialize(&SCHEMA_WITH_OPTIONAL, &mut deserializer).unwrap();

    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_without_optional() {
    let original = StructWithOptional {
        required: "Bob".to_string(),
        optional: None,
    };

    // Serialize to Document
    let doc = original
        .serialize_with_schema(&SCHEMA_WITH_OPTIONAL, DocumentParser)
        .unwrap();

    // Deserialize back
    let mut deserializer = DocumentDeserializer::new(&doc);
    let result = StructWithOptional::deserialize(&SCHEMA_WITH_OPTIONAL, &mut deserializer).unwrap();

    assert_eq!(original, result);
}

#[test]
fn test_simple_struct_roundtrip() {
    let original = SimpleStruct {
        field_a: "Hello".to_string(),
        field_b: 123,
    };

    // Serialize to Document
    let doc = original
        .serialize_with_schema(&SIMPLE_SCHEMA, DocumentParser)
        .unwrap();

    // Deserialize back
    let mut deserializer = DocumentDeserializer::new(&doc);
    let result = SimpleStruct::deserialize(&SIMPLE_SCHEMA, &mut deserializer).unwrap();

    assert_eq!(original, result);
}

#[test]
fn test_all_shapes_roundtrip() {
    let mut map = IndexMap::new();
    map.insert("key1".to_string(), "value1".to_string());
    map.insert("key2".to_string(), "value2".to_string());

    let original = AllShapes {
        string_field: "test string".to_string(),
        byte_field: 42i8,
        short_field: 1000i16,
        integer_field: 100000i32,
        long_field: 10000000000i64,
        float_field: 3.15f32,
        double_field: 2.5f64,
        boolean_field: true,
        blob_field: ByteBuffer::from(vec![1u8, 2u8, 3u8, 4u8]),
        timestamp_field: Instant::from_epoch_milliseconds(1234567890000).unwrap(),
        // big_integer_field: BigInt::from(123456789),
        // big_decimal_field: BigDecimal::from(42),
        list_field: vec![
            "item1".to_string(),
            "item2".to_string(),
            "item3".to_string(),
        ],
        map_field: map,
        optional_field: Some("optional value".to_string()),
        recursive_field: None,
    };

    // Serialize to Document
    let doc = original
        .serialize_with_schema(&ALL_SHAPES_SCHEMA, DocumentParser)
        .unwrap();

    // Deserialize back
    let mut deserializer = DocumentDeserializer::new(&doc);
    let result = AllShapes::deserialize(&ALL_SHAPES_SCHEMA, &mut deserializer).unwrap();

    assert_eq!(original, result);
}

#[test]
fn test_all_shapes_with_recursive_roundtrip() {
    let mut inner_map = IndexMap::new();
    inner_map.insert("inner_key".to_string(), "inner_value".to_string());

    let _inner = AllShapes {
        string_field: "inner string".to_string(),
        byte_field: 1i8,
        short_field: 2i16,
        integer_field: 3i32,
        long_field: 4i64,
        float_field: 1.0f32,
        double_field: 2.0f64,
        boolean_field: false,
        blob_field: ByteBuffer::from(vec![5u8, 6u8]),
        timestamp_field: Instant::from_epoch_milliseconds(987654321000).unwrap(),
        // big_integer_field: BigInt::from(999),
        // big_decimal_field: BigDecimal::from(888),
        list_field: vec!["inner_item".to_string()],
        map_field: inner_map,
        optional_field: None,
        recursive_field: None,
    };

    let mut outer_map = IndexMap::new();
    outer_map.insert("outer_key".to_string(), "outer_value".to_string());

    let original = AllShapes {
        string_field: "outer string".to_string(),
        byte_field: 10i8,
        short_field: 20i16,
        integer_field: 30i32,
        long_field: 40i64,
        float_field: 5.5f32,
        double_field: 6.6f64,
        boolean_field: true,
        blob_field: ByteBuffer::from(vec![7u8, 8u8, 9u8]),
        timestamp_field: Instant::from_epoch_milliseconds(1111111111000).unwrap(),
        // big_integer_field: BigInt::from(777),
        // big_decimal_field: BigDecimal::from(666),
        list_field: vec!["outer_item1".to_string(), "outer_item2".to_string()],
        map_field: outer_map,
        optional_field: Some("outer optional".to_string()),
        recursive_field: Some(Box::new(_inner)),
    };

    // Serialize to Document
    let doc = original
        .serialize_with_schema(&ALL_SHAPES_SCHEMA, DocumentParser)
        .unwrap();
    println!("Serialized document: {:#?}", doc);

    // Deserialize back
    let mut deserializer = DocumentDeserializer::new(&doc);
    let result = AllShapes::deserialize(&ALL_SHAPES_SCHEMA, &mut deserializer).unwrap();

    assert_eq!(original, result);
}

#[test]
fn test_macro_expansion() {
    macrotest::expand("tests/expand/*.rs");
}
