#![allow(dead_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};
use arbitrary::Unstructured;
use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;
use temporal_rs::Instant;
use crate::schema::{Document, Schema};
use crate::serde::de::{DeserializeWithSchema, Deserializer};

// ============================================================================
// Error
// ============================================================================

/// Yup and Yup
#[derive(Debug)]
pub struct ArbError(pub arbitrary::Error);
impl From<arbitrary::Error> for ArbError {
    fn from(value: arbitrary::Error) -> Self {
        ArbError(value)
    }
}
impl Error for ArbError {}
impl Display for ArbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl crate::serde::de::Error for ArbError {
    fn custom<T: Display>(msg: T) -> Self {
        ArbError(arbitrary::Error::IncorrectFormat)
    }
}

// ============================================================================
// Deserializer
// ============================================================================
use arbitrary::{Arbitrary};
use crate::prelude::RequiredTrait;

/// Yup and Yup
pub struct ArbitraryDeserializer<'de, 'arb>(pub &'de mut Unstructured<'arb>);
impl <'de, 'arb> Deserializer<'de> for ArbitraryDeserializer<'de, 'arb> {
    type Error = ArbError;

    fn read_bool(&mut self, _: &Schema) -> Result<bool, Self::Error> {
        Ok(bool::arbitrary(self.0)?)
    }

    fn read_byte(&mut self, _: &Schema) -> Result<i8, Self::Error> {
        Ok(i8::arbitrary(self.0)?)
    }

    fn read_short(&mut self, _: &Schema) -> Result<i16, Self::Error> {
        Ok(i16::arbitrary(self.0)?)
    }

    fn read_integer(&mut self, _: &Schema) -> Result<i32, Self::Error> {
        Ok(i32::arbitrary(self.0)?)
    }

    fn read_long(&mut self, _: &Schema) -> Result<i64, Self::Error> {
        Ok(i64::arbitrary(self.0)?)
    }

    fn read_float(&mut self, _: &Schema) -> Result<f32, Self::Error> {
        Ok(f32::arbitrary(self.0)?)
    }

    fn read_double(&mut self, _: &Schema) -> Result<f64, Self::Error> {
        Ok(f64::arbitrary(self.0)?)
    }

    fn read_big_integer(&mut self, _: &Schema) -> Result<BigInt, Self::Error> {
        Ok(BigInt::arbitrary(self.0)?)
    }

    fn read_big_decimal(&mut self, _: &Schema) -> Result<BigDecimal, Self::Error> {
        // TODO: beter way to impl arbitrary?
        let scale = i64::arbitrary(self.0)?;
        let big_int = BigInt::arbitrary(self.0)?;
        Ok(BigDecimal::from_bigint(big_int, scale))
    }

    fn read_string(&mut self, _: &Schema) -> Result<String, Self::Error> {
        Ok(String::arbitrary(self.0)?)
    }

    fn read_blob(&mut self, _: &Schema) -> Result<ByteBuffer, Self::Error> {
        todo!()
    }

    fn read_timestamp(&mut self, schema: &Schema) -> Result<Instant, Self::Error> {
        let millis = i64::arbitrary(self.0)?;
        Ok(Instant::from_epoch_milliseconds(millis)
            .map_err(|_| arbitrary::Error::IncorrectFormat)?)
    }

    fn read_document(&mut self, schema: &Schema) -> Result<Box<dyn Document>, Self::Error> {
        todo!()
    }

    fn read_struct<B, F>(&mut self, schema: &Schema, mut builder: B, consumer: F) -> Result<B, Self::Error>
    where
        B: DeserializeWithSchema<'de>,
        F: Fn(B, &Schema, &mut Self) -> Result<B, Self::Error>
    {
        // NOTE: We do not want unknown values as those are never serialized and
        // so are not relevant to these tests
        for (name, member_schema) in schema.members() {
            // if the member is optional, randomly pick if it should be set
            if !member_schema.contains_type::<RequiredTrait>() &&
                bool::arbitrary(self.0)?
            {
                continue;
            }
            builder = consumer(builder, member_schema, self)?;
        }
        Ok(builder)
    }

    fn read_list<T, F>(&mut self, schema: &Schema, state: &mut T, consumer: F) -> Result<(), Self::Error>
    where
        T: DeserializeWithSchema<'de>,
        F: Fn(&mut T, &Schema, &mut Self) -> Result<(), Self::Error>
    {
        let member_schema = schema.expect_member("member");
        // TODO: Could we use smarter size hints here?
        let len: usize = usize::arbitrary(self.0)?;
        for _ in 0..len {
            consumer(state, member_schema, self)?;
        }
        Ok(())
    }

    fn read_map<T, F>(&mut self, schema: &Schema, state: &mut T, consumer: F) -> Result<(), Self::Error>
    where
        T: DeserializeWithSchema<'de>,
        F: Fn(&mut T, String, &mut Self) -> Result<(), Self::Error>
    {
        let values_schema = schema.expect_member("value");
        let len: usize = usize::arbitrary(self.0)?;
        for _ in 0..len {
            consumer(state, String::arbitrary(self.0)?, self)?;
        }
        Ok(())
    }

    fn is_null(&mut self) -> bool {
        todo!()
    }

    fn read_null(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}