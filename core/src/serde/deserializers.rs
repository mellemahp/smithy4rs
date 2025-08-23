#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::missing_errors_doc)]

use std::{error::Error as StdError, hash::Hash, fmt::Display};

use indexmap::IndexMap;

use crate::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    schema::{Document, SchemaRef, SchemaShape},
};

/// Deserialize a shape with its pre-defined schema.
///
/// This trait provides an automatic, blanket implementation for all shapes
/// with both a [`SchemaShape`] and [`DeserializeWithSchema`] implementation.
pub trait Deserialize: SchemaShape + DeserializeWithSchema {
    /// Deserialize a shape with its pre-defined schema
    fn deserialize<'de, D: Deserializer<'de>>(&self, deserializer: D) -> Result<Self, D::Error>;
}

// Blanket implementation of deserialization for all types with schema

/// Schema-Guided deserialization implementation.
pub trait DeserializeWithSchema: Sized {
    /// Deserialize a Shape using a schema to guide the process
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error>;
}

/// Basically just a clone of the serde::Error trait.
/// We use our own to ensure we don't enforce a `serde` dependency on consumers.
pub trait Error: Sized + StdError {
    fn custom<T: Display>(msg: T) -> Self;
}

/// Schema-guided deserializer trait (mirrors Serializer)
pub trait Deserializer<'de>: Sized {
    /// Error type emitted on failed deserialization.
    type Error: Error;

    /// Begin to deserialize a variably sized structure or union.
    fn read_struct<T: DeserializeWithSchema>(
        self,
        schema: &SchemaRef,
    ) -> Result<T, Self::Error>;

    /// Begin to deserialize a variably sized map.
    fn read_map<K: DeserializeWithSchema, V: DeserializeWithSchema>(self, schema: &SchemaRef) -> Result<IndexMap<K, V>, Self::Error>;

    /// Begin to deserialize a variably sized list.x
    fn read_list<T: DeserializeWithSchema>(self, schema: &SchemaRef) -> Result<Vec<T>, Self::Error>;

    /// Deserialize a `boolean`
    fn read_boolean(self, schema: &SchemaRef) -> Result<bool, Self::Error>;

    /// Deserialize a byte (`i8`)
    fn read_byte(self, schema: &SchemaRef) -> Result<i8, Self::Error>;

    /// Deserialize a short (`i16`)
    fn read_short(self, schema: &SchemaRef) -> Result<i16, Self::Error>;

    /// Deserialize an integer (`i32`)
    fn read_integer(self, schema: &SchemaRef) -> Result<i32, Self::Error>;

    /// Deserialize a long (`i64`)
    fn read_long(self, schema: &SchemaRef) -> Result<i64, Self::Error>;

    /// Deserialize a float (`f32`)
    fn read_float(self, schema: &SchemaRef) -> Result<f32, Self::Error>;

    /// Deserialize a double (`f64`)
    fn read_double(self, schema: &SchemaRef) -> Result<f64, Self::Error>;

    /// Deserialize a [`BigInt`]
    fn read_big_integer(self, schema: &SchemaRef) -> Result<BigInt, Self::Error>;

    /// Deserialize a [`BigDecimal`]
    fn read_big_decimal(self, schema: &SchemaRef) -> Result<BigDecimal, Self::Error>;

    /// Deserialize a string
    fn read_string(self, schema: &SchemaRef) -> Result<String, Self::Error>;

    /// Deserialize a blob (i.e. a buffer)
    fn read_blob(self, schema: &SchemaRef) -> Result<ByteBuffer, Self::Error>;

    /// Deserialize a timestamp
    fn read_timestamp(self, schema: &SchemaRef) -> Result<Instant, Self::Error>;

    /// Deserialize an untyped [`Document`]
    fn read_document(self, schema: &SchemaRef) -> Result<Document, Self::Error>;

    /// Deserialize a `null` value
    fn read_null(self, schema: &SchemaRef) -> Result<(), Self::Error>;

    /// Check if the current value is null
    fn is_null(&self) -> bool {
        false
    }
}

// === Default implementations ===
impl<T: DeserializeWithSchema> DeserializeWithSchema for Vec<T> {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.read_list::<T>(schema) 
    }
}

impl<K, V> DeserializeWithSchema for IndexMap<K, V>
where
    K: DeserializeWithSchema + Hash + Eq,
    V: DeserializeWithSchema,
{
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        todo!()
    }
}

impl DeserializeWithSchema for bool {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.read_boolean(schema)
    }
}

impl DeserializeWithSchema for i8 {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.read_byte(schema)
    }
}

impl DeserializeWithSchema for i16 {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.read_short(schema)
    }
}

impl DeserializeWithSchema for i32 {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.read_integer(schema)
    }
}

impl DeserializeWithSchema for i64 {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.read_long(schema)
    }
}

impl DeserializeWithSchema for f32 {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.read_float(schema)
    }
}

impl DeserializeWithSchema for f64 {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.read_double(schema)
    }
}

impl DeserializeWithSchema for BigInt {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.read_big_integer(schema)
    }
}

impl DeserializeWithSchema for BigDecimal {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.read_big_decimal(schema)
    }
}

impl DeserializeWithSchema for ByteBuffer {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.read_blob(schema)
    }
}

impl DeserializeWithSchema for String {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.read_string(schema)
    }
}

impl DeserializeWithSchema for Instant {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.read_timestamp(schema)
    }
}


impl<T: DeserializeWithSchema> DeserializeWithSchema for Option<T> {
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        if deserializer.is_null() {
            deserializer.read_null(schema)?;
            Ok(None)
        } else {
            T::deserialize_with_schema(schema, deserializer).map(Some)
        }
    }
}