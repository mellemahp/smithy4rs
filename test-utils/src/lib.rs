//! Test utilities and common test schemas + shapes

// TODO(verify): We don't add required to anything yet
// TODO(test): Add constraint test shapes once we have validation

use indexmap::IndexMap;
use smithy4rs_core::{ByteBuffer, Instant, prelude::*, smithy};
use smithy4rs_core_derive::SmithyStruct;

smithy!("test#StringList": {
    list STRING_LIST_SCHEMA {
        member: STRING
    }
});

smithy!("test#StringMap": {
    map STRING_MAP_SCHEMA {
        key: STRING
        value: STRING
    }
});

smithy!("test#IntegerList": {
    list INTEGER_LIST_SCHEMA {
        member: INTEGER
    }
});

smithy!("test#AllPrimitivesStruct": {
    structure ALL_PRIMITIVES_STRUCT_SCHEMA {
        STRING: STRING = "string_field"
        BYTE: BYTE = "byte_field"
        SHORT: SHORT = "short_field"
        INTEGER: INTEGER = "integer_field"
        LONG: LONG = "long_field"
        FLOAT: FLOAT = "float_field"
        DOUBLE: DOUBLE = "double_field"
        BOOLEAN: BOOLEAN = "boolean_field"
        BLOB: BLOB = "blob_field"
        TIMESTAMP: TIMESTAMP = "timestamp_field"
    }
});

#[derive(SmithyStruct, Debug, PartialEq, Clone)]
#[smithy_schema(ALL_PRIMITIVES_STRUCT_SCHEMA)]
pub struct AllPrimitivesStruct {
    #[smithy_schema(STRING)]
    pub string_field: String,
    #[smithy_schema(BYTE)]
    pub byte_field: i8,
    #[smithy_schema(SHORT)]
    pub short_field: i16,
    #[smithy_schema(INTEGER)]
    pub integer_field: i32,
    #[smithy_schema(LONG)]
    pub long_field: i64,
    #[smithy_schema(FLOAT)]
    pub float_field: f32,
    #[smithy_schema(DOUBLE)]
    pub double_field: f64,
    #[smithy_schema(BOOLEAN)]
    pub boolean_field: bool,
    #[smithy_schema(BLOB)]
    pub blob_field: ByteBuffer,
    #[smithy_schema(TIMESTAMP)]
    pub timestamp_field: Instant,
}

smithy!("test#OptionalFieldsStruct": {
    structure OPTIONAL_FIELDS_STRUCT_SCHEMA {
        REQUIRED: STRING = "required_field"
        OPTIONAL: STRING = "optional_field"
    }
});

#[derive(SmithyStruct, Debug, PartialEq, Clone)]
#[smithy_schema(OPTIONAL_FIELDS_STRUCT_SCHEMA)]
pub struct OptionalFieldsStruct {
    #[smithy_schema(REQUIRED)]
    pub required_field: String,
    #[smithy_schema(OPTIONAL)]
    pub optional_field: Option<String>,
}

smithy!("test#NumericTypesStruct": {
    structure NUMERIC_TYPES_STRUCT_SCHEMA {
        BYTE: BYTE = "byte_val"
        SHORT: SHORT = "short_val"
        INT: INTEGER = "int_val"
        LONG: LONG = "long_val"
        FLOAT: FLOAT = "float_val"
        DOUBLE: DOUBLE = "double_val"
    }
});

#[derive(SmithyStruct, Debug, PartialEq, Clone)]
#[smithy_schema(NUMERIC_TYPES_STRUCT_SCHEMA)]
pub struct NumericTypesStruct {
    #[smithy_schema(BYTE)]
    pub byte_val: i8,
    #[smithy_schema(SHORT)]
    pub short_val: i16,
    #[smithy_schema(INT)]
    pub int_val: i32,
    #[smithy_schema(LONG)]
    pub long_val: i64,
    #[smithy_schema(FLOAT)]
    pub float_val: f32,
    #[smithy_schema(DOUBLE)]
    pub double_val: f64,
}

smithy!("test#SimpleStruct": {
    structure SIMPLE_STRUCT_SCHEMA {
        A: STRING = "field_a"
        B: INTEGER = "field_b"
    }
});

#[derive(SmithyStruct, Debug, PartialEq, Clone)]
#[smithy_schema(SIMPLE_STRUCT_SCHEMA)]
pub struct SimpleStruct {
    #[smithy_schema(A)]
    pub field_a: String,
    #[smithy_schema(B)]
    pub field_b: i32,
}

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

#[derive(SmithyStruct, Debug, PartialEq, Clone)]
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
smithy!("test#InnerStruct": {
    structure INNER_STRUCT_SCHEMA {
        A: STRING = "field_a"
        B: STRING = "field_b"
        C: STRING = "field_c"
    }
});

#[derive(SmithyStruct, Debug, PartialEq, Clone)]
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

#[derive(SmithyStruct, Debug, PartialEq, Clone)]
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
