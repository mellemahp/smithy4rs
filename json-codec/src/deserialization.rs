use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::Arc,
};

use jiter::{Jiter, JiterResult, Peek};
use smithy4rs_core::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    schema::{Document, SchemaRef},
    serde::deserializers::Deserializer,
};

use crate::errors::JsonSerdeError;

/// A JSON deserializer that uses jiter.
///
/// This deserializer reads JSON data from a byte slice and uses schemas
/// to guide the deserialization.
#[derive(Clone)]
pub struct JsonDeserializer<'de> {
    parser: Rc<Jiter<'de>>,
}

impl<'de> JsonDeserializer<'de> {
    /// Create a new JSON deserializer from a byte slice.
    pub fn new(data: &'de [u8]) -> Self {
        Self {
            parser: Rc::new(Jiter::new(data)),
        }
    }
}
impl<'de> Deref for JsonDeserializer<'de> {
    type Target = Jiter<'de>;

    fn deref(&self) -> &Self::Target {
        self.parser.as_ref()
    }
}
impl<'de> DerefMut for JsonDeserializer<'de> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Rc::get_mut(&mut self.parser).expect("Should be unwrappable")
    }
}

impl<'de, 'a: 'de> Deserializer<'de> for &'a mut JsonDeserializer<'de> {
    type Error = JsonSerdeError;

