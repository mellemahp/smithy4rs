use smithy4rs_core::{
    derive::{
        SmithyShape,
        smithy_union,
    },
    doc_map,
    prelude::{
        INTEGER,
        STRING,
        UNIT_SCHEMA,
    },
    schema::DynamicTrait,
    smithy,
};

smithy!("smithy.api#Unit": {
    /// Schema for [`Unit`]
    @DynamicTrait::from("smithy.api#unitType", doc_map![]);
    structure UNIT_SCHEMA {
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(UNIT_SCHEMA)]
pub struct Unit {
}

smithy!("com.test#MyUnion": {
    /// Schema for [`MyUnion`]
    union MY_UNION_SCHEMA {
        STRING_VARIANT: STRING = "string_variant"
        INTEGER_VARIANT: INTEGER = "integer_variant"
        UNIT_VARIANT: UNIT_SCHEMA = "unit_variant"
    }
});

#[smithy_union]
#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(MY_UNION_SCHEMA)]
pub enum MyUnion {
    #[smithy_schema(STRING_VARIANT)]
    StringVariant(String),
    #[smithy_schema(INTEGER_VARIANT)]
    IntegerVariant(i32),
    #[smithy_schema(UNIT_VARIANT)]
    UnitVariant(Unit),
}
