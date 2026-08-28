use smithy4rs_core::serde::deserializers::DeserializeWithSchema;
use smithy4rs_json_codec::{JsonDeserializer, JsonSerdeError};
use smithy4rs_test_utils::*;

#[test]
fn test_overflow_for_byte() {
    let json = br#"{"byteVal": 200, "shortVal": 0, "intVal": 0, "longVal": 0, "floatVal": 0.0, "doubleVal": 0.0}"#;
    let mut de = JsonDeserializer::new(json);
    let result = NumericTypesStructBuilder::deserialize_with_schema(&NUMERIC_TYPES_STRUCT, &mut de);

    // 200 exceeds i8::MAX (127), should error with range message
    match result {
        Err(JsonSerdeError::DeserializationError(msg)) => {
            assert!(
                msg.contains("out of range") && msg.contains("i8"),
                "Expected i8 range error, got: {msg}"
            );
        }
        Err(e) => panic!("Expected DeserializationError, got: {e:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[test]
fn test_overflow_for_short() {
    let json = br#"{"byteVal": 0, "shortVal": 40000, "intVal": 0, "longVal": 0, "floatVal": 0.0, "doubleVal": 0.0}"#;
    let mut de = JsonDeserializer::new(json);
    let result = NumericTypesStructBuilder::deserialize_with_schema(&NUMERIC_TYPES_STRUCT, &mut de);

    // 40000 exceeds i16::MAX (32767), should error with range message
    match result {
        Err(JsonSerdeError::DeserializationError(msg)) => {
            assert!(
                msg.contains("out of range") && msg.contains("i16"),
                "Expected i16 range error, got: {msg}"
            );
        }
        Err(e) => panic!("Expected DeserializationError, got: {e:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}
