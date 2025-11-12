use smithy4rs_core::{
    lazy_schema,
    prelude::*,
    schema::{Schema, SchemaRef, ShapeId},
    serde::{deserializers::Deserialize, serializers::SerializeWithSchema},
    traits,
};
use smithy4rs_core_derive::{DeserializableStruct, SerializableStruct};
use smithy4rs_json_codec::{JsonDeserializer, JsonSerializer};

// ============================================================================
// Test Structures
// ============================================================================

// Simple struct with primitives
lazy_schema!(
    PERSON_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#Person"), traits![]),
    (NAME, "name", STRING, traits![]),
    (AGE, "age", INTEGER, traits![]),
    (EMAIL, "email", STRING, traits![])
);

#[derive(Debug, PartialEq, SerializableStruct, DeserializableStruct)]
#[smithy_schema(PERSON_SCHEMA)]
struct Person {
    #[smithy_schema(NAME)]
    name: String,
    #[smithy_schema(AGE)]
    age: i32,
    #[smithy_schema(EMAIL)]
    email: String,
}

// Struct with optional fields
lazy_schema!(
    OPTIONAL_DATA_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#OptionalData"), traits![]),
    (REQUIRED_FIELD, "required_field", STRING, traits![]),
    (OPTIONAL_FIELD, "optional_field", STRING, traits![])
);

#[derive(Debug, PartialEq, SerializableStruct, DeserializableStruct)]
#[smithy_schema(OPTIONAL_DATA_SCHEMA)]
struct OptionalData {
    #[smithy_schema(REQUIRED_FIELD)]
    required_field: String,
    #[smithy_schema(OPTIONAL_FIELD)]
    optional_field: Option<String>,
}

// Struct with numeric types
lazy_schema!(
    NUMBERS_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#Numbers"), traits![]),
    (BYTE_VAL, "byte_val", BYTE, traits![]),
    (SHORT_VAL, "short_val", SHORT, traits![]),
    (INT_VAL, "int_val", INTEGER, traits![]),
    (LONG_VAL, "long_val", LONG, traits![]),
    (FLOAT_VAL, "float_val", FLOAT, traits![]),
    (DOUBLE_VAL, "double_val", DOUBLE, traits![])
);

#[derive(Debug, PartialEq, SerializableStruct, DeserializableStruct)]
#[smithy_schema(NUMBERS_SCHEMA)]
struct Numbers {
    #[smithy_schema(BYTE_VAL)]
    byte_val: i8,
    #[smithy_schema(SHORT_VAL)]
    short_val: i16,
    #[smithy_schema(INT_VAL)]
    int_val: i32,
    #[smithy_schema(LONG_VAL)]
    long_val: i64,
    #[smithy_schema(FLOAT_VAL)]
    float_val: f32,
    #[smithy_schema(DOUBLE_VAL)]
    double_val: f64,
}

// ============================================================================
// Roundtrip Tests
// ============================================================================

fn serialize_to_json<T: SerializeWithSchema>(value: &T, schema: &SchemaRef) -> Vec<u8> {
    let mut buf = Vec::new();
    let serializer = JsonSerializer::new(&mut buf);
    value.serialize_with_schema(schema, serializer).unwrap();
    buf
}

fn deserialize_from_json<'de, T: Deserialize<'de>>(data: &'de [u8], schema: &SchemaRef) -> T {
    let mut deserializer = JsonDeserializer::new(data);
    T::deserialize(schema, &mut deserializer).unwrap()
}

fn roundtrip<T>(value: &T, schema: &SchemaRef) -> T
where
    T: SerializeWithSchema + for<'de> Deserialize<'de>,
{
    let json = serialize_to_json(value, schema);
    println!("Serialized JSON: {}", String::from_utf8_lossy(&json));
    deserialize_from_json(&json, schema)
}

#[test]
fn test_person_roundtrip() {
    let person = Person {
        name: "Alice Smith".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };

    let result = roundtrip(&person, &PERSON_SCHEMA);
    assert_eq!(person, result);
}

#[test]
fn test_optional_data_with_value() {
    let data = OptionalData {
        required_field: "required".to_string(),
        optional_field: Some("optional".to_string()),
    };

    let result = roundtrip(&data, &OPTIONAL_DATA_SCHEMA);
    assert_eq!(data, result);
}

#[test]
fn test_optional_data_without_value() {
    let data = OptionalData {
        required_field: "required".to_string(),
        optional_field: None,
    };

    let result = roundtrip(&data, &OPTIONAL_DATA_SCHEMA);
    assert_eq!(data, result);
}

#[test]
fn test_numbers_roundtrip() {
    let numbers = Numbers {
        byte_val: 42,
        short_val: 1000,
        int_val: 100000,
        long_val: 1000000000000,
        float_val: 3.14,
        double_val: 2.718281828,
    };

    let result = roundtrip(&numbers, &NUMBERS_SCHEMA);
    assert_eq!(numbers, result);
}

#[test]
fn test_numbers_negative_values() {
    let numbers = Numbers {
        byte_val: -42,
        short_val: -1000,
        int_val: -100000,
        long_val: -1000000000000,
        float_val: -3.14,
        double_val: -2.718281828,
    };

    let result = roundtrip(&numbers, &NUMBERS_SCHEMA);
    assert_eq!(numbers, result);
}

#[test]
fn test_numbers_edge_cases() {
    let numbers = Numbers {
        byte_val: i8::MAX,
        short_val: i16::MIN,
        int_val: i32::MAX,
        long_val: i64::MIN,
        float_val: f32::MIN_POSITIVE,
        double_val: f64::MAX,
    };

    let result = roundtrip(&numbers, &NUMBERS_SCHEMA);
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
    let person = Person {
        name: "Test \"User\" with\nnewlines\tand\ttabs".to_string(),
        age: 25,
        email: "test@example.com".to_string(),
    };

    let result = roundtrip(&person, &PERSON_SCHEMA);
    assert_eq!(person, result);
}

#[test]
fn test_unicode_strings() {
    let person = Person {
        name: "Müller 李明 🎉".to_string(),
        age: 28,
        email: "test@例え.jp".to_string(),
    };

    let result = roundtrip(&person, &PERSON_SCHEMA);
    assert_eq!(person, result);
}

#[test]
fn test_empty_strings() {
    let person = Person {
        name: "".to_string(),
        age: 0,
        email: "".to_string(),
    };

    let result = roundtrip(&person, &PERSON_SCHEMA);
    assert_eq!(person, result);
}
