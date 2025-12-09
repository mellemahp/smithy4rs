use smithy4rs_core::{
    lazy_schema,
    prelude::{INTEGER, STRING},
    schema::{Schema, ShapeId},
    traits,
};
use smithy4rs_core_derive::SmithyStruct;

lazy_schema!(
    SIMPLE_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#SimpleStruct"), traits![]),
    (FIELD_A, "field_a", STRING, traits![]),
    (FIELD_B, "field_b", INTEGER, traits![]),
    (FIELD_C, "field_c", STRING, traits![])
);

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

lazy_schema!(
    NESTED_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#NESTED_STRUCT"), traits![]),
    (FIELD_D, "field_d", STRING, traits![]),
);

#[derive(SmithyStruct, Debug, PartialEq)]
#[smithy_schema(NESTED_SCHEMA)]
pub struct Nested {
    #[smithy_schema(FIELD_C)]
    pub field_a: String,
}
