use smithy4rs_core::prelude::STRING;
use smithy4rs_core::smithy;
use smithy4rs_core_derive::{SmithyShape, SmithyTraitImpl};

smithy!("test#SimpleTrait": {
    list NESTED {
        member: SIMPLE_SCHEMA
    }
});

#[derive(SmithyShape, SmithyTraitImpl, Clone)]
#[smithy_schema(NESTED)]
#[repr(transparent)]
pub struct NestedWrapper(Vec<Nested>);

smithy!("test#SimpleStruct": {
    structure SIMPLE_SCHEMA {
        A: STRING = "field_a"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(SIMPLE_SCHEMA)]
pub struct Nested {
    #[smithy_schema(A)]
    pub field_a: String,
}
