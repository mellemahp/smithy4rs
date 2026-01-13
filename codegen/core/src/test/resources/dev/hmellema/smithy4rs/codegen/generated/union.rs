use smithy4rs_core::{
    derive::{
        SmithyShape,
        smithy_union,
    },
    prelude::{
        INTEGER,
        STRING,
        UNIT_SCHEMA,
    },
    smithy,
};

smithy!("com.test#MyUnion": {
    union MY_UNION_SCHEMA {
        STRING_VARIANT: STRING = "string_variant"
        INTEGER_VARIANT: INTEGER = "integer_variant"
        UNIT_VARIANT: UNIT_SCHEMA = "unit_variant"
    }
});

/// My union
#[smithy_union]
#[derive(SmithyShape)]
#[smithy_schema(MY_UNION_SCHEMA)]
pub enum MyUnion {
    /// A union member
    #[smithy_schema(STRING_VARIANT)]
    StringVariant(String),
    #[smithy_schema(INTEGER_VARIANT)]
    IntegerVariant(i32),
    #[smithy_schema(UNIT_VARIANT)]
    UnitVariant(Unit),
}
