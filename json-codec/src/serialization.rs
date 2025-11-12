use smithy4rs_core::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    schema::{Document, SchemaRef},
    serde::serializers::{
        ListSerializer, MapSerializer, SerializeWithSchema, Serializer, StructSerializer,
    },
};

use crate::errors::JsonSerdeError;

// Pre-computed escape lookup table for fast string escaping
static ESCAPE: [u8; 256] = {
    const NONE: u8 = 0;
    const ESCAPE: u8 = 1;
    let mut table = [NONE; 256];

    table[b'"' as usize] = ESCAPE;
    table[b'\\' as usize] = ESCAPE;
    table[b'\n' as usize] = ESCAPE;
    table[b'\r' as usize] = ESCAPE;
    table[b'\t' as usize] = ESCAPE;

    // Control characters (0x00-0x1F)
    let mut i = 0;
    while i < 0x20 {
        table[i] = ESCAPE;
        i += 1;
    }

    table
};

/// High-performance JSON serializer that writes directly to a Vec<u8>.
///
/// This serializer is optimized for maximum throughput:
/// - Zero intermediate allocations
/// - Direct buffer manipulation
/// - Fast number formatting with itoa/ryu
/// - Optimized string escaping
#[repr(align(64))] // Cache-line aligned for better performance
pub struct JsonSerializer<'a> {
    buf: &'a mut Vec<u8>,
}

impl<'a> JsonSerializer<'a> {
    /// Create a new JSON serializer that writes to the given buffer.
    ///
    /// The buffer will be cleared before use.
    #[inline(always)]
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        buf.clear();
        Self { buf }
    }

    /// Create a new JSON serializer with a capacity hint.
    ///
    /// The buffer will be cleared and reserved to at least `capacity` bytes.
    #[inline(always)]
    pub fn with_capacity(buf: &'a mut Vec<u8>, capacity: usize) -> Self {
        buf.clear();
        buf.reserve(capacity);
        Self { buf }
    }

    /// Get the serialized JSON as a string slice.
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        // We only write valid UTF-8 to the buffer
        std::str::from_utf8(self.buf).expect("JSON is always valid UTF-8")
    }

    /// Get the serialized JSON as bytes.
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        self.buf
    }

    /// Push bytes to buffer (safe version with capacity check).
    #[inline(always)]
    fn push_bytes(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }
}

impl<'a> Serializer for JsonSerializer<'a> {
    type Error = JsonSerdeError;
    type Ok = ();
    type SerializeList = JsonListSerializer<'a>;
    type SerializeMap = JsonMapSerializer<'a>;
    type SerializeStruct = JsonStructSerializer<'a>;

    #[inline(always)]
    fn write_struct(
        self,
        _schema: &SchemaRef,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        start_json_object(self.buf);
        Ok(JsonStructSerializer {
            buf: self.buf,
            first: true,
        })
    }

    #[inline(always)]
    fn write_map(
        self,
        _schema: &SchemaRef,
        _len: usize,
    ) -> Result<Self::SerializeMap, Self::Error> {
        start_json_object(self.buf);
        Ok(JsonMapSerializer {
            buf: self.buf,
            first: true,
        })
    }

    #[inline(always)]
    fn write_list(
        self,
        _schema: &SchemaRef,
        _len: usize,
    ) -> Result<Self::SerializeList, Self::Error> {
        start_json_array(self.buf);
        Ok(JsonListSerializer {
            buf: self.buf,
            first: true,
        })
    }

    #[inline(always)]
    fn write_boolean(self, _schema: &SchemaRef, value: bool) -> Result<Self::Ok, Self::Error> {
        self.buf
            .extend_from_slice(if value { b"true" } else { b"false" });
        Ok(())
    }

    #[inline(always)]
    fn write_byte(self, _schema: &SchemaRef, value: i8) -> Result<Self::Ok, Self::Error> {
        write_json_integer(self.buf, value);
        Ok(())
    }

    #[inline(always)]
    fn write_short(self, _schema: &SchemaRef, value: i16) -> Result<Self::Ok, Self::Error> {
        write_json_integer(self.buf, value);
        Ok(())
    }

    #[inline(always)]
    fn write_integer(self, _schema: &SchemaRef, value: i32) -> Result<Self::Ok, Self::Error> {
        write_json_integer(self.buf, value);
        Ok(())
    }

    #[inline(always)]
    fn write_long(self, _schema: &SchemaRef, value: i64) -> Result<Self::Ok, Self::Error> {
        write_json_integer(self.buf, value);
        Ok(())
    }

    #[inline(always)]
    fn write_float(self, _schema: &SchemaRef, value: f32) -> Result<Self::Ok, Self::Error> {
        write_json_float(self.buf, value);
        Ok(())
    }

