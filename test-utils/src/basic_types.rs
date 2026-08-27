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
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = ALL_PRIMITIVES_STRUCT_SCHEMA)]
pub struct AllPrimitivesStruct {
    pub string_field: String,
    pub byte_field: i8,
    pub short_field: i16,
    pub integer_field: i32,
    pub long_field: i64,
    pub float_field: f32,
    pub double_field: f64,
    pub boolean_field: bool,
    pub blob_field: ByteBuffer,
    pub timestamp_field: Instant,
}

smithy!("test#OptionalFieldsStruct": {
    structure OPTIONAL_FIELDS_STRUCT_SCHEMA {
        requiredField: STRING
        optionalField: STRING
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = OPTIONAL_FIELDS_STRUCT_SCHEMA)]
pub struct OptionalFieldsStruct {
    pub required_field: String,
    pub optional_field: Option<String>,
}

smithy!("test#NumericTypesStruct": {
    structure NUMERIC_TYPES_STRUCT_SCHEMA {
        byteVal: BYTE
        shortVal: SHORT
        intVal: INTEGER
        longVal: LONG
        floatVal: FLOAT
        doubleVal: DOUBLE
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = NUMERIC_TYPES_STRUCT_SCHEMA)]
pub struct NumericTypesStruct {
    pub byte_val: i8,
    pub short_val: i16,
    pub int_val: i32,
    pub long_val: i64,
    pub float_val: f32,
    pub double_val: f64,
}

smithy!("test#SimpleStruct": {
    structure SIMPLE_STRUCT_SCHEMA {
        fieldA: STRING
        fieldB: INTEGER
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = SIMPLE_STRUCT_SCHEMA)]
pub struct SimpleStruct {
    pub field_a: String,
    pub field_b: i32,
}
