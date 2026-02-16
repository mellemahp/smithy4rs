use smithy4rs_core::serde::deserializers::DeserializeWithSchema;
use smithy4rs_json_codec::{JsonDeserializer, JsonSerdeError};
use smithy4rs_test_utils::*;

#[test]
fn test_unclosed_brace() {
    let json = b"{\"field_a\": \"test\"";
    let mut de = JsonDeserializer::new(json);
    let result = SimpleStructBuilder::deserialize_with_schema(&SIMPLE_STRUCT_SCHEMA, de);

    match result {
        Err(JsonSerdeError::DeserializationError(msg)) => {
            assert!(
                msg.contains("EOF while parsing"),
                "Expected EOF parsing error, got: {msg}"
            );
        }
        Err(e) => panic!("Expected DeserializationError, got: {e:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[test]
fn test_unclosed_bracket() {
    let json = b"[1, 2, 3";
    let mut de = JsonDeserializer::new(json);
    let result: Result<Vec<i32>, _> = Vec::deserialize_with_schema(&INTEGER_LIST_SCHEMA, de);

    match result {
        Err(JsonSerdeError::DeserializationError(msg)) => {
            assert!(
                msg.contains("EOF while parsing"),
                "Expected EOF parsing error, got: {msg}"
            );
        }
        Err(e) => panic!("Expected DeserializationError, got: {e:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[test]
fn test_trailing_comma() {
    let json = b"[1, 2, 3,]";
    let mut de = JsonDeserializer::new(json);
    let result: Result<Vec<i32>, _> = Vec::deserialize_with_schema(&INTEGER_LIST_SCHEMA, de);

    match result {
        Err(JsonSerdeError::DeserializationError(msg)) => {
            assert!(
                msg.contains("trailing comma"),
                "Expected trailing comma error, got: {msg}"
            );
        }
        Err(e) => panic!("Expected DeserializationError, got: {e:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}
