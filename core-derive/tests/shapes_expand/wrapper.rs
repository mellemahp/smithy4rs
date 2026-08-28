use smithy4rs_core::smithy;
use smithy4rs_core_derive::{SmithyShape, SmithyTrait};

smithy!(
    string test::StringTrait
);

#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[schema(schema = STRING_TRAIT)]
pub struct SimpleTrait(String);
