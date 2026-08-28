use smithy4rs_core::smithy;
use smithy4rs_core_derive::{SmithyShape, smithy_enum};

smithy!(
    intEnum test::SimpleIntEnum {
        A = 1
        B = 2
        C = 3
    }
);

#[smithy_enum]
#[derive(SmithyShape)]
#[schema(schema = SIMPLE_INT_ENUM)]
pub enum TestIntEnum {
    A = 1,
    B = 2,
    C = 3,
}
