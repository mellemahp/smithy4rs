use smithy4rs_core::{
    prelude::PatternTrait,
    smithy,
};

smithy!("com.test#StringWithPattern": {
    @PatternTrait::new("^[a-z]*$");
    string STRING_WITH_PATTERN
});
