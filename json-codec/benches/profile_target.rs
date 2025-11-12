use smithy4rs_core::{lazy_schema, prelude::*, schema::{Schema, ShapeId}, serde::serializers::SerializeWithSchema, traits};
use smithy4rs_core_derive::SerializableStruct;
use smithy4rs_json_codec::JsonSerializer;

// Simple struct for profiling
lazy_schema!(
    PERSON_SCHEMA,
    Schema::structure_builder(ShapeId::from("bench#Person"), traits![]),
    (NAME, "name", STRING, traits![]),
    (AGE, "age", INTEGER, traits![]),
    (EMAIL, "email", STRING, traits![])
);

#[derive(SerializableStruct, Clone)]
#[smithy_schema(PERSON_SCHEMA)]
struct Person {
    #[smithy_schema(NAME)]
    name: String,
    #[smithy_schema(AGE)]
    age: i32,
    #[smithy_schema(EMAIL)]
    email: String,
}

fn main() {
    let person = Person {
        name: "John Doe".to_string(),
        age: 35,
        email: "john.doe@example.com".to_string(),
    };

    let mut buf = Vec::with_capacity(128);

    // Run many iterations for profiling
    for _ in 0..1_000_000 {
        let serializer = JsonSerializer::new(&mut buf);
        person.serialize_with_schema(&PERSON_SCHEMA, serializer).unwrap();
    }

    println!("Done: {} bytes", buf.len());
}
