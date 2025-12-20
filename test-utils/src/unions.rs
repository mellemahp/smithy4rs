#![allow(dead_code)]

use smithy4rs_core::{prelude::*, smithy};
use smithy4rs_core_derive::{Dummy, smithy_union};

smithy!("test#SimpleUnion": {
    union UNION {
        A: STRING = "a"
        B: INTEGER = "b"
        C: UNIT = "c"
    }
});

#[smithy_union]
#[derive(Dummy, PartialEq, Debug)]
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
