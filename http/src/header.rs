//! HTTP header serialization and deserialization.
//!
//! Headers support scalar values, lists (comma-separated), and blobs (base64).
//! Timestamps default to `http-date` (RFC 7231) format.

use std::fmt::Write;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use smithy4rs_core::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    schema::Schema,
    serde::{de, never::Never, se},
};

use crate::error::HttpBindingError;

// --- Serialization ---

/// Serializer for HTTP header values.
///
/// Converts values to strings for HTTP headers.
/// - Scalars are converted to their string representation
/// - Lists are comma-separated
/// - Blobs are base64-encoded
/// - Timestamps default to HTTP-date format (RFC 7231)
pub struct HeaderSerializer<'a> {
    output: &'a mut String,
}

impl<'a> HeaderSerializer<'a> {
    /// Create a new header serializer that writes to the given output string.
    #[inline]
    pub fn new(output: &'a mut String) -> Self {
        Self { output }
    }
}

impl<'a> se::Serializer for HeaderSerializer<'a> {
    type Error = HttpBindingError;
    type Ok = ();
    type ListWriter = HeaderListSerializer<'a>;
    type MapWriter = Never<Self::Error>;
    type StructWriter = Never<Self::Error>;

    fn write_list(self, _schema: &Schema, _len: usize) -> Result<Self::ListWriter, Self::Error> {
        Ok(HeaderListSerializer {
            output: self.output,
        })
    }

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

    #[inline]
    fn write_big_integer(self, _schema: &Schema, value: &BigInt) -> Result<Self::Ok, Self::Error> {
        let _ = write!(self.output, "{value}");
        Ok(())
    }

    #[inline]
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
        // TODO: Check @mediaType trait - if present, base64 encode
        self.output.push_str(value);
        Ok(())
    }

    #[inline]
    fn write_blob(self, _schema: &Schema, value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        BASE64.encode_string(value.as_bytes(), self.output);
        Ok(())
    }

    fn write_timestamp(self, _schema: &Schema, _value: &Instant) -> Result<Self::Ok, Self::Error> {
        // TODO: Check @timestampFormat trait, default to HTTP_DATE (RFC 7231)
        Err(HttpBindingError::new(
            "timestamp header serialization is not yet implemented",
        ))
    }

    fn write_null(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        // Null headers are simply omitted
        Ok(())
    }

    fn skip(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// --- List Serialization (comma-separated) ---

/// Serializer for list values in HTTP headers.
///
/// Builds comma-separated output incrementally (avoids collect-then-join).
pub struct HeaderListSerializer<'a> {
    output: &'a mut String,
}

impl se::ListWriter for HeaderListSerializer<'_> {
    type Error = HttpBindingError;
    type Ok = ();

    fn write_element<T: se::SerializeWithSchema>(
        &mut self,
        schema: &Schema,
        value: &T,
    ) -> Result<(), Self::Error> {
        let start_len = self.output.len();
        if !self.output.is_empty() {
            self.output.push_str(", ");
        }
        let before_element = self.output.len();
        value.serialize_with_schema(schema, HeaderSerializer::new(self.output))?;

        let written = &self.output.as_bytes()[before_element..];
        if written.is_empty() {
            // Nothing written — remove the separator we added.
            if before_element > start_len {
                self.output.truncate(start_len);
            }
        } else {
            // Per RFC 9110 §5.6.4, list elements containing comma or quote
            // must be wrapped in a quoted-string.
            let mut needs_quoting = false;
            let mut needs_escaping = false;
            for &b in written {
                if b == b',' {
                    needs_quoting = true;
                } else if b == b'"' || b == b'\\' {
                    needs_quoting = true;
                    needs_escaping = true;
                    break;
                }
            }
            if needs_quoting && !needs_escaping {
                // Fast path: just wrap in quotes, no internal escaping needed.
                self.output.insert(before_element, '"');
                self.output.push('"');
            } else if needs_escaping {
                // Slow path: rebuild with escaping (rare — quotes/backslashes in header values).
                let raw = self.output[before_element..].to_string();
                self.output.truncate(before_element);
                self.output.push('"');
                for ch in raw.bytes() {
                    if ch == b'"' || ch == b'\\' {
                        self.output.push('\\');
                    }
                    self.output.push(ch as char);
                }
                self.output.push('"');
            }
        }
        Ok(())
    }

    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// --- Prefix Headers Serialization (@httpPrefixHeaders) ---

