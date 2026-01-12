use smithy4rs_core::{
    derive::{
        SmithyShape,
        smithy_enum,
    },
    smithy,
};

smithy!("com.test#MyIntEnum": {
    enum MY_INT_ENUM_SCHEMA {
        Third = 3
        Second = 2
        First = 1
    }
});

#[smithy_enum]
#[derive(SmithyShape)]
#[smithy_schema(MY_INT_ENUM_SCHEMA)]
pub enum TestEnum {
    Third = 3,
    Second = 2,
    First = 1,
}

smithy!("com.test#Suits": {
    enum SUITS_SCHEMA {
        Spade = "spade"
        Heart = "heart"
        Diamond = "diamond"
        Club = "club"
    }
});

#[smithy_enum]
#[derive(SmithyShape)]
#[smithy_schema(SUITS_SCHEMA)]
pub enum TestEnum {
    Spade = "spade",
    Heart = "heart",
    Diamond = "diamond",
    Club = "club",
}
