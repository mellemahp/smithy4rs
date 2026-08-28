use smithy4rs_core::{
    prelude::{INTEGER, STRING},
    schema::UNIT,
    smithy,
};
use smithy4rs_core_derive::{SmithyShape, smithy_union};

smithy!(
    union test::Union {
        a: STRING
        b: INTEGER
        c: UNIT
    }
);

#[smithy_union]
#[derive(SmithyShape)]
#[schema(schema = UNION)]
pub enum TestEnum {
    A(String),
    B(i32),
    // Unit variant
    C,
}