/// Serializer for `@httpPrefixHeaders` - a map where each entry becomes a header
/// with the key prefixed by a given string.
///
/// Writes directly to the output vec instead of using a callback.
pub struct PrefixHeadersSerializer<'a> {
    prefix: &'a str,
    output: &'a mut Vec<(String, String)>,
}

impl<'a> PrefixHeadersSerializer<'a> {
    /// Create a new prefix headers serializer.
    ///
    /// Each map entry produces a header with name = `prefix` + `map_key`.
    pub fn new(prefix: &'a str, output: &'a mut Vec<(String, String)>) -> Self {
        Self { prefix, output }
    }
}

impl<'a> se::Serializer for PrefixHeadersSerializer<'a> {
    type Error = HttpBindingError;
    type Ok = ();
    type ListWriter = Never<Self::Error>;
    type MapWriter = PrefixHeadersMapSerializer<'a>;
    type StructWriter = Never<Self::Error>;

    fn write_map(self, _schema: &Schema, _len: usize) -> Result<Self::MapWriter, Self::Error> {
        Ok(PrefixHeadersMapSerializer {
            prefix: self.prefix,
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

/// Map serializer for prefix headers.
pub struct PrefixHeadersMapSerializer<'a> {
    prefix: &'a str,
    output: &'a mut Vec<(String, String)>,
    // Reusable scratch buffer for serializing map keys.
    key_scratch: String,
}

impl se::MapWriter for PrefixHeadersMapSerializer<'_> {
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
        // Serialize the key (reuses scratch buffer across entries)
        self.key_scratch.clear();
        key.serialize_with_schema(key_schema, HeaderSerializer::new(&mut self.key_scratch))?;

        let mut header_value = String::new();
        value.serialize_with_schema(value_schema, HeaderSerializer::new(&mut header_value))?;

        if !header_value.is_empty() {
            let mut header_name = String::with_capacity(self.prefix.len() + self.key_scratch.len());
            header_name.push_str(self.prefix);
            header_name.push_str(&self.key_scratch);
            self.output.push((header_name, header_value));
        }
        Ok(())
    }

    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// --- Deserialization ---

/// Deserializer for HTTP header values.
///
/// Parses values from HTTP header strings.
/// - Lists are parsed from comma-separated values
/// - Blobs are base64-decoded
/// - Timestamps default to HTTP-date format (RFC 7231)
pub struct HeaderDeserializer<S: AsRef<str>> {
    input: S,
}

impl<S: AsRef<str>> HeaderDeserializer<S> {
    /// Create a new header deserializer.
    #[inline]
    pub fn new(input: S) -> Self {
        Self { input }
    }
}

impl<'de, S: AsRef<str>> de::Deserializer<'de> for HeaderDeserializer<S> {
    type Error = HttpBindingError;
    type StructReader = Never<Self::Error>;
    type ListReader = HeaderListDeserializer<S>;
    type MapReader = Never<Self::Error>;

    fn read_list(self, _schema: &Schema) -> Result<Self::ListReader, Self::Error> {
        Ok(HeaderListDeserializer {
            input: self.input,
            position: 0,
        })
    }

    #[inline]
    fn read_bool(self, _schema: &Schema) -> Result<bool, Self::Error> {
        match self.input.as_ref() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(HttpBindingError::new(format!(
                "invalid boolean header value: '{}'",
                self.input.as_ref()
            ))),
        }
    }

    #[inline]
    fn read_byte(self, _schema: &Schema) -> Result<i8, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid byte header value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_short(self, _schema: &Schema) -> Result<i16, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid short header value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_integer(self, _schema: &Schema) -> Result<i32, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid integer header value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_long(self, _schema: &Schema) -> Result<i64, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid long header value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_float(self, _schema: &Schema) -> Result<f32, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid float header value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_double(self, _schema: &Schema) -> Result<f64, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid double header value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    fn read_big_integer(self, _schema: &Schema) -> Result<BigInt, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid bigInteger header value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    fn read_big_decimal(self, _schema: &Schema) -> Result<BigDecimal, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid bigDecimal header value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_string(self, _schema: &Schema) -> Result<String, Self::Error> {
        // TODO: Check @mediaType trait - if present, base64 decode
        Ok(self.input.as_ref().to_string())
    }

    fn read_blob(self, _schema: &Schema) -> Result<ByteBuffer, Self::Error> {
        let bytes = BASE64
            .decode(self.input.as_ref())
            .map_err(|e| HttpBindingError::new(format!("invalid base64 header value: {e}")))?;
        Ok(ByteBuffer::from(bytes))
    }

    fn read_timestamp(self, _schema: &Schema) -> Result<Instant, Self::Error> {
        // TODO: Check @timestampFormat trait, default to HTTP_DATE (RFC 7231)
        Err(HttpBindingError::new(
            "timestamp header deserialization is not yet implemented",
        ))
    }

    fn read_null(self) -> Result<(), Self::Error> {
        if self.input.as_ref().is_empty() {
            Ok(())
        } else {
            Err(HttpBindingError::new(
                "expected empty header value for null",
            ))
        }
    }

    fn is_null(&mut self) -> bool {
        self.input.as_ref().is_empty()
    }
}

// --- List Deserialization ---

/// Deserializer for list values from comma-separated header values.
///
/// Lazily yields trimmed, non-empty elements — no pre-allocated Vec of ranges.
pub struct HeaderListDeserializer<S> {
    input: S,
    // Byte position into the input string for the next element.
    position: usize,
}

impl<'de, S: AsRef<str>> de::ListReader<'de> for HeaderListDeserializer<S> {
    type Error = HttpBindingError;

