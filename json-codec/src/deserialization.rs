use std::time::Instant;
use jiter::Jiter;
use smithy4rs_core::{BigDecimal, BigInt, ByteBuffer};
use smithy4rs_core::documents::Document;
use smithy4rs_core::schema::Schema;
use smithy4rs_core::serde::{Deserializer, ListMemberConsumer, MapMemberConsumer, StructMemberConsumer};

pub struct JsonDeserializer<'de> {
    jiter: Jiter<'de>,
}

impl <'de> JsonDeserializer<'de> {
    pub fn new(json_data: &'de str) -> Self {
        // TODO: support options?
        JsonDeserializer { jiter: Jiter::new(json_data.as_bytes()) }
    }

    // TODO: should there be an interface method for this?
    pub fn finish(&mut self) {
        self.jiter.finish().unwrap();
    }
}

impl Deserializer for JsonDeserializer<'_> {
    fn read_struct<T, C: StructMemberConsumer<T, Self>>(&mut self, schema: &Schema, state: &mut T, consumer: C) {
        // Parse first key.
        if let Ok(Some(first_key)) = self.jiter.known_object() {
            println!("FIRST KEY: {}", first_key);
            let member_schema = schema.get_member(first_key).expect("missing member");
            consumer.accept(state, member_schema, self);
        }
        // Continue parsing remaining keys
        // TODO: Is there a nicer way to express?
        while let Ok(Some(next_key)) = self.jiter.next_key() {
            println!("OTHER: {}", next_key);
            let member_schema = schema.get_member(next_key).expect("missing member");
            consumer.accept(state, member_schema, self);
        }
    }

    fn read_list<T>(&mut self, schema: &Schema, state: T, consumer: ListMemberConsumer<T, Self>) {
        todo!()
    }

    fn read_string_map<T>(schema: &Schema, state: T, consumer: MapMemberConsumer<String, T, Self>) {
        todo!()
    }

    fn read_boolean(&mut self, schema: &Schema) -> bool {
        todo!()
    }

    fn read_blob(&mut self, schema: &Schema) -> ByteBuffer {
        todo!()
    }

    fn read_byte(&mut self, schema: &Schema) -> u8 {
        todo!()
    }

    fn read_short(&mut self, schema: &Schema) -> i16 {
        todo!()
    }

    fn read_integer(&mut self, schema: &Schema) -> i32 {
        todo!()
    }

    fn read_long(&mut self, schema: &Schema) -> i64 {
        todo!()
    }

    fn read_float(&mut self, schema: &Schema) -> f32 {
        todo!()
    }

    fn read_double(&mut self, schema: &Schema) -> f64 {
        todo!()
    }

    fn read_big_integer(&mut self, schema: &Schema) -> BigInt {
        todo!()
    }

    fn read_big_decimal(&mut self, schema: &Schema) -> BigDecimal {
        todo!()
    }

    fn read_string(&mut self, schema: &Schema) -> &str {
        self.jiter.known_str().expect("Expected known str")
    }

    fn read_timestamp(&mut self, schema: &Schema) -> Instant {
        todo!()
    }

    fn read_document(&mut self, schema: &Schema) -> Document {
        todo!()
    }

    fn is_null() -> bool {
        todo!()
    }

    fn read_null<T>() {
        todo!()
    }
}