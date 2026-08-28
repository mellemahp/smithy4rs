use smithy4rs_core::{
    ByteBuffer,
    Instant,
    derive::SmithyShape,
    prelude::{
        BLOB,
        BOOLEAN,
        BYTE,
        DOUBLE,
        FLOAT,
        INTEGER,
        LONG,
        RequiredTrait,
        SHORT,
        STRING,
        TIMESTAMP,
    },
    smithy,
};

smithy!("com.test#AllPrimitivesStruct": {
    /// Schema for [`AllPrimitivesStruct`]
    structure ALL_PRIMITIVES_STRUCT_SCHEMA {
        @RequiredTrait::builder().build();
        stringField: STRING
        @RequiredTrait::builder().build();
        byteField: BYTE
        shortField: SHORT
        @RequiredTrait::builder().build();
        integerField: INTEGER
        longField: LONG
        floatField: FLOAT
        @RequiredTrait::builder().build();
        doubleField: DOUBLE
        booleanField: BOOLEAN
        blobField: BLOB
        @RequiredTrait::builder().build();
        timestampField: TIMESTAMP
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = ALL_PRIMITIVES_STRUCT_SCHEMA)]
pub struct AllPrimitivesStruct {
    pub string_field: String,
    pub byte_field: i8,
    pub short_field: Option<i16>,
    pub integer_field: i32,
    pub long_field: Option<i64>,
    pub float_field: Option<f32>,
    pub double_field: f64,
    pub boolean_field: Option<bool>,
    pub blob_field: Option<ByteBuffer>,
    pub timestamp_field: Instant,
}
