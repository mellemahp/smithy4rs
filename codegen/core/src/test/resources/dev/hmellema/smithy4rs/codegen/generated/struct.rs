smithy!("com.test#AllPrimitivesStruct": {
    structure ALL_PRIMITIVES_STRUCT_SCHEMA {
        STRING_FIELD: STRING = "string_field"
        BYTE_FIELD: BYTE = "byte_field"
        SHORT_FIELD: SHORT = "short_field"
        INTEGER_FIELD: INTEGER = "integer_field"
        LONG_FIELD: LONG = "long_field"
        FLOAT_FIELD: FLOAT = "float_field"
        DOUBLE_FIELD: DOUBLE = "double_field"
        BOOLEAN_FIELD: BOOLEAN = "boolean_field"
        BLOB_FIELD: BLOB = "blob_field"
        TIMESTAMP_FIELD: TIMESTAMP = "timestamp_field"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(ALL_PRIMITIVES_STRUCT_SCHEMA)]
pub struct AllPrimitivesStruct {
    #[smithy_schema(STRING_FIELD)]
    pub string_field: String,
    #[smithy_schema(BYTE_FIELD)]
    pub byte_field: i8,
    #[smithy_schema(SHORT_FIELD)]
    pub short_field: i16,
    #[smithy_schema(INTEGER_FIELD)]
    pub integer_field: i32,
    #[smithy_schema(LONG_FIELD)]
    pub long_field: long,
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
}
