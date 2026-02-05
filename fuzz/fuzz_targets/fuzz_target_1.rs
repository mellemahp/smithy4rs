#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use smithy4rs_core::derive::SmithyShape;
use smithy4rs_core::prelude::{INTEGER, STRING};
use smithy4rs_core::serde::arbitrary::ArbitraryDeserializer;
use smithy4rs_core::serde::de::DeserializeWithSchema;
use smithy4rs_core::serde::ShapeBuilder;
use smithy4rs_core::smithy;

smithy!("test#SimpleStruct": {
    structure SIMPLE_STRUCT_SCHEMA {
        A: STRING = "field_a"
        B: INTEGER = "field_b"
    }
});

#[derive(SmithyShape, PartialEq, Clone)]
#[smithy_schema(SIMPLE_STRUCT_SCHEMA)]
pub struct SimpleStruct {
    #[smithy_schema(A)]
    pub field_a: String,
    #[smithy_schema(B)]
    pub field_b: i32,
}

impl <'a> Arbitrary<'a> for SimpleStruct {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        SimpleStructBuilder::deserialize_with_schema(
            &SIMPLE_STRUCT_SCHEMA,
            &mut ArbitraryDeserializer(u)
        ).map_err(|e| e.0)?
            .build()
            .map_err(|_| arbitrary::Error::IncorrectFormat)
    }
}

fuzz_target!(|data: SimpleStruct| {
    // fuzzed code goes here
    // STUFF AND THINGS!!!! 
});
