<<<<<<< HEAD
use smithy4rs_core::smithy;
use smithy4rs_core_derive::{SmithyShape, SmithyTraitImpl};

smithy!("test#SimpleTrait": {
    string STRING_TRAIT
});

#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(STRING_TRAIT)]
pub struct SimpleTrait(String);
=======
// use smithy4rs_core::smithy;
// use smithy4rs_core_derive::SmithyShape;
//
// smithy!("test#SimpleTrait": {
//     string STRING_TRAIT
// });
//
// #[derive(SmithyShape, PartialEq, Clone)]
// #[smithy_schema(STRING_TRAIT)]
// pub struct SimpleTrait(String);
>>>>>>> a99d247 (Update approach to document conversion for improved ergonomics)
