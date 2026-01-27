use smithy4rs_core::IndexMap;
use smithy4rs_core::schema::Document;
use smithy4rs_core::string_map;
use smithy4rs_core_derive::SmithyTraitImpl;

#[derive(SmithyTraitImpl, Debug)]
#[smithy_trait_id("com.example#TestMapTrait")]
pub struct TestMapTrait(IndexMap<String, String>, Box<dyn Document>);

fn test() {
    let _ = TestMapTrait::new(string_map!["a" => "b"]);
}