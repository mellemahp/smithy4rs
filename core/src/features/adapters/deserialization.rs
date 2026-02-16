#![allow(dead_code)]

use std::{
    error::Error as StdError,
    fmt,
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
};

use serde::de::{DeserializeSeed, Error as SerdeDeError, MapAccess, SeqAccess, Visitor};

use crate::{
    schema::{Schema, ShapeType},
    serde::deserializers::{
        DeserializeWithSchema, Deserializer, Error as DeserError, ListReader, MapReader,
        StructReader,
    },
};

//========================================================================
// Errors
//========================================================================

/// Error wrapper to bridge serde errors with our error type
#[derive(Debug)]
pub struct DeserdeErrorWrapper<E: SerdeDeError>(E);

impl<E: SerdeDeError> Display for DeserdeErrorWrapper<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<E: SerdeDeError> StdError for DeserdeErrorWrapper<E> {}

impl<E: SerdeDeError> DeserError for DeserdeErrorWrapper<E> {
    fn custom<T: Display>(msg: T) -> Self {
        DeserdeErrorWrapper(E::custom(msg))
    }
}

impl<E: SerdeDeError> From<E> for DeserdeErrorWrapper<E> {
    fn from(e: E) -> Self {
        DeserdeErrorWrapper(e)
    }
}

//========================================================================
// Never Reader Types (for unused GAT slots)
//========================================================================

/// A reader type that can never be constructed.
/// Used for GAT slots that are not applicable in a given context.
/// Generic over E to satisfy GAT `Error = Self::Error` bounds.
pub struct NeverStructReader<E>(PhantomData<fn() -> E>, std::convert::Infallible);

impl<'de, E: DeserError> StructReader<'de> for NeverStructReader<E> {
    type Error = E;

    fn read_member(&mut self) -> Result<Option<Schema>, Self::Error> {
        match self.1 {}
    }

    fn read_value<T: DeserializeWithSchema<'de>>(
        &mut self,
        _schema: &Schema,
    ) -> Result<T, Self::Error> {
        match self.1 {}
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        match self.1 {}
    }
}

pub struct NeverListReader<E>(PhantomData<fn() -> E>, std::convert::Infallible);

impl<'de, E: DeserError> ListReader<'de> for NeverListReader<E> {
    type Error = E;

    fn read_element<T: DeserializeWithSchema<'de>>(
        &mut self,
        _schema: &Schema,
    ) -> Result<Option<T>, Self::Error> {
        match self.1 {}
    }
}

pub struct NeverMapReader<E>(PhantomData<fn() -> E>, std::convert::Infallible);

impl<'de, E: DeserError> MapReader<'de> for NeverMapReader<E> {
    type Error = E;

    fn read_key(&mut self) -> Result<Option<String>, Self::Error> {
        match self.1 {}
    }

    fn read_value<V: DeserializeWithSchema<'de>>(
        &mut self,
        _schema: &Schema,
    ) -> Result<V, Self::Error> {
        match self.1 {}
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        match self.1 {}
    }
}

//========================================================================
// Reader Types
//========================================================================

/// Wraps serde's `SeqAccess` to implement our `ListReader` trait.
pub struct SerdeListReader<'de, S: SeqAccess<'de>> {
    seq_access: S,
    _phantom: PhantomData<&'de ()>,
}

impl<'de, S: SeqAccess<'de>> ListReader<'de> for SerdeListReader<'de, S> {
    type Error = DeserdeErrorWrapper<S::Error>;

    fn read_element<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<Option<T>, Self::Error> {
        let seed = SchemaSeed::<T>::new(schema);
        self.seq_access
            .next_element_seed(seed)
            .map_err(DeserdeErrorWrapper)
    }

    fn size_hint(&self) -> Option<usize> {
        self.seq_access.size_hint()
    }
}

/// Wraps serde's `MapAccess` to implement our `StructReader` trait.
pub struct SerdeStructReader<'de, M: MapAccess<'de>> {
    map_access: M,
    schema: Schema,
    _phantom: PhantomData<&'de ()>,
}

impl<'de, M: MapAccess<'de>> StructReader<'de> for SerdeStructReader<'de, M> {
    type Error = DeserdeErrorWrapper<M::Error>;

    fn read_member(&mut self) -> Result<Option<Schema>, Self::Error> {
        loop {
            match self
                .map_access
                .next_key::<String>()
                .map_err(DeserdeErrorWrapper)?
            {
                Some(key) => {
                    if let Some(member_schema) = self.schema.get_member(&key) {
                        return Ok(Some(member_schema.clone()));
                    }
                    // Unknown key — skip the value internally
                    self.map_access
                        .next_value::<serde::de::IgnoredAny>()
                        .map_err(DeserdeErrorWrapper)?;
                }
                None => return Ok(None),
            }
        }
    }

