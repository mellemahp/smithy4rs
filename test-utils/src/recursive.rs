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
        stringField: STRING
        integerField: INTEGER
        listField: STRING_LIST_SCHEMA
        mapField: STRING_MAP_SCHEMA
        optionalField: STRING
        next: (@self)
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = RECURSIVE_SHAPES_STRUCT_SCHEMA)]
pub struct RecursiveShapesStruct {
    pub string_field: String,
    pub integer_field: i32,
    pub list_field: Vec<String>,
    pub map_field: IndexMap<String, String>,
    pub optional_field: Option<String>,
    pub next: Option<Box<RecursiveShapesStruct>>,
}
