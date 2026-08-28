use smithy4rs_core::{
    prelude::{INTEGER, STRING},
    smithy,
};
use smithy4rs_core_derive::SmithyShape;

smithy!(
    structure test::SimpleSchema {
        fieldA: STRING
        fieldB: INTEGER
        fieldC: STRING
    }
);

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = SIMPLE_SCHEMA)]
pub struct SimpleStruct {
    pub field_a: String,
    #[schema(default = 0)]
    pub field_b: i32,
    pub field_c: Option<Nested>,
}

smithy!(
    structure test::NestedSchema {
        fieldD: STRING
    }
);

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = NESTED_SCHEMA)]
pub struct Nested {
    pub field_d: String,
}
