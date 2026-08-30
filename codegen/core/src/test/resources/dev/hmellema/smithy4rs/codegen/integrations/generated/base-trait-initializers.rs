use smithy4rs_core::{
    derive::SmithyShape,
    prelude::{
        JsonNameTrait,
        STRING,
        SparseTrait,
    },
    smithy,
};

smithy!(
    /// Schema for [`MyStruct`]
    structure com::test::MyStruct {
        @JsonNameTrait::new("stuff");
        withStringTrait: STRING
    }
);

#[derive(SmithyShape, PartialEq, Clone)]
#[non_exhaustive]
#[schema(schema = MY_STRUCT)]
pub struct MyStruct {
    pub with_string_trait: Option<String>,
}

smithy!(
    @SparseTrait::builder().build();
    list com::test::WithAnnotationTrait {
        member: STRING
    }
);
