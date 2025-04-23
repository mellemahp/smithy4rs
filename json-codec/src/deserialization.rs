#![allow(unused_variables)]

use std::time::Instant;
use jiter::{Jiter, NumberAny, NumberInt, Peek};
use smithy4rs_core::{BigDecimal, BigInt, ByteBuffer};
use smithy4rs_core::documents::Document;
use smithy4rs_core::schema::Schema;
use smithy4rs_core::serde::de::{Deserializer, ListConsumer, StringMapConsumer, StructConsumer};
use crate::errors::JsonSerdeError;

pub struct JsonDeserializer<'de> {
    jiter: Jiter<'de>,
}

impl <'de> JsonDeserializer<'de> {
    pub fn new(json_data: &'de str) -> Self {
        // TODO: support options?
        JsonDeserializer {
            jiter: Jiter::new(json_data.as_bytes())
        }
    }

    fn known_int(&mut self) -> Result<i64, JsonSerdeError> {
        let peek = self.jiter.peek()?;
        match self.jiter.known_number(peek)? {
            NumberAny::Int(num_int) => match num_int {
                NumberInt::Int(i) => Ok(i),
                NumberInt::BigInt(_) => Err(JsonSerdeError::DeserializationError("Unexpected Big int value".to_string()))
            },
            NumberAny::Float(_) => Err(JsonSerdeError::DeserializationError("Unexpected float value".to_string())),
        }
    }

    fn known_float(&mut self) -> Result<f64, JsonSerdeError> {
        let peek = self.jiter.peek()?;
        match self.jiter.known_number(peek)? {
            NumberAny::Int(_) => Err(JsonSerdeError::DeserializationError("Unexpected int value".to_string())),
            NumberAny::Float(f) => Ok(f),
        }
    }
}

impl Deserializer for JsonDeserializer<'_> {
    type Error = JsonSerdeError;

    fn read_struct<T>(&mut self, schema: &Schema, state: &mut T, consumer: StructConsumer<T, Self>) -> Result<(), Self::Error> {
        // Parse first key.
        if let Ok(Some(first_key)) = self.jiter.known_object() {
            let member_schema = schema.get_member(first_key).expect("missing member");
            consumer(state, member_schema, self)?;
        }

        // Continue parsing remaining keys
        while let Ok(Some(next_key)) = self.jiter.next_key() {
            let member_schema = schema.get_member(next_key).expect("missing member");
            consumer(state, member_schema, self)?;
        }

        // Return empty if no failures
        Ok(())
    }

    fn read_list<T>(&mut self, schema: &Schema, state: T, consumer: ListConsumer<T, Self>) -> Result<(), Self::Error> {
        todo!()
    }

    fn read_string_map<T>(schema: &Schema, state: T, consumer: StringMapConsumer<T, Self>) -> Result<(), Self::Error> {
        todo!()
    }

    fn read_boolean(&mut self, schema: &Schema) -> Result<bool, Self::Error> {
        let peek = self.jiter.peek()?;
        Ok(self.jiter.known_bool(peek)?)
    }

    fn read_blob(&mut self, schema: &Schema) -> Result<ByteBuffer, Self::Error> {
        todo!()
    }

    fn read_byte(&mut self, schema: &Schema) -> Result<u8, Self::Error> {
        Ok(self.known_int()? as u8)
    }

    fn read_short(&mut self, _: &Schema) -> Result<i16, Self::Error> {
        Ok(self.known_int()? as i16)
    }

    fn read_integer(&mut self, _: &Schema) -> Result<i32, Self::Error> {
        Ok(self.known_int()? as i32)

    }

    fn read_long(&mut self, schema: &Schema) -> Result<i64, Self::Error> {
        Ok(self.known_int()?)
    }

    fn read_float(&mut self, schema: &Schema) -> Result<f32, Self::Error> {
        Ok(self.known_float()? as f32)
    }

    fn read_double(&mut self, schema: &Schema) -> Result<f64, Self::Error> {
        Ok(self.known_float()?)
    }

    fn read_big_integer(&mut self, schema: &Schema) -> Result<BigInt, Self::Error> {
        let peek = self.jiter.peek()?;
        match self.jiter.known_number(peek)? {
            NumberAny::Int(number_any) => match number_any {
                NumberInt::Int(_) => Err(JsonSerdeError::DeserializationError("Unexpected int value".to_string())),
                NumberInt::BigInt(i) => Ok(i)
            }
            NumberAny::Float(_) => Err(JsonSerdeError::DeserializationError("Unexpected float value".to_string())),
        }
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

    fn is_null(&mut self) -> bool {
        let Ok(peek) = self.jiter.peek() else {
            return false;
        };
        peek == Peek::Null
    }

    fn read_null<T>(&mut self) -> Result<(), Self::Error> {
        self.jiter.known_null()?;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Self::Error> {
        self.jiter.finish()?;
        Ok(())
    }
}