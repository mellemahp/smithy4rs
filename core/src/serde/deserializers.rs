#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::missing_errors_doc)]

use std::{error::Error as StdError, hash::Hash, fmt::Display};

use indexmap::IndexMap;

use crate::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    schema::{Document, SchemaRef, SchemaShape, ShapeId},
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
impl<T: SchemaShape + DeserializeWithSchema> Deserialize for T {
    fn deserialize<'de, D: Deserializer<'de>>(&self, deserializer: D) -> Result<Self, D::Error> {
        Self::deserialize_with_schema(self.schema(), deserializer)
    }
}

/// Schema-Guided deserialization implementation.
pub trait DeserializeWithSchema: Sized {
    /// Deserialize a Shape using a schema to guide the process
    fn deserialize_with_schema<'de, D: Deserializer<'de>>(
        schema: &SchemaRef,
        deserializer: D,
    ) -> Result<Self, D::Error>;
}

/// List Deserializer that can be called in a loop to deserialize list values
pub trait ListDeserializer<'de> {
    /// Must match the `Error` type of our `Deserializer`.
    type Error: Error;

    /// Deserialize a sequence element.
    fn deserialize_element<T>(
        &mut self,
        element_schema: &SchemaRef,
    ) -> Result<Option<T>, Self::Error>
    where
        T: DeserializeWithSchema;

    /// Finish deserializing a sequence.
    fn finish(self) -> Result<(), Self::Error>;
}

/// Map Deserializer that can be called in a loop to deserialize map values
pub trait MapDeserializer<'de> {
    /// Must match the `Error` type of our [`Deserializer`].
    type Error: Error;

    /// Deserialize a single map entry
    fn deserialize_entry<K, V>(
        &mut self,
        key_schema: &SchemaRef,
        value_schema: &SchemaRef,
    ) -> Result<Option<(K, V)>, Self::Error>
    where
        K: DeserializeWithSchema,
        V: DeserializeWithSchema;

    /// Finish deserializing a map.
    fn finish(self) -> Result<(), Self::Error>;
}

/// Struct Deserializer for deserializing structure and union types
pub trait StructDeserializer<'de> {
    /// Must match the `Error` type of our [`Deserializer`].
    type Error: Error;

    /// Check if discriminator matches expected value (for union types).
    /// 
    /// Returns true if the discriminator matches or if no discriminator checking
    /// is needed. Default implementation accepts all discriminators.
    fn check_discriminator(&mut self, expected: &ShapeId) -> Result<bool, Self::Error> {
        Ok(true)
    }

    /// Deserialize a member from the struct
    fn deserialize_member<T>(
        &mut self,
        member_schema: &SchemaRef,
    ) -> Result<Option<T>, Self::Error>
    where
        T: DeserializeWithSchema;

    /// Deserialize an optional member.
    ///
    /// This method will return `None` for any optional members that are missing,
    /// otherwise it deserializes and returns `Some(value)`.
    fn deserialize_optional_member<T: DeserializeWithSchema>(
        &mut self,
        member_schema: &SchemaRef,
    ) -> Result<Option<T>, Self::Error> {
        self.deserialize_member(member_schema)
    }

    /// Finish deserializing a structure.
    fn finish(self) -> Result<(), Self::Error>;
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

    /// Type returned from [`read_list`] for deserializing the contents of a list.
    type DeserializeList: ListDeserializer<'de, Error = Self::Error>;

    /// Type returned from [`read_map`] for deserializing the contents of a map.
    type DeserializeMap: MapDeserializer<'de, Error = Self::Error>;

    /// Type returned from [`read_struct`] for deserializing the contents of a struct or union.
    type DeserializeStruct: StructDeserializer<'de, Error = Self::Error>;

    /// Begin to deserialize a variably sized structure or union.
    fn read_struct(
        self,
        schema: &SchemaRef,
    ) -> Result<Self::DeserializeStruct, Self::Error>;

    /// Begin to deserialize a variably sized map.
    fn read_map(self, schema: &SchemaRef) -> Result<Self::DeserializeMap, Self::Error>;

    /// Begin to deserialize a variably sized list.
    fn read_list(self, schema: &SchemaRef) -> Result<Self::DeserializeList, Self::Error>;

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
        let mut list_deser = deserializer.read_list(schema)?;
        let element_schema = schema.expect_member("member");
        let mut vec = Vec::new();

        while let Some(element) = list_deser.deserialize_element::<T>(element_schema)? {
            vec.push(element);
        }

        list_deser.finish()?;
        Ok(vec)
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
        let mut map_deser = deserializer.read_map(schema)?;
        let key_schema = schema.get_member("key").expect("Should have key schema");
        let value_schema = schema
            .get_member("value")
            .expect("Should have value schema");
        let mut map = IndexMap::new();

        while let Some((key, value)) = map_deser.deserialize_entry::<K, V>(key_schema, value_schema)? {
            map.insert(key, value);
        }

        map_deser.finish()?;
        Ok(map)
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