use smithy4rs_core::serde::deserializers::DeserializeWithSchema;
use smithy4rs_json_codec::{JsonDeserializer, JsonSerdeError};
use smithy4rs_test_utils::*;

#[test]
fn test_missing_required_field() {
    // SimpleStruct requires both field_a (String) and field_b (i32)
    let json = br#"{"field_a": "test"}"#;
    let mut de = JsonDeserializer::new(json);

    // Deserialization succeeds
    let builder = SimpleStructBuilder::deserialize_with_schema(&SIMPLE_STRUCT_SCHEMA, &mut de)
        .expect("Deserialization should succeed");

    // But build should fail for missing required field
    let err = builder
        .build()
        .expect_err("Build should fail for missing field");

    // ValidationErrors should contain "Required" for the missing field
    let err_str = err.to_string();
    assert!(
        err_str.contains("Required"),
        "Expected required field error, got: {err_str}"
    );
}

#[test]
fn test_null_for_required_field() {
    let json = br#"{"field_a": null, "field_b": 42}"#;
    let mut de = JsonDeserializer::new(json);
    let result = SimpleStructBuilder::deserialize_with_schema(&SIMPLE_STRUCT_SCHEMA, &mut de);

    // This should error during deserialization because field_a is a required String
    match result {
        Err(JsonSerdeError::DeserializationError(msg)) => {
            assert!(
                msg.contains("string"),
                "Expected string parsing error for null, got: {msg}"
            );
        }
        Err(e) => panic!("Expected DeserializationError, got: {e:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}
