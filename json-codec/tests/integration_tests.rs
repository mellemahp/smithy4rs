mod shapes;

use crate::shapes::{Nested, SerializeMe};
use smithy4rs_core::serde::de::{Deserializable, ShapeBuilder};
use smithy4rs_core::serde::se::Serializable;
use smithy4rs_json_codec::{JsonDeserializer, JsonSerializer};

#[test]
fn serializes_to_json() {
    let mut output = JsonSerializer::new();
    let structure = SerializeMe::builder()
        .member_a("Hello")
        .member_b("World")
        .nested(Nested::builder().member_c("Yeah").build())
        .build();
    structure
        .serialize(&mut output)
        .expect("serialization failed");
    if let JsonSerializer::Root(Some(value)) = output {
        println!("DEBUGGING: {}", json::stringify(value));
    }
    //println!("OUTPUT: {}", output.to_string());
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
    let output = SerializeMe::builder()
        .deserialize(&mut deserializer)
        .expect("Should be able to deserialize")
        .build();
    assert_eq!(output.member_a, "Hello");
    assert_eq!(output.member_b, "World");
    assert_eq!(output.nested.member_c, "Yeah");
}
