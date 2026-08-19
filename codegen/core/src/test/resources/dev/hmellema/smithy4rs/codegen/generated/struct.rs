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
        STRING_FIELD: STRING = "string_field"
        @RequiredTrait::builder().build();
        BYTE_FIELD: BYTE = "byte_field"
        SHORT_FIELD: SHORT = "short_field"
        @RequiredTrait::builder().build();
        INTEGER_FIELD: INTEGER = "integer_field"
        LONG_FIELD: LONG = "long_field"
        FLOAT_FIELD: FLOAT = "float_field"
        @RequiredTrait::builder().build();
        DOUBLE_FIELD: DOUBLE = "double_field"
        BOOLEAN_FIELD: BOOLEAN = "boolean_field"
        BLOB_FIELD: BLOB = "blob_field"
        @RequiredTrait::builder().build();
        TIMESTAMP_FIELD: TIMESTAMP = "timestamp_field"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = ALL_PRIMITIVES_STRUCT_SCHEMA)]
pub struct AllPrimitivesStruct {
    #[schema(schema = STRING_FIELD)]
    pub string_field: String,
    #[schema(schema = BYTE_FIELD)]
    pub byte_field: i8,
    #[schema(schema = SHORT_FIELD)]
    pub short_field: Option<i16>,
    #[schema(schema = INTEGER_FIELD)]
    pub integer_field: i32,
    #[schema(schema = LONG_FIELD)]
    pub long_field: Option<i64>,
    #[schema(schema = FLOAT_FIELD)]
    pub float_field: Option<f32>,
    #[schema(schema = DOUBLE_FIELD)]
    pub double_field: f64,
    #[schema(schema = BOOLEAN_FIELD)]
    pub boolean_field: Option<bool>,
    #[schema(schema = BLOB_FIELD)]
    pub blob_field: Option<ByteBuffer>,
    #[schema(schema = TIMESTAMP_FIELD)]
    pub timestamp_field: Instant,
}
