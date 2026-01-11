//! Utilities for implementing [`Debug`] for generated shapes
//!
//! Smithy shape debug implementations are similar to the default
//! implementation derived by the `Debug` macro. However, unlike
//! the default implementation the Smithy implementations must respect
//! the `@sensitive` trait. Fields and structures with this trait should
//! _always_ be redacted when written to a string in order to avoid leaking
//! sensitive info into logs and API responses.
//!
//! ## Derived Debug Implementations
//!
//! The `SmithyShape` derive macro will automatically derive a `Debug` implementation
//! for Smithy Shapes.
//!
use core::fmt;
use std::fmt::{Debug, DebugList, DebugMap, DebugStruct, Display, Error, Formatter};

use log::error;
use thiserror::Error;

use crate::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    schema::{Document, SchemaRef, SmithyTrait, prelude::SensitiveTrait},
    serde::{
        debug::FmtError::Custom,
        se::{ListSerializer, MapSerializer, SerializeWithSchema, Serializer, StructSerializer},
    },
};
// ============================================================================
// Wrapper
// ============================================================================

/// Wrapper struct used to adapt debug implementations to use schema-base serialization
///
/// This class should not be used directly by users. Instead, users should use generated
/// `Debug` implementation for shapes.
pub struct DebugWrapper<'a, T: SerializeWithSchema>(&'a SchemaRef, &'a T);
impl<'a, T: SerializeWithSchema> DebugWrapper<'a, T> {
    /// Construct a new Debug wrapper to format type `T` using the provided schema.
    pub const fn new(schema: &'a SchemaRef, value: &'a T) -> Self {
        DebugWrapper(schema, value)
    }
}
impl<T: SerializeWithSchema> Debug for DebugWrapper<'_, T> {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        self.1
            .serialize_with_schema(self.0, DebugSerializer { fmt })
            .map_err(|e| {
                error!("Encountered error while printing debug repr: {}", e);
                Error
            })
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Errors that can occur when serializing a shape into a string representation
#[derive(Error, Debug)]
#[doc(hidden)]
pub enum FmtError {
    #[error(transparent)]
    Fmt(#[from] Error),
    #[error("Expected Member Schema but found: {0}")]
    ExpectedMember(String),
    #[error("Formatting error: {0}")]
    Custom(String),
}
use crate::serde::se::Error as SerdeError;
impl SerdeError for FmtError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Custom(msg.to_string())
    }
}

// ============================================================================
// Debug Serializer
// ============================================================================

macro_rules! redact {
    ($self:ident, $schema:ident, $value:ident) => {
        if $schema.contains_type::<SensitiveTrait>() {
            $self.fmt.write_str(REDACTED_ITEM)?;
        } else {
            Debug::fmt(&$value, $self.fmt)?;
        }
    };
}

/// Serializer used to generate `Debug` implementations that respect `@sensitive` fields.
struct DebugSerializer<'a, 'b: 'a> {
    fmt: &'a mut Formatter<'b>,
}
const REDACTED_ITEM: &str = "**REDACTED**";
const REDACTED_LIST: &str = "[**REDACTED**]";
const REDACTED_MAP: &str = "{**REDACTED**}";

impl<'a, 'b> Serializer for DebugSerializer<'a, 'b> {
    type Error = FmtError;
    type Ok = ();
    type SerializeList = DebugListSerializer<'a, 'b>;
    type SerializeMap = DebugMapSerializer<'a, 'b>;
    type SerializeStruct = DebugStructSerializer<'a, 'b>;

    fn write_struct(
        self,
        schema: &SchemaRef,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        if schema.contains_type::<SensitiveTrait>() {
            self.fmt.write_str(schema.id().name())?;
            // Replace entire structure contents with redacted placeholder
            self.fmt.write_str(REDACTED_MAP)?;
            Ok(DebugStructSerializer::Redacted)
        } else {
            Ok(DebugStructSerializer::Unredacted(
                self.fmt.debug_struct(schema.id().name()),
            ))
        }
    }

    fn write_map(self, schema: &SchemaRef, _: usize) -> Result<Self::SerializeMap, Self::Error> {
        if schema.contains_type::<SensitiveTrait>() {
            // Replace entire map with redacted placeholder
            self.fmt.write_str(REDACTED_MAP)?;
            Ok(DebugMapSerializer::Redacted)
        } else {
            Ok(DebugMapSerializer::Unredacted(self.fmt.debug_map()))
        }
    }

    fn write_list(self, schema: &SchemaRef, _: usize) -> Result<Self::SerializeList, Self::Error> {
        if schema.contains_type::<SensitiveTrait>() {
            // Replace entire list with redacted placeholder
            self.fmt.write_str(REDACTED_LIST)?;
            Ok(DebugListSerializer::Redacted)
        } else {
            Ok(DebugListSerializer::Unredacted(self.fmt.debug_list()))
        }
    }

    #[inline]
    fn write_boolean(self, schema: &SchemaRef, value: bool) -> Result<Self::Ok, Self::Error> {
        redact!(self, schema, value);
        Ok(())
    }

