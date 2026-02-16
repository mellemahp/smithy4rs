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
    serde::de::{DeserializeWithSchema, Deserializer, ListReader, MapReader, StructReader},
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
// Reader Types
// ============================================================================

/// [`StructReader`] implementation for arbitrary deserialization.
pub struct ArbitraryStructReader<'a, 'u> {
    u: &'a mut Unstructured<'u>,
    members: Vec<(String, Schema)>,
    index: usize,
}

impl<'de, 'u> StructReader<'de> for ArbitraryStructReader<'_, 'u> {
    type Error = Error;

    fn read_name(&mut self) -> Result<Option<String>, Self::Error> {
        if self.index >= self.members.len() {
            return Ok(None);
        }
        let name = self.members[self.index].0.clone();
        self.index += 1;
        Ok(Some(name))
    }

    fn read_value<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<T, Self::Error> {
        T::deserialize_with_schema(schema, &mut ArbitraryDeserializer::new(self.u, schema))
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// [`ListReader`] implementation for arbitrary deserialization.
pub struct ArbitraryListReader<'a, 'u> {
    u: &'a mut Unstructured<'u>,
    remaining: usize,
}

impl<'de, 'u> ListReader<'de> for ArbitraryListReader<'_, 'u> {
    type Error = Error;

    fn read_element<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<Option<T>, Self::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        let value =
            T::deserialize_with_schema(schema, &mut ArbitraryDeserializer::new(self.u, schema))?;
        Ok(Some(value))
    }
}

/// [`MapReader`] implementation for arbitrary deserialization.
pub struct ArbitraryMapReader<'a, 'u> {
    u: &'a mut Unstructured<'u>,
    remaining: usize,
}

