use indexmap::IndexMap;
use smithy4rs_core::{prelude::*, smithy};
use smithy4rs_core_derive::SmithyShape;

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
        STRING: STRING = "string_field"
        INTEGER: INTEGER = "integer_field"
        LIST: STRING_LIST_SCHEMA = "list_field"
        MAP: STRING_MAP_SCHEMA = "map_field"
        OPTIONAL: STRING = "optional_field"
        NEXT: (@self) = "next"
    }
});

#[derive(SmithyShape, Debug, PartialEq, Clone)]
#[smithy_schema(RECURSIVE_SHAPES_STRUCT_SCHEMA)]
pub struct RecursiveShapesStruct {
    #[smithy_schema(STRING)]
    pub string_field: String,
    #[smithy_schema(INTEGER)]
    pub integer_field: i32,
    #[smithy_schema(LIST)]
    pub list_field: Vec<String>,
    #[smithy_schema(MAP)]
    pub map_field: IndexMap<String, String>,
    #[smithy_schema(OPTIONAL)]
    pub optional_field: Option<String>,
    #[smithy_schema(NEXT)]
    pub next: Option<Box<RecursiveShapesStruct>>,
}