    fn read_value<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<T, Self::Error> {
        let seed = SchemaSeed::<T>::new(schema);
        self.map_access
            .next_value_seed(seed)
            .map_err(DeserdeErrorWrapper)
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.map_access
            .next_value::<serde::de::IgnoredAny>()
            .map_err(DeserdeErrorWrapper)?;
        Ok(())
    }

    fn size_hint(&self) -> Option<usize> {
        self.map_access.size_hint()
    }
}

/// Wraps serde's `MapAccess` to implement our `MapReader` trait.
pub struct SerdeMapReader<'de, M: MapAccess<'de>> {
    map_access: M,
    _phantom: PhantomData<&'de ()>,
}

impl<'de, M: MapAccess<'de>> MapReader<'de> for SerdeMapReader<'de, M> {
    type Error = DeserdeErrorWrapper<M::Error>;

    fn read_key(&mut self) -> Result<Option<String>, Self::Error> {
        self.map_access
            .next_key::<String>()
            .map_err(DeserdeErrorWrapper)
    }

    fn read_value<V: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<V, Self::Error> {
        let seed = SchemaSeed::<V>::new(schema);
        self.map_access
            .next_value_seed(seed)
            .map_err(DeserdeErrorWrapper)
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.map_access
            .next_value::<serde::de::IgnoredAny>()
            .map_err(DeserdeErrorWrapper)?;
        Ok(())
    }

    fn size_hint(&self) -> Option<usize> {
        self.map_access.size_hint()
    }
}

//========================================================================
// Context Deserializers
//========================================================================

/// A deserializer wrapping serde's `SeqAccess` for list deserialization.
///
/// This implements our `Deserializer` trait in a list context, where
/// `read_list()` returns the underlying `SerdeListReader`.
struct SeqAccessDeserializer<'de, S: SeqAccess<'de>> {
    seq_access: Option<S>,
    _phantom: PhantomData<&'de ()>,
}

impl<'de, S: SeqAccess<'de>> SeqAccessDeserializer<'de, S> {
    fn new(seq_access: S) -> Self {
        Self {
            seq_access: Some(seq_access),
            _phantom: PhantomData,
        }
    }
}

impl<'de, S: SeqAccess<'de>> Deserializer<'de> for SeqAccessDeserializer<'de, S> {
    type Error = DeserdeErrorWrapper<S::Error>;
    type StructReader<'a>
        = NeverStructReader<Self::Error>
    where
        Self: 'a;
    type ListReader<'a>
        = SerdeListReader<'de, S>
    where
        Self: 'a;
    type MapReader<'a>
        = NeverMapReader<Self::Error>
    where
        Self: 'a;

    fn read_list(&mut self) -> Result<Self::ListReader<'_>, Self::Error> {
        let seq_access = self
            .seq_access
            .take()
            .ok_or_else(|| Self::Error::custom("read_list called more than once"))?;
        Ok(SerdeListReader {
            seq_access,
            _phantom: PhantomData,
        })
    }
}

/// A deserializer wrapping serde's `MapAccess` for struct/map deserialization.
///
/// This implements our `Deserializer` trait in a map context, where
/// `read_struct()` and `read_map()` return the underlying reader.
struct MapAccessDeserializer<'de, M: MapAccess<'de>> {
    map_access: Option<M>,
    _phantom: PhantomData<&'de ()>,
}

impl<'de, M: MapAccess<'de>> MapAccessDeserializer<'de, M> {
    fn new(map_access: M) -> Self {
        Self {
            map_access: Some(map_access),
            _phantom: PhantomData,
        }
    }
}

impl<'de, M: MapAccess<'de>> Deserializer<'de> for MapAccessDeserializer<'de, M> {
    type Error = DeserdeErrorWrapper<M::Error>;
    type StructReader<'a>
        = SerdeStructReader<'de, M>
    where
        Self: 'a;
    type ListReader<'a>
        = NeverListReader<Self::Error>
    where
        Self: 'a;
    type MapReader<'a>
        = SerdeMapReader<'de, M>
    where
        Self: 'a;

    fn read_struct(&mut self, schema: &Schema) -> Result<Self::StructReader<'_>, Self::Error> {
        let map_access = self
            .map_access
            .take()
            .ok_or_else(|| Self::Error::custom("read_struct called more than once"))?;
        Ok(SerdeStructReader {
            map_access,
            schema: schema.clone(),
            _phantom: PhantomData,
        })
    }

    fn read_map(&mut self) -> Result<Self::MapReader<'_>, Self::Error> {
        let map_access = self
            .map_access
            .take()
            .ok_or_else(|| Self::Error::custom("read_map called more than once"))?;
        Ok(SerdeMapReader {
            map_access,
            _phantom: PhantomData,
        })
    }
}

//========================================================================
// Schema Seed (Public API)
//========================================================================

/// A [`DeserializeSeed`] that carries a schema to guide deserialization.
///
/// This allows us to use serde's deserialization infrastructure while
/// having our schema guide the process.
pub struct SchemaSeed<'a, T> {
    schema: &'a Schema,
    _phantom: PhantomData<T>,
}

