use smithy4rs_core::{
    prelude::{INTEGER, STRING},
    smithy,
};
use smithy4rs_core_derive::SmithyShape;

smithy!("test#SimpleStruct": {
    structure SIMPLE_SCHEMA {
        A: STRING = "field_a"
        B: INTEGER = "field_b"
        C: STRING = "field_c"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(SIMPLE_SCHEMA)]
pub struct SimpleStruct {
    #[smithy_schema(A)]
    pub field_a: String,
    #[smithy_schema(B)]
    #[default(0)]
    pub field_b: i32,
    #[smithy_schema(C)]
    pub field_c: Option<Nested>,
}

smithy!("test#NESTED_STRUCT": {
    structure NESTED_SCHEMA {
        D: STRING = "field_d"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(NESTED_SCHEMA)]
pub struct Nested {
    #[smithy_schema(D)]
    pub field_a: String,
}
