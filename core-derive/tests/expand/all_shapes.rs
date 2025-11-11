use std::sync::{Arc, LazyLock};

use indexmap::IndexMap;
use smithy4rs_core::{
    // BigDecimal, BigInt,
    ByteBuffer,
    Instant,
    prelude::*,
    schema::{Schema, SchemaBuilder, SchemaRef, ShapeId},
    traits,
};
use smithy4rs_core_derive::{DeserializableStruct, SerializableStruct};

pub static STRING_LIST_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::list_builder("test#StringList", traits![])
        .put_member("member", &STRING, traits![])
        .build()
});

pub static STRING_MAP_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::map_builder(ShapeId::from("test#StringMap"), traits![])
        .put_member("key", &STRING, traits![])
        .put_member("value", &STRING, traits![])
        .build()
});

// Use Arc<SchemaBuilder> for recursive schemas
pub static ALL_SHAPES_BUILDER: LazyLock<Arc<SchemaBuilder>> = LazyLock::new(|| {
    Arc::new(Schema::structure_builder(
        ShapeId::from("test#AllShapes"),
        traits![],
    ))
});

pub static ALL_SHAPES_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_BUILDER
        .put_member("string_field", &STRING, traits![])
        .put_member("byte_field", &BYTE, traits![])
        .put_member("short_field", &SHORT, traits![])
        .put_member("integer_field", &INTEGER, traits![])
        .put_member("long_field", &LONG, traits![])
        .put_member("float_field", &FLOAT, traits![])
        .put_member("double_field", &DOUBLE, traits![])
        .put_member("boolean_field", &BOOLEAN, traits![])
        .put_member("blob_field", &BLOB, traits![])
        .put_member("timestamp_field", &TIMESTAMP, traits![])
        // TODO: Uncomment when BigInt/BigDecimal serialization is fixed
        // .put_member("big_integer_field", &BIG_INTEGER, traits![])
        // .put_member("big_decimal_field", &BIG_DECIMAL, traits![])
        .put_member("list_field", &STRING_LIST_SCHEMA, traits![])
        .put_member("map_field", &STRING_MAP_SCHEMA, traits![])
        .put_member("optional_field", &STRING, traits![])
        .put_member("recursive_field", &*ALL_SHAPES_BUILDER, traits![]) // Self-reference!
        .build()
});

static STRING_FIELD: LazyLock<&SchemaRef> =
    LazyLock::new(|| ALL_SHAPES_SCHEMA.expect_member("string_field"));
static BYTE_FIELD: LazyLock<&SchemaRef> =
    LazyLock::new(|| ALL_SHAPES_SCHEMA.expect_member("byte_field"));
static SHORT_FIELD: LazyLock<&SchemaRef> =
    LazyLock::new(|| ALL_SHAPES_SCHEMA.expect_member("short_field"));
static INTEGER_FIELD: LazyLock<&SchemaRef> =
    LazyLock::new(|| ALL_SHAPES_SCHEMA.expect_member("integer_field"));
static LONG_FIELD: LazyLock<&SchemaRef> =
    LazyLock::new(|| ALL_SHAPES_SCHEMA.expect_member("long_field"));
static FLOAT_FIELD: LazyLock<&SchemaRef> =
    LazyLock::new(|| ALL_SHAPES_SCHEMA.expect_member("float_field"));
static DOUBLE_FIELD: LazyLock<&SchemaRef> =
    LazyLock::new(|| ALL_SHAPES_SCHEMA.expect_member("double_field"));
static BOOLEAN_FIELD: LazyLock<&SchemaRef> =
    LazyLock::new(|| ALL_SHAPES_SCHEMA.expect_member("boolean_field"));
static BLOB_FIELD: LazyLock<&SchemaRef> =
    LazyLock::new(|| ALL_SHAPES_SCHEMA.expect_member("blob_field"));
static TIMESTAMP_FIELD: LazyLock<&SchemaRef> =
    LazyLock::new(|| ALL_SHAPES_SCHEMA.expect_member("timestamp_field"));
static LIST_FIELD: LazyLock<&SchemaRef> =
    LazyLock::new(|| ALL_SHAPES_SCHEMA.expect_member("list_field"));
static MAP_FIELD: LazyLock<&SchemaRef> =
    LazyLock::new(|| ALL_SHAPES_SCHEMA.expect_member("map_field"));
static OPTIONAL_FIELD: LazyLock<&SchemaRef> =
    LazyLock::new(|| ALL_SHAPES_SCHEMA.expect_member("optional_field"));
static RECURSIVE_FIELD: LazyLock<&SchemaRef> =
    LazyLock::new(|| ALL_SHAPES_SCHEMA.expect_member("recursive_field"));

#[derive(SerializableStruct, DeserializableStruct, Debug, PartialEq)]
#[smithy_schema(ALL_SHAPES_SCHEMA)]
pub struct AllShapes {
    #[smithy_schema(STRING_FIELD)]
    pub string_field: String,
    #[smithy_schema(BYTE_FIELD)]
    pub byte_field: i8,
    #[smithy_schema(SHORT_FIELD)]
    pub short_field: i16,
    #[smithy_schema(INTEGER_FIELD)]
    pub integer_field: i32,
    #[smithy_schema(LONG_FIELD)]
    pub long_field: i64,
    #[smithy_schema(FLOAT_FIELD)]
    pub float_field: f32,
    #[smithy_schema(DOUBLE_FIELD)]
    pub double_field: f64,
    #[smithy_schema(BOOLEAN_FIELD)]
    pub boolean_field: bool,
    #[smithy_schema(BLOB_FIELD)]
    pub blob_field: ByteBuffer,
    #[smithy_schema(TIMESTAMP_FIELD)]
    pub timestamp_field: Instant,
    // TODO: Uncomment when BigInt/BigDecimal serialization is fixed
    // #[smithy_schema(BIG_INTEGER_FIELD)]
    // pub big_integer_field: BigInt,
    // #[smithy_schema(BIG_DECIMAL_FIELD)]
    // pub big_decimal_field: BigDecimal,
    #[smithy_schema(LIST_FIELD)]
    pub list_field: Vec<String>,
    #[smithy_schema(MAP_FIELD)]
    pub map_field: IndexMap<String, String>,
    #[smithy_schema(OPTIONAL_FIELD)]
    pub optional_field: Option<String>,
    #[smithy_schema(RECURSIVE_FIELD)]
    pub recursive_field: Option<Box<AllShapes>>,
}