impl<'a, T> SchemaSeed<'a, T> {
    /// Create a new [`SchemaSeed`] instance.
    pub fn new(schema: &'a Schema) -> Self {
        Self {
            schema,
            _phantom: PhantomData,
        }
    }
}

impl<'a, 'de, T> DeserializeSeed<'de> for SchemaSeed<'a, T>
where
    T: DeserializeWithSchema<'de>,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Dispatch based on schema type to appropriate serde deserialize method
        match self.schema.shape_type() {
            ShapeType::List => {
                // Tell serde we expect a sequence
                deserializer.deserialize_seq(ListVisitor {
                    schema: self.schema,
                    _phantom: PhantomData,
                })
            }
            ShapeType::Structure | ShapeType::Map | ShapeType::Union => {
                // Tell serde we expect a map/object
                deserializer.deserialize_map(MapVisitor {
                    schema: self.schema,
                    _phantom: PhantomData,
                })
            }
            ShapeType::IntEnum | ShapeType::Enum => {
                T::deserialize_with_schema(self.schema, &mut EnumWrapper::new(deserializer))
                    .map_err(|e| e.0)
            }
            // Primitives are deserialized through the PrimitiveWrapper
            ShapeType::Boolean
            | ShapeType::Byte
            | ShapeType::Short
            | ShapeType::Integer
            | ShapeType::Long
            | ShapeType::Float
            | ShapeType::Double
            | ShapeType::BigInteger
            | ShapeType::BigDecimal
            | ShapeType::String => {
                T::deserialize_with_schema(self.schema, &mut PrimitiveWrapper::new(deserializer))
                    .map_err(|e| e.0)
            }
            _ => Err(D::Error::custom(format!(
                "Unsupported shape type for deserialization: {:?}",
                self.schema.shape_type()
            ))),
        }
    }
}

//========================================================================
// Visitors
//========================================================================

/// Visitor for lists - receives a [`SeqAccess`] and creates adapter
struct ListVisitor<'a, T> {
    schema: &'a Schema,
    _phantom: PhantomData<T>,
}

impl<'a, 'de, T: DeserializeWithSchema<'de>> Visitor<'de> for ListVisitor<'a, T> {
    type Value = T;

    fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "a list")
    }

    fn visit_seq<A>(self, seq: A) -> Result<T, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut deserializer = SeqAccessDeserializer::new(seq);
        T::deserialize_with_schema(self.schema, &mut deserializer)
            .map_err(|e| A::Error::custom(format!("{}", e)))
    }
}

/// Visitor for maps, structs, and unions - receives `MapAccess` and creates adapter
struct MapVisitor<'a, T> {
    schema: &'a Schema,
    _phantom: PhantomData<T>,
}

impl<'a, 'de, T: DeserializeWithSchema<'de>> Visitor<'de> for MapVisitor<'a, T> {
    type Value = T;

    fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "a struct/map")
    }

    fn visit_map<A>(self, map: A) -> Result<T, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut deserializer = MapAccessDeserializer::new(map);
        T::deserialize_with_schema(self.schema, &mut deserializer)
            .map_err(|e| A::Error::custom(format!("{}", e)))
    }
}

//========================================================================
// Enum Deserializer
//========================================================================

/// Wraps a serde `Deserializer` for deserializing string and integer enums.
struct EnumWrapper<'de, D: serde::Deserializer<'de>> {
    deserializer: Option<D>,
    _phantom: PhantomData<&'de ()>,
}

impl<'de, D: serde::Deserializer<'de>> EnumWrapper<'de, D> {
    fn new(deserializer: D) -> Self {
        EnumWrapper {
            deserializer: Some(deserializer),
            _phantom: PhantomData,
        }
    }
}

impl<'de, D: serde::Deserializer<'de>> Deserializer<'de> for EnumWrapper<'de, D> {
    type Error = DeserdeErrorWrapper<D::Error>;
    type StructReader<'a>
        = NeverStructReader<Self::Error>
    where
        Self: 'a;
    type ListReader<'a>
        = NeverListReader<Self::Error>
    where
        Self: 'a;
    type MapReader<'a>
        = NeverMapReader<Self::Error>
    where
        Self: 'a;