    fn read_bool(self, _schema: &SchemaRef) -> Result<bool, Self::Error> {
        self.next_bool().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read bool: {}", e))
        })
    }

    fn read_byte(self, _schema: &SchemaRef) -> Result<i8, Self::Error> {
        let value = self.next_int().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read byte: {}", e))
        })?;

        match value {
            jiter::NumberInt::Int(i) => i8::try_from(i).map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Value out of range for i8: {}", e))
            }),
            jiter::NumberInt::BigInt(b) => {
                let i = b.to_string().parse::<i8>().map_err(|e| {
                    JsonSerdeError::DeserializationError(format!(
                        "BigInt value out of range for i8: {}",
                        e
                    ))
                })?;
                Ok(i)
            }
        }
    }

    fn read_short(self, _schema: &SchemaRef) -> Result<i16, Self::Error> {
        let value = self.next_int().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read short: {}", e))
        })?;

        match value {
            jiter::NumberInt::Int(i) => i16::try_from(i).map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Value out of range for i16: {}", e))
            }),
            jiter::NumberInt::BigInt(b) => {
                let i = b.to_string().parse::<i16>().map_err(|e| {
                    JsonSerdeError::DeserializationError(format!(
                        "BigInt value out of range for i16: {}",
                        e
                    ))
                })?;
                Ok(i)
            }
        }
    }

    fn read_integer(self, _schema: &SchemaRef) -> Result<i32, Self::Error> {
        let value = self.next_int().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read integer: {}", e))
        })?;

        match value {
            jiter::NumberInt::Int(i) => i32::try_from(i).map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Value out of range for i32: {}", e))
            }),
            jiter::NumberInt::BigInt(b) => {
                let i = b.to_string().parse::<i32>().map_err(|e| {
                    JsonSerdeError::DeserializationError(format!(
                        "BigInt value out of range for i32: {}",
                        e
                    ))
                })?;
                Ok(i)
            }
        }
    }

    fn read_long(self, _schema: &SchemaRef) -> Result<i64, Self::Error> {
        let value = self.next_int().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read long: {}", e))
        })?;

        match value {
            jiter::NumberInt::Int(i) => Ok(i),
            jiter::NumberInt::BigInt(b) => b.to_string().parse::<i64>().map_err(|e| {
                JsonSerdeError::DeserializationError(format!(
                    "BigInt value out of range for i64: {}",
                    e
                ))
            }),
        }
    }

    fn read_float(self, _schema: &SchemaRef) -> Result<f32, Self::Error> {
        self.next_float()
            .map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Failed to read float: {}", e))
            })
            .map(|v| v as f32)
    }

    fn read_double(self, _schema: &SchemaRef) -> Result<f64, Self::Error> {
        self.next_float().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read double: {}", e))
        })
    }

    fn read_big_integer(self, _schema: &SchemaRef) -> Result<BigInt, Self::Error> {
        let s = self.next_str().map_err(|e| {
            JsonSerdeError::DeserializationError(format!(
                "Failed to read big integer string: {}",
                e
            ))
        })?;

        s.parse::<BigInt>().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to parse big integer: {}", e))
        })
    }

    fn read_big_decimal(self, _schema: &SchemaRef) -> Result<BigDecimal, Self::Error> {
        let s = self.next_str().map_err(|e| {
            JsonSerdeError::DeserializationError(format!(
                "Failed to read big decimal string: {}",
                e
            ))
        })?;

        s.parse::<BigDecimal>().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to parse big decimal: {}", e))
        })
    }

    fn read_string(self, _schema: &SchemaRef) -> Result<String, Self::Error> {
        self.known_str()
            .map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Failed to read string: {}", e))
            })
            .map(|s| s.to_string())
    }

    fn read_blob(self, _schema: &SchemaRef) -> Result<ByteBuffer, Self::Error> {
        // Blobs in JSON are base64-encoded strings
        let s = self.next_str().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read blob string: {}", e))
        })?;

        // For now, just convert the string to bytes
        // TODO: Add base64 decoding
        Ok(ByteBuffer::from(s.as_bytes()))
    }

    fn read_timestamp(self, _schema: &SchemaRef) -> Result<Instant, Self::Error> {
        let s = self.next_str().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read timestamp string: {}", e))
        })?;

        // TODO: timestampFormat handling
        Instant::from_utf8(s.as_bytes()).map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to parse timestamp: {}", e))
        })
    }

    fn read_document(self, _schema: &SchemaRef) -> Result<Box<dyn Document>, Self::Error> {
        todo!("Support deserialization of documents")
    }

    fn read_struct<B, F>(
        mut self,
        schema: &SchemaRef,
        mut builder: B,
        consumer: F,
    ) -> Result<B, Self::Error>
    where
        F: Fn(B, &SchemaRef, Self) -> Result<B, Self::Error>,
    {
        // next_object() returns the first key, or None for empty object
        let maybe_key = self
            .next_object()
            .map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Expected object start: {}", e))
            })?
            .map(|s| s.to_string());

        // Process all subsequent keys
        while let Some(key) = &maybe_key {
            match schema.get_member(key) {
                Some(member_schema) => {
                    builder = consumer(builder, member_schema)?;
                }
                None => {}
            }
        }

        Ok(builder)
    }

    fn read_list<T, F>(
        self,
        _schema: &SchemaRef,
        state: &mut T,
        mut consumer: F,
    ) -> Result<(), Self::Error>
    where
        F: FnMut(&mut T, &SchemaRef, Self) -> Result<(), Self::Error>,
    {
        // Get the member schema for list elements
        let member_schema = _schema.get_member("member").ok_or_else(|| {
            JsonSerdeError::DeserializationError("List schema missing member".to_string())
        })?;

        // next_array() returns Option<Peek> for the first element (or None for empty array)
        let mut maybe_peek = self.next_array().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Expected array start: {}", e))
        })?;

        // Process all elements (first and subsequent)
        while maybe_peek.is_some() {
            consumer(state, member_schema, &mut *self)?;

            // Get next element
            maybe_peek = self.array_step().map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Failed to read array element: {}", e))
            })?;
        }

        Ok(())
    }

    fn read_map<T, F>(
        self,
        _schema: &SchemaRef,
        state: &mut T,
        mut consumer: F,
    ) -> Result<(), Self::Error>
    where
        F: FnMut(&mut T, String, Self) -> Result<(), Self::Error>,
    {
        // next_object() returns the first key, or None for empty object
        let mut maybe_key = self.next_object().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Expected object start: {}", e))
        })?;

        // Process all keys (first and subsequent)
        while let Some(key) = maybe_key {
            consumer(state, key.to_owned(), self)?;

            // Get next key
            maybe_key = self.parser.next_key().map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Failed to read map key: {}", e))
            })?;
        }

        Ok(())
    }

    fn is_null(&mut self) -> bool {
        matches!(self.peek(), Ok(jiter::Peek::Null))
    }

    fn read_null(self) -> Result<(), Self::Error> {
        self.next_null()
            .map_err(|e| JsonSerdeError::DeserializationError(format!("Expected null: {}", e)))
    }

    fn unknown(self, name: &str) -> Result<(), Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use smithy4rs_core::prelude::*;

    use super::*;

    // TODO(test): Add comprehensive suite here

    #[test]
    fn test_read_primitives() {
        let mut de = JsonDeserializer::new("true".as_bytes());
        assert!(de.read_bool(&BOOLEAN).unwrap());

        let mut de = JsonDeserializer::new("42".as_bytes());
        assert_eq!(de.read_integer(&INTEGER).unwrap(), 42);

        let mut de = JsonDeserializer::new("1.234".as_bytes());
        assert!((de.read_float(&FLOAT).unwrap() - 1.234).abs() < 0.001);

        let mut de = JsonDeserializer::new("\"hello\"".as_bytes());
        assert_eq!(de.read_string(&STRING).unwrap(), "hello");
    }

    #[test]
    fn test_is_null() {
        let mut de = &mut JsonDeserializer::new("null".as_bytes());
        assert!(de.is_null());

        let mut de = &mut JsonDeserializer::new("42".as_bytes());
        assert!(!de.is_null());

        let mut de = &mut JsonDeserializer::new("\"string\"".as_bytes());
        assert!(!de.is_null());
    }
}
