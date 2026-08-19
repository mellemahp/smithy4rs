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
#[schema(schema = ALL_PRIMITIVES_DEFAULTS_SCHEMA)]
pub struct AllPrimitivesDefaults {
    #[schema(schema = STRING, default = "default".to_string())]
    pub string_field: String,
    #[schema(schema = BYTE, default = 0)]
    pub byte_field: i8,
    #[schema(schema = SHORT, default = 0)]
    pub short_field: i16,
    #[schema(schema = INTEGER, default = 0)]
    pub integer_field: i32,
    #[schema(schema = LONG, default = 0)]
    pub long_field: i64,
    #[schema(schema = FLOAT, default = 0.0)]
    pub float_field: f32,
    #[schema(schema = DOUBLE, default = 0.0)]
    pub double_field: f64,
    #[schema(schema = BOOLEAN, default = true)]
    pub boolean_field: bool,
    #[schema(schema = BLOB, default = ByteBuffer::default())]
    pub blob_field: ByteBuffer,
    #[schema(schema = TIMESTAMP, default = Instant::from_epoch_milliseconds(1000000).expect("Epoch milliseconds must be set"))]
    pub timestamp_field: Instant,
}
