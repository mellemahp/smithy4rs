use smithy4rs_core::{
    derive::{SmithyShape, smithy_enum},
    prelude::{JsonNameTrait, STRING},
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
#[schema(schema = SIMPLE_ENUM)]
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
#[schema(schema = SIMPLE_INT_ENUM)]
pub enum TestIntEnum {
    A = 1,
    B = 2,
    C = 3,
}

smithy!("com.example#Rename": {
    structure RENAME {
        @JsonNameTrait::new("renamed");
        A: STRING = "a"
    }
});
#[derive(SmithyShape)]
#[schema(schema = RENAME)]
pub struct TestRename {
    #[schema(schema = A)]
    pub a: String,
}
