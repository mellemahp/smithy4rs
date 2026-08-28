use smithy4rs_core::smithy;
use smithy4rs_core_derive::{SmithyShape, SmithyTrait};

smithy!("test#SimpleTrait": {
    string STRING_TRAIT
});

#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[schema(schema = STRING_TRAIT)]
pub struct SimpleTrait(String);
