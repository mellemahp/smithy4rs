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
#[smithy_schema(NESTED_ENUM_SCHEMA)]
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
#[smithy_schema(NESTED_INT_ENUM_SCHEMA)]
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
#[smithy_schema(DEFAULT_STRUCTURE_SCHEMA)]
pub struct DefaultStructure {
    #[default(true)]
    #[smithy_schema(BOOLEAN)]
    pub boolean: bool,
    #[default(BigDecimal::from_str("1E+309").unwrap())]
    #[smithy_schema(BIG_DECIMAL)]
    pub big_decimal: BigDecimal,
    #[default(BigDecimal::from_str("1.3").unwrap())]
    #[smithy_schema(BIG_DECIMAL_WITH_DOUBLE_DEFAULT)]
    pub big_decimal_with_double_default: BigDecimal,
    #[default(BigDecimal::from_str("5").unwrap())]
    #[smithy_schema(BIG_DECIMAL_WITH_LONG_DEFAULT)]
    pub big_decimal_with_long_default: BigDecimal,
    #[default(BigInt::from_str("123456789123456789123456789123456789123456789123456789").unwrap())]
    #[smithy_schema(BIG_INTEGER)]
    pub big_integer: BigInt,
    #[default(BigInt::from_str("1").unwrap())]
    #[smithy_schema(BIG_INTEGER_WITH_LONG_DEFAULT)]
    pub big_integer_with_long_default: BigInt,
    #[default(1i8)]
    #[smithy_schema(BYTE)]
    pub byte: i8,
    #[default(1.0f64)]
    #[smithy_schema(DOUBLE)]
    pub double: f64,
    #[default(1.0f32)]
    #[smithy_schema(FLOAT)]
    pub float: f32,
    #[default(1i32)]
    #[smithy_schema(INTEGER)]
    pub integer: i32,
    #[default(1i64)]
    #[smithy_schema(LONG)]
    pub long: i64,
    #[default(1i16)]
    #[smithy_schema(SHORT)]
    pub short: i16,
    #[default("default".to_string())]
    #[smithy_schema(STRING)]
    pub string: String,
    #[default(ByteBuffer::from_bytes("YmxvYg==".as_bytes()))]
    #[smithy_schema(BLOB)]
    pub blob: ByteBuffer,
    #[no_builder]
    #[default(true.into())]
    #[smithy_schema(BOOL_DOC)]
    pub bool_doc: Box<dyn Document>,
    #[no_builder]
    #[default("string".into())]
    #[smithy_schema(STRING_DOC)]
    pub string_doc: Box<dyn Document>,
    #[no_builder]
    #[default(1i64.into())]
    #[smithy_schema(NUMBER_DOC)]
    pub number_doc: Box<dyn Document>,
    #[no_builder]
    #[default(1.2f64.into())]
    #[smithy_schema(FLOATING_POINTNUMBER_DOC)]
    pub floating_pointnumber_doc: Box<dyn Document>,
    #[no_builder]
    #[default(Vec::<Box<dyn Document>>::new().into())]
    #[smithy_schema(LIST_DOC)]
    pub list_doc: Box<dyn Document>,
    #[no_builder]
    #[default(IndexMap::<String, Box<dyn Document>>::default().into())]
    #[smithy_schema(MAP_DOC)]
    pub map_doc: Box<dyn Document>,
    #[default(Vec::<String>::new())]
    #[smithy_schema(LIST)]
    pub list: Vec<String>,
    #[default(IndexMap::<String, String>::default())]
    #[smithy_schema(MAP)]
    pub map: IndexMap<String, String>,
    #[default()]
    #[smithy_schema(TIMESTAMP)]
    pub timestamp: Instant,
    #[no_builder]
    #[default(NestedEnum::A)]
    #[smithy_schema(ENUM)]
    pub enum: NestedEnum,
    #[no_builder]
    #[default(NestedIntEnum::A)]
    #[smithy_schema(INT_ENUM)]
    pub int_enum: NestedIntEnum,
}
