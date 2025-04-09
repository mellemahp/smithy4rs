mod shapes;

use smithy4rs_core::serde::{Serializable, ShapeBuilder};
use smithy4rs_json_codec::{JsonDeserializer, JsonSerializer};
use crate::shapes::{Nested, SerializeMe, SerializeMeBuilder};

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

#[test]
fn deserializes_from_json() {
    let data = r#"{
        "a":"Hello",
        "b":"World",
        "nested":{
            "c":"Yeah"
        }
    }"#;
    let mut deserializer = JsonDeserializer::new(data);
    let mut builder = SerializeMe::builder();
    builder.deserialize(&mut deserializer);
    let output = builder.build();
    assert_eq!(output.member_a, "Hello");
    assert_eq!(output.member_b, "World");
    assert_eq!(output.nested.member_c, "Yeah");
}