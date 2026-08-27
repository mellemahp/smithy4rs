use smithy4rs_core::{
    prelude::{INTEGER, STRING},
    schema::UNIT,
    smithy,
};
use smithy4rs_core_derive::{SmithyShape, smithy_union};

smithy!("test#SimpleUnion": {
    union UNION {
        A: STRING = "a"
        B: INTEGER = "b"
        C: UNIT = "c"
    }
});

#[smithy_union]
#[derive(SmithyShape)]
#[schema(schema = UNION)]
pub enum TestEnum {
    #[schema(schema = A)]
    A(String),
    #[schema(schema = B)]
    B(i32),
    // Unit variant
    #[schema(schema = C)]
    C,
}
