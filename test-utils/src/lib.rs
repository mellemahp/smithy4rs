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
        ALL_PRIMITIVES_STRING: STRING = "string_field"
        ALL_PRIMITIVES_BYTE: BYTE = "byte_field"
        ALL_PRIMITIVES_SHORT: SHORT = "short_field"
        ALL_PRIMITIVES_INTEGER: INTEGER = "integer_field"
        ALL_PRIMITIVES_LONG: LONG = "long_field"
        ALL_PRIMITIVES_FLOAT: FLOAT = "float_field"
        ALL_PRIMITIVES_DOUBLE: DOUBLE = "double_field"
        ALL_PRIMITIVES_BOOLEAN: BOOLEAN = "boolean_field"
        ALL_PRIMITIVES_BLOB: BLOB = "blob_field"
        ALL_PRIMITIVES_TIMESTAMP: TIMESTAMP = "timestamp_field"
    }
});

#[derive(SmithyStruct, Debug, PartialEq)]
#[smithy_schema(ALL_PRIMITIVES_STRUCT_SCHEMA)]
pub struct AllPrimitivesStruct {
    #[smithy_schema(ALL_PRIMITIVES_STRING)]
    pub string_field: String,
    #[smithy_schema(ALL_PRIMITIVES_BYTE)]
    pub byte_field: i8,
    #[smithy_schema(ALL_PRIMITIVES_SHORT)]
    pub short_field: i16,
    #[smithy_schema(ALL_PRIMITIVES_INTEGER)]
    pub integer_field: i32,
    #[smithy_schema(ALL_PRIMITIVES_LONG)]
    pub long_field: i64,
    #[smithy_schema(ALL_PRIMITIVES_FLOAT)]
    pub float_field: f32,
    #[smithy_schema(ALL_PRIMITIVES_DOUBLE)]
    pub double_field: f64,
    #[smithy_schema(ALL_PRIMITIVES_BOOLEAN)]
    pub boolean_field: bool,
    #[smithy_schema(ALL_PRIMITIVES_BLOB)]
    pub blob_field: ByteBuffer,
    #[smithy_schema(ALL_PRIMITIVES_TIMESTAMP)]
    pub timestamp_field: Instant,
}

smithy!("test#OptionalFieldsStruct": {
    structure OPTIONAL_FIELDS_STRUCT_SCHEMA {
        OPTIONAL_REQUIRED: STRING = "required_field"
        OPTIONAL_OPTIONAL: STRING = "optional_field"
    }
});

#[derive(SmithyStruct, Debug, PartialEq)]
#[smithy_schema(OPTIONAL_FIELDS_STRUCT_SCHEMA)]
pub struct OptionalFieldsStruct {
    #[smithy_schema(OPTIONAL_REQUIRED)]
    pub required_field: String,
    #[smithy_schema(OPTIONAL_OPTIONAL)]
    pub optional_field: Option<String>,
}
smithy!("test#NumericTypesStruct": {
    structure NUMERIC_TYPES_STRUCT_SCHEMA {
        NUMERIC_BYTE: BYTE = "byte_val"
        NUMERIC_SHORT: SHORT = "short_val"
        NUMERIC_INT: INTEGER = "int_val"
        NUMERIC_LONG: LONG = "long_val"
        NUMERIC_FLOAT: FLOAT = "float_val"
        NUMERIC_DOUBLE: DOUBLE = "double_val"
    }
});

#[derive(SmithyStruct, Debug, PartialEq)]
#[smithy_schema(NUMERIC_TYPES_STRUCT_SCHEMA)]
pub struct NumericTypesStruct {
    #[smithy_schema(NUMERIC_BYTE)]
    pub byte_val: i8,
    #[smithy_schema(NUMERIC_SHORT)]
    pub short_val: i16,
    #[smithy_schema(NUMERIC_INT)]
    pub int_val: i32,
    #[smithy_schema(NUMERIC_LONG)]
    pub long_val: i64,
    #[smithy_schema(NUMERIC_FLOAT)]
    pub float_val: f32,
    #[smithy_schema(NUMERIC_DOUBLE)]
    pub double_val: f64,
}

