#![no_main]

use libfuzzer_sys::fuzz_target;
use smithy4rs_core::{
    derive::SmithyShape,
    prelude::{INTEGER, RequiredTrait, STRING},
    schema::StaticSchemaShape,
    serde::validation::{DefaultValidator, Validator},
    smithy,
};

smithy!("test#SimpleStruct": {
    structure SIMPLE_STRUCT_SCHEMA {
        A: STRING = "field_a"
        @RequiredTrait;
        B: INTEGER = "field_b"
    }
});

// TODO: Replace with validation fuzzer. This is just to verify functionality
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(SIMPLE_STRUCT_SCHEMA)]
pub struct SimpleStruct {
    #[smithy_schema(A)]
    pub field_a: Option<String>,
    #[smithy_schema(B)]
    pub field_b: i32,
}

fuzz_target!(|data: SimpleStruct| {
    let _ = DefaultValidator::new().validate(SimpleStruct::schema(), &data);
});
