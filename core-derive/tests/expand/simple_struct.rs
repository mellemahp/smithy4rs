use smithy4rs_core::{
    prelude::{INTEGER, STRING},
    schema::ShapeId,
    traits,
};
use smithy4rs_core_derive::SmithyStruct;

smithy!("test#SimpleStruct": {
    structure SIMPLE_SCHEMA {
        FIELD_A: STRING = "field_a"
        FIELD_B: INTEGER = "field_b"
        FIELD_C: STRING = "field_c"
    }
});

#[derive(SmithyStruct, Debug, PartialEq)]
#[smithy_schema(SIMPLE_SCHEMA)]
pub struct SimpleStruct {
    #[smithy_schema(FIELD_A)]
    pub field_a: String,
    #[smithy_schema(FIELD_B)]
    pub field_b: i32,
    #[smithy_schema(FIELD_C)]
    pub field_c: Option<Nested>,
}

smithy!("test#NESTED_STRUCT": {
    structure NESTED_SCHEMA {
        FIELD_D: STRING = "field_d"
    }
});

#[derive(SmithyStruct, Debug, PartialEq)]
#[smithy_schema(NESTED_SCHEMA)]
pub struct Nested {
    #[smithy_schema(FIELD_C)]
    pub field_a: String,
}
