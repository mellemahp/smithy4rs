use smithy4rs_core::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    schema::Schema,
    serde::deserializers::{
        DeserializeWithSchema, Deserializer, ListReader, MapReader, StructReader,
    },
};

use crate::errors::JsonSerdeError;

/// A JSON deserializer that uses jiter.
///
/// This deserializer reads JSON data from a byte slice and uses schemas
/// to guide the deserialization.
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
///
/// Iterates through JSON object keys and values. The `'de` lifetime is the
/// input data lifetime.
///
/// Callers must alternate `read_member()` and `read_value()`/`skip_value()` calls.
pub struct JsonStructReader<'de> {
    parser: jiter::Jiter<'de>,
    started: bool,
}

/// Reader for JSON list elements.
///
/// Iterates through JSON array elements. The `'de` lifetime is the input data
/// lifetime.
pub struct JsonListReader<'de> {
    parser: jiter::Jiter<'de>,
    started: bool,
}

/// Reader for JSON map entries.
///
/// Iterates through JSON object key-value pairs as map entries. The `'de`
/// lifetime is the input data lifetime.
///
/// Callers must alternate `read_key()` and `read_value()`/`skip_value()` calls.
pub struct JsonMapReader<'de> {
    parser: jiter::Jiter<'de>,
    started: bool,
}

// ============================================================================
// Deserializer Implementation
// ============================================================================

impl<'de> Deserializer<'de> for JsonDeserializer<'de> {
    type Error = JsonSerdeError;
    type StructReader = JsonStructReader<'de>;
    type ListReader = JsonListReader<'de>;
    type MapReader = JsonMapReader<'de>;

    fn read_bool(mut self, _schema: &Schema) -> Result<bool, Self::Error> {
        self.parser.next_bool().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read bool: {}", e))
        })
    }

    fn read_byte(mut self, _schema: &Schema) -> Result<i8, Self::Error> {
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

    fn read_short(mut self, _schema: &Schema) -> Result<i16, Self::Error> {
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

    fn read_integer(mut self, _schema: &Schema) -> Result<i32, Self::Error> {
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

    fn read_long(mut self, _schema: &Schema) -> Result<i64, Self::Error> {
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

    fn read_float(mut self, _schema: &Schema) -> Result<f32, Self::Error> {
        self.parser
            .next_float()
            .map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Failed to read float: {}", e))
            })
            .map(|v| v as f32)
    }

    fn read_double(mut self, _schema: &Schema) -> Result<f64, Self::Error> {
        self.parser.next_float().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read double: {}", e))
        })
    }

    fn read_big_integer(mut self, _schema: &Schema) -> Result<BigInt, Self::Error> {
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

    fn read_big_decimal(mut self, _schema: &Schema) -> Result<BigDecimal, Self::Error> {
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

    fn read_string(mut self, _schema: &Schema) -> Result<String, Self::Error> {
        self.parser
            .next_str()
            .map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Failed to read string: {}", e))
            })
            .map(|s| s.to_string())
    }

    fn read_blob(mut self, _schema: &Schema) -> Result<ByteBuffer, Self::Error> {
        // Blobs in JSON are base64-encoded strings
        let s = self.parser.next_str().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read blob string: {}", e))
        })?;

        // For now, just convert the string to bytes
        // TODO: Add base64 decoding
        Ok(ByteBuffer::from(s.as_bytes()))
    }

    fn read_timestamp(mut self, _schema: &Schema) -> Result<Instant, Self::Error> {
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

    fn read_struct(self, _schema: &Schema) -> Result<Self::StructReader, Self::Error> {
        Ok(JsonStructReader {
            parser: self.parser,
            started: false,
        })
    }

    fn read_list(self, _schema: &Schema) -> Result<Self::ListReader, Self::Error> {
        Ok(JsonListReader {
            parser: self.parser,
            started: false,
        })
    }

    fn read_map(self, _schema: &Schema) -> Result<Self::MapReader, Self::Error> {
        Ok(JsonMapReader {
            parser: self.parser,
            started: false,
        })
    }

    fn is_null(&mut self) -> bool {
        matches!(self.parser.peek(), Ok(jiter::Peek::Null))
    }

    fn read_null(mut self) -> Result<(), Self::Error> {
        self.parser
            .next_null()
            .map_err(|e| JsonSerdeError::DeserializationError(format!("Expected null: {}", e)))
    }
}

