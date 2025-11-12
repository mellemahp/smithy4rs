use smithy4rs_core::{
    lazy_schema, prelude::*,
    schema::{Schema, ShapeId},
    serde::{deserializers::Deserialize, serializers::SerializeWithSchema},
    traits
};
use smithy4rs_core_derive::{DeserializableStruct, SerializableStruct};
use smithy4rs_json_codec::{JsonDeserializer, JsonSerializer};

// Simple struct for profiling
lazy_schema!(
    PERSON_SCHEMA,
    Schema::structure_builder(ShapeId::from("bench#Person"), traits![]),
    (NAME, "name", STRING, traits![]),
    (AGE, "age", INTEGER, traits![]),
    (EMAIL, "email", STRING, traits![])
);

#[derive(SerializableStruct, DeserializableStruct, Clone)]
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
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("both");

    let person = Person {
        name: "John Doe".to_string(),
        age: 35,
        email: "john.doe@example.com".to_string(),
    };

    match mode {
        "ser" => profile_serialization(&person),
        "deser" => profile_deserialization(&person),
        "both" => {
            profile_serialization(&person);
            profile_deserialization(&person);
        }
        _ => {
            eprintln!("Usage: profile_target [ser|deser|both]");
            std::process::exit(1);
        }
    }
}

fn profile_serialization(person: &Person) {
    let mut buf = Vec::with_capacity(128);

    // Run many iterations for profiling
    for _ in 0..1_000_000 {
        buf.clear();
        let serializer = JsonSerializer::new(&mut buf);
        person.serialize_with_schema(&PERSON_SCHEMA, serializer).unwrap();
    }

    println!("Serialization done: {} bytes", buf.len());
}

fn profile_deserialization(person: &Person) {
    // Pre-serialize to get JSON data
    let mut buf = Vec::new();
    let serializer = JsonSerializer::new(&mut buf);
    person.serialize_with_schema(&PERSON_SCHEMA, serializer).unwrap();

    // Run many iterations for profiling
    let mut result = None;
    for _ in 0..1_000_000 {
        let mut deserializer = JsonDeserializer::new(&buf);
        result = Some(Person::deserialize(&PERSON_SCHEMA, &mut deserializer).unwrap());
    }

    println!("Deserialization done: name={}", result.unwrap().name);
}
