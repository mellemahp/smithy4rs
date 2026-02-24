//! HTTP URI label serialization and deserialization.
//!
//! URI labels are path segments like `{bucket}` in `/buckets/{bucket}`.
//! Only scalar values are supported (no lists, maps, or blobs).
//! Timestamps default to `date-time` (ISO 8601) format.

use std::fmt::Write;

use smithy4rs_core::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    schema::{Document, Schema},
    serde::{de, never::Never, se},
};

use crate::error::HttpBindingError;

// --- Serialization ---

/// Serializer for HTTP URI label values.
///
/// Converts scalar values to strings for URI path interpolation.
/// Lists, maps, structs, and blobs are not supported in URI labels.
pub struct LabelSerializer<'a> {
    output: &'a mut String,
}

impl<'a> LabelSerializer<'a> {
    /// Create a new label serializer that writes to the given output string.
    #[inline]
    pub fn new(output: &'a mut String) -> Self {
        Self { output }
    }
}

impl se::Serializer for LabelSerializer<'_> {
    type Error = HttpBindingError;
    type Ok = ();
    type ListWriter = Never<Self::Error>;
    type MapWriter = Never<Self::Error>;
    type StructWriter = Never<Self::Error>;

    fn write_struct(
        self,
        _schema: &Schema,
        _len: usize,
    ) -> Result<Self::StructWriter, Self::Error> {
        Err(HttpBindingError::new(
            "structs cannot be serialized to URI labels",
        ))
    }

    fn write_map(self, _schema: &Schema, _len: usize) -> Result<Self::MapWriter, Self::Error> {
        Err(HttpBindingError::new(
            "maps cannot be serialized to URI labels",
        ))
    }

    fn write_list(self, _schema: &Schema, _len: usize) -> Result<Self::ListWriter, Self::Error> {
        Err(HttpBindingError::new(
            "lists cannot be serialized to URI labels",
        ))
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
    fn write_string(self, schema: &Schema, value: &str) -> Result<Self::Ok, Self::Error> {
        if value.is_empty() {
            return Err(HttpBindingError::new(format!(
                "HTTP label for `{:?}` cannot be empty",
                schema.id()
            )));
        }
        self.output.push_str(value);
        Ok(())
    }

    fn write_blob(self, _schema: &Schema, _value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        Err(HttpBindingError::new(
            "blobs cannot be serialized to URI labels",
        ))
    }

    fn write_timestamp(self, _schema: &Schema, _value: &Instant) -> Result<Self::Ok, Self::Error> {
        // TODO: Check @timestampFormat trait, default to DATE_TIME (ISO 8601)
        Err(HttpBindingError::new(
            "timestamp label serialization is not yet implemented",
        ))
    }

    fn write_document(
        self,
        _schema: &Schema,
        _value: &Box<dyn Document>,
    ) -> Result<Self::Ok, Self::Error> {
        Err(HttpBindingError::new(
            "documents cannot be serialized to URI labels",
        ))
    }

    fn write_null(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Err(HttpBindingError::new(
            "null values cannot be serialized to URI labels",
        ))
    }

    #[inline]
    fn skip(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// --- Deserialization ---

/// Deserializer for HTTP URI label values.
///
/// Parses scalar values from URI path segments.
pub struct LabelDeserializer<S: AsRef<str>> {
    input: S,
}

impl<S: AsRef<str>> LabelDeserializer<S> {
    /// Create a new label deserializer.
    #[inline]
    pub fn new(input: S) -> Self {
        Self { input }
    }
}

impl<'de, S: AsRef<str>> de::Deserializer<'de> for LabelDeserializer<S> {
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
                "invalid boolean label value: '{}'",
                self.input.as_ref()
            ))),
        }
    }

    #[inline]
    fn read_byte(self, _schema: &Schema) -> Result<i8, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid byte label value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_short(self, _schema: &Schema) -> Result<i16, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid short label value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_integer(self, _schema: &Schema) -> Result<i32, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid integer label value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_long(self, _schema: &Schema) -> Result<i64, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid long label value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_float(self, _schema: &Schema) -> Result<f32, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid float label value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    #[inline]
    fn read_double(self, _schema: &Schema) -> Result<f64, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid double label value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    fn read_big_integer(self, _schema: &Schema) -> Result<BigInt, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid bigInteger label value: '{}'",
                self.input.as_ref()
            ))
        })
    }

    fn read_big_decimal(self, _schema: &Schema) -> Result<BigDecimal, Self::Error> {
        self.input.as_ref().parse().map_err(|_| {
            HttpBindingError::new(format!(
                "invalid bigDecimal label value: '{}'",
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
            "timestamp label deserialization is not yet implemented",
        ))
    }
}

#[cfg(test)]
mod tests {
    use smithy4rs_core::{
        schema::prelude::{BOOLEAN, DOUBLE, FLOAT, INTEGER, STRING},
        serde::{de::Deserializer, se::Serializer},
    };

    use super::*;

    // --- LabelSerializer ---

    #[test]
    fn serialize_string() {
        let mut out = String::new();
        LabelSerializer::new(&mut out)
            .write_string(&STRING, "hello")
            .unwrap();
        assert_eq!(out, "hello");
    }

    #[test]
    fn serialize_empty_string_error() {
        let mut out = String::new();
        assert!(
            LabelSerializer::new(&mut out)
                .write_string(&STRING, "")
                .is_err()
        );
    }

    #[test]
    fn serialize_integer() {
        let mut out = String::new();
        LabelSerializer::new(&mut out)
            .write_integer(&INTEGER, 123)
            .unwrap();
        assert_eq!(out, "123");
    }

    #[test]
    fn serialize_boolean() {
        let mut out = String::new();
        LabelSerializer::new(&mut out)
            .write_boolean(&BOOLEAN, true)
            .unwrap();
        assert_eq!(out, "true");
    }

    #[test]
    fn serialize_float() {
        let mut out = String::new();
        LabelSerializer::new(&mut out)
            .write_float(&FLOAT, 1.5)
            .unwrap();
        assert_eq!(out, "1.5");
    }

    #[test]
    fn serialize_blob_error() {
        let mut out = String::new();
        let blob = ByteBuffer::from(vec![1, 2, 3]);
        assert!(
            LabelSerializer::new(&mut out)
                .write_blob(&STRING, &blob)
                .is_err()
        );
    }

    #[test]
    fn serialize_null_error() {
        let mut out = String::new();
        assert!(LabelSerializer::new(&mut out).write_null(&STRING).is_err());
    }

    // --- LabelDeserializer ---

    #[test]
    fn deserialize_string() {
        let val: String = LabelDeserializer::new("hello")
            .read_string(&STRING)
            .unwrap();
        assert_eq!(val, "hello");
    }

    #[test]
    fn deserialize_integer() {
        let val: i32 = LabelDeserializer::new("42").read_integer(&INTEGER).unwrap();
        assert_eq!(val, 42);
    }

    #[test]
    fn deserialize_boolean_true() {
        let val: bool = LabelDeserializer::new("true").read_bool(&BOOLEAN).unwrap();
        assert!(val);
    }

    #[test]
    fn deserialize_boolean_invalid() {
        assert!(LabelDeserializer::new("yes").read_bool(&BOOLEAN).is_err());
    }

    #[test]
    fn deserialize_float() {
        let val: f64 = LabelDeserializer::new("3.14").read_double(&DOUBLE).unwrap();
        assert!((val - 3.14).abs() < 0.001);
    }
}
