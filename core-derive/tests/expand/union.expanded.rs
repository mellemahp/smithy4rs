use smithy4rs_core::prelude::{INTEGER, STRING, UNIT};
use smithy4rs_core_derive::{Dummy, smithy_union};
#[smithy_schema(UNION)]
#[smithy_union_enum]
pub enum TestEnum {
    #[smithy_schema(A)]
    A(String),
    #[smithy_schema(B)]
    B(i32),
    #[smithy_schema(C)]
    C,
    #[automatically_derived]
    #[doc(hidden)]
    Unknown(String),
}
