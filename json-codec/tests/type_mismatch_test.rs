use smithy4rs_core::serde::deserializers::DeserializeWithSchema;
use smithy4rs_json_codec::{JsonDeserializer, JsonSerdeError};
use smithy4rs_test_utils::*;

#[test]
fn test_string_for_integer() {
    let json = br#"{"field_a": "test", "field_b": "not_an_int"}"#;
    let mut de = JsonDeserializer::new(json);
    let result = SimpleStructBuilder::deserialize_with_schema(&SIMPLE_STRUCT_SCHEMA, &mut de);

    match result {
        Err(JsonSerdeError::DeserializationError(msg)) => {
            assert!(
                msg.contains("integer"),
                "Expected integer parsing error, got: {msg}"
            );
        }
        Err(e) => panic!("Expected DeserializationError, got: {e:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[test]
fn test_integer_for_string() {
    let json = br#"{"field_a": 123, "field_b": 456}"#;
    let mut de = JsonDeserializer::new(json);
    let result = SimpleStructBuilder::deserialize_with_schema(&SIMPLE_STRUCT_SCHEMA, &mut de);

    match result {
        Err(JsonSerdeError::DeserializationError(msg)) => {
            assert!(
                msg.contains("string"),
                "Expected string parsing error, got: {msg}"
            );
        }
        Err(e) => panic!("Expected DeserializationError, got: {e:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[test]
fn test_array_for_object() {
    let json = b"[]";
    let mut de = JsonDeserializer::new(json);
    let result = SimpleStructBuilder::deserialize_with_schema(&SIMPLE_STRUCT_SCHEMA, &mut de);

    match result {
        Err(JsonSerdeError::DeserializationError(msg)) => {
            assert!(
                msg.contains("object"),
                "Expected object parsing error, got: {msg}"
            );
        }
        Err(e) => panic!("Expected DeserializationError, got: {e:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[test]
fn test_object_for_array() {
    let json = b"{}";
    let mut de = JsonDeserializer::new(json);
    let result: Result<Vec<i32>, _> = Vec::deserialize_with_schema(&INTEGER_LIST_SCHEMA, &mut de);

    match result {
        Err(JsonSerdeError::DeserializationError(msg)) => {
            assert!(
                msg.contains("array"),
                "Expected array parsing error, got: {msg}"
            );
        }
        Err(e) => panic!("Expected DeserializationError, got: {e:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[test]
fn test_bool_for_integer() {
    let json = br#"{"field_a": "test", "field_b": true}"#;
    let mut de = JsonDeserializer::new(json);
    let result = SimpleStructBuilder::deserialize_with_schema(&SIMPLE_STRUCT_SCHEMA, &mut de);

    match result {
        Err(JsonSerdeError::DeserializationError(msg)) => {
            assert!(
                msg.contains("integer"),
                "Expected integer parsing error, got: {msg}"
            );
        }
        Err(e) => panic!("Expected DeserializationError, got: {e:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}
