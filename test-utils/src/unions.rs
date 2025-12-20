use smithy4rs_core::smithy;
use smithy4rs_core_derive::{smithy_union, Dummy};
use smithy4rs_core::prelude::*;

smithy!("test#SimpleUnion": {
    union UNION {
        A: STRING = "field_a"
        B: INTEGER = "field_b"
        C: UNIT = "field_c"
    }
});

#[smithy_union]
#[derive(Dummy)]
#[smithy_schema(UNION)]
pub enum TestEnum {
    #[smithy_schema(A)]
    A(String),
    #[smithy_schema(B)]
    B(i32),
    // Unit variant
    #[smithy_schema(C)]
    C
}