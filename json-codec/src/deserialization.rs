use smithy4rs_core::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    schema::Schema,
    serde::deserializers::{
        DeserializeWithSchema, Deserializer, ListReader, MapReader, StructReader,
    },
};

use crate::errors::JsonSerdeError;

/// A JSON deserializer that uses jiter.
pub struct JsonDeserializer<'de> {
    parser: jiter::Jiter<'de>,
}

impl<'de> JsonDeserializer<'de> {
    /// Create a new JSON deserializer from a byte slice.
    pub fn new(data: &'de [u8]) -> Self {
        Self {
            parser: jiter::Jiter::new(data),
        }
    }
}

/// Reader for JSON struct members.
pub struct JsonStructReader<'de, 'a> {
    de: &'a mut JsonDeserializer<'de>,
    started: bool,
}

/// Reader for JSON list elements.
pub struct JsonListReader<'de, 'a> {
    de: &'a mut JsonDeserializer<'de>,
    started: bool,
}

/// Reader for JSON map entries.
pub struct JsonMapReader<'de, 'a> {
    de: &'a mut JsonDeserializer<'de>,
    started: bool,
}

// ============================================================================
// Deserializer Implementation (only one needed!)
// ============================================================================

impl<'de, 'a> Deserializer<'de> for &'a mut JsonDeserializer<'de> {
    type Error = JsonSerdeError;
    type StructReader = JsonStructReader<'de, 'a>;
    type ListReader = JsonListReader<'de, 'a>;
    type MapReader = JsonMapReader<'de, 'a>;

    fn read_bool(self, _schema: &Schema) -> Result<bool, Self::Error> {
        self.parser.next_bool().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read bool: {}", e))
        })
    }

    fn read_byte(self, _schema: &Schema) -> Result<i8, Self::Error> {
        let value = self.parser.next_int().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read byte: {}", e))
        })?;

        match value {
            jiter::NumberInt::Int(i) => i8::try_from(i).map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Value out of range for i8: {}", e))
            }),
            jiter::NumberInt::BigInt(b) => b.to_string().parse::<i8>().map_err(|e| {
                JsonSerdeError::DeserializationError(format!(
                    "BigInt value out of range for i8: {}",
                    e
                ))
            }),
        }
    }

    fn read_short(self, _schema: &Schema) -> Result<i16, Self::Error> {
        let value = self.parser.next_int().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read short: {}", e))
        })?;

        match value {
            jiter::NumberInt::Int(i) => i16::try_from(i).map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Value out of range for i16: {}", e))
            }),
            jiter::NumberInt::BigInt(b) => b.to_string().parse::<i16>().map_err(|e| {
                JsonSerdeError::DeserializationError(format!(
                    "BigInt value out of range for i16: {}",
                    e
                ))
            }),
        }
    }

    fn read_integer(self, _schema: &Schema) -> Result<i32, Self::Error> {
        let value = self.parser.next_int().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read integer: {}", e))
        })?;

        match value {
            jiter::NumberInt::Int(i) => i32::try_from(i).map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Value out of range for i32: {}", e))
            }),
            jiter::NumberInt::BigInt(b) => b.to_string().parse::<i32>().map_err(|e| {
                JsonSerdeError::DeserializationError(format!(
                    "BigInt value out of range for i32: {}",
                    e
                ))
            }),
        }
    }

    fn read_long(self, _schema: &Schema) -> Result<i64, Self::Error> {
        let value = self.parser.next_int().map_err(|e| {
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

    fn read_float(self, _schema: &Schema) -> Result<f32, Self::Error> {
        self.parser
            .next_float()
            .map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Failed to read float: {}", e))
            })
            .map(|v| v as f32)
    }

    fn read_double(self, _schema: &Schema) -> Result<f64, Self::Error> {
        self.parser.next_float().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read double: {}", e))
        })
    }

    fn read_big_integer(self, _schema: &Schema) -> Result<BigInt, Self::Error> {
        let s = self.parser.next_str().map_err(|e| {
            JsonSerdeError::DeserializationError(format!(
                "Failed to read big integer string: {}",
                e
            ))
        })?;

        s.parse::<BigInt>().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to parse big integer: {}", e))
        })
    }

    fn read_big_decimal(self, _schema: &Schema) -> Result<BigDecimal, Self::Error> {
        let s = self.parser.next_str().map_err(|e| {
            JsonSerdeError::DeserializationError(format!(
                "Failed to read big decimal string: {}",
                e
            ))
        })?;

        s.parse::<BigDecimal>().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to parse big decimal: {}", e))
        })
    }

    fn read_string(self, _schema: &Schema) -> Result<String, Self::Error> {
        self.parser
            .next_str()
            .map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Failed to read string: {}", e))
            })
            .map(|s| s.to_string())
    }

    fn read_blob(self, _schema: &Schema) -> Result<ByteBuffer, Self::Error> {
        let s = self.parser.next_str().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read blob string: {}", e))
        })?;

        // TODO: Add base64 decoding
        Ok(ByteBuffer::from(s.as_bytes()))
    }

    fn read_timestamp(self, _schema: &Schema) -> Result<Instant, Self::Error> {
        let s = self.parser.next_str().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read timestamp string: {}", e))
        })?;

        // TODO: timestampFormat handling
        Instant::from_utf8(s.as_bytes()).map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to parse timestamp: {}", e))
        })
    }

    fn read_document(
        self,
        _schema: &Schema,
    ) -> Result<Box<dyn smithy4rs_core::schema::Document>, Self::Error> {
        todo!("Support deserialization of documents")
    }

    fn read_null(self) -> Result<(), Self::Error> {
        self.parser
            .next_null()
            .map_err(|e| JsonSerdeError::DeserializationError(format!("Expected null: {}", e)))
    }

    fn read_struct(self, _schema: &Schema) -> Result<Self::StructReader, Self::Error> {
        Ok(JsonStructReader {
            de: self,
            started: false,
        })
    }

    fn read_list(self, _schema: &Schema) -> Result<Self::ListReader, Self::Error> {
        Ok(JsonListReader {
            de: self,
            started: false,
        })
    }

    fn read_map(self, _schema: &Schema) -> Result<Self::MapReader, Self::Error> {
        Ok(JsonMapReader {
            de: self,
            started: false,
        })
    }

    fn is_null(&mut self) -> bool {
        matches!(self.parser.peek(), Ok(jiter::Peek::Null))
    }
}

