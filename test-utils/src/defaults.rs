#![allow(dead_code)]

use smithy4rs_core::{ByteBuffer, Instant, derive::SmithyShape, prelude::*, smithy};

smithy!(
    structure test::AllPrimitivesDefaults {
        stringField: STRING
        byteField: BYTE
        shortField: SHORT
        integerField: INTEGER
        longField: LONG
        floatField: FLOAT
        doubleField: DOUBLE
        booleanField: BOOLEAN
        blobField: BLOB
        timestampField: TIMESTAMP
    }
);

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = ALL_PRIMITIVES_DEFAULTS)]
pub struct AllPrimitivesDefaults {
    #[schema(default = "default".to_string())]
    pub string_field: String,
    #[schema(default = 0)]
    pub byte_field: i8,
    #[schema(default = 0)]
    pub short_field: i16,
    #[schema(default = 0)]
    pub integer_field: i32,
    #[schema(default = 0)]
    pub long_field: i64,
    #[schema(default = 0.0)]
    pub float_field: f32,
    #[schema(default = 0.0)]
    pub double_field: f64,
    #[schema(default = true)]
    pub boolean_field: bool,
    #[schema(default = ByteBuffer::default())]
    pub blob_field: ByteBuffer,
    #[schema(default = Instant::from_epoch_milliseconds(1000000).expect("Epoch milliseconds must be set"))]
    pub timestamp_field: Instant,
}
