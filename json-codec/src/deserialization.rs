#![allow(unused_variables)]

use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;
use crate::errors::JsonSerdeError;
use jiter::{Jiter, NumberAny, NumberInt, Peek};
use smithy4rs_core::schema::{Schema, SchemaRef};
use smithy4rs_core::schema::documents::Document;
use smithy4rs_core::serde::de::{Deserialize, Deserializer, MapReader};
use smithy4rs_core::{BigDecimal, BigInt, ByteBuffer};
use smithy4rs_core::serde::deserializers::ListReader;

pub struct JsonDeserializer<'de> {
    generator: &'de mut Jiter<'de>,
}

impl<'de> JsonDeserializer<'de> {
    pub fn new(jiter: &'de mut Jiter<'de>) -> Self {
        // TODO: support options?
        JsonDeserializer {
            generator: jiter,
        }
    }

    pub fn from(generator: &'de mut Jiter<'de>) -> Self {
        JsonDeserializer { generator }
    }

    fn known_int(&mut self) -> Result<i64, JsonSerdeError> {
        let peek = self.generator.peek()?;
        match self.generator.known_number(peek)? {
            NumberAny::Int(num_int) => match num_int {
                NumberInt::Int(i) => Ok(i),
                NumberInt::BigInt(_) => Err(JsonSerdeError::DeserializationError(
                    "Unexpected Big int value".to_string(),
                )),
            },
            NumberAny::Float(_) => Err(JsonSerdeError::DeserializationError(
                "Unexpected float value".to_string(),
            )),
        }
    }

    fn known_float(&mut self) -> Result<f64, JsonSerdeError> {
        let peek = self.generator.peek()?;
        match self.generator.known_number(peek)? {
            NumberAny::Int(_) => Err(JsonSerdeError::DeserializationError(
                "Unexpected int value".to_string(),
            )),
            NumberAny::Float(f) => Ok(f),
        }
    }
}

impl Deserializer<'_> for JsonDeserializer<'_> {
    type Error = JsonSerdeError;
    type MapReader<'m>  = JsonMapReader;
    type ListReader<'l> = JsonListReader<'l>;

    fn read_struct<'a, T>(self, schema: &SchemaRef, builder: impl smithy4rs_core::serde::builders::ShapeBuilder<'a, T>) -> Result<(), Self::Error> {
        todo!()
    }

    fn read_list(self, schema: &SchemaRef) -> Result<Self::ListReader<'_>, Self::Error> {
        Ok(JsonListReader::new(self))
    }

    fn read_map(self, schema: &SchemaRef) -> Result<Self::MapReader<'_>, Self::Error> {
        todo!()
    }

    //     // Continue parsing remaining keys
    //     while let Ok(Some(next_key)) = self.generator.next_key() {
    //         let member_schema = schema.get_member(next_key).expect("missing member");
    //         consumer(state, member_schema, self)?;
    //     }
    //
    //     // Return empty if no failures
    //     Ok(())
    // }


    fn read_boolean(mut self, schema: &SchemaRef) -> Result<bool, Self::Error> {
        let peek = self.generator.peek()?;
        Ok(self.generator.known_bool(peek)?)
    }

    fn read_blob(mut self, schema: &SchemaRef) -> Result<ByteBuffer, Self::Error> {
        todo!()
    }

    fn read_byte(mut self, schema: &SchemaRef) -> Result<i8, Self::Error> {
        Ok(self.known_int()? as i8)
    }

    fn read_short(mut self, _: &SchemaRef) -> Result<i16, Self::Error> {
        Ok(self.known_int()? as i16)
    }

    fn read_integer(mut self, _: &SchemaRef) -> Result<i32, Self::Error> {
        Ok(self.known_int()? as i32)
    }

    fn read_long(mut self, schema: &SchemaRef) -> Result<i64, Self::Error> {
        Ok(self.known_int()?)
    }

    fn read_float(mut self, schema: &SchemaRef) -> Result<f32, Self::Error> {
        Ok(self.known_float()? as f32)
    }

    fn read_double(mut self, schema: &SchemaRef) -> Result<f64, Self::Error> {
        Ok(self.known_float()?)
    }

    fn read_big_integer(self, schema: &SchemaRef) -> Result<BigInt, Self::Error> {
        let peek = self.generator.peek()?;
        match self.generator.known_number(peek)? {
            NumberAny::Int(number_any) => match number_any {
                NumberInt::Int(_) => Err(JsonSerdeError::DeserializationError(
                    "Unexpected int value".to_string(),
                )),
                NumberInt::BigInt(i) => Ok(i),
            },
            NumberAny::Float(_) => Err(JsonSerdeError::DeserializationError(
                "Unexpected float value".to_string(),
            )),
        }
    }

    fn read_big_decimal(self, schema: &SchemaRef) -> Result<BigDecimal, Self::Error> {
        todo!()
    }

    fn read_string(self, _: &SchemaRef) -> Result<String, Self::Error> {
        Ok(self.generator.known_str()?.to_string())
    }

    fn read_timestamp(self, schema: &SchemaRef) -> Result<Instant, Self::Error> {
        todo!()
    }

    fn read_document(self, schema: &SchemaRef) -> Result<Document, Self::Error> {
        todo!()
    }

    fn is_null(&self, schema: &SchemaRef) -> bool {
        todo!()
        // let Ok(peek) = self.generator.peek() else {
        //     return false;
        // };
        // peek == Peek::Null
    }

    fn read_null<T>(&mut self) -> Result<(), Self::Error> {
        self.generator.known_null()?;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Self::Error> {
        self.generator.finish()?;
        Ok(())
    }
}

struct JsonListReader<'l> {
    is_first: bool,
    parent: JsonDeserializer<'l>,
}
impl JsonListReader<'_> {
    fn new(parent: JsonDeserializer) -> JsonListReader {
        JsonListReader { is_first: true, parent, }
    }
}
impl ListReader<'_> for JsonListReader<'_> {
    type Error = JsonSerdeError;

    fn size_hint(&self) -> Option<usize> {
        // TODO: How to actually hint size of array?
        None
    }

    fn read_item<'i, I>(&mut self, schema: &SchemaRef) -> Result<Option<I>, Self::Error>
    where
        I: Deserialize<'i>
    {
        // Increment array values.
        let next = if self.is_first {
            self.is_first = false;
            self.parent.generator.next_array()?
        } else {
            self.parent.generator.array_step()?
        };

        // No array values to read. Return
        if next.is_none() {
            return Ok(None);
        };

        // Get member schema for deserializing
        let Some(list_schema) = schema.as_list() else {
            Err(JsonSerdeError::DeserializationError("Expected list".to_string()))?
        };
        let value_deser = JsonDeserializer::from(self.parent.generator);
        let value = I::deserialize(&list_schema.member, value_deser)?;
        Ok(Some(value))
    }
}

struct JsonMapReader {}
impl MapReader<'_> for JsonMapReader {
    type Error = JsonSerdeError;

    fn size_hint(&self) -> Option<usize> {
        todo!()
    }

    fn read_entry<'k, 'v, K, V>(&mut self, schema: &SchemaRef) -> Result<Option<(K, V)>, Self::Error>
    where
        K: Deserialize<'k> + Hash + Eq,
        V: Deserialize<'v>
    {
        todo!()
    }
}
