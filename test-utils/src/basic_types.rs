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
        STRING: STRING = "stringField"
        BYTE: BYTE = "byteField"
        SHORT: SHORT = "shortField"
        INTEGER: INTEGER = "integerField"
        LONG: LONG = "longField"
        FLOAT: FLOAT = "floatField"
        DOUBLE: DOUBLE = "doubleField"
        BOOLEAN: BOOLEAN = "booleanField"
        BLOB: BLOB = "blobField"
        TIMESTAMP: TIMESTAMP = "timestampField"
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
        REQUIRED: STRING = "requiredField"
        OPTIONAL: STRING = "optionalField"
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
        BYTE: BYTE = "byteVal"
        SHORT: SHORT = "shortVal"
        INT: INTEGER = "intVal"
        LONG: LONG = "longVal"
        FLOAT: FLOAT = "floatVal"
        DOUBLE: DOUBLE = "doubleVal"
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
        A: STRING = "fieldA"
        B: INTEGER = "fieldB"
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
