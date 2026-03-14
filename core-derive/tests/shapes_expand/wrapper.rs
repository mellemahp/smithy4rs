use smithy4rs_core::smithy;
use smithy4rs_core_derive::{SmithyShape, SmithyTraitImpl};

smithy!("test#SimpleTrait": {
    string STRING_TRAIT
});

#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(STRING_TRAIT)]
pub struct SimpleTrait(String);
