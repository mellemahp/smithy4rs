use smithy4rs_core::schema::{Document,};
use smithy4rs_core_derive::SmithyTraitImpl;

// List Trait
#[derive(SmithyTraitImpl, Debug)]
#[smithy_trait_id("com.example#TestListTrait")]
pub struct TestListTrait(Vec<String>, Box<dyn Document>);

