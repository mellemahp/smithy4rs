//! Test utilities and common test shapes for smithy4rs
//!
//! This crate provides reusable test shapes and schemas following Smithy protocol test
//! conventions. Shape names are self-describing to clearly indicate what they test.

use std::sync::LazyLock;

use indexmap::IndexMap;
use smithy4rs_core::{
    ByteBuffer, Instant, lazy_schema,
    prelude::*,
    schema::{Schema, SchemaRef, ShapeId},
    traits,
};
use smithy4rs_core_derive::{DeserializableStruct, SchemaShape, SerializableStruct, SmithyStruct};

// ============================================================================
// Basic Collection Types
// ============================================================================

lazy_schema!(
    STRING_LIST_SCHEMA,
    Schema::list_builder("test#StringList", traits![]),
    ("member", STRING, traits![])
);

lazy_schema!(
    STRING_MAP_SCHEMA,
    Schema::map_builder(ShapeId::from("test#StringMap"), traits![]),
    ("key", STRING, traits![]),
    ("value", STRING, traits![])
);

lazy_schema!(
    INTEGER_LIST_SCHEMA,
    Schema::list_builder("test#IntegerList", traits![]),
    ("member", INTEGER, traits![])
);

// ============================================================================
// Primitive Type Test Structures
// ============================================================================

/// Tests all Smithy primitive types in a single structure
lazy_schema!(
    ALL_PRIMITIVES_STRUCT_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#AllPrimitivesStruct"), traits![]),
    (ALL_PRIMITIVES_STRING, "string_field", STRING, traits![]),
    (ALL_PRIMITIVES_BYTE, "byte_field", BYTE, traits![]),
    (ALL_PRIMITIVES_SHORT, "short_field", SHORT, traits![]),
    (ALL_PRIMITIVES_INTEGER, "integer_field", INTEGER, traits![]),
    (ALL_PRIMITIVES_LONG, "long_field", LONG, traits![]),
    (ALL_PRIMITIVES_FLOAT, "float_field", FLOAT, traits![]),
    (ALL_PRIMITIVES_DOUBLE, "double_field", DOUBLE, traits![]),
    (ALL_PRIMITIVES_BOOLEAN, "boolean_field", BOOLEAN, traits![]),
    (ALL_PRIMITIVES_BLOB, "blob_field", BLOB, traits![]),
    (
        ALL_PRIMITIVES_TIMESTAMP,
        "timestamp_field",
        TIMESTAMP,
        traits![]
    )
);

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

/// Tests optional field handling
lazy_schema!(
    OPTIONAL_FIELDS_STRUCT_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#OptionalFieldsStruct"), traits![]),
    (OPTIONAL_REQUIRED, "required_field", STRING, traits![]),
    (OPTIONAL_OPTIONAL, "optional_field", STRING, traits![])
);

#[derive(SmithyStruct, Debug, PartialEq)]
#[smithy_schema(OPTIONAL_FIELDS_STRUCT_SCHEMA)]
pub struct OptionalFieldsStruct {
    #[smithy_schema(OPTIONAL_REQUIRED)]
    pub required_field: String,
    #[smithy_schema(OPTIONAL_OPTIONAL)]
    pub optional_field: Option<String>,
}

/// Tests numeric type handling across all integer and floating point types
lazy_schema!(
    NUMERIC_TYPES_STRUCT_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#NumericTypesStruct"), traits![]),
    (NUMERIC_BYTE, "byte_val", BYTE, traits![]),
    (NUMERIC_SHORT, "short_val", SHORT, traits![]),
    (NUMERIC_INT, "int_val", INTEGER, traits![]),
    (NUMERIC_LONG, "long_val", LONG, traits![]),
    (NUMERIC_FLOAT, "float_val", FLOAT, traits![]),
    (NUMERIC_DOUBLE, "double_val", DOUBLE, traits![])
);

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

/// Simple two-field structure for basic macro expansion tests
lazy_schema!(
    SIMPLE_STRUCT_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#SimpleStruct"), traits![]),
    (SIMPLE_FIELD_A, "field_a", STRING, traits![]),
    (SIMPLE_FIELD_B, "field_b", INTEGER, traits![])
);

#[derive(SmithyStruct, Debug, PartialEq)]
#[smithy_schema(SIMPLE_STRUCT_SCHEMA)]
pub struct SimpleStruct {
    #[smithy_schema(SIMPLE_FIELD_A)]
    pub field_a: String,
    #[smithy_schema(SIMPLE_FIELD_B)]
    pub field_b: i32,
}

// ============================================================================
// Recursive Structure Test Types
// ============================================================================

/// Tests recursive self-reference using the (@self) syntax in lazy_schema
lazy_schema!(
    RECURSIVE_SHAPES_STRUCT_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#RecursiveShapesStruct"), traits![]),
    (RECURSIVE_SHAPES_STRING, "string_field", STRING, traits![]),
    (RECURSIVE_SHAPES_INTEGER, "integer_field", INTEGER, traits![]),
    (RECURSIVE_SHAPES_LIST, "list_field", STRING_LIST_SCHEMA, traits![]),
    (RECURSIVE_SHAPES_MAP, "map_field", STRING_MAP_SCHEMA, traits![]),
    (RECURSIVE_SHAPES_OPTIONAL, "optional_field", STRING, traits![]),
    (RECURSIVE_SHAPES_NEXT, "next", (@self), traits![])
);

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
    pub next: Option<Box<RecursiveShapesStruct>>,
}

// ============================================================================
// Nested Collections Test
// ============================================================================

/// Inner struct for testing nested structures
lazy_schema!(
    INNER_STRUCT_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#InnerStruct"), traits![]),
    (INNER_FIELD_A, "field_a", STRING, traits![]),
    (INNER_FIELD_B, "field_b", STRING, traits![]),
    (INNER_FIELD_C, "field_c", STRING, traits![])
);

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

pub static INNER_STRUCT_LIST_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::list_builder(ShapeId::from("test#InnerStructList"), traits![])
        .put_member("member", &INNER_STRUCT_SCHEMA, traits![])
        .build()
});

pub static INNER_STRUCT_MAP_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::map_builder(ShapeId::from("test#InnerStructMap"), traits![])
        .put_member("key", &STRING, traits![])
        .put_member("value", &INNER_STRUCT_SCHEMA, traits![])
        .build()
});

/// Tests nested structures in various collection types
lazy_schema!(
    NESTED_COLLECTIONS_STRUCT_SCHEMA,
    Schema::structure_builder(ShapeId::from("test#NestedCollectionsStruct"), traits![]),
    (NESTED_NAME, "name", STRING, traits![]),
    (NESTED_COUNT, "count", INTEGER, traits![]),
    (
        NESTED_SINGLE,
        "single_nested",
        INNER_STRUCT_SCHEMA,
        traits![]
    ),
    (
        NESTED_OPTIONAL,
        "optional_nested",
        INNER_STRUCT_SCHEMA,
        traits![]
    ),
    (
        NESTED_LIST,
        "list_nested",
        INNER_STRUCT_LIST_SCHEMA,
        traits![]
    ),
    (NESTED_MAP, "map_nested", INNER_STRUCT_MAP_SCHEMA, traits![])
);

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
