use smithy4rs_core::{
    doc_map,
    schema::DynamicTrait,
    smithy,
};

smithy!("com.test#MyString": {
    @DynamicTrait::from("com.test#myAnnotationTrait", doc_map![]);
    string MY_STRING
});
