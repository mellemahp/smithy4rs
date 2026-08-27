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
        boolean: BOOLEAN
        bigDecimal: BIG_DECIMAL
        bigDecimalWithDoubleDefault: BIG_DECIMAL
        bigDecimalWithLongDefault: BIG_DECIMAL
        bigInteger: BIG_INTEGER
        bigIntegerWithLongDefault: BIG_INTEGER
        byte: BYTE
        double: DOUBLE
        float: FLOAT
        integer: INTEGER
        long: LONG
        short: SHORT
        string: STRING
        blob: BLOB
        boolDoc: DOCUMENT
        stringDoc: DOCUMENT
        numberDoc: DOCUMENT
        floatingPointnumberDoc: DOCUMENT
        listDoc: DOCUMENT
        mapDoc: DOCUMENT
        list: LIST_OF_STRING
        map: STRING_STRING_MAP
        timestamp: TIMESTAMP
        enum: NESTED_ENUM_SCHEMA
        intEnum: NESTED_INT_ENUM_SCHEMA
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = DEFAULT_STRUCTURE_SCHEMA)]
pub struct DefaultStructure {
    #[schema(default = true)]
    pub boolean: bool,
    #[schema(default = BigDecimal::from_str("1E+309").unwrap())]
    pub big_decimal: BigDecimal,
    #[schema(default = BigDecimal::from_str("1.3").unwrap())]
    pub big_decimal_with_double_default: BigDecimal,
    #[schema(default = BigDecimal::from_str("5").unwrap())]
    pub big_decimal_with_long_default: BigDecimal,
    #[schema(default = BigInt::from_str("123456789123456789123456789123456789123456789123456789").unwrap())]
    pub big_integer: BigInt,
    #[schema(default = BigInt::from_str("1").unwrap())]
    pub big_integer_with_long_default: BigInt,
    #[schema(default = 1i8)]
    pub byte: i8,
    #[schema(default = 1.0f64)]
    pub double: f64,
    #[schema(default = 1.0f32)]
    pub float: f32,
    #[schema(default = 1i32)]
    pub integer: i32,
    #[schema(default = 1i64)]
    pub long: i64,
    #[schema(default = 1i16)]
    pub short: i16,
    #[schema(default = "default".to_string())]
    pub string: String,
    #[schema(default = ByteBuffer::from_bytes("YmxvYg==".as_bytes()))]
    pub blob: ByteBuffer,
    #[schema(default = true.into(), no_builder)]
    pub bool_doc: Box<dyn Document>,
    #[schema(default = "string".into(), no_builder)]
    pub string_doc: Box<dyn Document>,
    #[schema(default = 1i64.into(), no_builder)]
    pub number_doc: Box<dyn Document>,
    #[schema(default = 1.2f64.into(), no_builder)]
    pub floating_pointnumber_doc: Box<dyn Document>,
    #[schema(default = Vec::<Box<dyn Document>>::new().into(), no_builder)]
    pub list_doc: Box<dyn Document>,
    #[schema(default = IndexMap::<String, Box<dyn Document>>::default().into(), no_builder)]
    pub map_doc: Box<dyn Document>,
    #[schema(default)]
    pub list: Vec<String>,
    #[schema(default)]
    pub map: IndexMap<String, String>,
    #[schema(default = )]
    pub timestamp: Instant,
    #[schema(default = NestedEnum::A, no_builder)]
    pub enum: NestedEnum,
    #[schema(default = NestedIntEnum::A, no_builder)]
    pub int_enum: NestedIntEnum,
}