    fn read_element<T: de::DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<Option<T>, Self::Error> {
        loop {
            let remaining = &self.input.as_ref()[self.position..];
            let bytes = remaining.as_bytes();
            if bytes.is_empty() {
                return Ok(None);
            }

            // Quote-aware comma splitting per RFC 9110 §5.6.4.
            // Check first non-whitespace byte to detect quoted strings.
            let ws_len = bytes
                .iter()
                .position(|&b| b != b' ' && b != b'\t')
                .unwrap_or(bytes.len());

            if bytes.get(ws_len) == Some(&b'"') {
                // Quoted string — scan for closing quote, respecting backslash escapes.
                let qbytes = &bytes[ws_len..];
                let mut i = 1; // skip opening quote
                while i < qbytes.len() {
                    if qbytes[i] == b'\\' {
                        i += 2; // skip escaped char
                    } else if qbytes[i] == b'"' {
                        i += 1; // past closing quote
                        break;
                    } else {
                        i += 1;
                    }
                }
                let quoted_end = ws_len + i;
                let inner = &remaining[ws_len + 1..ws_len + i.saturating_sub(1)];

                // Advance past comma (if any) after the quoted value.
                let after = &bytes[quoted_end..];
                self.position += quoted_end
                    + after
                        .iter()
                        .position(|&b| b == b',')
                        .map_or(after.len(), |idx| idx + 1);

                // Unescape: only allocate if backslash escapes are present.
                if inner.contains('\\') {
                    let mut unescaped = String::with_capacity(inner.len());
                    let ibytes = inner.as_bytes();
                    let mut j = 0;
                    while j < ibytes.len() {
                        if ibytes[j] == b'\\' && j + 1 < ibytes.len() {
                            j += 1;
                        }
                        unescaped.push(ibytes[j] as char);
                        j += 1;
                    }
                    return T::deserialize_with_schema(schema, HeaderDeserializer::new(unescaped))
                        .map(Some);
                }
                return T::deserialize_with_schema(schema, HeaderDeserializer::new(inner))
                    .map(Some);
            }

            // Unquoted — split on comma.
            let (element, rest_len) = bytes
                .iter()
                .position(|&b| b == b',')
                .map_or((remaining, 0), |idx| {
                    (&remaining[..idx], remaining.len() - idx - 1)
                });

            self.position = self.input.as_ref().len() - rest_len;

            let trimmed = element.trim();
            if !trimmed.is_empty() {
                return T::deserialize_with_schema(schema, HeaderDeserializer::new(trimmed))
                    .map(Some);
            }
            // Empty element after trim — skip and try the next one
        }
    }

