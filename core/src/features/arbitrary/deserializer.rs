#![allow(dead_code)]

use std::{
    fmt::{Display, Formatter},
    ops::Div,
};

use arbitrary::{Arbitrary, Unstructured};
use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;
use temporal_rs::Instant;

use crate::{
    schema::{Document, Schema, SchemaValue, ShapeType},
    serde::de::{DeserializeWithSchema, Deserializer},
};
// ============================================================================
// Errors
// ============================================================================

/// Error type to bridge between deserialization errors
/// and [`arbitrary::Error`] errors.
#[derive(Debug)]
pub struct Error(arbitrary::Error);
impl Error {
    /// Create a new [`Error`] wrapper instance
    pub const fn new(inner: arbitrary::Error) -> Self {
        Self(inner)
    }
}
impl From<arbitrary::Error> for Error {
    fn from(value: arbitrary::Error) -> Self {
        Error(value)
    }
}
impl From<Error> for arbitrary::Error {
    fn from(value: Error) -> Self {
        value.0
    }
}
impl std::error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl crate::serde::de::Error for Error {
    fn custom<T: Display>(_msg: T) -> Self {
        Error(arbitrary::Error::IncorrectFormat)
    }
}

// ============================================================================
// Deserializer
// ============================================================================

/// Deserializer that constructs and arbitrary Schema-Defined shape from unstructured data.
///
/// This deserializer is used to implement [`Arbitrary`] for generated
/// shapes, allowing them to support fuzzing.
pub struct ArbitraryDeserializer<'de, 'arb>(pub &'de mut Unstructured<'arb>);
impl<'de, 'arb> Deserializer<'de> for ArbitraryDeserializer<'de, 'arb> {
    type Error = Error;

    #[inline]
    fn read_bool(&mut self, _: &Schema) -> Result<bool, Self::Error> {
        Ok(bool::arbitrary(self.0)?)
    }

    #[inline]
    fn read_byte(&mut self, _: &Schema) -> Result<i8, Self::Error> {
        Ok(i8::arbitrary(self.0)?)
    }

    #[inline]
    fn read_short(&mut self, _: &Schema) -> Result<i16, Self::Error> {
        Ok(i16::arbitrary(self.0)?)
    }

    #[inline]
    fn read_integer(&mut self, schema: &Schema) -> Result<i32, Self::Error> {
        if schema.shape_type() == &ShapeType::Enum {
            let SchemaValue::IntEnum(enum_schema) = &**schema else {
                return Err(arbitrary::Error::IncorrectFormat.into());
            };
            let value_index = usize::arbitrary(self.0)? % enum_schema.values().len();
            Ok(*enum_schema
                .values()
                .get_index(value_index)
                .ok_or(arbitrary::Error::IncorrectFormat)?)
        } else {
            Ok(i32::arbitrary(self.0)?)
        }
    }

    #[inline]
    fn read_long(&mut self, _: &Schema) -> Result<i64, Self::Error> {
        Ok(i64::arbitrary(self.0)?)
    }

    #[inline]
    fn read_float(&mut self, _: &Schema) -> Result<f32, Self::Error> {
        Ok(f32::arbitrary(self.0)?)
    }

    #[inline]
    fn read_double(&mut self, _: &Schema) -> Result<f64, Self::Error> {
        Ok(f64::arbitrary(self.0)?)
    }

    #[inline]
    fn read_big_integer(&mut self, _: &Schema) -> Result<BigInt, Self::Error> {
        Ok(BigInt::arbitrary(self.0)?)
    }

    fn read_big_decimal(&mut self, schema: &Schema) -> Result<BigDecimal, Self::Error> {
        let scale = i64::arbitrary(self.0)?;
        let big_decimal = BigDecimal::from_bigint(self.read_big_integer(schema)?, scale);
        // divide by a random number
        let divisor = f32::arbitrary(self.0)?;
        Ok(big_decimal.div(divisor))
    }

    #[inline]
    fn read_string(&mut self, schema: &Schema) -> Result<String, Self::Error> {
        if schema.shape_type() == &ShapeType::Enum {
            let SchemaValue::Enum(enum_schema) = &**schema else {
                return Err(arbitrary::Error::IncorrectFormat.into());
            };
            let value_index = usize::arbitrary(self.0)? % enum_schema.values().len();
            Ok(enum_schema
                .values()
                .get_index(value_index)
                .ok_or(arbitrary::Error::IncorrectFormat)?
                .to_string())
        } else {
            Ok(String::arbitrary(self.0)?)
        }
    }

    #[inline]
    fn read_blob(&mut self, _: &Schema) -> Result<ByteBuffer, Self::Error> {
        let bytes = Vec::<u8>::arbitrary(self.0)?;
        Ok(ByteBuffer::from_bytes(bytes.as_slice()))
    }

    fn read_timestamp(&mut self, _: &Schema) -> Result<Instant, Self::Error> {
        // TODO: bound input
        let millis = i64::arbitrary(self.0)?;
        Ok(Instant::from_epoch_milliseconds(millis)
            .map_err(|_| arbitrary::Error::IncorrectFormat)?)
    }

    fn read_document(&mut self, _schema: &Schema) -> Result<Box<dyn Document>, Self::Error> {
        todo!()
    }

    fn read_struct<B, F>(
        &mut self,
        schema: &Schema,
        mut state: B,
        consumer: F,
    ) -> Result<B, Self::Error>
    where
        B: DeserializeWithSchema<'de>,
        F: Fn(B, &Schema, &mut Self) -> Result<B, Self::Error>,
    {
        // NOTE: We do not want unknown values as those are never serialized and
        // so are not relevant to these tests
        if schema.shape_type() == &ShapeType::Union {
            // pick a random member
            let idx = usize::arbitrary(self.0)? % schema.members().len();
            let (_, member) = schema
                .members()
                .get_index(idx)
                .ok_or(arbitrary::Error::IncorrectFormat)?;
            return consumer(state, member, self);
        }
        // For regular structs just walk through each member and set.
        for member_schema in schema.members().values() {
            state = consumer(state, member_schema, self)?;
        }
        Ok(state)
    }

    fn read_list<T, F>(
        &mut self,
        schema: &Schema,
        state: &mut T,
        consumer: F,
    ) -> Result<(), Self::Error>
    where
        T: DeserializeWithSchema<'de>,
        F: Fn(&mut T, &Schema, &mut Self) -> Result<(), Self::Error>,
    {
        let member_schema = schema.expect_member("member");
        let len: usize = usize::arbitrary(self.0)?;
        for _ in 0..len {
            consumer(state, member_schema, self)?;
        }
        Ok(())
    }

    fn read_map<T, F>(
        &mut self,
        _schema: &Schema,
        state: &mut T,
        consumer: F,
    ) -> Result<(), Self::Error>
    where
        T: DeserializeWithSchema<'de>,
        F: Fn(&mut T, String, &mut Self) -> Result<(), Self::Error>,
    {
        let len: usize = usize::arbitrary(self.0)?;
        for _ in 0..len {
            consumer(state, String::arbitrary(self.0)?, self)?;
        }
        Ok(())
    }

    fn is_null(&mut self) -> bool {
        // Randomly select if an optional value is null
        bool::arbitrary(self.0).unwrap_or(true)
    }

    fn read_null(&mut self) -> Result<(), Self::Error> {
        // Do nothing on null read.
        Ok(())
    }
}
