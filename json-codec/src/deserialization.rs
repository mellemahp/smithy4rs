use std::str::FromStr;
use std::time::Instant;
use jiter::{Jiter, NumberAny, NumberInt, Peek};
use smithy4rs_core::{BigDecimal, BigInt, ByteBuffer};
use smithy4rs_core::documents::Document;
use smithy4rs_core::schema::Schema;
use smithy4rs_core::serde::{Deserializer, ListMemberConsumer, MapMemberConsumer, StructMemberConsumer};
use crate::errors::JsonSerdeError;

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

    fn known_int(&mut self) -> Result<i64, JsonSerdeError> {
        let peek = self.jiter.peek()?;
        match self.jiter.known_number(peek)? {
            NumberAny::Int(numInt) => match numInt {
                NumberInt::Int(i) => Ok(i),
                NumberInt::BigInt(_) => Err(JsonSerdeError::DeserializationError("Unexpected Big int value".to_string()))
            },
            NumberAny::Float(_) => Err(JsonSerdeError::DeserializationError("Unexpected float value".to_string())),
        }
    }
}

impl Deserializer for JsonDeserializer<'_> {
    type Error = JsonSerdeError;

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

    fn read_boolean(&mut self, schema: &Schema) -> Result<bool, Self::Error> {
        todo!()
    }

    fn read_blob(&mut self, schema: &Schema) -> Result<ByteBuffer, Self::Error> {
        todo!()
    }

    fn read_byte(&mut self, schema: &Schema) -> Result<u8, Self::Error> {
        Ok(self.known_int()? as u8)
    }

    fn read_short(&mut self, schema: &Schema) -> Result<i16, Self::Error> {
        todo!()
    }

    fn read_integer(&mut self, schema: &Schema) -> Result<i32, Self::Error> {
        todo!()
    }

    fn read_long(&mut self, schema: &Schema) -> Result<i64, Self::Error> {
        todo!()
    }

    fn read_float(&mut self, schema: &Schema) -> Result<f32, Self::Error> {
        todo!()
    }

    fn read_double(&mut self, schema: &Schema) -> Result<f64, Self::Error> {
        todo!()
    }

    fn read_big_integer(&mut self, schema: &Schema) -> Result<BigInt, Self::Error> {
        todo!()
    }

    fn read_big_decimal(&mut self, schema: &Schema) -> Result<BigDecimal, Self::Error> {
        todo!()
    }

    fn read_string(&mut self, _: &Schema) -> Result<&str, Self::Error> {
        Ok(self.jiter.known_str()?)
    }

    fn read_timestamp(&mut self, schema: &Schema) -> Result<Instant, Self::Error> {
        todo!()
    }

    fn read_document(&mut self, schema: &Schema) -> Result<Document, Self::Error> {
        todo!()
    }

    fn is_null() -> bool {
        todo!()
    }

    fn read_null<T>() -> Result<(), Self::Error> {
        todo!()
    }
}