use smithy4rs_core::{
    derive::SmithyShape,
    prelude::{
        JsonNameTrait,
        STRING,
        SparseTrait,
    },
    smithy,
};

smithy!("com.test#MyStruct": {
    /// Schema for [`MyStruct`]
    structure MY_STRUCT_SCHEMA {
        @JsonNameTrait::new("stuff");
        withStringTrait: STRING
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[schema(schema = MY_STRUCT_SCHEMA)]
pub struct MyStruct {
    pub with_string_trait: Option<String>,
}

smithy!("com.test#WithAnnotationTrait": {
    @SparseTrait::builder().build();
    list WITH_ANNOTATION_TRAIT {
        member: STRING
    }
});
