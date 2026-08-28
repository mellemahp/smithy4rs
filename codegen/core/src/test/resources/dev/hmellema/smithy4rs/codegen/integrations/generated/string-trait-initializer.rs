use smithy4rs_core::{
    derive::{
        SmithyShape,
        SmithyTrait,
    },
    schema::DynamicTrait,
    smithy,
};

smithy!("com.test#genericTrait": {
    string GENERIC_TRAIT
});

#[derive(SmithyShape, SmithyTrait, Clone)]
#[schema(schema = GENERIC_TRAIT)]
#[repr(transparent)]
pub struct GenericTraitTrait(String);

smithy!("com.test#WithGeneric": {
    @DynamicTrait::from("com.test#genericTrait", "stuff");
    string WITH_GENERIC
});
