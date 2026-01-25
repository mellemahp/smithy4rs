use smithy4rs_core::{
    derive::SmithyShape,
    doc_map,
    schema::DynamicTrait,
    smithy,
};

smithy!("com.test#AppliedTo": {
    /// Schema for [`AppliedTo`]
    @DynamicTrait::from("com.test#floatTrait", 2.0);
    @DynamicTrait::from("com.test#myCustomStruct", doc_map!["a" => "str", "b" => 2]);
    @DynamicTrait::from("com.test#intTrait", 1);
    @DynamicTrait::from("com.test#otherListTrait", vec![1, 2, 3]);
    @DynamicTrait::from("com.test#stringTrait", "stuff");
    @DynamicTrait::from("com.test#stringListTrait", vec!["a", "b", "c"]);
    structure APPLIED_TO_SCHEMA {
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(APPLIED_TO_SCHEMA)]
pub struct AppliedTo {
}
