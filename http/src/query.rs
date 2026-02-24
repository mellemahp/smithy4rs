//! HTTP query parameter serialization and deserialization.
//!
//! Query parameters support scalar values and lists (as repeated keys).
//! Timestamps default to `date-time` (ISO 8601) format.
//! Blobs are not supported in query parameters.

use std::fmt::Write;

use smithy4rs_core::{
    BigDecimal, BigInt, Instant,
    schema::Schema,
    serde::{de, never::Never, se},
};

use crate::error::HttpBindingError;

// --- Serialization ---

/// Serializer for HTTP query parameter values.
///
/// Converts values to strings for query string encoding.
/// - Scalars are converted to their string representation
/// - Lists produce multiple values (for repeated keys)
/// - Blobs are not supported
/// - Timestamps default to date-time format (ISO 8601)
pub struct QuerySerializer<'a> {
    output: &'a mut String,
}

impl<'a> QuerySerializer<'a> {
    /// Create a new query serializer that writes to the given output string.
    #[inline]
    pub fn new(output: &'a mut String) -> Self {
        Self { output }
    }
}

impl se::Serializer for QuerySerializer<'_> {
    type Error = HttpBindingError;
    type Ok = ();
    type ListWriter = Never<Self::Error>;
    type MapWriter = Never<Self::Error>;
    type StructWriter = Never<Self::Error>;

    #[inline]
    fn write_boolean(self, _schema: &Schema, value: bool) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(if value { "true" } else { "false" });
        Ok(())
    }

    #[inline]
    fn write_byte(self, _schema: &Schema, value: i8) -> Result<Self::Ok, Self::Error> {
        let mut buf = itoa::Buffer::new();
        self.output.push_str(buf.format(value));
        Ok(())
    }

    #[inline]
    fn write_short(self, _schema: &Schema, value: i16) -> Result<Self::Ok, Self::Error> {
        let mut buf = itoa::Buffer::new();
        self.output.push_str(buf.format(value));
        Ok(())
    }

    #[inline]
    fn write_integer(self, _schema: &Schema, value: i32) -> Result<Self::Ok, Self::Error> {
        let mut buf = itoa::Buffer::new();
        self.output.push_str(buf.format(value));
        Ok(())
    }

    #[inline]
    fn write_long(self, _schema: &Schema, value: i64) -> Result<Self::Ok, Self::Error> {
        let mut buf = itoa::Buffer::new();
        self.output.push_str(buf.format(value));
        Ok(())
    }

    #[inline]
    fn write_float(self, _schema: &Schema, value: f32) -> Result<Self::Ok, Self::Error> {
        let mut buf = ryu::Buffer::new();
        self.output.push_str(buf.format(value));
        Ok(())
    }

    #[inline]
    fn write_double(self, _schema: &Schema, value: f64) -> Result<Self::Ok, Self::Error> {
        let mut buf = ryu::Buffer::new();
        self.output.push_str(buf.format(value));
        Ok(())
    }

    fn write_big_integer(self, _schema: &Schema, value: &BigInt) -> Result<Self::Ok, Self::Error> {
        let _ = write!(self.output, "{value}");
        Ok(())
    }

    fn write_big_decimal(
        self,
        _schema: &Schema,
        value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        let _ = write!(self.output, "{value}");
        Ok(())
    }

    #[inline]
    fn write_string(self, _schema: &Schema, value: &str) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(value);
        Ok(())
    }

    #[inline]
    fn write_null(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn skip(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// --- QueryParams Serialization (@httpQueryParams) ---

/// Serializer for `@httpQueryParams` - a map where each entry becomes a query parameter.
///
/// Supports `Map<String, String>` and `Map<String, List<String>>`.
/// Writes directly to the output vec instead of using a callback.
pub struct QueryParamsSerializer<'a> {
    output: &'a mut Vec<(String, String)>,
}

impl<'a> QueryParamsSerializer<'a> {
    /// Create a new query params serializer that writes to the given output vec.
    pub fn new(output: &'a mut Vec<(String, String)>) -> Self {
        Self { output }
    }
}

impl<'a> se::Serializer for QueryParamsSerializer<'a> {
    type Error = HttpBindingError;
    type Ok = ();
    type ListWriter = Never<Self::Error>;
    type MapWriter = QueryParamsMapSerializer<'a>;
    type StructWriter = Never<Self::Error>;

    fn write_map(self, _schema: &Schema, _len: usize) -> Result<Self::MapWriter, Self::Error> {
        Ok(QueryParamsMapSerializer {
            output: self.output,
            key_scratch: String::new(),
        })
    }

    fn write_null(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn skip(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Map serializer for `@httpQueryParams`.
pub struct QueryParamsMapSerializer<'a> {
    output: &'a mut Vec<(String, String)>,
    // Reusable scratch buffer for serializing map keys.
    key_scratch: String,
}

impl se::MapWriter for QueryParamsMapSerializer<'_> {
    type Error = HttpBindingError;
    type Ok = ();

    fn write_entry<K, V>(
        &mut self,
        key_schema: &Schema,
        value_schema: &Schema,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: se::SerializeWithSchema,
        V: se::SerializeWithSchema,
    {
        // Serialize the key to get the query param name (reuses scratch buffer)
        self.key_scratch.clear();
        key.serialize_with_schema(key_schema, QuerySerializer::new(&mut self.key_scratch))?;

        // Serialize the value — could be a string or a list of strings.
        // Key is borrowed since it may be used multiple times for list values.
        value.serialize_with_schema(
            value_schema,
            QueryParamsValueSerializer {
                key: &self.key_scratch,
                output: &mut *self.output,
            },
        )?;

        Ok(())
    }

    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Serializer for query param values (string or list of strings).
struct QueryParamsValueSerializer<'a> {
    key: &'a str,
    output: &'a mut Vec<(String, String)>,
}

impl<'a> se::Serializer for QueryParamsValueSerializer<'a> {
    type Error = HttpBindingError;
    type Ok = ();
    type ListWriter = QueryParamsListSerializer<'a>;
    type MapWriter = Never<Self::Error>;
    type StructWriter = Never<Self::Error>;

    fn write_list(self, _schema: &Schema, _len: usize) -> Result<Self::ListWriter, Self::Error> {
        Ok(QueryParamsListSerializer {
            key: self.key,
            output: self.output,
        })
    }

    fn write_string(self, _schema: &Schema, value: &str) -> Result<Self::Ok, Self::Error> {
        self.output.push((self.key.to_owned(), value.to_owned()));
        Ok(())
    }

    fn write_null(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn skip(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// List serializer for `@httpQueryParams` list values.
struct QueryParamsListSerializer<'a> {
    key: &'a str,
    output: &'a mut Vec<(String, String)>,
}

impl se::ListWriter for QueryParamsListSerializer<'_> {
    type Error = HttpBindingError;
    type Ok = ();

    fn write_element<T: se::SerializeWithSchema>(
        &mut self,
        schema: &Schema,
        value: &T,
    ) -> Result<(), Self::Error> {
        let mut element_str = String::new();
        value.serialize_with_schema(schema, QuerySerializer::new(&mut element_str))?;
        self.output.push((self.key.to_owned(), element_str));
        Ok(())
    }

    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// --- Deserialization ---

/// Deserializer for HTTP query parameter values.
///
/// Parses values from query string values.
/// - Blobs are not supported
/// - Timestamps default to date-time format (ISO 8601)
pub struct QueryDeserializer<S: AsRef<str>> {
    input: S,
}

impl<S: AsRef<str>> QueryDeserializer<S> {
    /// Create a new query deserializer for a single value.
    #[inline]
    pub fn new(input: S) -> Self {
        Self { input }
    }
}

impl<'de, S: AsRef<str>> de::Deserializer<'de> for QueryDeserializer<S> {
    type Error = HttpBindingError;
    type StructReader = Never<Self::Error>;
    type ListReader = Never<Self::Error>;
    type MapReader = Never<Self::Error>;

    #[inline]
    fn read_bool(self, _schema: &Schema) -> Result<bool, Self::Error> {
        match self.input.as_ref() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(HttpBindingError::new(format!(
                "invalid boolean query value: '{}'",
                self.input.as_ref()
            ))),
        }
    }

    #[inline]
    fn read_byte(self, _schema: &Schema) -> Result<i8, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid byte query value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_short(self, _schema: &Schema) -> Result<i16, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid short query value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_integer(self, _schema: &Schema) -> Result<i32, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid integer query value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_long(self, _schema: &Schema) -> Result<i64, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid long query value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_float(self, _schema: &Schema) -> Result<f32, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid float query value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_double(self, _schema: &Schema) -> Result<f64, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid double query value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    fn read_big_integer(self, _schema: &Schema) -> Result<BigInt, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid bigInteger query value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    fn read_big_decimal(self, _schema: &Schema) -> Result<BigDecimal, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid bigDecimal query value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_string(self, _schema: &Schema) -> Result<String, Self::Error> {
        Ok(self.input.as_ref().to_string())
    }

    fn read_timestamp(self, _schema: &Schema) -> Result<Instant, Self::Error> {
        // TODO: Check @timestampFormat trait, default to DATE_TIME (ISO 8601)
        Err(HttpBindingError::new(
            "timestamp query deserialization is not yet implemented",
        ))
    }

    fn read_null(self) -> Result<(), Self::Error> {
        if self.input.as_ref().is_empty() {
            Ok(())
        } else {
            Err(HttpBindingError::new("expected empty query value for null"))
        }
    }

    fn is_null(&mut self) -> bool {
        self.input.as_ref().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use smithy4rs_core::{
        schema::prelude::{BOOLEAN, INTEGER, STRING},
        serde::{de::Deserializer, se::Serializer},
    };

    use super::*;

    // --- QuerySerializer ---

    #[test]
    fn serialize_string() {
        let mut out = String::new();
        QuerySerializer::new(&mut out)
            .write_string(&STRING, "hello")
            .unwrap();
        assert_eq!(out, "hello");
    }

    #[test]
    fn serialize_integer() {
        let mut out = String::new();
        QuerySerializer::new(&mut out)
            .write_integer(&INTEGER, 42)
            .unwrap();
        assert_eq!(out, "42");
    }

    #[test]
    fn serialize_boolean() {
        let mut out = String::new();
        QuerySerializer::new(&mut out)
            .write_boolean(&BOOLEAN, true)
            .unwrap();
        assert_eq!(out, "true");
    }

    #[test]
    fn serialize_null_empty() {
        let mut out = String::new();
        QuerySerializer::new(&mut out).write_null(&STRING).unwrap();
        assert!(out.is_empty());
    }

    // --- QueryDeserializer ---

    #[test]
    fn deserialize_string() {
        let val: String = QueryDeserializer::new("hello")
            .read_string(&STRING)
            .unwrap();
        assert_eq!(val, "hello");
    }

    #[test]
    fn deserialize_integer() {
        let val: i32 = QueryDeserializer::new("42").read_integer(&INTEGER).unwrap();
        assert_eq!(val, 42);
    }

    #[test]
    fn deserialize_boolean() {
        let val: bool = QueryDeserializer::new("true").read_bool(&BOOLEAN).unwrap();
        assert!(val);
    }

    #[test]
    fn deserialize_null_empty_ok() {
        QueryDeserializer::new("").read_null().unwrap();
    }

    #[test]
    fn deserialize_null_non_empty_error() {
        assert!(QueryDeserializer::new("something").read_null().is_err());
    }

    #[test]
    fn deserialize_invalid_integer() {
        assert!(
            QueryDeserializer::new("abc")
                .read_integer(&INTEGER)
                .is_err()
        );
    }

    #[test]
    fn is_null_check() {
        assert!(QueryDeserializer::new("").is_null());
        assert!(!QueryDeserializer::new("x").is_null());
    }
}
