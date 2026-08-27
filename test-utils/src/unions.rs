#![allow(dead_code)]

use smithy4rs_core::{
    derive::{SmithyShape, smithy_union},
    schema::{
        UNIT,
        prelude::{INTEGER, STRING},
    },
    smithy,
};

smithy!("test#SimpleUnion": {
    union UNION {
        a: STRING
        b: INTEGER
        c: UNIT
    }
});

#[smithy_union]
#[derive(SmithyShape, PartialEq)]
#[schema(schema = UNION)]
pub enum TestUnion {
    A(String),
    B(i32),
    // Unit variant
    C,
}
