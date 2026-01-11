use smithy4rs_core::{
    IndexMap,
    derive::SmithyShape,
    schema::prelude::{INTEGER, STRING},
    smithy,
};

smithy!("test#InnerStruct": {
    structure INNER_STRUCT_SCHEMA {
        A: STRING = "field_a"
        B: STRING = "field_b"
        C: STRING = "field_c"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(INNER_STRUCT_SCHEMA)]
pub struct InnerStruct {
    #[smithy_schema(A)]
    pub field_a: String,
    #[smithy_schema(B)]
    pub field_b: String,
    #[smithy_schema(C)]
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
        SINGLE: INNER_STRUCT_SCHEMA = "single_nested"
        OPTIONAL: INNER_STRUCT_SCHEMA = "optional_nested"
        LIST: INNER_STRUCT_LIST_SCHEMA = "list_nested"
        MAP: INNER_STRUCT_MAP_SCHEMA = "map_nested"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(NESTED_COLLECTIONS_STRUCT_SCHEMA)]
pub struct NestedCollectionsStruct {
    #[smithy_schema(NAME)]
    pub name: String,
    #[smithy_schema(COUNT)]
    pub count: i32,
    #[smithy_schema(SINGLE)]
    pub single_nested: InnerStruct,
    #[smithy_schema(OPTIONAL)]
    pub optional_nested: Option<InnerStruct>,
    #[smithy_schema(LIST)]
    pub list_nested: Vec<InnerStruct>,
    #[smithy_schema(MAP)]
    pub map_nested: IndexMap<String, InnerStruct>,
}
