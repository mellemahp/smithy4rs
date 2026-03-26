use smithy4rs_core::{
    derive::{
        SmithyShape,
        SmithyTraitImpl,
    },
    schema::DynamicTrait,
    smithy,
};

#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(GENERIC_TRAIT)]
#[repr(transparent)]
pub struct GenericTraitTrait(String);

smithy!("com.test#genericTrait": {
    string GENERIC_TRAIT
});

smithy!("com.test#WithGeneric": {
    @DynamicTrait::from("com.test#genericTrait", "stuff");
    string WITH_GENERIC
});
