use smithy4rs_core::{
    ByteBuffer, Instant,
    derive::SmithyShape,
    schema::prelude::{
        BLOB, BOOLEAN, BYTE, DOUBLE, FLOAT, INTEGER, LONG, SHORT, STRING, TIMESTAMP,
    },
    smithy,
};

smithy!("test#IntegerList": {
    list INTEGER_LIST_SCHEMA {
        member: INTEGER
    }
});

smithy!("test#AllPrimitivesStruct": {
    structure ALL_PRIMITIVES_STRUCT_SCHEMA {
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
#[schema(schema = ALL_PRIMITIVES_STRUCT_SCHEMA)]
pub struct AllPrimitivesStruct {
    #[schema(schema = STRING)]
    pub string_field: String,
    #[schema(schema = BYTE)]
    pub byte_field: i8,
    #[schema(schema = SHORT)]
    pub short_field: i16,
    #[schema(schema = INTEGER)]
    pub integer_field: i32,
    #[schema(schema = LONG)]
    pub long_field: i64,
    #[schema(schema = FLOAT)]
    pub float_field: f32,
    #[schema(schema = DOUBLE)]
    pub double_field: f64,
    #[schema(schema = BOOLEAN)]
    pub boolean_field: bool,
    #[schema(schema = BLOB)]
    pub blob_field: ByteBuffer,
    #[schema(schema = TIMESTAMP)]
    pub timestamp_field: Instant,
}

smithy!("test#OptionalFieldsStruct": {
    structure OPTIONAL_FIELDS_STRUCT_SCHEMA {
        REQUIRED: STRING = "required_field"
        OPTIONAL: STRING = "optional_field"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = OPTIONAL_FIELDS_STRUCT_SCHEMA)]
pub struct OptionalFieldsStruct {
    #[schema(schema = REQUIRED)]
    pub required_field: String,
    #[schema(schema = OPTIONAL)]
    pub optional_field: Option<String>,
}

smithy!("test#NumericTypesStruct": {
    structure NUMERIC_TYPES_STRUCT_SCHEMA {
        BYTE: BYTE = "byte_val"
        SHORT: SHORT = "short_val"
        INT: INTEGER = "int_val"
        LONG: LONG = "long_val"
        FLOAT: FLOAT = "float_val"
        DOUBLE: DOUBLE = "double_val"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = NUMERIC_TYPES_STRUCT_SCHEMA)]
pub struct NumericTypesStruct {
    #[schema(schema = BYTE)]
    pub byte_val: i8,
    #[schema(schema = SHORT)]
    pub short_val: i16,
    #[schema(schema = INT)]
    pub int_val: i32,
    #[schema(schema = LONG)]
    pub long_val: i64,
    #[schema(schema = FLOAT)]
    pub float_val: f32,
    #[schema(schema = DOUBLE)]
    pub double_val: f64,
}

smithy!("test#SimpleStruct": {
    structure SIMPLE_STRUCT_SCHEMA {
        A: STRING = "field_a"
        B: INTEGER = "field_b"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = SIMPLE_STRUCT_SCHEMA)]
pub struct SimpleStruct {
    #[schema(schema = A)]
    pub field_a: String,
    #[schema(schema = B)]
    pub field_b: i32,
}
