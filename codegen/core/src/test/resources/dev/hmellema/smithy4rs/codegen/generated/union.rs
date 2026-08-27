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
#[schema(schema = UNIT_SCHEMA)]
pub struct Unit {
}

smithy!("com.test#MyUnion": {
    /// Schema for [`MyUnion`]
    union MY_UNION_SCHEMA {
        string_variant: STRING
        integer_variant: INTEGER
        unit_variant: UNIT_SCHEMA
    }
});

#[smithy_union]
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = MY_UNION_SCHEMA)]
pub enum MyUnion {
    StringVariant(String),
    IntegerVariant(i32),
    UnitVariant(Unit),
}
