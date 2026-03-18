use smithy4rs_core::{
    BigDecimal,
    BigInt,
    ByteBuffer,
    Document,
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
#[derive(SmithyShape)]
#[smithy_schema(NESTED_ENUM_SCHEMA)]
pub enum NestedEnum {
    A = "A",
    B = "B",
}

smithy!("smithy.java.codegen.test.structures#NestedIntEnum": {
    /// Schema for [`NestedIntEnum`]
    enum NESTED_INT_ENUM_SCHEMA {
        A = 1
        B = 2
    }
});

#[smithy_enum]
#[derive(SmithyShape)]
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
    #[smithy_schema(BOOLEAN)]
    pub boolean: bool,
    #[smithy_schema(BIG_DECIMAL)]
    pub big_decimal: BigDecimal,
    #[smithy_schema(BIG_DECIMAL_WITH_DOUBLE_DEFAULT)]
    pub big_decimal_with_double_default: BigDecimal,
    #[smithy_schema(BIG_DECIMAL_WITH_LONG_DEFAULT)]
    pub big_decimal_with_long_default: BigDecimal,
    #[smithy_schema(BIG_INTEGER)]
    pub big_integer: BigInt,
    #[smithy_schema(BIG_INTEGER_WITH_LONG_DEFAULT)]
    pub big_integer_with_long_default: BigInt,
    #[smithy_schema(BYTE)]
    pub byte: i8,
    #[smithy_schema(DOUBLE)]
    pub double: f64,
    #[smithy_schema(FLOAT)]
    pub float: f32,
    #[smithy_schema(INTEGER)]
    pub integer: i32,
    #[smithy_schema(LONG)]
    pub long: i64,
    #[smithy_schema(SHORT)]
    pub short: i16,
    #[smithy_schema(STRING)]
    pub string: String,
    #[smithy_schema(BLOB)]
    pub blob: ByteBuffer,
    #[smithy_schema(BOOL_DOC)]
    pub bool_doc: Document,
    #[smithy_schema(STRING_DOC)]
    pub string_doc: Document,
    #[smithy_schema(NUMBER_DOC)]
    pub number_doc: Document,
    #[smithy_schema(FLOATING_POINTNUMBER_DOC)]
    pub floating_pointnumber_doc: Document,
    #[smithy_schema(LIST_DOC)]
    pub list_doc: Document,
    #[smithy_schema(MAP_DOC)]
    pub map_doc: Document,
    #[smithy_schema(LIST)]
    pub list: Vec<String>,
    #[smithy_schema(MAP)]
    pub map: IndexMap<String, String>,
    #[smithy_schema(TIMESTAMP)]
    pub timestamp: Instant,
    #[smithy_schema(ENUM)]
    pub enum: NestedEnum,
    #[smithy_schema(INT_ENUM)]
    pub int_enum: NestedIntEnum,
}