    fn read_integer(&mut self, _schema: &Schema) -> Result<i32, Self::Error> {
        struct IntegerVisitor;

        impl<'de> Visitor<'de> for IntegerVisitor {
            type Value = i32;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("an integer value")
            }

            fn visit_i8<E: SerdeDeError>(self, v: i8) -> Result<Self::Value, E> {
                Ok(i32::from(v))
            }

            fn visit_i16<E: SerdeDeError>(self, v: i16) -> Result<Self::Value, E> {
                Ok(i32::from(v))
            }

            fn visit_i32<E: SerdeDeError>(self, v: i32) -> Result<Self::Value, E> {
                Ok(v)
            }

            fn visit_i64<E: SerdeDeError>(self, v: i64) -> Result<Self::Value, E> {
                v.try_into().map_err(SerdeDeError::custom)
            }

            fn visit_i128<E: SerdeDeError>(self, v: i128) -> Result<Self::Value, E> {
                v.try_into().map_err(SerdeDeError::custom)
            }

            fn visit_u8<E: SerdeDeError>(self, v: u8) -> Result<Self::Value, E> {
                Ok(i32::from(v))
            }

            fn visit_u16<E: SerdeDeError>(self, v: u16) -> Result<Self::Value, E> {
                Ok(i32::from(v))
            }

            fn visit_u32<E: SerdeDeError>(self, v: u32) -> Result<Self::Value, E> {
                v.try_into().map_err(SerdeDeError::custom)
            }

            fn visit_u64<E: SerdeDeError>(self, v: u64) -> Result<Self::Value, E> {
                v.try_into().map_err(SerdeDeError::custom)
            }

            fn visit_u128<E: SerdeDeError>(self, v: u128) -> Result<Self::Value, E> {
                v.try_into().map_err(SerdeDeError::custom)
            }
        }

        Ok(self
            .deserializer
            .take()
            .ok_or_else(|| DeserdeErrorWrapper(D::Error::custom("could not access deserializer")))?
            .deserialize_i32(IntegerVisitor)?)
    }

    fn read_string(&mut self, _schema: &Schema) -> Result<String, Self::Error> {
        struct StringVisitor;

        impl<'de> Visitor<'de> for StringVisitor {
            type Value = String;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string value")
            }

            fn visit_str<E: SerdeDeError>(self, v: &str) -> Result<Self::Value, E> {
                Ok(v.to_string())
            }

            fn visit_string<E: SerdeDeError>(self, v: String) -> Result<Self::Value, E> {
                Ok(v)
            }
        }

        Ok(self
            .deserializer
            .take()
            .ok_or_else(|| DeserdeErrorWrapper(D::Error::custom("could not access deserializer")))?
            .deserialize_string(StringVisitor)?)
    }
}

//========================================================================
// Primitive Deserializer
//========================================================================

/// Wraps a serde `Deserializer` for deserializing primitive types.
struct PrimitiveWrapper<'de, D: serde::Deserializer<'de>> {
    deserializer: Option<D>,
    _phantom: PhantomData<&'de ()>,
}

impl<'de, D: serde::Deserializer<'de>> PrimitiveWrapper<'de, D> {
    fn new(deserializer: D) -> Self {
        PrimitiveWrapper {
            deserializer: Some(deserializer),
            _phantom: PhantomData,
        }
    }

    fn take_deserializer(&mut self) -> Result<D, DeserdeErrorWrapper<D::Error>> {
        self.deserializer
            .take()
            .ok_or_else(|| DeserdeErrorWrapper(D::Error::custom("deserializer already consumed")))
    }
}

impl<'de, D: serde::Deserializer<'de>> Deserializer<'de> for PrimitiveWrapper<'de, D> {
    type Error = DeserdeErrorWrapper<D::Error>;
    type StructReader<'a>
        = NeverStructReader<Self::Error>
    where
        Self: 'a;
    type ListReader<'a>
        = NeverListReader<Self::Error>
    where
        Self: 'a;
    type MapReader<'a>
        = NeverMapReader<Self::Error>
    where
        Self: 'a;

