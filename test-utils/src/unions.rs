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
        A: STRING = "a"
        B: INTEGER = "b"
        C: UNIT = "c"
    }
});

#[smithy_union]
#[derive(SmithyShape, PartialEq)]
#[schema(schema = UNION)]
pub enum TestUnion {
    #[schema(schema = A)]
    A(String),
    #[schema(schema = B)]
    B(i32),
    // Unit variant
    #[schema(schema = C)]
    C,
}
