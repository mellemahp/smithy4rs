mod shapes;

use smithy4rs_core::serde::Serializable;
use smithy4rs_json_codec::JsonSerializer;
use crate::shapes::{Nested, SerializeMe};

#[test]
fn serializes_to_json() {
    let mut output = JsonSerializer::new();
    let structure = SerializeMe {
        member_a: "Hello".to_string(),
        member_b: "World".to_string(),
        nested: Nested { member_c: "Yeah".to_string() },
    };
    structure.serialize(&mut output);
    println!("OUTPUT: {}", output.string);
}