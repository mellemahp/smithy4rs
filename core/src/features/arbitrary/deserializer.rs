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
    index: usize,
    set_union: bool,
}

impl<'de> StructReader<'de> for ArbitraryStructReader<'_, '_> {
    type Error = Error;

    fn read_member<'a>(&mut self, schema: &'a Schema) -> Result<Option<&'a Schema>, Self::Error> {
        // NOTE: We do not want unknown values as those are never serialized and
        // so are not relevant to these tests
        if schema.shape_type() == &ShapeType::Union {
            if self.set_union {
                // If a union value has already been set, do not set again
                return Ok(None);
            }

            self.set_union = true;
            // pick a random member
            let idx = usize::arbitrary(self.u)? % schema.members().len();
            let (_, member_schema) = schema
                .members()
                .get_index(idx)
                .ok_or(arbitrary::Error::IncorrectFormat)?;
            return Ok(Some(member_schema));
        };
        // === Normal Structures ====
        // If there are no more members to set, return empty
        if self.index >= schema.members().len() {
            return Ok(None);
        }
        // Get the next member
        let (_, member_schema) = schema
            .members()
            .get_index(self.index)
            .ok_or(arbitrary::Error::IncorrectFormat)?;
        self.index += 1;

        Ok(Some(member_schema))
    }

    fn read_value<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<T, Self::Error> {
        T::deserialize_with_schema(schema, ArbitraryDeserializer { u: self.u })
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

impl<'de> ListReader<'de> for ArbitraryListReader<'_, '_> {
    type Error = Error;

    fn read_element<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<Option<T>, Self::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        let value = T::deserialize_with_schema(schema, ArbitraryDeserializer { u: self.u })?;
        Ok(Some(value))
    }
}

/// [`MapReader`] implementation for arbitrary deserialization.
pub struct ArbitraryMapReader<'a, 'u> {
    u: &'a mut Unstructured<'u>,
    remaining: usize,
}

impl<'de> MapReader<'de> for ArbitraryMapReader<'_, '_> {
    type Error = Error;

    fn read_key(&mut self) -> Result<Option<String>, Self::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        Ok(Some(String::arbitrary(self.u)?))
    }

    #[inline]
    fn read_value<V: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<V, Self::Error> {
        V::deserialize_with_schema(schema, ArbitraryDeserializer { u: self.u })
    }

    #[inline]
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
    u: &'a mut Unstructured<'u>,
}

impl<'a, 'u> ArbitraryDeserializer<'a, 'u> {
    /// Create a new [`ArbitraryDeserializer`] instance.
    #[inline]
    pub fn new(u: &'a mut Unstructured<'u>) -> Self {
        Self { u }
    }
}

impl<'de, 'a, 'u> Deserializer<'de> for ArbitraryDeserializer<'a, 'u> {
    type Error = Error;
    type StructReader = ArbitraryStructReader<'a, 'u>;
    type ListReader = ArbitraryListReader<'a, 'u>;
    type MapReader = ArbitraryMapReader<'a, 'u>;

    #[inline]
    fn read_bool(self, _: &Schema) -> Result<bool, Self::Error> {
        Ok(bool::arbitrary(self.u)?)
    }

    #[inline]
    fn read_byte(self, _: &Schema) -> Result<i8, Self::Error> {
        Ok(i8::arbitrary(self.u)?)
    }

    #[inline]
    fn read_short(self, _: &Schema) -> Result<i16, Self::Error> {
        Ok(i16::arbitrary(self.u)?)
    }

    fn read_integer(self, schema: &Schema) -> Result<i32, Self::Error> {
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
    fn read_long(self, _: &Schema) -> Result<i64, Self::Error> {
        Ok(i64::arbitrary(self.u)?)
    }

    #[inline]
    fn read_float(self, _: &Schema) -> Result<f32, Self::Error> {
        Ok(f32::arbitrary(self.u)?)
    }

    #[inline]
    fn read_double(self, _: &Schema) -> Result<f64, Self::Error> {
        Ok(f64::arbitrary(self.u)?)
    }

    #[inline]
    fn read_big_integer(self, _: &Schema) -> Result<BigInt, Self::Error> {
        Ok(BigInt::arbitrary(self.u)?)
    }

    fn read_big_decimal(self, _schema: &Schema) -> Result<BigDecimal, Self::Error> {
        let scale = i64::arbitrary(self.u)?;
        let big_int = BigInt::arbitrary(self.u)?;
        let big_decimal = BigDecimal::from_bigint(big_int, scale);
        // divide by a random number
        let divisor = f32::arbitrary(self.u)?;
        Ok(big_decimal.div(divisor))
    }

    fn read_string(self, schema: &Schema) -> Result<String, Self::Error> {
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
    fn read_blob(self, _: &Schema) -> Result<ByteBuffer, Self::Error> {
        let bytes = Vec::<u8>::arbitrary(self.u)?;
        Ok(ByteBuffer::from_bytes(bytes.as_slice()))
    }

    fn read_timestamp(self, _: &Schema) -> Result<Instant, Self::Error> {
        // TODO: bound input
        let millis = i64::arbitrary(self.u)?;
        Ok(Instant::from_epoch_milliseconds(millis)
            .map_err(|_| arbitrary::Error::IncorrectFormat)?)
    }

    fn read_document(self, _schema: &Schema) -> Result<Box<dyn Document>, Self::Error> {
        todo!()
    }

    fn read_struct(self, _schema: &Schema) -> Result<Self::StructReader, Self::Error> {
        Ok(ArbitraryStructReader {
            u: self.u,
            index: 0,
            set_union: false,
        })
    }

    fn read_list(self, _schema: &Schema) -> Result<Self::ListReader, Self::Error> {
        let len = usize::arbitrary(self.u)?;
        Ok(ArbitraryListReader {
            u: self.u,
            remaining: len,
        })
    }

    fn read_map(self, _schema: &Schema) -> Result<Self::MapReader, Self::Error> {
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

    fn read_null(self) -> Result<(), Self::Error> {
        // Do nothing on null read.
        Ok(())
    }
}