impl<'de, 'u> MapReader<'de> for ArbitraryMapReader<'_, 'u> {
    type Error = Error;

    fn read_key(&mut self) -> Result<Option<String>, Self::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        Ok(Some(String::arbitrary(self.u)?))
    }

    fn read_value<V: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<V, Self::Error> {
        V::deserialize_with_schema(schema, &mut ArbitraryDeserializer::new(self.u, schema))
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// ============================================================================
// Deserializer
// ============================================================================

/// Deserializer that constructs an arbitrary Schema-Defined shape from unstructured data.
///
/// This deserializer is used to implement [`Arbitrary`] for generated
/// shapes, allowing them to support fuzzing.
pub struct ArbitraryDeserializer<'a, 'u> {
    /// The unstructured data source for generating arbitrary values.
    pub u: &'a mut Unstructured<'u>,
    schema: &'a Schema,
}

impl<'a, 'u> ArbitraryDeserializer<'a, 'u> {
    /// Create a new [`ArbitraryDeserializer`] instance.
    pub fn new(u: &'a mut Unstructured<'u>, schema: &'a Schema) -> Self {
        Self { u, schema }
    }
}

impl<'de, 'a, 'u> Deserializer<'de> for ArbitraryDeserializer<'a, 'u> {
    type Error = Error;
    type StructReader<'r>
        = ArbitraryStructReader<'r, 'u>
    where
        Self: 'r;
    type ListReader<'r>
        = ArbitraryListReader<'r, 'u>
    where
        Self: 'r;
    type MapReader<'r>
        = ArbitraryMapReader<'r, 'u>
    where
        Self: 'r;

    #[inline]
    fn read_bool(&mut self, _: &Schema) -> Result<bool, Self::Error> {
        Ok(bool::arbitrary(self.u)?)
    }

    #[inline]
    fn read_byte(&mut self, _: &Schema) -> Result<i8, Self::Error> {
        Ok(i8::arbitrary(self.u)?)
    }

    #[inline]
    fn read_short(&mut self, _: &Schema) -> Result<i16, Self::Error> {
        Ok(i16::arbitrary(self.u)?)
    }

    #[inline]
    fn read_integer(&mut self, schema: &Schema) -> Result<i32, Self::Error> {
        if schema.shape_type() == &ShapeType::Enum {
            let SchemaValue::IntEnum(enum_schema) = &**schema else {
                return Err(arbitrary::Error::IncorrectFormat.into());
            };
            let value_index = usize::arbitrary(self.u)? % enum_schema.values().len();
            Ok(*enum_schema
                .values()
                .get_index(value_index)
                .ok_or(arbitrary::Error::IncorrectFormat)?)
        } else {
            Ok(i32::arbitrary(self.u)?)
        }
    }

    #[inline]
    fn read_long(&mut self, _: &Schema) -> Result<i64, Self::Error> {
        Ok(i64::arbitrary(self.u)?)
    }

    #[inline]
    fn read_float(&mut self, _: &Schema) -> Result<f32, Self::Error> {
        Ok(f32::arbitrary(self.u)?)
    }

    #[inline]
    fn read_double(&mut self, _: &Schema) -> Result<f64, Self::Error> {
        Ok(f64::arbitrary(self.u)?)
    }

    #[inline]
    fn read_big_integer(&mut self, _: &Schema) -> Result<BigInt, Self::Error> {
        Ok(BigInt::arbitrary(self.u)?)
    }

    fn read_big_decimal(&mut self, schema: &Schema) -> Result<BigDecimal, Self::Error> {
        let scale = i64::arbitrary(self.u)?;
        let big_decimal = BigDecimal::from_bigint(self.read_big_integer(schema)?, scale);
        // divide by a random number
        let divisor = f32::arbitrary(self.u)?;
        Ok(big_decimal.div(divisor))
    }

    #[inline]
    fn read_string(&mut self, schema: &Schema) -> Result<String, Self::Error> {
        if schema.shape_type() == &ShapeType::Enum {
            let SchemaValue::Enum(enum_schema) = &**schema else {
                return Err(arbitrary::Error::IncorrectFormat.into());
            };
            let value_index = usize::arbitrary(self.u)? % enum_schema.values().len();
            Ok(enum_schema
                .values()
                .get_index(value_index)
                .ok_or(arbitrary::Error::IncorrectFormat)?
                .to_string())
        } else {
            Ok(String::arbitrary(self.u)?)
        }
    }

    #[inline]
    fn read_blob(&mut self, _: &Schema) -> Result<ByteBuffer, Self::Error> {
        let bytes = Vec::<u8>::arbitrary(self.u)?;
        Ok(ByteBuffer::from_bytes(bytes.as_slice()))
    }

    fn read_timestamp(&mut self, _: &Schema) -> Result<Instant, Self::Error> {
        // TODO: bound input
        let millis = i64::arbitrary(self.u)?;
        Ok(Instant::from_epoch_milliseconds(millis)
            .map_err(|_| arbitrary::Error::IncorrectFormat)?)
    }

    fn read_document(&mut self, _schema: &Schema) -> Result<Box<dyn Document>, Self::Error> {
        todo!()
    }

    fn read_struct(&mut self) -> Result<Self::StructReader<'_>, Self::Error> {
        // NOTE: We do not want unknown values as those are never serialized and
        // so are not relevant to these tests
        let members = if self.schema.shape_type() == &ShapeType::Union {
            // pick a random member
            let idx = usize::arbitrary(self.u)? % self.schema.members().len();
            let (name, schema) = self
                .schema
                .members()
                .get_index(idx)
                .ok_or(arbitrary::Error::IncorrectFormat)?;
            vec![(name.clone(), schema.clone())]
        } else {
            // For regular structs, yield all members
            self.schema
                .members()
                .iter()
                .map(|(name, schema)| (name.clone(), schema.clone()))
                .collect()
        };
        Ok(ArbitraryStructReader {
            u: self.u,
            members,
            index: 0,
        })
    }

    fn read_list(&mut self) -> Result<Self::ListReader<'_>, Self::Error> {
        let len = usize::arbitrary(self.u)?;
        Ok(ArbitraryListReader {
            u: self.u,
            remaining: len,
        })
    }

    fn read_map(&mut self) -> Result<Self::MapReader<'_>, Self::Error> {
        let len = usize::arbitrary(self.u)?;
        Ok(ArbitraryMapReader {
            u: self.u,
            remaining: len,
        })
    }

    fn is_null(&mut self) -> bool {
        // Randomly select if an optional value is null
        bool::arbitrary(self.u).unwrap_or(true)
    }

    fn read_null(&mut self) -> Result<(), Self::Error> {
        // Do nothing on null read.
        Ok(())
    }
}