// ============================================================================
// StructReader Implementation
// ============================================================================

impl<'de> StructReader<'de> for JsonStructReader<'de> {
    type Error = JsonSerdeError;

    fn read_member<'a>(&mut self, schema: &'a Schema) -> Result<Option<&'a Schema>, Self::Error> {
        loop {
            let maybe_key = if !self.started {
                self.started = true;
                self.parser.next_object().map_err(|e| {
                    JsonSerdeError::DeserializationError(format!("Expected object start: {}", e))
                })?
            } else {
                self.parser.next_key().map_err(|e| {
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
                    // Unknown key — skip the value internally
                    self.parser.next_skip().map_err(|e| {
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
        // Create a temporary deserializer that borrows our parser
        // This is a bit awkward but necessary since DeserializeWithSchema expects a Deserializer
        let de = JsonDeserializerRef { parser: &mut self.parser };
        T::deserialize_with_schema(schema, de)
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.parser.next_skip().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to skip value: {}", e))
        })
    }
}

// ============================================================================
// ListReader Implementation
// ============================================================================

impl<'de> ListReader<'de> for JsonListReader<'de> {
    type Error = JsonSerdeError;

    fn read_element<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<Option<T>, Self::Error> {
        let has_element = if !self.started {
            self.started = true;
            self.parser
                .next_array()
                .map_err(|e| {
                    JsonSerdeError::DeserializationError(format!("Expected array start: {}", e))
                })?
                .is_some()
        } else {
            self.parser
                .array_step()
                .map_err(|e| {
                    JsonSerdeError::DeserializationError(format!("Failed to advance array: {}", e))
                })?
                .is_some()
        };

        if !has_element {
            return Ok(None);
        }

        let de = JsonDeserializerRef { parser: &mut self.parser };
        T::deserialize_with_schema(schema, de).map(Some)
    }
}

// ============================================================================
// MapReader Implementation
// ============================================================================

impl<'de> MapReader<'de> for JsonMapReader<'de> {
    type Error = JsonSerdeError;

    fn read_key(&mut self) -> Result<Option<String>, Self::Error> {
        let maybe_key = if !self.started {
            self.started = true;
            self.parser.next_object().map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Expected object start: {}", e))
            })?
        } else {
            self.parser.next_key().map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Failed to read map key: {}", e))
            })?
        };

        Ok(maybe_key.map(|s| s.to_string()))
    }

    fn read_value<V: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<V, Self::Error> {
        let de = JsonDeserializerRef { parser: &mut self.parser };
        V::deserialize_with_schema(schema, de)
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.parser.next_skip().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to skip value: {}", e))
        })
    }
}

// ============================================================================
// Internal: Deserializer that borrows a parser (for nested deserialization)
// ============================================================================

struct JsonDeserializerRef<'de, 'a> {
    parser: &'a mut jiter::Jiter<'de>,
}

impl<'de, 'a> Deserializer<'de> for JsonDeserializerRef<'de, 'a> {
    type Error = JsonSerdeError;
    type StructReader = JsonStructReaderRef<'de, 'a>;
    type ListReader = JsonListReaderRef<'de, 'a>;
    type MapReader = JsonMapReaderRef<'de, 'a>;

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
        Ok(ByteBuffer::from(s.as_bytes()))
    }

    fn read_timestamp(self, _schema: &Schema) -> Result<Instant, Self::Error> {
        let s = self.parser.next_str().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to read timestamp string: {}", e))
        })?;
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

    fn read_struct(self, _schema: &Schema) -> Result<Self::StructReader, Self::Error> {
        Ok(JsonStructReaderRef {
            parser: self.parser,
            started: false,
        })
    }

    fn read_list(self, _schema: &Schema) -> Result<Self::ListReader, Self::Error> {
        Ok(JsonListReaderRef {
            parser: self.parser,
            started: false,
        })
    }

    fn read_map(self, _schema: &Schema) -> Result<Self::MapReader, Self::Error> {
        Ok(JsonMapReaderRef {
            parser: self.parser,
            started: false,
        })
    }

    fn is_null(&mut self) -> bool {
        matches!(self.parser.peek(), Ok(jiter::Peek::Null))
    }

    fn read_null(self) -> Result<(), Self::Error> {
        self.parser
            .next_null()
            .map_err(|e| JsonSerdeError::DeserializationError(format!("Expected null: {}", e)))
    }
}

