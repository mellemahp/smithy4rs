use smithy4rs_core::{
    prelude::PatternTrait,
    smithy,
};

smithy!(
    @PatternTrait::new("^[a-z]*$");
    string com::test::StringWithPattern
);
