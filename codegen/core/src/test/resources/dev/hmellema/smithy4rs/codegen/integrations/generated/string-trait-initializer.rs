use smithy4rs_core::{
    derive::{
        SmithyShape,
        SmithyTrait,
    },
    schema::DynamicTrait,
    smithy,
};

smithy!(
    string com::test::genericTrait
);

#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = GENERIC_TRAIT)]
#[repr(transparent)]
pub struct GenericTraitTrait(String);

smithy!(
    @DynamicTrait::from("com.test#genericTrait", "stuff");
    string com::test::WithGeneric
);