    #[inline(always)]
    fn write_double(self, _schema: &SchemaRef, value: f64) -> Result<Self::Ok, Self::Error> {
        write_json_double(self.buf, value);
        Ok(())
    }

    #[inline(always)]
    fn write_big_integer(
        self,
        _schema: &SchemaRef,
        value: &BigInt,
    ) -> Result<Self::Ok, Self::Error> {
        // BigInt Display is reasonably fast
        use std::fmt::Write;
        write!(StringWriter(self.buf), "{value}").map_err(JsonSerdeError::from)?;
        Ok(())
    }

    #[inline(always)]
    fn write_big_decimal(
        self,
        _schema: &SchemaRef,
        value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        use std::fmt::Write;
        write!(StringWriter(self.buf), "{value}").map_err(JsonSerdeError::from)?;
        Ok(())
    }

    #[inline(always)]
    fn write_string(self, _schema: &SchemaRef, value: &str) -> Result<Self::Ok, Self::Error> {
        write_json_string(self.buf, value);
        Ok(())
    }

    #[inline]
    fn write_blob(self, _schema: &SchemaRef, _value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        todo!("Blob serialization (base64 encoding) not yet implemented")
    }

    #[inline]
    fn write_timestamp(
        self,
        _schema: &SchemaRef,
        _value: &Instant,
    ) -> Result<Self::Ok, Self::Error> {
        todo!("Timestamp serialization not yet implemented")
    }

    #[inline]
    fn write_document(
        self,
        _schema: &SchemaRef,
        _value: &Document,
    ) -> Result<Self::Ok, Self::Error> {
        todo!("Document serialization not yet implemented")
    }

    #[inline(always)]
    fn write_null(mut self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.push_bytes(b"null");
        Ok(())
    }

    #[inline(always)]
    fn skip(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        // Skip does nothing for JSON
        Ok(())
    }
}

pub struct JsonListSerializer<'a> {
    buf: &'a mut Vec<u8>,
    first: bool,
}

impl<'a> ListSerializer for JsonListSerializer<'a> {
    type Error = JsonSerdeError;
    type Ok = ();

    #[inline(always)]
    fn serialize_element<T>(
        &mut self,
        element_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
    {
        if !self.first {
            write_json_comma(self.buf);
        }
        self.first = false;

        // Direct serialization - no intermediate buffer!
        let serializer = JsonSerializer { buf: self.buf };
        value.serialize_with_schema(element_schema, serializer)?;

        Ok(())
    }

    #[inline(always)]
    fn end(self, _schema: &SchemaRef) -> Result<(), Self::Error> {
        end_json_array(self.buf);
        Ok(())
    }
}

pub struct JsonMapSerializer<'a> {
    buf: &'a mut Vec<u8>,
    first: bool,
}

impl<'a> MapSerializer for JsonMapSerializer<'a> {
    type Error = JsonSerdeError;
    type Ok = ();

    #[inline(always)]
    fn serialize_entry<K, V>(
        &mut self,
        key_schema: &SchemaRef,
        value_schema: &SchemaRef,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: ?Sized + SerializeWithSchema,
        V: ?Sized + SerializeWithSchema,
    {
        if !self.first {
            write_json_comma(self.buf);
        }
        self.first = false;

        // Serialize key directly to buffer
        let key_serializer = JsonSerializer { buf: self.buf };
        key.serialize_with_schema(key_schema, key_serializer)?;

        write_json_colon(self.buf);

        // Serialize value directly to buffer
        let value_serializer = JsonSerializer { buf: self.buf };
        value.serialize_with_schema(value_schema, value_serializer)?;

        Ok(())
    }

    #[inline(always)]
    fn end(self, _schema: &SchemaRef) -> Result<(), Self::Error> {
        end_json_object(self.buf);
        Ok(())
    }
}

pub struct JsonStructSerializer<'a> {
    buf: &'a mut Vec<u8>,
    first: bool,
}

impl<'a> StructSerializer for JsonStructSerializer<'a> {
    type Error = JsonSerdeError;
    type Ok = ();