    fn read_bool(&mut self, _schema: &Schema) -> Result<bool, Self::Error> {
        struct BoolVisitor;
        impl<'de> Visitor<'de> for BoolVisitor {
            type Value = bool;
            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str("a boolean")
            }
            fn visit_bool<E: SerdeDeError>(self, v: bool) -> Result<Self::Value, E> {
                Ok(v)
            }
        }
        self.take_deserializer()?
            .deserialize_bool(BoolVisitor)
            .map_err(DeserdeErrorWrapper)
    }

    fn read_byte(&mut self, _schema: &Schema) -> Result<i8, Self::Error> {
        struct ByteVisitor;
        impl<'de> Visitor<'de> for ByteVisitor {
            type Value = i8;
            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str("a byte (i8)")
            }
            fn visit_i8<E: SerdeDeError>(self, v: i8) -> Result<Self::Value, E> {
                Ok(v)
            }
            fn visit_i64<E: SerdeDeError>(self, v: i64) -> Result<Self::Value, E> {
                v.try_into().map_err(SerdeDeError::custom)
            }
            fn visit_u64<E: SerdeDeError>(self, v: u64) -> Result<Self::Value, E> {
                v.try_into().map_err(SerdeDeError::custom)
            }
        }
        self.take_deserializer()?
            .deserialize_i8(ByteVisitor)
            .map_err(DeserdeErrorWrapper)
    }

    fn read_short(&mut self, _schema: &Schema) -> Result<i16, Self::Error> {
        struct ShortVisitor;
        impl<'de> Visitor<'de> for ShortVisitor {
            type Value = i16;
            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str("a short (i16)")
            }
            fn visit_i16<E: SerdeDeError>(self, v: i16) -> Result<Self::Value, E> {
                Ok(v)
            }
            fn visit_i64<E: SerdeDeError>(self, v: i64) -> Result<Self::Value, E> {
                v.try_into().map_err(SerdeDeError::custom)
            }
            fn visit_u64<E: SerdeDeError>(self, v: u64) -> Result<Self::Value, E> {
                v.try_into().map_err(SerdeDeError::custom)
            }
        }
        self.take_deserializer()?
            .deserialize_i16(ShortVisitor)
            .map_err(DeserdeErrorWrapper)
    }

    fn read_integer(&mut self, _schema: &Schema) -> Result<i32, Self::Error> {
        struct IntegerVisitor;
        impl<'de> Visitor<'de> for IntegerVisitor {
            type Value = i32;
            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str("an integer (i32)")
            }
            fn visit_i32<E: SerdeDeError>(self, v: i32) -> Result<Self::Value, E> {
                Ok(v)
            }
            fn visit_i64<E: SerdeDeError>(self, v: i64) -> Result<Self::Value, E> {
                v.try_into().map_err(SerdeDeError::custom)
            }
            fn visit_u64<E: SerdeDeError>(self, v: u64) -> Result<Self::Value, E> {
                v.try_into().map_err(SerdeDeError::custom)
            }
        }
        self.take_deserializer()?
            .deserialize_i32(IntegerVisitor)
            .map_err(DeserdeErrorWrapper)
    }

    fn read_long(&mut self, _schema: &Schema) -> Result<i64, Self::Error> {
        struct LongVisitor;
        impl<'de> Visitor<'de> for LongVisitor {
            type Value = i64;
            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str("a long (i64)")
            }
            fn visit_i64<E: SerdeDeError>(self, v: i64) -> Result<Self::Value, E> {
                Ok(v)
            }
            fn visit_u64<E: SerdeDeError>(self, v: u64) -> Result<Self::Value, E> {
                v.try_into().map_err(SerdeDeError::custom)
            }
        }
        self.take_deserializer()?
            .deserialize_i64(LongVisitor)
            .map_err(DeserdeErrorWrapper)
    }

    fn read_float(&mut self, _schema: &Schema) -> Result<f32, Self::Error> {
        struct FloatVisitor;
        impl<'de> Visitor<'de> for FloatVisitor {
            type Value = f32;
            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str("a float (f32)")
            }
            fn visit_f32<E: SerdeDeError>(self, v: f32) -> Result<Self::Value, E> {
                Ok(v)
            }
            fn visit_f64<E: SerdeDeError>(self, v: f64) -> Result<Self::Value, E> {
                Ok(v as f32)
            }
            fn visit_i64<E: SerdeDeError>(self, v: i64) -> Result<Self::Value, E> {
                Ok(v as f32)
            }
            fn visit_u64<E: SerdeDeError>(self, v: u64) -> Result<Self::Value, E> {
                Ok(v as f32)
            }
        }
        self.take_deserializer()?
            .deserialize_f32(FloatVisitor)
            .map_err(DeserdeErrorWrapper)
    }

    fn read_double(&mut self, _schema: &Schema) -> Result<f64, Self::Error> {
        struct DoubleVisitor;
        impl<'de> Visitor<'de> for DoubleVisitor {
            type Value = f64;
            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str("a double (f64)")
            }
            fn visit_f64<E: SerdeDeError>(self, v: f64) -> Result<Self::Value, E> {
                Ok(v)
            }
            fn visit_i64<E: SerdeDeError>(self, v: i64) -> Result<Self::Value, E> {
                Ok(v as f64)
            }
            fn visit_u64<E: SerdeDeError>(self, v: u64) -> Result<Self::Value, E> {
                Ok(v as f64)
            }
        }
        self.take_deserializer()?
            .deserialize_f64(DoubleVisitor)
            .map_err(DeserdeErrorWrapper)
    }

    fn read_string(&mut self, _schema: &Schema) -> Result<String, Self::Error> {
        struct StringVisitor;
        impl<'de> Visitor<'de> for StringVisitor {
            type Value = String;
            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str("a string")
            }
            fn visit_str<E: SerdeDeError>(self, v: &str) -> Result<Self::Value, E> {
                Ok(v.to_string())
            }
            fn visit_string<E: SerdeDeError>(self, v: String) -> Result<Self::Value, E> {
                Ok(v)
            }
        }
        self.take_deserializer()?
            .deserialize_string(StringVisitor)
            .map_err(DeserdeErrorWrapper)
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use smithy4rs_core_derive::{SmithyShape, smithy_enum, smithy_union};

    use super::*;
    use crate::{prelude::*, smithy};

    // Test list schema
    smithy!("test#StringList": {
        list STRING_LIST_SCHEMA {
            member: STRING
        }
    });

    #[test]
    fn test_list_of_strings() {
        let json = r#"["hello", "world", "test"]"#;

        let seed = SchemaSeed::<Vec<String>>::new(&STRING_LIST_SCHEMA);
        let result: Vec<String> = seed
            .deserialize(&mut serde_json::Deserializer::from_str(json))
            .unwrap();

        assert_eq!(result, vec!["hello", "world", "test"]);
    }

    smithy!("test#OptionalFieldsStruct": {
        structure OPTIONAL_FIELDS_STRUCT_SCHEMA {
            REQUIRED: STRING = "required_field"
            OPTIONAL: STRING = "optional_field"
        }
    });

    #[derive(SmithyShape, PartialEq, Clone)]
    #[smithy_schema(OPTIONAL_FIELDS_STRUCT_SCHEMA)]
    pub struct OptionalFieldsStruct {
        #[smithy_schema(REQUIRED)]
        required_field: String,
        #[smithy_schema(OPTIONAL)]
        optional_field: Option<String>,
    }

    #[test]
    fn test_simple_struct_with_serde_json() {
        let json = r#"{
            "required_field": "hello",
            "optional_field": "world"
        }"#;

        let result: OptionalFieldsStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.required_field, "hello");
        assert_eq!(result.optional_field, Some("world".to_string()));
    }

    #[test]
    fn test_simple_struct_with_optional_none() {
        let json = r#"{
            "required_field": "hello"
        }"#;

        let result: OptionalFieldsStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.required_field, "hello");
        assert_eq!(result.optional_field, None);
    }

    // Nested struct tests
    smithy!("test#NestedStruct": {
        structure NESTED_STRUCT_SCHEMA {
            FIELD_A: STRING = "field_a"
            FIELD_B: STRING = "field_b"
        }
    });

    #[derive(SmithyShape, PartialEq, Clone)]
    #[smithy_schema(NESTED_STRUCT_SCHEMA)]
    pub struct NestedStruct {
        #[smithy_schema(FIELD_A)]
        field_a: String,
        #[smithy_schema(FIELD_B)]
        field_b: String,
    }

    // List schema for tags
    smithy!("test#TagsList": {
        list TAGS_LIST_SCHEMA {
            member: STRING
        }
    });

    smithy!("test#ParentStruct": {
        structure PARENT_STRUCT_SCHEMA {
            NAME: STRING = "name"
            NESTED: NESTED_STRUCT_SCHEMA = "nested"
            OPTIONAL_NESTED: NESTED_STRUCT_SCHEMA = "optional_nested"
            TAGS: TAGS_LIST_SCHEMA = "tags"
        }
    });

    #[derive(SmithyShape, PartialEq, Clone)]
    #[smithy_schema(PARENT_STRUCT_SCHEMA)]
    pub struct ParentStruct {
        #[smithy_schema(NAME)]
        name: String,
        #[smithy_schema(NESTED)]
        nested: NestedStruct,
        #[smithy_schema(OPTIONAL_NESTED)]
        optional_nested: Option<NestedStruct>,
        #[smithy_schema(TAGS)]
        tags: Vec<String>,
    }

    smithy!("test#MultiPrimitive": {
        structure MULTI_PRIMITIVE_SCHEMA {
            STRING_FIELD: STRING = "string_field"
            INT_FIELD: INTEGER = "int_field"
            BOOL_FIELD: BOOLEAN = "bool_field"
            FLOAT_FIELD: FLOAT = "float_field"
        }
    });

    #[derive(SmithyShape, PartialEq, Clone)]
    #[smithy_schema(MULTI_PRIMITIVE_SCHEMA)]
    pub struct MultiPrimitive {
        #[smithy_schema(STRING_FIELD)]
        string_field: String,
        #[smithy_schema(INT_FIELD)]
        int_field: i32,
        #[smithy_schema(BOOL_FIELD)]
        bool_field: bool,
        #[smithy_schema(FLOAT_FIELD)]
        float_field: f32,
    }

    #[test]
    fn test_multiple_primitives() {
        let json = r#"{
            "string_field": "test",
            "int_field": 42,
            "bool_field": true,
            "float_field": 3.1111
        }"#;

        let result: MultiPrimitive = serde_json::from_str(json).unwrap();

        assert_eq!(result.string_field, "test");
        assert_eq!(result.int_field, 42);
        assert!(result.bool_field);
        assert_eq!(result.float_field, 3.1111);
    }

    #[test]
    fn test_unknown_fields_ignored() {
        let json = r#"{
            "required_field": "hello",
            "optional_field": "world",
            "unknown_field": "should be ignored",
            "another_unknown": 123
        }"#;

        let result: OptionalFieldsStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.required_field, "hello");
        assert_eq!(result.optional_field, Some("world".to_string()));
    }

    // Test nested list in struct
    smithy!("test#StructWithList": {
        structure STRUCT_WITH_LIST_SCHEMA {
            NAME: STRING = "name"
            TAGS: STRING_LIST_SCHEMA = "tags"
        }
    });

    #[derive(SmithyShape, PartialEq, Clone)]
    #[smithy_schema(STRUCT_WITH_LIST_SCHEMA)]
    pub struct StructWithList {
        #[smithy_schema(NAME)]
        name: String,
        #[smithy_schema(TAGS)]
        tags: Vec<String>,
    }

    #[test]
    fn test_struct_with_nested_list() {
        let json = r#"{
            "name": "test",
            "tags": ["a", "b", "c"]
        }"#;

        let result: StructWithList = serde_json::from_str(json).unwrap();

        assert_eq!(result.name, "test");
        assert_eq!(result.tags, vec!["a", "b", "c"]);
    }

    // Comprehensive deep nesting test
    smithy!("test#Address": {
        structure ADDRESS_SCHEMA {
            STREET: STRING = "street"
            CITY: STRING = "city"
            ZIP: INTEGER = "zipCode"
        }
    });

    #[derive(SmithyShape, PartialEq, Clone)]
    #[smithy_schema(ADDRESS_SCHEMA)]
    pub struct Address {
        #[smithy_schema(STREET)]
        street: String,
        #[smithy_schema(CITY)]
        city: String,
        #[smithy_schema(ZIP)]
        zip_code: i32,
    }

    smithy!("test#PhoneList": {
        list PHONE_LIST_SCHEMA {
            member: STRING
        }
    });

    smithy!("test#Contact": {
        structure CONTACT_SCHEMA {
            EMAIL: STRING = "email"
            PHONES: PHONE_LIST_SCHEMA = "phones"
            ADDRESS: ADDRESS_SCHEMA = "address"
            BACKUP: ADDRESS_SCHEMA = "backupAddress"
        }
    });

    #[derive(SmithyShape, PartialEq, Clone)]
    #[smithy_schema(CONTACT_SCHEMA)]
    pub struct Contact {
        #[smithy_schema(EMAIL)]
        email: String,
        #[smithy_schema(PHONES)]
        phones: Vec<String>,
        #[smithy_schema(ADDRESS)]
        address: Address,
        #[smithy_schema(BACKUP)]
        backup_address: Option<Address>,
    }

    smithy!("test#Hobby": {
        structure HOBBY_SCHEMA {
            NAME: STRING = "name"
            YEARS: INTEGER = "yearsOfExperience"
        }
    });

    #[derive(SmithyShape, PartialEq, Clone)]
    #[smithy_schema(HOBBY_SCHEMA)]
    pub struct Hobby {
        #[smithy_schema(NAME)]
        name: String,
        #[smithy_schema(YEARS)]
        years_of_experience: i32,
    }

    smithy!("test#HobbyList": {
        list HOBBY_LIST_SCHEMA {
            member: HOBBY_SCHEMA
        }
    });

    smithy!("test#StringMap": {
        map STRING_MAP_SCHEMA {
            key: STRING
            value: STRING
        }
    });

    smithy!("test#Person": {
        structure PERSON_SCHEMA {
            NAME: STRING = "name"
            AGE: INTEGER = "age"
            ACTIVE: BOOLEAN = "isActive"
            SCORE: FLOAT = "score"
            CONTACT: CONTACT_SCHEMA = "contact"
            HOBBIES: HOBBY_LIST_SCHEMA = "hobbies"
            METADATA: STRING_MAP_SCHEMA = "metadata"
            NOTES: STRING = "notes"
        }
    });

    #[derive(SmithyShape, PartialEq, Clone)]
    #[smithy_schema(PERSON_SCHEMA)]
    pub struct Person {
        #[smithy_schema(NAME)]
        name: String,
        #[smithy_schema(AGE)]
        age: i32,
        #[smithy_schema(ACTIVE)]
        is_active: bool,
        #[smithy_schema(SCORE)]
        score: f32,
        #[smithy_schema(CONTACT)]
        contact: Contact,
        #[smithy_schema(HOBBIES)]
        hobbies: Vec<Hobby>,
        #[smithy_schema(METADATA)]
        metadata: IndexMap<String, String>,
        #[smithy_schema(NOTES)]
        notes: Option<String>,
    }

    #[test]
    fn test_comprehensive_nested_structures() {
        let json = r#"{
            "name": "Alice Johnson",
            "age": 32,
            "isActive": true,
            "score": 95.5,
            "contact": {
                "email": "alice@example.com",
                "phones": ["+1-555-0100", "+1-555-0101"],
                "address": {
                    "street": "123 Main St",
                    "city": "Springfield",
                    "zipCode": 12345
                },
                "backupAddress": {
                    "street": "456 Oak Ave",
                    "city": "Shelbyville",
                    "zipCode": 67890
                }
            },
            "hobbies": [
                {
                    "name": "Photography",
                    "yearsOfExperience": 5
                },
                {
                    "name": "Rock Climbing",
                    "yearsOfExperience": 3
                }
            ],
            "metadata": {
                "department": "Engineering",
                "team": "Backend",
                "location": "Remote"
            },
            "notes": "Excellent performance",
            "unknownField": "should be ignored"
        }"#;

        let result: Person = serde_json::from_str(json).unwrap();

        // Verify top-level fields
        assert_eq!(result.name, "Alice Johnson");
        assert_eq!(result.age, 32);
        assert!(result.is_active);
        assert_eq!(result.score, 95.5);

        // Verify nested contact
        assert_eq!(result.contact.email, "alice@example.com");
        assert_eq!(result.contact.phones, vec!["+1-555-0100", "+1-555-0101"]);

        // Verify nested address
        assert_eq!(result.contact.address.street, "123 Main St");
        assert_eq!(result.contact.address.city, "Springfield");
        assert_eq!(result.contact.address.zip_code, 12345);

        // Verify optional nested address
        assert!(result.contact.backup_address.is_some());
        let backup = result.contact.backup_address.unwrap();
        assert_eq!(backup.street, "456 Oak Ave");
        assert_eq!(backup.city, "Shelbyville");
        assert_eq!(backup.zip_code, 67890);

        // Verify list of structs
        assert_eq!(result.hobbies.len(), 2);
        assert_eq!(result.hobbies[0].name, "Photography");
        assert_eq!(result.hobbies[0].years_of_experience, 5);
        assert_eq!(result.hobbies[1].name, "Rock Climbing");
        assert_eq!(result.hobbies[1].years_of_experience, 3);

        // Verify map
        assert_eq!(result.metadata.len(), 3);
        assert_eq!(
            result.metadata.get("department"),
            Some(&"Engineering".to_string())
        );
        assert_eq!(result.metadata.get("team"), Some(&"Backend".to_string()));
        assert_eq!(result.metadata.get("location"), Some(&"Remote".to_string()));

        // Verify optional field
        assert_eq!(result.notes, Some("Excellent performance".to_string()));
    }

    #[test]
    fn test_comprehensive_with_missing_optional_fields() {
        let json = r#"{
            "name": "Bob Smith",
            "age": 28,
            "isActive": false,
            "score": 87.3,
            "contact": {
                "email": "bob@example.com",
                "phones": [],
                "address": {
                    "street": "789 Elm St",
                    "city": "Capital City",
                    "zipCode": 54321
                }
            },
            "hobbies": [],
            "metadata": {}
        }"#;

        let result: Person = serde_json::from_str(json).unwrap();

        assert_eq!(result.name, "Bob Smith");
        assert_eq!(result.age, 28);
        assert!(!result.is_active);
        assert_eq!(result.score, 87.3);
        assert_eq!(result.contact.email, "bob@example.com");
        assert_eq!(result.contact.phones, Vec::<String>::new());
        assert!(result.contact.backup_address.is_none());
        assert_eq!(result.hobbies, Vec::<Hobby>::new());
        assert_eq!(result.metadata, IndexMap::<String, String>::new());
        assert!(result.notes.is_none());
    }

    smithy!("test#IpAddr": {
        union IP_ADDR {
            V4: STRING = "v4"
            V6: STRING = "v6"
        }
    });

    #[smithy_union]
    #[derive(SmithyShape, PartialEq)]
    #[smithy_schema(IP_ADDR)]
    pub enum IpAddr {
        #[smithy_schema(V4)]
        V4(String),
        #[smithy_schema(V6)]
        V6(String),
    }

    #[test]
    fn test_union_deserialize() {
        let json = r#"{
            "v4": "192.168.5.0"
        }"#;
        let result: IpAddr = serde_json::from_str(json).unwrap();
        let IpAddr::V4(value) = result else {
            panic!("Expected v4 address")
        };
        assert_eq!(value, "192.168.5.0")
    }

    smithy!("test#StringEnum": {
        enum A_OR_B {
            A = "a"
            B = "b"
        }
    });

    #[smithy_enum]
    #[derive(SmithyShape)]
    #[smithy_schema(A_OR_B)]
    pub enum AorB {
        A = "a",
        B = "b",
    }

    #[test]
    fn test_enum_deserialize() {
        let json = r#""a""#;
        let result: AorB = serde_json::from_str(json).unwrap();
        let AorB::A = result else {
            panic!("Expected a")
        };
    }

    smithy!("test#IntEnum": {
        intEnum C_OR_D {
            C = 1
            D = 2
        }
    });

    #[smithy_enum]
    #[derive(SmithyShape)]
    #[smithy_schema(C_OR_D)]
    pub enum CorD {
        C = 1,
        D = 2,
    }

    #[test]
    fn test_int_enum_deserialize() {
        let json = "2";
        let result: CorD = serde_json::from_str(json).unwrap();
        let CorD::D = result else {
            panic!("Expected D")
        };
    }
}
