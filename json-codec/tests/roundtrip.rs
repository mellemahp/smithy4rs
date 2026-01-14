use smithy4rs_core::{
    schema::SchemaRef,
    serde::{Buildable, ShapeBuilder, de::DeserializeWithSchema, serializers::SerializeWithSchema},
};
use smithy4rs_json_codec::{JsonDeserializer, JsonSerializer};
use smithy4rs_test_utils::*;

// ============================================================================
// Roundtrip Test Helpers
// ============================================================================

fn serialize_to_json<T: SerializeWithSchema>(value: &T, schema: &SchemaRef) -> Vec<u8> {
    let mut buf = Vec::new();
    let serializer = JsonSerializer::new(&mut buf);
    value.serialize_with_schema(schema, serializer).unwrap();
    buf
}

fn deserialize_from_json<'de, B: ShapeBuilder<'de, T>, T: Buildable<'de, B>>(
    data: &'de [u8],
    schema: &SchemaRef,
) -> T {
    let mut deserializer = JsonDeserializer::new(data);
    B::deserialize_with_schema(schema, &mut deserializer)
        .unwrap()
        .build()
        .unwrap()
}

fn roundtrip<T, B>(value: &T, schema: &SchemaRef) -> T
where
    B: for<'de> ShapeBuilder<'de, T>,
    T: SerializeWithSchema + for<'de> Buildable<'de, B>,
{
    let json = serialize_to_json(value, schema);
    println!("Serialized JSON: {}", String::from_utf8_lossy(&json));
    deserialize_from_json(&json, schema)
}

// ============================================================================
// Roundtrip Tests
// ============================================================================

#[test]
fn test_optional_data_with_value() {
    let data = OptionalFieldsStructBuilder::new()
        .required_field("required".to_string())
        .optional_field("optional".to_string())
        .build()
        .unwrap();

    let result = roundtrip(&data, &OPTIONAL_FIELDS_STRUCT_SCHEMA);
    assert_eq!(data, result);
}

#[test]
fn test_optional_data_without_value() {
    // Don't set optional_field - it will be None
    let data = OptionalFieldsStructBuilder::new()
        .required_field("required".to_string())
        .build()
        .unwrap();

    let result = roundtrip(&data, &OPTIONAL_FIELDS_STRUCT_SCHEMA);
    assert_eq!(data, result);
}

#[test]
fn test_numbers_roundtrip() {
    let numbers = NumericTypesStructBuilder::new()
        .byte_val(42)
        .short_val(1000i16)
        .int_val(100000)
        .long_val(1000000000000i64)
        .float_val(1.234)
        .double_val(1.23456789)
        .build()
        .unwrap();

    let result = roundtrip(&numbers, &NUMERIC_TYPES_STRUCT_SCHEMA);
    assert_eq!(numbers, result);
}

#[test]
fn test_numbers_negative_values() {
    let numbers = NumericTypesStructBuilder::new()
        .byte_val(-42)
        .short_val(-1000i16)
        .int_val(-100000)
        .long_val(-1000000000000i64)
        .float_val(-1.234)
        .double_val(-1.23456789)
        .build()
        .unwrap();

    let result = roundtrip(&numbers, &NUMERIC_TYPES_STRUCT_SCHEMA);
    assert_eq!(numbers, result);
}

#[test]
fn test_numbers_edge_cases() {
    let numbers = NumericTypesStructBuilder::new()
        .byte_val(i8::MAX)
        .short_val(i16::MIN)
        .int_val(i32::MAX)
        .long_val(i64::MIN)
        .float_val(f32::MIN_POSITIVE)
        .double_val(f64::MAX)
        .build()
        .unwrap();

    let result = roundtrip(&numbers, &NUMERIC_TYPES_STRUCT_SCHEMA);
    assert_eq!(numbers.byte_val, result.byte_val);
    assert_eq!(numbers.short_val, result.short_val);
    assert_eq!(numbers.int_val, result.int_val);
    assert_eq!(numbers.long_val, result.long_val);
    // For floats, we need approximate comparison
    assert!((numbers.float_val - result.float_val).abs() < f32::EPSILON);
    assert!((numbers.double_val - result.double_val).abs() < f64::EPSILON);
}

#[test]
fn test_special_characters_in_strings() {
    let data = SimpleStructBuilder::new()
        .field_a("Test \"string\" with\nnewlines\tand\ttabs".to_string())
        .field_b(42)
        .build()
        .unwrap();

    let result = roundtrip(&data, &SIMPLE_STRUCT_SCHEMA);
    assert_eq!(data, result);
}

#[test]
fn test_unicode_strings() {
    let data = SimpleStructBuilder::new()
        .field_a("Müller 李明 🎉".to_string())
        .field_b(123)
        .build()
        .unwrap();

    let result = roundtrip(&data, &SIMPLE_STRUCT_SCHEMA);
    assert_eq!(data, result);
}

#[test]
fn test_empty_strings() {
    let data = SimpleStructBuilder::new()
        .field_a("".to_string())
        .field_b(0)
        .build()
        .unwrap();

    let result = roundtrip(&data, &SIMPLE_STRUCT_SCHEMA);
    assert_eq!(data, result);
}

#[test]
fn test_union() {
    let data = TestUnion::A("stuff".to_string());
    let json = serialize_to_json(&data, &UNION);
    println!("Serialized JSON: {}", String::from_utf8_lossy(&json));
    let mut deserializer = JsonDeserializer::new(&json);
    let result = TestUnion::deserialize_with_schema(&UNION, &mut deserializer).unwrap();
    assert_eq!(data, result);
}
