use smithy4rs_core::{
    BigDecimal,
    BigInt,
    ByteBuffer,
    IndexMap,
    Instant,
    derive::{
        SmithyShape,
        smithy_enum,
    },
    prelude::{
        BIG_DECIMAL,
        BIG_INTEGER,
        BLOB,
        BOOLEAN,
        BYTE,
        DOCUMENT,
        DOUBLE,
        FLOAT,
        INTEGER,
        LONG,
        SHORT,
        STRING,
        TIMESTAMP,
    },
    schema::Document,
    smithy,
};

smithy!("smithy.java.codegen.test.structures#NestedEnum": {
    /// Schema for [`NestedEnum`]
    enum NESTED_ENUM_SCHEMA {
        A = "A"
        B = "B"
    }
});

#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = NESTED_ENUM_SCHEMA)]
pub enum NestedEnum {
    A = "A",
    B = "B",
}

smithy!("smithy.java.codegen.test.structures#NestedIntEnum": {
    /// Schema for [`NestedIntEnum`]
    intEnum NESTED_INT_ENUM_SCHEMA {
        A = 1
        B = 2
    }
});

#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = NESTED_INT_ENUM_SCHEMA)]
pub enum NestedIntEnum {
    A = 1,
    B = 2,
}

smithy!("smithy.java.codegen.test.structures#ListOfString": {
    list LIST_OF_STRING {
        member: STRING
    }
});

smithy!("smithy.java.codegen.test.structures#StringStringMap": {
    map STRING_STRING_MAP {
        key: STRING
        value: STRING
    }
});

smithy!("smithy.java.codegen.test.structures#DefaultStructure": {
    /// Schema for [`DefaultStructure`]
    structure DEFAULT_STRUCTURE_SCHEMA {
        BOOLEAN: BOOLEAN = "boolean"
        BIG_DECIMAL: BIG_DECIMAL = "bigDecimal"
        BIG_DECIMAL_WITH_DOUBLE_DEFAULT: BIG_DECIMAL = "bigDecimalWithDoubleDefault"
        BIG_DECIMAL_WITH_LONG_DEFAULT: BIG_DECIMAL = "bigDecimalWithLongDefault"
        BIG_INTEGER: BIG_INTEGER = "bigInteger"
        BIG_INTEGER_WITH_LONG_DEFAULT: BIG_INTEGER = "bigIntegerWithLongDefault"
        BYTE: BYTE = "byte"
        DOUBLE: DOUBLE = "double"
        FLOAT: FLOAT = "float"
        INTEGER: INTEGER = "integer"
        LONG: LONG = "long"
        SHORT: SHORT = "short"
        STRING: STRING = "string"
        BLOB: BLOB = "blob"
        BOOL_DOC: DOCUMENT = "boolDoc"
        STRING_DOC: DOCUMENT = "stringDoc"
        NUMBER_DOC: DOCUMENT = "numberDoc"
        FLOATING_POINTNUMBER_DOC: DOCUMENT = "floatingPointnumberDoc"
        LIST_DOC: DOCUMENT = "listDoc"
        MAP_DOC: DOCUMENT = "mapDoc"
        LIST: LIST_OF_STRING = "list"
        MAP: STRING_STRING_MAP = "map"
        TIMESTAMP: TIMESTAMP = "timestamp"
        ENUM: NESTED_ENUM_SCHEMA = "enum"
        INT_ENUM: NESTED_INT_ENUM_SCHEMA = "intEnum"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = DEFAULT_STRUCTURE_SCHEMA)]
pub struct DefaultStructure {
    #[schema(schema = BOOLEAN, default = true)]
    pub boolean: bool,
    #[schema(schema = BIG_DECIMAL, default = BigDecimal::from_str("1E+309").unwrap())]
    pub big_decimal: BigDecimal,
    #[schema(schema = BIG_DECIMAL_WITH_DOUBLE_DEFAULT, default = BigDecimal::from_str("1.3").unwrap())]
    pub big_decimal_with_double_default: BigDecimal,
    #[schema(schema = BIG_DECIMAL_WITH_LONG_DEFAULT, default = BigDecimal::from_str("5").unwrap())]
    pub big_decimal_with_long_default: BigDecimal,
    #[schema(schema = BIG_INTEGER, default = BigInt::from_str("123456789123456789123456789123456789123456789123456789").unwrap())]
    pub big_integer: BigInt,
    #[schema(schema = BIG_INTEGER_WITH_LONG_DEFAULT, default = BigInt::from_str("1").unwrap())]
    pub big_integer_with_long_default: BigInt,
    #[schema(schema = BYTE, default = 1i8)]
    pub byte: i8,
    #[schema(schema = DOUBLE, default = 1.0f64)]
    pub double: f64,
    #[schema(schema = FLOAT, default = 1.0f32)]
    pub float: f32,
    #[schema(schema = INTEGER, default = 1i32)]
    pub integer: i32,
    #[schema(schema = LONG, default = 1i64)]
    pub long: i64,
    #[schema(schema = SHORT, default = 1i16)]
    pub short: i16,
    #[schema(schema = STRING, default = "default".to_string())]
    pub string: String,
    #[schema(schema = BLOB, default = ByteBuffer::from_bytes("YmxvYg==".as_bytes()))]
    pub blob: ByteBuffer,
    #[schema(schema = BOOL_DOC, default = true.into(), no_builder)]
    pub bool_doc: Box<dyn Document>,
    #[schema(schema = STRING_DOC, default = "string".into(), no_builder)]
    pub string_doc: Box<dyn Document>,
    #[schema(schema = NUMBER_DOC, default = 1i64.into(), no_builder)]
    pub number_doc: Box<dyn Document>,
    #[schema(schema = FLOATING_POINTNUMBER_DOC, default = 1.2f64.into(), no_builder)]
    pub floating_pointnumber_doc: Box<dyn Document>,
    #[schema(schema = LIST_DOC, default = Vec::<Box<dyn Document>>::new().into(), no_builder)]
    pub list_doc: Box<dyn Document>,
    #[schema(schema = MAP_DOC, default = IndexMap::<String, Box<dyn Document>>::default().into(), no_builder)]
    pub map_doc: Box<dyn Document>,
    #[schema(schema = LIST, default)]
    pub list: Vec<String>,
    #[schema(schema = MAP, default)]
    pub map: IndexMap<String, String>,
    #[schema(schema = TIMESTAMP, default = )]
    pub timestamp: Instant,
    #[schema(schema = ENUM, default = NestedEnum::A, no_builder)]
    pub enum: NestedEnum,
    #[schema(schema = INT_ENUM, default = NestedIntEnum::A, no_builder)]
    pub int_enum: NestedIntEnum,
}