// ============================================================================
// StructReader Implementation
// ============================================================================

impl<'de> StructReader<'de> for JsonStructReader<'de, '_> {
    type Error = JsonSerdeError;

    fn read_member<'a>(&mut self, schema: &'a Schema) -> Result<Option<&'a Schema>, Self::Error> {
        loop {
            let maybe_key = if !self.started {
                self.started = true;
                self.de.parser.next_object().map_err(|e| {
                    JsonSerdeError::DeserializationError(format!("Expected object start: {}", e))
                })?
            } else {
                self.de.parser.next_key().map_err(|e| {
                    JsonSerdeError::DeserializationError(format!(
                        "Failed to read object key: {}",
                        e
                    ))
                })?
            };

            match maybe_key {
                Some(key) => {
                    if let Some(member_schema) = schema.get_member(key) {
                        return Ok(Some(member_schema));
                    }
                    // Unknown key — skip the value
                    self.de.parser.next_skip().map_err(|e| {
                        JsonSerdeError::DeserializationError(format!("Failed to skip value: {}", e))
                    })?;
                }
                None => return Ok(None),
            }
        }
    }

    fn read_value<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<T, Self::Error> {
        T::deserialize_with_schema(schema, &mut *self.de)
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.de.parser.next_skip().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to skip value: {}", e))
        })
    }
}

// ============================================================================
// ListReader Implementation
// ============================================================================

impl<'de> ListReader<'de> for JsonListReader<'de, '_> {
    type Error = JsonSerdeError;

    fn read_element<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<Option<T>, Self::Error> {
        let has_element = if !self.started {
            self.started = true;
            self.de
                .parser
                .next_array()
                .map_err(|e| {
                    JsonSerdeError::DeserializationError(format!("Expected array start: {}", e))
                })?
                .is_some()
        } else {
            self.de
                .parser
                .array_step()
                .map_err(|e| {
                    JsonSerdeError::DeserializationError(format!("Failed to advance array: {}", e))
                })?
                .is_some()
        };

        if !has_element {
            return Ok(None);
        }

        T::deserialize_with_schema(schema, &mut *self.de).map(Some)
    }
}

// ============================================================================
// MapReader Implementation
// ============================================================================

impl<'de> MapReader<'de> for JsonMapReader<'de, '_> {
    type Error = JsonSerdeError;

    fn read_key(&mut self) -> Result<Option<String>, Self::Error> {
        let maybe_key = if !self.started {
            self.started = true;
            self.de.parser.next_object().map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Expected object start: {}", e))
            })?
        } else {
            self.de.parser.next_key().map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Failed to read map key: {}", e))
            })?
        };

        Ok(maybe_key.map(|s| s.to_string()))
    }

    fn read_value<V: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<V, Self::Error> {
        V::deserialize_with_schema(schema, &mut *self.de)
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.de.parser.next_skip().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to skip value: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use smithy4rs_core::prelude::*;

    use super::*;

    #[test]
    fn test_read_primitives() {
        let mut de = JsonDeserializer::new(b"true");
        assert!((&mut de).read_bool(&BOOLEAN).unwrap());

        let mut de = JsonDeserializer::new(b"42");
        assert_eq!((&mut de).read_integer(&INTEGER).unwrap(), 42);

        let mut de = JsonDeserializer::new(b"1.234");
        assert!(((&mut de).read_float(&FLOAT).unwrap() - 1.234).abs() < 0.001);

        let mut de = JsonDeserializer::new(b"\"hello\"");
        assert_eq!((&mut de).read_string(&STRING).unwrap(), "hello");
    }

    #[test]
    fn test_is_null() {
        let mut de = JsonDeserializer::new(b"null");
        assert!((&mut de).is_null());

        let mut de = JsonDeserializer::new(b"42");
        assert!(!(&mut de).is_null());

        let mut de = JsonDeserializer::new(b"\"string\"");
        assert!(!(&mut de).is_null());
    }
}
