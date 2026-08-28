use smithy4rs_core::{
    derive::{
        SmithyShape,
        smithy_enum,
    },
    smithy,
};

smithy!(
    /// Schema for [`MyIntEnum`]
    intEnum com::test::MyIntEnum {
        Third = 3
        Second = 2
        First = 1
    }
);

#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = MY_INT_ENUM)]
pub enum MyIntEnum {
    Third = 3,
    Second = 2,
    First = 1,
}

smithy!(
    /// Schema for [`Suits`]
    enum com::test::Suits {
        Spade = "spade"
        Heart = "heart"
        Diamond = "diamond"
        Club = "club"
    }
);

#[smithy_enum]
#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = SUITS)]
pub enum Suits {
    Spade = "spade",
    Heart = "heart",
    Diamond = "diamond",
    Club = "club",
}