// ============================================================================
// Ref-based readers for nested deserialization
// ============================================================================

struct JsonStructReaderRef<'de, 'a> {
    parser: &'a mut jiter::Jiter<'de>,
    started: bool,
}

struct JsonListReaderRef<'de, 'a> {
    parser: &'a mut jiter::Jiter<'de>,
    started: bool,
}

struct JsonMapReaderRef<'de, 'a> {
    parser: &'a mut jiter::Jiter<'de>,
    started: bool,
}

impl<'de> StructReader<'de> for JsonStructReaderRef<'de, '_> {
    type Error = JsonSerdeError;

    fn read_member<'a>(&mut self, schema: &'a Schema) -> Result<Option<&'a Schema>, Self::Error> {
        loop {
            let maybe_key = if !self.started {
                self.started = true;
                self.parser.next_object().map_err(|e| {
                    JsonSerdeError::DeserializationError(format!("Expected object start: {}", e))
                })?
            } else {
                self.parser.next_key().map_err(|e| {
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
                    // Unknown key — skip the value internally
                    self.parser.next_skip().map_err(|e| {
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
        let de = JsonDeserializerRef { parser: self.parser };
        T::deserialize_with_schema(schema, de)
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.parser.next_skip().map_err(|e| {
            JsonSerdeError::DeserializationError(format!("Failed to skip value: {}", e))
        })
    }
}

impl<'de> ListReader<'de> for JsonListReaderRef<'de, '_> {
    type Error = JsonSerdeError;

    fn read_element<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<Option<T>, Self::Error> {
        let has_element = if !self.started {
            self.started = true;
            self.parser
                .next_array()
                .map_err(|e| {
                    JsonSerdeError::DeserializationError(format!("Expected array start: {}", e))
                })?
                .is_some()
        } else {
            self.parser
                .array_step()
                .map_err(|e| {
                    JsonSerdeError::DeserializationError(format!("Failed to advance array: {}", e))
                })?
                .is_some()
        };

        if !has_element {
            return Ok(None);
        }

        let de = JsonDeserializerRef { parser: self.parser };
        T::deserialize_with_schema(schema, de).map(Some)
    }
}

impl<'de> MapReader<'de> for JsonMapReaderRef<'de, '_> {
    type Error = JsonSerdeError;

    fn read_key(&mut self) -> Result<Option<String>, Self::Error> {
        let maybe_key = if !self.started {
            self.started = true;
            self.parser.next_object().map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Expected object start: {}", e))
            })?
        } else {
            self.parser.next_key().map_err(|e| {
                JsonSerdeError::DeserializationError(format!("Failed to read map key: {}", e))
            })?
        };

        Ok(maybe_key.map(|s| s.to_string()))
    }

    fn read_value<V: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<V, Self::Error> {
        let de = JsonDeserializerRef { parser: self.parser };
        V::deserialize_with_schema(schema, de)
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.parser.next_skip().map_err(|e| {
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
        let de = JsonDeserializer::new(b"true");
        assert!(de.read_bool(&BOOLEAN).unwrap());

        let de = JsonDeserializer::new(b"42");
        assert_eq!(de.read_integer(&INTEGER).unwrap(), 42);

        let de = JsonDeserializer::new(b"1.234");
        assert!((de.read_float(&FLOAT).unwrap() - 1.234).abs() < 0.001);

        let de = JsonDeserializer::new(b"\"hello\"");
        assert_eq!(de.read_string(&STRING).unwrap(), "hello");
    }

    #[test]
    fn test_is_null() {
        let mut de = JsonDeserializer::new(b"null");
        assert!(de.is_null());

        let mut de = JsonDeserializer::new(b"42");
        assert!(!de.is_null());

        let mut de = JsonDeserializer::new(b"\"string\"");
        assert!(!de.is_null());
    }
}
