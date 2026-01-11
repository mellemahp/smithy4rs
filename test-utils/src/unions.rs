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
#[smithy_schema(UNION)]
pub enum TestUnion {
    #[smithy_schema(A)]
    A(String),
    #[smithy_schema(B)]
    B(i32),
    // Unit variant
    #[smithy_schema(C)]
    C,
}