    fn size_hint(&self) -> Option<usize> {
        None
    }
}

#[cfg(test)]
mod tests {
    use smithy4rs_core::{
        schema::prelude::{BLOB, BOOLEAN, DOUBLE, FLOAT, INTEGER, LONG, STRING},
        serde::{
            de::{Deserializer, ListReader},
            se::{ListWriter, Serializer},
        },
    };

    use super::*;

    // --- HeaderSerializer ---

    #[test]
    fn serialize_boolean() {
        let mut out = String::new();
        HeaderSerializer::new(&mut out)
            .write_boolean(&BOOLEAN, true)
            .unwrap();
        assert_eq!(out, "true");

        out.clear();
        HeaderSerializer::new(&mut out)
            .write_boolean(&BOOLEAN, false)
            .unwrap();
        assert_eq!(out, "false");
    }

    #[test]
    fn serialize_integer() {
        let mut out = String::new();
        HeaderSerializer::new(&mut out)
            .write_integer(&INTEGER, 42)
            .unwrap();
        assert_eq!(out, "42");
    }

    #[test]
    fn serialize_long() {
        let mut out = String::new();
        HeaderSerializer::new(&mut out)
            .write_long(&LONG, 9_999_999_999)
            .unwrap();
        assert_eq!(out, "9999999999");
    }

    #[test]
    fn serialize_float() {
        let mut out = String::new();
        HeaderSerializer::new(&mut out)
            .write_float(&FLOAT, 3.14)
            .unwrap();
        assert!(out.starts_with("3.14"));
    }

    #[test]
    fn serialize_double() {
        let mut out = String::new();
        HeaderSerializer::new(&mut out)
            .write_double(&DOUBLE, 2.718281828)
            .unwrap();
        assert!(out.starts_with("2.71828"));
    }

    #[test]
    fn serialize_string() {
        let mut out = String::new();
        HeaderSerializer::new(&mut out)
            .write_string(&STRING, "hello")
            .unwrap();
        assert_eq!(out, "hello");
    }

    #[test]
    fn serialize_blob_base64() {
        let mut out = String::new();
        let blob = ByteBuffer::from(vec![0, 1, 2, 3]);
        HeaderSerializer::new(&mut out)
            .write_blob(&BLOB, &blob)
            .unwrap();
        assert_eq!(out, "AAECAw==");
    }

    #[test]
    fn serialize_null_empty() {
        let mut out = String::new();
        HeaderSerializer::new(&mut out).write_null(&STRING).unwrap();
        assert!(out.is_empty());
    }

    // --- HeaderListSerializer ---

    #[test]
    fn serialize_list_simple() {
        let mut out = String::new();
        let mut w = HeaderSerializer::new(&mut out)
            .write_list(&STRING, 3)
            .unwrap();
        w.write_element(&STRING, &"a".to_string()).unwrap();
        w.write_element(&STRING, &"b".to_string()).unwrap();
        w.write_element(&STRING, &"c".to_string()).unwrap();
        w.end(&STRING).unwrap();
        assert_eq!(out, "a, b, c");
    }

    #[test]
    fn serialize_list_value_with_comma() {
        let mut out = String::new();
        let mut w = HeaderSerializer::new(&mut out)
            .write_list(&STRING, 1)
            .unwrap();
        w.write_element(&STRING, &"a,b".to_string()).unwrap();
        w.end(&STRING).unwrap();
        assert_eq!(out, "\"a,b\"");
    }

