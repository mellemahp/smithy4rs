use smithy4rs_core::{
    schema::DynamicTrait,
    smithy,
};

smithy!("com.test#WithGeneric": {
    @DynamicTrait::from("com.test#genericTrait", "stuff");
    string WITH_GENERIC
});
