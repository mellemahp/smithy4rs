use smithy4rs_core::{
    IndexMap,
    derive::SmithyShape,
    schema::prelude::{INTEGER, STRING},
    smithy,
};

smithy!("test#InnerStruct": {
    structure INNER_STRUCT_SCHEMA {
        fieldA: STRING
        fieldB: STRING
        fieldC: STRING
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = INNER_STRUCT_SCHEMA)]
pub struct InnerStruct {
    pub field_a: String,
    pub field_b: String,
    pub field_c: String,
}

smithy!("test#InnerStructList": {
    list INNER_STRUCT_LIST_SCHEMA {
        member: INNER_STRUCT_SCHEMA
    }
});
smithy!("test#InnerStructMap": {
    map INNER_STRUCT_MAP_SCHEMA {
        key: STRING
        value: INNER_STRUCT_SCHEMA
    }
});
smithy!("test#NestedCollectionsStruct": {
    structure NESTED_COLLECTIONS_STRUCT_SCHEMA {
        name: STRING
        count: INTEGER
        singleNested: INNER_STRUCT_SCHEMA
        optionalNested: INNER_STRUCT_SCHEMA
        listNested: INNER_STRUCT_LIST_SCHEMA
        mapNested: INNER_STRUCT_MAP_SCHEMA
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = NESTED_COLLECTIONS_STRUCT_SCHEMA)]
pub struct NestedCollectionsStruct {
    pub name: String,
    pub count: i32,
    pub single_nested: InnerStruct,
    pub optional_nested: Option<InnerStruct>,
    pub list_nested: Vec<InnerStruct>,
    pub map_nested: IndexMap<String, InnerStruct>,
}
