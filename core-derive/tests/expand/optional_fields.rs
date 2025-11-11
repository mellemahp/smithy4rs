use smithy4rs_core::{
    lazy_schema,
    prelude::{INTEGER, STRING},
    schema::{Schema, ShapeId},
    traits,
};
use smithy4rs_core_derive::{DeserializableStruct, SerializableStruct};

lazy_schema!(
    SCHEMA_WITH_OPTIONAL,
    Schema::structure_builder(ShapeId::from("test#StructWithOptional"), traits![]),
    (REQUIRED_FIELD, "required", STRING, traits![]),
    (OPTIONAL_FIELD, "optional", INTEGER, traits![])
);

#[derive(SerializableStruct, DeserializableStruct, Debug, PartialEq)]
#[smithy_schema(SCHEMA_WITH_OPTIONAL)]
pub struct StructWithOptional {
    #[smithy_schema(REQUIRED_FIELD)]
    pub required: String,
    #[smithy_schema(OPTIONAL_FIELD)]
    pub optional: Option<i32>,
}
