//! Test utilities and common test shapes for smithy4rs
//!
//! This crate provides reusable test shapes and schemas following Smithy protocol test
//! conventions. Shape names are self-describing to clearly indicate what they test.

use indexmap::IndexMap;
use smithy4rs_core::{
    lazy_schema,
    prelude::*,
    schema::{Schema, ShapeId},
    traits,
    ByteBuffer,
    Instant,
};
use smithy4rs_core_derive::{DeserializableStruct, SerializableStruct};

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
    (ALL_PRIMITIVES_TIMESTAMP, "timestamp_field", TIMESTAMP, traits![])
);

#[derive(SerializableStruct, DeserializableStruct, Debug, PartialEq)]
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

#[derive(Debug, PartialEq, SerializableStruct, DeserializableStruct)]
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

#[derive(Debug, PartialEq, SerializableStruct, DeserializableStruct)]
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

#[derive(SerializableStruct, DeserializableStruct, Debug, PartialEq)]
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

#[derive(SerializableStruct, DeserializableStruct, Debug, PartialEq)]
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