    #[inline]
    fn write_byte(self, schema: &SchemaRef, value: i8) -> Result<Self::Ok, Self::Error> {
        redact!(self, schema, value);
        Ok(())
    }

    #[inline]
    fn write_short(self, schema: &SchemaRef, value: i16) -> Result<Self::Ok, Self::Error> {
        redact!(self, schema, value);
        Ok(())
    }

    #[inline]
    fn write_integer(self, schema: &SchemaRef, value: i32) -> Result<Self::Ok, Self::Error> {
        redact!(self, schema, value);
        Ok(())
    }

    #[inline]
    fn write_long(self, schema: &SchemaRef, value: i64) -> Result<Self::Ok, Self::Error> {
        redact!(self, schema, value);
        Ok(())
    }

    #[inline]
    fn write_float(self, schema: &SchemaRef, value: f32) -> Result<Self::Ok, Self::Error> {
        redact!(self, schema, value);
        Ok(())
    }

    #[inline]
    fn write_double(self, schema: &SchemaRef, value: f64) -> Result<Self::Ok, Self::Error> {
        redact!(self, schema, value);
        Ok(())
    }

    #[inline]
    fn write_big_integer(
        self,
        schema: &SchemaRef,
        value: &BigInt,
    ) -> Result<Self::Ok, Self::Error> {
        redact!(self, schema, value);
        Ok(())
    }

    #[inline]
    fn write_big_decimal(
        self,
        schema: &SchemaRef,
        value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        redact!(self, schema, value);
        Ok(())
    }

    #[inline]
    fn write_string(self, schema: &SchemaRef, value: &str) -> Result<Self::Ok, Self::Error> {
        redact!(self, schema, value);
        Ok(())
    }

    #[inline]
    fn write_blob(self, schema: &SchemaRef, value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        redact!(self, schema, value);
        Ok(())
    }

    #[inline]
    fn write_timestamp(self, schema: &SchemaRef, value: &Instant) -> Result<Self::Ok, Self::Error> {
        redact!(self, schema, value);
        Ok(())
    }

    #[inline]
    fn write_document(
        self,
        schema: &SchemaRef,
        value: &Box<dyn Document>,
    ) -> Result<Self::Ok, Self::Error> {
        redact!(self, schema, value);
        Ok(())
    }

    #[inline]
    fn write_null(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        self.fmt.write_str("null")?;
        Ok(())
    }

    #[inline]
    fn skip(self, _schema: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

enum DebugListSerializer<'a, 'b: 'a> {
    Unredacted(DebugList<'a, 'b>),
    Redacted,
}
impl ListSerializer for DebugListSerializer<'_, '_> {
    type Error = FmtError;
    type Ok = ();

    fn serialize_element<T>(
        &mut self,
        element_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        let DebugListSerializer::Unredacted(inner) = self else {
            // Redacted lists do not write any entries.
            return Ok(());
        };
        inner.entry(&DebugWrapper::new(element_schema, value));
        Ok(())
    }

    #[inline]
    fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        if let DebugListSerializer::Unredacted(mut inner) = self {
            inner.finish()?;
        }
        Ok(())
    }
}

enum DebugMapSerializer<'a, 'b: 'a> {
    Unredacted(DebugMap<'a, 'b>),
    Redacted,
}

impl MapSerializer for DebugMapSerializer<'_, '_> {
    type Error = FmtError;
    type Ok = ();

    fn serialize_entry<K, V>(
        &mut self,
        key_schema: &SchemaRef,
        value_schema: &SchemaRef,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: SerializeWithSchema,
        V: SerializeWithSchema,
    {
        let DebugMapSerializer::Unredacted(inner) = self else {
            // Redacted lists do not write any entries.
            return Ok(());
        };
        inner.entry(
            &DebugWrapper::new(key_schema, key),
            &DebugWrapper::new(value_schema, value),
        );
        Ok(())
    }

    #[inline]
    fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        if let DebugMapSerializer::Unredacted(mut inner) = self {
            inner.finish()?;
        }
        Ok(())
    }
}

enum DebugStructSerializer<'a, 'b: 'a> {
    Unredacted(DebugStruct<'a, 'b>),
    Redacted,
}

impl StructSerializer for DebugStructSerializer<'_, '_> {
    type Error = FmtError;
    type Ok = ();

    fn serialize_member<T>(
        &mut self,
        member_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        let DebugStructSerializer::Unredacted(inner) = self else {
            // Redacted lists do not write any entries.
            return Ok(());
        };
        let Some(me) = member_schema.as_member() else {
            return Err(FmtError::ExpectedMember(format!(
                "{:?}",
                member_schema.id()
            )));
        };
        inner.field(me.name.as_str(), &DebugWrapper::new(member_schema, value));
        Ok(())
    }

    fn serialize_member_named<T>(
        &mut self,
        member_name: &str,
        member_schema: &SchemaRef,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        let DebugStructSerializer::Unredacted(inner) = self else {
            // Redacted lists do not write any entries.
            return Ok(());
        };
        inner.field(member_name, &DebugWrapper(member_schema, value));
        Ok(())
    }

