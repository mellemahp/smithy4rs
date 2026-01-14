#![allow(dead_code)]

use smithy4rs_core::{ByteBuffer, Instant, derive::SmithyShape, prelude::*, smithy};

smithy!("test#AllPrimitivesDefaults": {
    structure ALL_PRIMITIVES_DEFAULTS_SCHEMA {
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

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(ALL_PRIMITIVES_DEFAULTS_SCHEMA)]
pub struct AllPrimitivesDefaults {
    #[smithy_schema(STRING)]
    #[default("default".to_string())]
    pub string_field: String,
    #[smithy_schema(BYTE)]
    #[default(0)]
    pub byte_field: i8,
    #[smithy_schema(SHORT)]
    #[default(0)]
    pub short_field: i16,
    #[smithy_schema(INTEGER)]
    #[default(0)]
    pub integer_field: i32,
    #[smithy_schema(LONG)]
    #[default(0)]
    pub long_field: i64,
    #[smithy_schema(FLOAT)]
    #[default(0.0)]
    pub float_field: f32,
    #[smithy_schema(DOUBLE)]
    #[default(0.0)]
    pub double_field: f64,
    #[smithy_schema(BOOLEAN)]
    #[default(true)]
    pub boolean_field: bool,
    #[smithy_schema(BLOB)]
    #[default(ByteBuffer::default())]
    pub blob_field: ByteBuffer,
    #[smithy_schema(TIMESTAMP)]
    #[default(Instant::from_epoch_milliseconds(1000000).expect("Epoch milliseconds must be set"))]
    pub timestamp_field: Instant,
}
