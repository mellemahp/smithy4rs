use smithy4rs_core::{
    derive::{
        SmithyShape,
        SmithyTrait,
    },
    doc_map,
    schema::DynamicTrait,
    smithy,
};

smithy!(
    /// Schema for [`MyAnnotationTraitTrait`]
    structure com::test::myAnnotationTrait {
    }
);

#[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
#[schema(schema = MY_ANNOTATION_TRAIT)]
pub struct MyAnnotationTraitTrait {
}

smithy!(
    @DynamicTrait::from("com.test#myAnnotationTrait", doc_map![]);
    string com::test::MyString
);