    #[inline(always)]
    fn serialize_member<T>(
        &mut self,
        member_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
    {
        if !self.first {
            write_json_comma(self.buf);
        }
        self.first = false;

        // Get the member name from the schema
        let member = member_schema.as_member().ok_or_else(|| {
            JsonSerdeError::SerializationError("Expected member schema".to_string())
        })?;

        // Write field name as JSON string
        write_json_string(self.buf, &member.name);
        write_json_colon(self.buf);

        // Serialize value directly to buffer
        let value_serializer = JsonSerializer { buf: self.buf };
        value.serialize_with_schema(member_schema, value_serializer)?;

        Ok(())
    }

    #[inline(always)]
    fn serialize_member_named<T>(
        &mut self,
        member_name: &str,
        member_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + SerializeWithSchema,
    {
        if !self.first {
            write_json_comma(self.buf);
        }
        self.first = false;

        // Write field name as JSON string (optimized: no schema lookup needed!)
        write_json_string(self.buf, member_name);
        write_json_colon(self.buf);

        // Serialize value directly to buffer
        let value_serializer = JsonSerializer { buf: self.buf };
        value.serialize_with_schema(member_schema, value_serializer)?;

        Ok(())
    }

    #[inline(always)]
    fn end(self, _schema: &SchemaRef) -> Result<(), Self::Error> {
        end_json_object(self.buf);
        Ok(())
    }
}

// Helper functions for writing JSON primitives

/// Start a JSON object.
#[inline(always)]
fn start_json_object(buf: &mut Vec<u8>) {
    buf.push(b'{');
}

/// End a JSON object.
#[inline(always)]
fn end_json_object(buf: &mut Vec<u8>) {
    buf.push(b'}');
}

/// Start a JSON array.
#[inline(always)]
fn start_json_array(buf: &mut Vec<u8>) {
    buf.push(b'[');
}

/// End a JSON array.
#[inline(always)]
fn end_json_array(buf: &mut Vec<u8>) {
    buf.push(b']');
}

/// Write a JSON field separator (comma).
#[inline(always)]
fn write_json_comma(buf: &mut Vec<u8>) {
    buf.push(b',');
}

/// Write a JSON key-value separator (colon).
#[inline(always)]
fn write_json_colon(buf: &mut Vec<u8>) {
    buf.push(b':');
}

/// Fast integer serialization using itoa.
#[inline(always)]
fn write_json_integer<T: itoa::Integer>(buf: &mut Vec<u8>, value: T) {
    buf.extend_from_slice(itoa::Buffer::new().format(value).as_bytes());
}

/// Optimized JSON string escaping using lookup table.
///
/// This is significantly faster than character-by-character iteration
/// for ASCII-heavy strings (which is the common case).
#[inline]
fn write_json_string(buf: &mut Vec<u8>, s: &str) {
    buf.push(b'"');

    let bytes = s.as_bytes();
    let mut start = 0;

    // Fast path: scan for characters that need escaping
    for (i, &byte) in bytes.iter().enumerate() {
        if byte < 128 && ESCAPE[byte as usize] == 0 {
            // Common case: no escape needed
            continue;
        }

        // Found a character that needs escaping
        // Write everything up to this point
        if start < i {
            buf.extend_from_slice(&bytes[start..i]);
        }

        // Write the escape sequence
        match byte {
            b'"' => buf.extend_from_slice(b"\\\""),
            b'\\' => buf.extend_from_slice(b"\\\\"),
            b'\n' => buf.extend_from_slice(b"\\n"),
            b'\r' => buf.extend_from_slice(b"\\r"),
            b'\t' => buf.extend_from_slice(b"\\t"),
            b if b < 0x20 => {
                // Control character
                buf.extend_from_slice(b"\\u00");
                buf.push(HEX_DIGITS[(b >> 4) as usize]);
                buf.push(HEX_DIGITS[(b & 0x0F) as usize]);
            }
            _ => {
                // Non-ASCII - write as-is (already valid UTF-8)
                buf.push(byte);
            }
        }

        start = i + 1;
    }

    // Write remaining unescaped portion
    if start < bytes.len() {
        buf.extend_from_slice(&bytes[start..]);
    }

    buf.push(b'"');
}

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Fast float serialization with special value handling.
#[inline(always)]
fn write_json_float(buf: &mut Vec<u8>, value: f32) {
    if value.is_finite() {
        // ryu is 2-5x faster than standard formatting
        buf.extend_from_slice(ryu::Buffer::new().format_finite(value).as_bytes());
    } else if value.is_nan() {
        buf.extend_from_slice(b"\"NaN\"");
    } else if value.is_sign_positive() {
        buf.extend_from_slice(b"\"Infinity\"");
    } else {
        buf.extend_from_slice(b"\"-Infinity\"");
    }
}

/// Fast double serialization with special value handling.
#[inline(always)]
fn write_json_double(buf: &mut Vec<u8>, value: f64) {
    if value.is_finite() {
        // ryu is 2-5x faster than standard formatting
        buf.extend_from_slice(ryu::Buffer::new().format_finite(value).as_bytes());
    } else if value.is_nan() {
        buf.extend_from_slice(b"\"NaN\"");
    } else if value.is_sign_positive() {
        buf.extend_from_slice(b"\"Infinity\"");
    } else {
        buf.extend_from_slice(b"\"-Infinity\"");
    }
}

/// Helper struct for writing Display types to Vec<u8>.
struct StringWriter<'a>(&'a mut Vec<u8>);

impl std::fmt::Write for StringWriter<'_> {
    #[inline(always)]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}