    #[test]
    fn serialize_list_value_with_quote() {
        let mut out = String::new();
        let mut w = HeaderSerializer::new(&mut out)
            .write_list(&STRING, 1)
            .unwrap();
        w.write_element(&STRING, &"say\"hi".to_string()).unwrap();
        w.end(&STRING).unwrap();
        assert_eq!(out, "\"say\\\"hi\"");
    }

    #[test]
    fn serialize_list_value_with_backslash() {
        let mut out = String::new();
        let mut w = HeaderSerializer::new(&mut out)
            .write_list(&STRING, 1)
            .unwrap();
        w.write_element(&STRING, &"a\\b".to_string()).unwrap();
        w.end(&STRING).unwrap();
        assert_eq!(out, "\"a\\\\b\"");
    }

    #[test]
    fn serialize_list_skips_empty_element() {
        let mut out = String::new();
        let mut w = HeaderSerializer::new(&mut out)
            .write_list(&STRING, 2)
            .unwrap();
        w.write_element(&STRING, &"a".to_string()).unwrap();
        // write_null produces empty string which gets removed
        HeaderSerializer::new(&mut String::new())
            .write_null(&STRING)
            .unwrap();
        w.write_element(&STRING, &"b".to_string()).unwrap();
        w.end(&STRING).unwrap();
        assert_eq!(out, "a, b");
    }

    // --- HeaderDeserializer ---

    #[test]
    fn deserialize_boolean() {
        let val: bool = HeaderDeserializer::new("true").read_bool(&BOOLEAN).unwrap();
        assert!(val);

        let val: bool = HeaderDeserializer::new("false")
            .read_bool(&BOOLEAN)
            .unwrap();
        assert!(!val);
    }

    #[test]
    fn deserialize_integer() {
        let val: i32 = HeaderDeserializer::new("42")
            .read_integer(&INTEGER)
            .unwrap();
        assert_eq!(val, 42);
    }

    #[test]
    fn deserialize_float() {
        let val: f32 = HeaderDeserializer::new("3.14").read_float(&FLOAT).unwrap();
        assert!((val - 3.14).abs() < 0.001);
    }

    #[test]
    fn deserialize_string() {
        let val: String = HeaderDeserializer::new("hello")
            .read_string(&STRING)
            .unwrap();
        assert_eq!(val, "hello");
    }

    #[test]
    fn deserialize_blob_base64() {
        let val: ByteBuffer = HeaderDeserializer::new("AAECAw==")
            .read_blob(&BLOB)
            .unwrap();
        assert_eq!(val.as_bytes(), &[0, 1, 2, 3]);
    }

    #[test]
    fn deserialize_invalid_integer() {
        assert!(
            HeaderDeserializer::new("abc")
                .read_integer(&INTEGER)
                .is_err()
        );
    }

    #[test]
    fn deserialize_null_empty() {
        HeaderDeserializer::new("").read_null().unwrap();
    }

    #[test]
    fn deserialize_null_non_empty_error() {
        assert!(HeaderDeserializer::new("something").read_null().is_err());
    }

    // --- HeaderListDeserializer ---

    fn read_string_list(input: &str) -> Vec<String> {
        let mut reader = HeaderDeserializer::new(input).read_list(&STRING).unwrap();
        let mut items = Vec::new();
        while let Some(item) = reader.read_element::<String>(&STRING).unwrap() {
            items.push(item);
        }
        items
    }

    #[test]
    fn deserialize_list_simple() {
        assert_eq!(read_string_list("a, b, c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn deserialize_list_quoted_preserves_comma() {
        assert_eq!(
            read_string_list("hello, \"a,b\", value"),
            vec!["hello", "a,b", "value"]
        );
    }

    #[test]
    fn deserialize_list_escaped_quote() {
        assert_eq!(read_string_list(r#""say\"hi""#), vec!["say\"hi"]);
    }

    #[test]
    fn deserialize_list_whitespace_trimming() {
        assert_eq!(read_string_list("  a ,  b  , c  "), vec!["a", "b", "c"]);
    }

    #[test]
    fn deserialize_list_empty_elements_skipped() {
        assert_eq!(read_string_list("a,,b, ,c"), vec!["a", "b", "c"]);
    }
}
