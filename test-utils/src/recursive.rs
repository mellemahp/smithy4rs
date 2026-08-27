use smithy4rs_core::{
    IndexMap,
    derive::SmithyShape,
    schema::prelude::{INTEGER, STRING},
    smithy,
};

smithy!("test#StringMap": {
    map STRING_MAP_SCHEMA {
        key: STRING
        value: STRING
    }
});

smithy!("test#StringList": {
    list STRING_LIST_SCHEMA {
        member: STRING
    }
});

smithy!("test#RecursiveShapesStruct": {
    structure RECURSIVE_SHAPES_STRUCT_SCHEMA {
        STRING: STRING = "stringField"
        INTEGER: INTEGER = "integerField"
        LIST: STRING_LIST_SCHEMA = "listField"
        MAP: STRING_MAP_SCHEMA = "mapField"
        OPTIONAL: STRING = "optionalField"
        NEXT: (@self) = "next"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = RECURSIVE_SHAPES_STRUCT_SCHEMA)]
pub struct RecursiveShapesStruct {
    #[schema(schema = STRING)]
    pub string_field: String,
    #[schema(schema = INTEGER)]
    pub integer_field: i32,
    #[schema(schema = LIST)]
    pub list_field: Vec<String>,
    #[schema(schema = MAP)]
    pub map_field: IndexMap<String, String>,
    #[schema(schema = OPTIONAL)]
    pub optional_field: Option<String>,
    #[schema(schema = NEXT)]
    pub next: Option<Box<RecursiveShapesStruct>>,
}
