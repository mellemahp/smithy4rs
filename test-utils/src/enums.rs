use smithy4rs_core::{
    derive::{SmithyShape, smithy_enum},
    prelude::{JsonNameTrait, STRING},
    smithy,
};

smithy!(
    enum test::StringEnum {
        A = "a"
        B = "b"
        C = "c"
    }
);

#[smithy_enum]
#[derive(SmithyShape)]
#[schema(schema = STRING_ENUM)]
pub enum TestEnum {
    A = "a",
    B = "b",
    C = "c",
}

smithy!(
    intEnum test::IntEnum {
        A = 1
        B = 2
        C = 3
    }
);

#[smithy_enum]
#[derive(SmithyShape)]
#[schema(schema = INT_ENUM)]
pub enum TestIntEnum {
    A = 1,
    B = 2,
    C = 3,
}

smithy!(
    structure com::example::Rename {
        @JsonNameTrait::new("renamed");
        a: STRING
    }
);
#[derive(SmithyShape)]
#[schema(schema = RENAME)]
pub struct TestRename {
    pub a: String,
}