    #[inline]
    fn end(self, _: &SchemaRef) -> Result<Self::Ok, Self::Error> {
        if let DebugStructSerializer::Unredacted(mut inner) = self {
            inner.finish()?;
        }
        Ok(())
    }
}

// ============================================================================
// Type impls
// ============================================================================
impl Debug for Box<dyn Document> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("Document");
        s.field("schema", &self.schema());
        s.field("value", &DebugWrapper::new(self.schema(), self));
        self.discriminator()
            .map(|v| s.field("discriminator", &v.id()));
        s.finish()
    }
}

impl Debug for dyn SmithyTrait {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_tuple(self.id().id());
        // Only print document value if it is non-null.
        if !&self.value().is_null() {
            // Use the debug wrapper to avoid writing document wrapper info into the trait debug.
            s.field(&DebugWrapper::new(self.value().schema(), self.value()));
        }
        s.finish()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        IndexMap,
        derive::SmithyShape,
        schema::prelude::{MediaTypeTrait, STRING},
        smithy,
    };

    smithy!("com.example#Map": {
        map MAP_SCHEMA {
            key: STRING
            value: STRING
        }
    });
    smithy!("com.example#List": {
        list LIST_SCHEMA {
            member: STRING
        }
    });
    smithy!("com.example#Shape": {
        structure SCHEMA {
            A: STRING = "a"
            @SensitiveTrait;
            B: STRING = "b"
            C: STRING = "c"
            MAP: MAP_SCHEMA = "map"
            LIST: LIST_SCHEMA = "list"
        }
    });
    smithy!("com.example#Shape": {
        structure REDACTED_AGGREGATES {
            @SensitiveTrait;
            MAP_REDACT: MAP_SCHEMA = "map"
            @SensitiveTrait;
            @MediaTypeTrait::new("application/json");
            LIST_REDACT: LIST_SCHEMA = "list"
        }
    });

    #[derive(SmithyShape)]
    #[smithy_schema(SCHEMA)]
    pub struct SerializeMe {
        #[smithy_schema(A)]
        pub member_a: String,
        #[smithy_schema(B)]
        pub member_b: String,
        #[smithy_schema(C)]
        pub member_optional: Option<String>,
        #[smithy_schema(LIST)]
        pub member_list: Vec<String>,
        #[smithy_schema(MAP)]
        pub member_map: IndexMap<String, String>,
    }

    #[derive(SmithyShape)]
    #[smithy_schema(REDACTED_AGGREGATES)]
    pub struct RedactMe {
        #[smithy_schema(LIST_REDACT)]
        pub member_list: Vec<String>,
        #[smithy_schema(MAP_REDACT)]
        pub member_map: IndexMap<String, String>,
    }

    #[test]
    fn debug_prints_shape() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let list = vec!["a".to_string(), "b".to_string()];
        let struct_to_write = SerializeMe {
            member_a: "a".to_string(),
            member_b: "b".to_string(),
            member_optional: Some("c".to_string()),
            member_map: map,
            member_list: list,
        };
        assert_eq!(
            format!("{struct_to_write:?}"),
            "Shape { member_a: \"a\", member_b: **REDACTED**, member_optional: \"c\", member_list: [\"a\", \"b\"], member_map: {\"a\": \"b\"} }"
        );
    }

    #[test]
    fn debug_respects_pretty_print() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let list = vec!["a".to_string(), "b".to_string()];
        let struct_to_write = SerializeMe {
            member_a: "a".to_string(),
            member_b: "b".to_string(),
            member_optional: Some("c".to_string()),
            member_map: map,
            member_list: list,
        };
        assert_eq!(
            format!("{struct_to_write:#?}"),
            r#"Shape {
    member_a: "a",
    member_b: **REDACTED**,
    member_optional: "c",
    member_list: [
        "a",
        "b",
    ],
    member_map: {
        "a": "b",
    },
}"#
        );
    }

    #[test]
    fn redacts_aggregates() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let list = vec!["a".to_string(), "b".to_string()];
        let struct_to_write = RedactMe {
            member_list: list,
            member_map: map,
        };
        let output = format!("{struct_to_write:?}");
        assert_eq!(
            output,
            "Shape { member_list: [**REDACTED**], member_map: {**REDACTED**} }"
        );
    }

    #[test]
    fn document_conversion_retains_redaction() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let list = vec!["a".to_string(), "b".to_string()];
        let struct_to_write = RedactMe {
            member_list: list,
            member_map: map,
        };
        let document: Box<dyn Document> = struct_to_write.into();
        let output = format!("{document:#?}");
        let expected = r#"Document {
    schema: StructSchema {
        shape_type: Structure,
        id: "com.example#Shape",
        traits: [],
        members: {
            "map": {
                "target": "com.example#Map",
                "traits": [
                    smithy.api#sensitive,
                ],
            },
            "list": {
                "target": "com.example#List",
                "traits": [
                    smithy.api#mediaType(
                        "application/json",
                    ),
                    smithy.api#sensitive,
                ],
            },
        },
    },
    value: Shape {
        list: [**REDACTED**],
        map: {**REDACTED**},
    },
    discriminator: "com.example#Shape",
}"#;
        assert_eq!(output, expected);
    }
}
