mod shapes;

use crate::shapes::{Nested, SCHEMA, SerializeMe};
use indexmap::IndexMap;
use smithy4rs_core::serde::de::{Deserializable, ShapeBuilder};
use smithy4rs_core::serde::se::Serialize;
use smithy4rs_json_codec::{JsonDeserializer, JsonSerializer};

#[test]
fn serializes_to_json() {
    let mut output = JsonSerializer::new();
    let mut map = IndexMap::new();
    map.insert("a".to_string(), Nested::builder().member_c("stuff").build());
    map.insert(
        "b".to_string(),
        Nested::builder().member_c("things").build(),
    );
    let structure = SerializeMe::builder()
        .member_a("Hello")
        .member_b("World")
        .nested(Nested::builder().member_c("Yeah").build())
        .map_nested(map)
        .build();
    structure
        .serialize(&SCHEMA, &mut output)
        .expect("serialization failed");
    if let Some(value) = output.value {
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
