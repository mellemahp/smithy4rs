use smithy4rs_core::{
    IndexMap,
    derive::SmithyShape,
    schema::prelude::{INTEGER, STRING},
    smithy,
};

smithy!("test#InnerStruct": {
    structure INNER_STRUCT_SCHEMA {
        A: STRING = "fieldA"
        B: STRING = "fieldB"
        C: STRING = "fieldC"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = INNER_STRUCT_SCHEMA)]
pub struct InnerStruct {
    #[schema(schema = A)]
    pub field_a: String,
    #[schema(schema = B)]
    pub field_b: String,
    #[schema(schema = C)]
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
        NAME: STRING = "name"
        COUNT: INTEGER = "count"
        SINGLE: INNER_STRUCT_SCHEMA = "singleNested"
        OPTIONAL: INNER_STRUCT_SCHEMA = "optionalNested"
        LIST: INNER_STRUCT_LIST_SCHEMA = "listNested"
        MAP: INNER_STRUCT_MAP_SCHEMA = "mapNested"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = NESTED_COLLECTIONS_STRUCT_SCHEMA)]
pub struct NestedCollectionsStruct {
    #[schema(schema = NAME)]
    pub name: String,
    #[schema(schema = COUNT)]
    pub count: i32,
    #[schema(schema = SINGLE)]
    pub single_nested: InnerStruct,
    #[schema(schema = OPTIONAL)]
    pub optional_nested: Option<InnerStruct>,
    #[schema(schema = LIST)]
    pub list_nested: Vec<InnerStruct>,
    #[schema(schema = MAP)]
    pub map_nested: IndexMap<String, InnerStruct>,
}
