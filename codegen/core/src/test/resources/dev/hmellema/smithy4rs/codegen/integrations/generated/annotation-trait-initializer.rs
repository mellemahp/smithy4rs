use smithy4rs_core::{
    derive::{
        SmithyShape,
        SmithyTraitImpl,
    },
    doc_map,
    schema::DynamicTrait,
    smithy,
};

smithy!("com.test#myAnnotationTrait": {
    /// Schema for [`MyAnnotationTraitTrait`]
    structure MY_ANNOTATION_TRAIT_SCHEMA {
    }
});

#[derive(SmithyShape, SmithyTraitImpl, PartialEq, Clone)]
#[smithy_schema(MY_ANNOTATION_TRAIT_SCHEMA)]
pub struct MyAnnotationTraitTrait {
}

smithy!("com.test#MyString": {
    @DynamicTrait::from("com.test#myAnnotationTrait", doc_map![]);
    string MY_STRING
});
