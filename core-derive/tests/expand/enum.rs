use smithy4rs_core_derive::{SmithyShape, smithy_enum};

smithy!("test#SimpleStruct": {
    enum SIMPLE_ENUM {
        A = "a"
        B = "b"
        C = "c"
    }
});

#[smithy_enum]
#[derive(SmithyShape)]
#[smithy_schema(SIMPLE_ENUM)]
pub enum TestEnum {
    A = "a",
    B = "b",
    C = "c"
}
