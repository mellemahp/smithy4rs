use smithy4rs_core::{
    derive::{
        SmithyShape,
        smithy_union,
    },
    doc_map,
    prelude::{
        INTEGER,
        STRING,
        UNIT,
    },
    schema::DynamicTrait,
    smithy,
};

smithy!(
    /// Schema for [`Unit`]
    @DynamicTrait::from("smithy.api#unitType", doc_map![]);
    structure smithy::api::Unit {
    }
);

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = UNIT)]
pub struct Unit {
}

smithy!(
    /// Schema for [`MyUnion`]
    union com::test::MyUnion {
        string_variant: STRING
        integer_variant: INTEGER
        unit_variant: UNIT
    }
);

#[smithy_union]
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = MY_UNION)]
pub enum MyUnion {
    StringVariant(String),
    IntegerVariant(i32),
    UnitVariant(Unit),
}
