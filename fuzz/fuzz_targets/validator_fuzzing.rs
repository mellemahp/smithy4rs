#![no_main]

use libfuzzer_sys::fuzz_target;
use smithy4rs_core::{
    derive::SmithyShape,
    prelude::{INTEGER, RequiredTrait, STRING},
    schema::StaticSchemaShape,
    serde::validation::{DefaultValidator, Validator},
    smithy,
};

smithy!(
    structure test::SimpleStruct {
        fieldA: STRING
        @RequiredTrait::builder().build();
        fieldB: INTEGER
    }
);

// TODO: Replace with validation fuzzer. This is just to verify functionality
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = SIMPLE_STRUCT)]
pub struct SimpleStruct {
    pub field_a: Option<String>,
    pub field_b: i32,
}

fuzz_target!(|data: SimpleStruct| {
    let _ = DefaultValidator::new().validate(SimpleStruct::schema(), &data);
});
