use smithy4rs_core::{
    derive::{SmithyShape, smithy_enum},
    smithy,
};

smithy!("test#StringEnum": {
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
    C = "c",
}

smithy!("test#IntEnum": {
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
    C = 3,
}