smithy!("test#SimpleStruct": {
    structure SIMPLE_STRUCT_SCHEMA {
        SIMPLE_FIELD_A: STRING = "field_a"
        SIMPLE_FIELD_B: INTEGER = "field_b"
    }
});

#[derive(SmithyStruct, Debug, PartialEq)]
#[smithy_schema(SIMPLE_STRUCT_SCHEMA)]
pub struct SimpleStruct {
    #[smithy_schema(SIMPLE_FIELD_A)]
    pub field_a: String,
    #[smithy_schema(SIMPLE_FIELD_B)]
    pub field_b: i32,
}
smithy!("test#RecursiveShapesStruct": {
    structure RECURSIVE_SHAPES_STRUCT_SCHEMA {
        RECURSIVE_SHAPES_STRING: STRING = "string_field"
        RECURSIVE_SHAPES_INTEGER: INTEGER = "integer_field"
        RECURSIVE_SHAPES_LIST: STRING_LIST_SCHEMA = "list_field"
        RECURSIVE_SHAPES_MAP: STRING_MAP_SCHEMA = "map_field"
        RECURSIVE_SHAPES_OPTIONAL: STRING = "optional_field"
        RECURSIVE_SHAPES_NEXT: (@self) = "next"
    }
});

#[derive(SmithyStruct, Debug, PartialEq)]
#[smithy_schema(RECURSIVE_SHAPES_STRUCT_SCHEMA)]
pub struct RecursiveShapesStruct {
    #[smithy_schema(RECURSIVE_SHAPES_STRING)]
    pub string_field: String,
    #[smithy_schema(RECURSIVE_SHAPES_INTEGER)]
    pub integer_field: i32,
    #[smithy_schema(RECURSIVE_SHAPES_LIST)]
    pub list_field: Vec<String>,
    #[smithy_schema(RECURSIVE_SHAPES_MAP)]
    pub map_field: IndexMap<String, String>,
    #[smithy_schema(RECURSIVE_SHAPES_OPTIONAL)]
    pub optional_field: Option<String>,
    #[smithy_schema(RECURSIVE_SHAPES_NEXT)]
    // TODO: Does this actually need boxing? Shapes themselves shouldnt
    //       recurse, only their schemas.
    pub next: Option<Box<RecursiveShapesStruct>>,
}
smithy!("test#InnerStruct": {
    structure INNER_STRUCT_SCHEMA {
        INNER_FIELD_A: STRING = "field_a"
        INNER_FIELD_B: STRING = "field_b"
        INNER_FIELD_C: STRING = "field_c"
    }
});

#[derive(SmithyStruct, Debug, PartialEq)]
#[smithy_schema(INNER_STRUCT_SCHEMA)]
pub struct InnerStruct {
    #[smithy_schema(INNER_FIELD_A)]
    pub field_a: String,
    #[smithy_schema(INNER_FIELD_B)]
    pub field_b: String,
    #[smithy_schema(INNER_FIELD_C)]
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
        NESTED_NAME: STRING = "name"
        NESTED_COUNT: INTEGER = "count"
        NESTED_SINGLE: INNER_STRUCT_SCHEMA = "single_nested"
        NESTED_OPTIONAL: INNER_STRUCT_SCHEMA = "optional_nested"
        NESTED_LIST: INNER_STRUCT_LIST_SCHEMA = "list_nested"
        NESTED_MAP: INNER_STRUCT_MAP_SCHEMA = "map_nested"
    }
});

#[derive(SmithyStruct, Debug, PartialEq)]
#[smithy_schema(NESTED_COLLECTIONS_STRUCT_SCHEMA)]
pub struct NestedCollectionsStruct {
    #[smithy_schema(NESTED_NAME)]
    pub name: String,
    #[smithy_schema(NESTED_COUNT)]
    pub count: i32,
    #[smithy_schema(NESTED_SINGLE)]
    pub single_nested: InnerStruct,
    #[smithy_schema(NESTED_OPTIONAL)]
    pub optional_nested: Option<InnerStruct>,
    #[smithy_schema(NESTED_LIST)]
    pub list_nested: Vec<InnerStruct>,
    #[smithy_schema(NESTED_MAP)]
    pub map_nested: IndexMap<String, InnerStruct>,
}
