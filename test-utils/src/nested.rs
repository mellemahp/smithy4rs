use smithy4rs_core::{
    IndexMap,
    derive::SmithyShape,
    schema::prelude::{INTEGER, STRING},
    smithy,
};

smithy!(
    structure test::InnerStruct {
        fieldA: STRING
        fieldB: STRING
        fieldC: STRING
    }
);

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = INNER_STRUCT)]
pub struct InnerStruct {
    pub field_a: String,
    pub field_b: String,
    pub field_c: String,
}

smithy!(
    list test::InnerStructList {
        member: INNER_STRUCT
    }
);
smithy!(
    map test::InnerStructMap {
        key: STRING
        value: INNER_STRUCT
    }
);
smithy!(
    structure test::NestedCollectionsStruct {
        name: STRING
        count: INTEGER
        singleNested: INNER_STRUCT
        optionalNested: INNER_STRUCT
        listNested: INNER_STRUCT_LIST
        mapNested: INNER_STRUCT_MAP
    }
);

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = NESTED_COLLECTIONS_STRUCT)]
pub struct NestedCollectionsStruct {
    pub name: String,
    pub count: i32,
    pub single_nested: InnerStruct,
    pub optional_nested: Option<InnerStruct>,
    pub list_nested: Vec<InnerStruct>,
    pub map_nested: IndexMap<String, InnerStruct>,
}
