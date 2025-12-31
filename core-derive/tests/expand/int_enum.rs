use smithy4rs_core_derive::{SmithyShape, smithy_enum};

smithy!("test#SimpleStruct": {
    intEnum SIMPLE_INT_ENUM {
        A = 1
        B = 2
        C = 3
    }
});

#[smithy_enum]
#[derive(SmithyShape)]
#[smithy_schema(SIMPLE_INT_ENUM)]
pub enum TestIntEnum {
    A = 1,
    B = 2,
    C = 3
}