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
        WITH_STRING_TRAIT: STRING = "withStringTrait"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(MY_STRUCT_SCHEMA)]
pub struct MyStruct {
    #[smithy_schema(WITH_STRING_TRAIT)]
    pub with_string_trait: String,
}

smithy!("com.test#WithAnnotationTrait": {
    @SparseTrait;
    list WITH_ANNOTATION_TRAIT {
        member: STRING
    }
});
