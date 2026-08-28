use smithy4rs_core::{
    IndexMap,
    derive::SmithyShape,
    schema::prelude::{INTEGER, STRING},
    smithy,
};

smithy!(
    map test::StringMap {
        key: STRING
        value: STRING
    }
);

smithy!(
    list test::StringList {
        member: STRING
    }
);

smithy!(
    structure test::RecursiveShapesStruct {
        stringField: STRING
        integerField: INTEGER
        listField: STRING_LIST
        mapField: STRING_MAP
        optionalField: STRING
        next: (@self)
    }
);

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = RECURSIVE_SHAPES_STRUCT)]
pub struct RecursiveShapesStruct {
    pub string_field: String,
    pub integer_field: i32,
    pub list_field: Vec<String>,
    pub map_field: IndexMap<String, String>,
    pub optional_field: Option<String>,
    pub next: Option<Box<RecursiveShapesStruct>>,
}
