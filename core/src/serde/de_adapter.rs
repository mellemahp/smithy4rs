#![allow(dead_code)]

use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
};

use serde::de::{DeserializeSeed, Error as SerdeDeError, MapAccess, SeqAccess, Visitor};

use crate::{
    schema::{SchemaRef, ShapeType, get_shape_type},
    serde::deserializers::{DeserializeWithSchema, Error as DeserError},
};

/// Error wrapper to bridge serde errors with our error type
#[derive(Debug)]
pub struct DeserdeErrorWrapper<E: SerdeDeError>(E);

impl<E: SerdeDeError> Display for DeserdeErrorWrapper<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

/// A DeserializeSeed that carries a schema to guide deserialization.
///
/// This allows us to use serde's deserialization infrastructure while
/// having our schema guide the process.
pub struct SchemaSeed<'a, T> {
    schema: &'a SchemaRef,
    _phantom: PhantomData<T>,
}

impl<'a, T> SchemaSeed<'a, T> {
    pub fn new(schema: &'a SchemaRef) -> Self {
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
        // Dispatch based on schema type
        match get_shape_type(self.schema).map_err(D::Error::custom)? {
            ShapeType::Boolean => {
                // Primitives should be deserialized through MapAccess/SeqAccess next_value()
                Err(D::Error::custom(
                    "Boolean primitives cannot be deserialized through SchemaSeed directly",
                ))
            }

            ShapeType::Byte => Err(D::Error::custom(
                "Byte primitives cannot be deserialized through SchemaSeed directly",
            )),

            ShapeType::Short => Err(D::Error::custom(
                "Short primitives cannot be deserialized through SchemaSeed directly",
            )),

            ShapeType::Integer | ShapeType::IntEnum => Err(D::Error::custom(
                "Integer primitives cannot be deserialized through SchemaSeed directly",
            )),

            ShapeType::Long => Err(D::Error::custom(
                "Long primitives cannot be deserialized through SchemaSeed directly",
            )),

            ShapeType::Float => Err(D::Error::custom(
                "Float primitives cannot be deserialized through SchemaSeed directly",
            )),

            ShapeType::Double => Err(D::Error::custom(
                "Double primitives cannot be deserialized through SchemaSeed directly",
            )),

            ShapeType::String | ShapeType::Enum => Err(D::Error::custom(
                "String primitives cannot be deserialized through SchemaSeed directly",
            )),

            ShapeType::Structure | ShapeType::Union => {
                // For structs, we need a visitor that handles MapAccess
                struct StructVisitor<'a, T> {
                    schema: &'a SchemaRef,
                    _phantom: PhantomData<T>,
                }

                impl<'a, 'de, T: DeserializeWithSchema<'de>> Visitor<'de> for StructVisitor<'a, T> {
                    type Value = T;

                    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                        write!(f, "struct {}", self.schema.id().name())
                    }

                    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<T, A::Error> {
                        // Create an adapter that bridges MapAccess to our Deserializer
                        let mut adapter = MapAccessAdapter {
                            map_access: &mut map,
                            schema: self.schema,
                            _phantom: PhantomData,
                        };

                        // Use our deserialization system
                        T::deserialize_with_schema(self.schema, &mut adapter)
                            .map_err(|e| A::Error::custom(format!("Deserialization error: {}", e)))
                    }
                }

                deserializer.deserialize_map(StructVisitor {
                    schema: self.schema,
                    _phantom: PhantomData,
                })
            }

            ShapeType::List => {
                // For lists, we need a visitor that handles SeqAccess
                struct ListVisitor<'a, T> {
                    schema: &'a SchemaRef,
                    _phantom: PhantomData<T>,
                }

                impl<'a, 'de, T: DeserializeWithSchema<'de>> Visitor<'de> for ListVisitor<'a, T> {
                    type Value = T;

                    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                        write!(f, "list")
                    }

                    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<T, A::Error> {
                        // Create an adapter that bridges SeqAccess to our Deserializer
                        let mut adapter = SeqAccessAdapter {
                            seq_access: &mut seq,
                            schema: self.schema,
                            _phantom: PhantomData,
                        };

                        // Use our deserialization system
                        T::deserialize_with_schema(self.schema, &mut adapter)
                            .map_err(|e| A::Error::custom(format!("Deserialization error: {}", e)))
                    }
                }

                deserializer.deserialize_seq(ListVisitor {
                    schema: self.schema,
                    _phantom: PhantomData,
                })
            }

            ShapeType::Map => {
                // Maps in JSON are also represented as objects (MapAccess)
                struct MapVisitor<'a, T> {
                    schema: &'a SchemaRef,
                    _phantom: PhantomData<T>,
                }

                impl<'a, 'de, T: DeserializeWithSchema<'de>> Visitor<'de> for MapVisitor<'a, T> {
                    type Value = T;

                    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                        write!(f, "map")
                    }

                    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<T, A::Error> {
                        let mut adapter = MapAccessAdapter {
                            map_access: &mut map,
                            schema: self.schema,
                            _phantom: PhantomData,
                        };

                        T::deserialize_with_schema(self.schema, &mut adapter)
                            .map_err(|e| A::Error::custom(format!("Deserialization error: {}", e)))
                    }
                }

                deserializer.deserialize_map(MapVisitor {
                    schema: self.schema,
                    _phantom: PhantomData,
                })
            }

            _ => Err(D::Error::custom(format!(
                "Unsupported shape type for deserialization: {:?}",
                get_shape_type(self.schema)
            ))),
        }
    }
}

// Adapters that bridge serde's access types to our Deserializer trait

struct MapAccessAdapter<'a, 'de, M: MapAccess<'de>> {
    map_access: &'a mut M,
    schema: &'a SchemaRef,
    _phantom: PhantomData<&'de ()>,
}

struct SeqAccessAdapter<'a, 'de, S: SeqAccess<'de>> {
    seq_access: &'a mut S,
    schema: &'a SchemaRef,
    _phantom: PhantomData<&'de ()>,
}

use crate::{
    BigDecimal, BigInt, ByteBuffer, Instant, schema::Document,
    serde::deserializers::Deserializer as OurDeserializer,
};

impl<'a, 'de, M: MapAccess<'de>> OurDeserializer<'de> for MapAccessAdapter<'a, 'de, M> {
    type Error = DeserdeErrorWrapper<M::Error>;

    fn read_struct<B, F>(
        &mut self,
        schema: &SchemaRef,
        mut builder: B,
        mut consumer: F,
    ) -> Result<B, Self::Error>
    where
        F: FnMut(B, &SchemaRef, &mut Self) -> Result<B, Self::Error>,
    {
        // Iterate through the map using MapAccess
        while let Some(key) = self.map_access.next_key::<String>()? {
            // Look up the member schema
            if let Some(member_schema) = schema.get_member(&key) {
                // Get the target schema (dereference member to get the actual shape)
                let target_schema = if let Some(member) = member_schema.as_member() {
                    &*member.target
                } else {
                    member_schema
                };

                // Check if this is a nested structure
                let is_nested_struct = matches!(
                    get_shape_type(target_schema),
                    Ok(ShapeType::Structure | ShapeType::Union)
                );

                if is_nested_struct {
                    // For nested structures, we use next_value_seed with a custom seed.
                    // The seed will create a new MapAccessAdapter for the nested map
                    // and call the consumer with it.
                    struct NestedStructSeed<'a, B, F> {
                        target_schema: &'a SchemaRef,
                        member_schema: &'a SchemaRef,
                        builder: Option<B>,
                        consumer: F,
                    }

                    impl<'a, 'de, B, F> DeserializeSeed<'de> for NestedStructSeed<'a, B, F>
                    where
                        B: 'de,
                        F: FnMut(
                            B,
                            &'a SchemaRef,
                            &mut dyn ErasedDeserializer<'de>,
                        ) -> Result<B, Box<dyn StdError + 'de>>,
                    {
                        type Value = B;

                        fn deserialize<D>(mut self, deserializer: D) -> Result<B, D::Error>
                        where
                            D: serde::Deserializer<'de>,
                        {
                            struct NestedVisitor<'a, B, F> {
                                target_schema: &'a SchemaRef,
                                member_schema: &'a SchemaRef,
                                builder: Option<B>,
                                consumer: F,
                            }

                            impl<'a, 'de, B, F> Visitor<'de> for NestedVisitor<'a, B, F>
                            where
                                B: 'de,
                                F: FnMut(
                                    B,
                                    &'a SchemaRef,
                                    &mut dyn ErasedDeserializer<'de>,
                                )
                                    -> Result<B, Box<dyn StdError + 'de>>,
                            {
                                type Value = B;

                                fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                                    write!(f, "a map for nested struct")
                                }

                                fn visit_map<A: MapAccess<'de>>(
                                    mut self,
                                    mut map: A,
                                ) -> Result<B, A::Error> {
                                    let mut adapter = MapAccessAdapter {
                                        map_access: &mut map,
                                        schema: self.target_schema,
                                        _phantom: PhantomData,
                                    };

                                    // Create an erased version of the adapter
                                    let mut erased: &mut dyn ErasedDeserializer<'de> = &mut adapter;

                                    let builder = self.builder.take().unwrap();
                                    (self.consumer)(builder, self.member_schema, erased).map_err(
                                        |e| A::Error::custom(format!("Nested struct error: {}", e)),
                                    )
                                }
                            }

                            deserializer.deserialize_map(NestedVisitor {
                                target_schema: self.target_schema,
                                member_schema: self.member_schema,
                                builder: self.builder.take(),
                                consumer: self.consumer,
                            })
                        }
                    }

                    // Create an erased consumer that the seed can use
                    let erased_consumer =
                        |b: B, s: &SchemaRef, d: &mut dyn ErasedDeserializer<'de>| {
                            // Call deserialize_nested which will invoke the consumer with the correct adapter type
                            d.deserialize_nested(b, s, &mut |b2, s2, adapter| {
                                consumer(b2, s2, adapter)
                                    .map_err(|e| Box::new(e) as Box<dyn StdError + 'de>)
                            })
                        };

                    let seed = NestedStructSeed {
                        target_schema,
                        member_schema,
                        builder: Some(builder),
                        consumer: erased_consumer,
                    };

                    builder = self.map_access.next_value_seed(seed)?;
                } else {
                    // For primitives, call the consumer normally
                    builder = consumer(builder, member_schema, self)?;
                }
            } else {
                // Unknown field - skip it
                self.map_access.next_value::<serde::de::IgnoredAny>()?;
            }
        }

        Ok(builder)
    }

    fn read_bool(&mut self, _schema: &SchemaRef) -> Result<bool, Self::Error> {
        self.map_access.next_value().map_err(DeserdeErrorWrapper)
    }

    fn read_byte(&mut self, _schema: &SchemaRef) -> Result<i8, Self::Error> {
        self.map_access.next_value().map_err(DeserdeErrorWrapper)
    }

    fn read_short(&mut self, _schema: &SchemaRef) -> Result<i16, Self::Error> {
        self.map_access.next_value().map_err(DeserdeErrorWrapper)
    }

    fn read_integer(&mut self, _schema: &SchemaRef) -> Result<i32, Self::Error> {
        self.map_access.next_value().map_err(DeserdeErrorWrapper)
    }

    fn read_long(&mut self, _schema: &SchemaRef) -> Result<i64, Self::Error> {
        self.map_access.next_value().map_err(DeserdeErrorWrapper)
    }

    fn read_float(&mut self, _schema: &SchemaRef) -> Result<f32, Self::Error> {
        self.map_access.next_value().map_err(DeserdeErrorWrapper)
    }

    fn read_double(&mut self, _schema: &SchemaRef) -> Result<f64, Self::Error> {
        self.map_access.next_value().map_err(DeserdeErrorWrapper)
    }

    fn read_string(&mut self, _schema: &SchemaRef) -> Result<String, Self::Error> {
        self.map_access.next_value().map_err(DeserdeErrorWrapper)
    }

    fn read_big_integer(&mut self, _schema: &SchemaRef) -> Result<BigInt, Self::Error> {
        Err(DeserError::custom(
            "BigInteger deserialization not yet implemented",
        ))
    }

    fn read_big_decimal(&mut self, _schema: &SchemaRef) -> Result<BigDecimal, Self::Error> {
        Err(DeserError::custom(
            "BigDecimal deserialization not yet implemented",
        ))
    }

    fn read_blob(&mut self, _schema: &SchemaRef) -> Result<ByteBuffer, Self::Error> {
        Err(DeserError::custom(
            "Blob deserialization not yet implemented",
        ))
    }

    fn read_timestamp(&mut self, _schema: &SchemaRef) -> Result<Instant, Self::Error> {
        Err(DeserError::custom(
            "Timestamp deserialization not yet implemented",
        ))
    }

    fn read_document(&mut self, _schema: &SchemaRef) -> Result<Document, Self::Error> {
        Err(DeserError::custom(
            "Document deserialization not yet implemented",
        ))
    }

    fn read_list<T, F>(
        &mut self,
        _schema: &SchemaRef,
        _state: &mut T,
        _consumer: F,
    ) -> Result<(), Self::Error>
    where
        F: FnMut(&mut T, &SchemaRef, &mut Self) -> Result<(), Self::Error>,
    {
        // Nested list - use SchemaSeed with SeqAccess
        Err(DeserError::custom(
            "Nested list deserialization not yet fully implemented",
        ))
    }

    fn read_map<T, F>(
        &mut self,
        _schema: &SchemaRef,
        _state: &mut T,
        _consumer: F,
    ) -> Result<(), Self::Error>
    where
        F: FnMut(&mut T, String, &mut Self) -> Result<(), Self::Error>,
    {
        // Nested map - use SchemaSeed with MapAccess
        Err(DeserError::custom(
            "Nested map deserialization not yet fully implemented",
        ))
    }

    fn is_null(&mut self) -> bool {
        false
    }

    fn read_null(&mut self) -> Result<(), Self::Error> {
        self.map_access
            .next_value::<()>()
            .map_err(DeserdeErrorWrapper)
    }
}

impl<'a, 'de, S: SeqAccess<'de>> OurDeserializer<'de> for SeqAccessAdapter<'a, 'de, S> {
    type Error = DeserdeErrorWrapper<S::Error>;

    fn read_list<T, F>(
        &mut self,
        schema: &SchemaRef,
        _state: &mut T,
        _consumer: F,
    ) -> Result<(), Self::Error>
    where
        F: FnMut(&mut T, &SchemaRef, &mut Self) -> Result<(), Self::Error>,
    {
        let _member_schema = schema
            .get_member("member")
            .ok_or_else(|| DeserdeErrorWrapper::<S::Error>::custom("List schema missing member"))?;

        // Iterate through the sequence
        while self
            .seq_access
            .next_element::<serde::de::IgnoredAny>()?
            .is_some()
        {
            // TODO: We need to call consumer, but this consumes the element
            // Need to rethink this
        }

        Err(DeserError::custom(
            "SeqAccess list iteration not yet fully implemented",
        ))
    }

    fn read_struct<B, F>(
        &mut self,
        _schema: &SchemaRef,
        _builder: B,
        _consumer: F,
    ) -> Result<B, Self::Error>
    where
        F: FnMut(B, &SchemaRef, &mut Self) -> Result<B, Self::Error>,
    {
        Err(DeserError::custom(
            "Cannot read struct from sequence context",
        ))
    }

    fn read_bool(&mut self, _schema: &SchemaRef) -> Result<bool, Self::Error> {
        Err(DeserError::custom(
            "Cannot read primitives from sequence context",
        ))
    }

    fn read_byte(&mut self, _schema: &SchemaRef) -> Result<i8, Self::Error> {
        Err(DeserError::custom(
            "Cannot read primitives from sequence context",
        ))
    }

    fn read_short(&mut self, _schema: &SchemaRef) -> Result<i16, Self::Error> {
        Err(DeserError::custom(
            "Cannot read primitives from sequence context",
        ))
    }

    fn read_integer(&mut self, _schema: &SchemaRef) -> Result<i32, Self::Error> {
        Err(DeserError::custom(
            "Cannot read primitives from sequence context",
        ))
    }

    fn read_long(&mut self, _schema: &SchemaRef) -> Result<i64, Self::Error> {
        Err(DeserError::custom(
            "Cannot read primitives from sequence context",
        ))
    }

    fn read_float(&mut self, _schema: &SchemaRef) -> Result<f32, Self::Error> {
        Err(DeserError::custom(
            "Cannot read primitives from sequence context",
        ))
    }

    fn read_double(&mut self, _schema: &SchemaRef) -> Result<f64, Self::Error> {
        Err(DeserError::custom(
            "Cannot read primitives from sequence context",
        ))
    }

    fn read_string(&mut self, _schema: &SchemaRef) -> Result<String, Self::Error> {
        Err(DeserError::custom(
            "Cannot read primitives from sequence context",
        ))
    }

    fn read_big_integer(&mut self, _schema: &SchemaRef) -> Result<BigInt, Self::Error> {
        Err(DeserError::custom(
            "Cannot read primitives from sequence context",
        ))
    }

    fn read_big_decimal(&mut self, _schema: &SchemaRef) -> Result<BigDecimal, Self::Error> {
        Err(DeserError::custom(
            "Cannot read primitives from sequence context",
        ))
    }

    fn read_blob(&mut self, _schema: &SchemaRef) -> Result<ByteBuffer, Self::Error> {
        Err(DeserError::custom(
            "Cannot read primitives from sequence context",
        ))
    }

    fn read_timestamp(&mut self, _schema: &SchemaRef) -> Result<Instant, Self::Error> {
        Err(DeserError::custom(
            "Cannot read primitives from sequence context",
        ))
    }

    fn read_document(&mut self, _schema: &SchemaRef) -> Result<Document, Self::Error> {
        Err(DeserError::custom(
            "Cannot read primitives from sequence context",
        ))
    }

    fn read_map<T, F>(
        &mut self,
        _schema: &SchemaRef,
        _state: &mut T,
        _consumer: F,
    ) -> Result<(), Self::Error>
    where
        F: FnMut(&mut T, String, &mut Self) -> Result<(), Self::Error>,
    {
        Err(DeserError::custom("Cannot read map from sequence context"))
    }

    fn is_null(&mut self) -> bool {
        false
    }

    fn read_null(&mut self) -> Result<(), Self::Error> {
        Err(DeserError::custom("Cannot read null from sequence context"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithy4rs_core_derive::{DeserializableStruct, SchemaShape};

    use crate::{
        lazy_schema,
        prelude::*,
        schema::{Schema, ShapeId, StaticSchemaShape},
        traits,
    };

    lazy_schema!(
        OPTIONAL_FIELDS_STRUCT_SCHEMA,
        Schema::structure_builder(ShapeId::from("test#OptionalFieldsStruct"), traits![]),
        (OPTIONAL_REQUIRED, "required_field", STRING, traits![]),
        (OPTIONAL_OPTIONAL, "optional_field", STRING, traits![])
    );

    #[derive(SchemaShape, DeserializableStruct, Debug, PartialEq)]
    #[smithy_schema(OPTIONAL_FIELDS_STRUCT_SCHEMA)]
    struct OptionalFieldsStruct {
        #[smithy_schema(OPTIONAL_REQUIRED)]
        required_field: String,
        #[smithy_schema(OPTIONAL_OPTIONAL)]
        optional_field: Option<String>,
    }

    impl<'de> serde::Deserialize<'de> for OptionalFieldsStruct {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let seed = SchemaSeed::<OptionalFieldsStruct>::new(OptionalFieldsStruct::schema());
            seed.deserialize(deserializer)
        }
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
    lazy_schema!(
        NESTED_STRUCT_SCHEMA,
        Schema::structure_builder(ShapeId::from("test#NestedStruct"), traits![]),
        (NESTED_FIELD_A, "field_a", STRING, traits![]),
        (NESTED_FIELD_B, "field_b", STRING, traits![])
    );

    #[derive(SchemaShape, DeserializableStruct, Debug, PartialEq)]
    #[smithy_schema(NESTED_STRUCT_SCHEMA)]
    struct NestedStruct {
        #[smithy_schema(NESTED_FIELD_A)]
        field_a: String,
        #[smithy_schema(NESTED_FIELD_B)]
        field_b: String,
    }

    impl<'de> serde::Deserialize<'de> for NestedStruct {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let seed = SchemaSeed::<NestedStruct>::new(NestedStruct::schema());
            seed.deserialize(deserializer)
        }
    }

    lazy_schema!(
        PARENT_STRUCT_SCHEMA,
        Schema::structure_builder(ShapeId::from("test#ParentStruct"), traits![]),
        (PARENT_NAME, "name", STRING, traits![]),
        (PARENT_NESTED, "nested", NESTED_STRUCT_SCHEMA, traits![]),
        (
            PARENT_OPTIONAL_NESTED,
            "optional_nested",
            NESTED_STRUCT_SCHEMA,
            traits![]
        )
    );

    #[derive(SchemaShape, DeserializableStruct, Debug, PartialEq)]
    #[smithy_schema(PARENT_STRUCT_SCHEMA)]
    struct ParentStruct {
        #[smithy_schema(PARENT_NAME)]
        name: String,
        #[smithy_schema(PARENT_NESTED)]
        nested: NestedStruct,
        #[smithy_schema(PARENT_OPTIONAL_NESTED)]
        optional_nested: Option<NestedStruct>,
    }

    impl<'de> serde::Deserialize<'de> for ParentStruct {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let seed = SchemaSeed::<ParentStruct>::new(ParentStruct::schema());
            seed.deserialize(deserializer)
        }
    }

    #[test]
    fn test_nested_struct() {
        let json = r#"{
            "name": "parent",
            "nested": {
                "field_a": "value_a",
                "field_b": "value_b"
            },
            "optional_nested": {
                "field_a": "optional_a",
                "field_b": "optional_b"
            }
        }"#;

        let result: ParentStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.name, "parent");
        assert_eq!(result.nested.field_a, "value_a");
        assert_eq!(result.nested.field_b, "value_b");
        assert!(result.optional_nested.is_some());
        let optional = result.optional_nested.unwrap();
        assert_eq!(optional.field_a, "optional_a");
        assert_eq!(optional.field_b, "optional_b");
    }

    #[test]
    fn test_nested_struct_with_optional_none() {
        let json = r#"{
            "name": "parent",
            "nested": {
                "field_a": "value_a",
                "field_b": "value_b"
            }
        }"#;

        let result: ParentStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.name, "parent");
        assert_eq!(result.nested.field_a, "value_a");
        assert_eq!(result.nested.field_b, "value_b");
        assert!(result.optional_nested.is_none());
    }

    #[test]
    fn test_deeply_nested_struct() {
        lazy_schema!(
            DEEPLY_NESTED_SCHEMA,
            Schema::structure_builder(ShapeId::from("test#DeeplyNested"), traits![]),
            (DEEPLY_NAME, "name", STRING, traits![]),
            (DEEPLY_PARENT, "parent", PARENT_STRUCT_SCHEMA, traits![])
        );

        #[derive(SchemaShape, DeserializableStruct, Debug, PartialEq)]
        #[smithy_schema(DEEPLY_NESTED_SCHEMA)]
        struct DeeplyNested {
            #[smithy_schema(DEEPLY_NAME)]
            name: String,
            #[smithy_schema(DEEPLY_PARENT)]
            parent: ParentStruct,
        }

        impl<'de> serde::Deserialize<'de> for DeeplyNested {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let seed = SchemaSeed::<DeeplyNested>::new(DeeplyNested::schema());
                seed.deserialize(deserializer)
            }
        }

        let json = r#"{
            "name": "deep",
            "parent": {
                "name": "parent",
                "nested": {
                    "field_a": "value_a",
                    "field_b": "value_b"
                },
                "optional_nested": {
                    "field_a": "optional_a",
                    "field_b": "optional_b"
                }
            }
        }"#;

        let result: DeeplyNested = serde_json::from_str(json).unwrap();

        assert_eq!(result.name, "deep");
        assert_eq!(result.parent.name, "parent");
        assert_eq!(result.parent.nested.field_a, "value_a");
        assert_eq!(result.parent.nested.field_b, "value_b");
        assert!(result.parent.optional_nested.is_some());
    }

    #[test]
    fn test_multiple_primitives() {
        lazy_schema!(
            MULTI_PRIMITIVE_SCHEMA,
            Schema::structure_builder(ShapeId::from("test#MultiPrimitive"), traits![]),
            (MULTI_STRING, "string_field", STRING, traits![]),
            (MULTI_INT, "int_field", INTEGER, traits![]),
            (MULTI_BOOL, "bool_field", BOOLEAN, traits![]),
            (MULTI_FLOAT, "float_field", FLOAT, traits![])
        );

        #[derive(SchemaShape, DeserializableStruct, Debug, PartialEq)]
        #[smithy_schema(MULTI_PRIMITIVE_SCHEMA)]
        struct MultiPrimitive {
            #[smithy_schema(MULTI_STRING)]
            string_field: String,
            #[smithy_schema(MULTI_INT)]
            int_field: i32,
            #[smithy_schema(MULTI_BOOL)]
            bool_field: bool,
            #[smithy_schema(MULTI_FLOAT)]
            float_field: f32,
        }

        impl<'de> serde::Deserialize<'de> for MultiPrimitive {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let seed = SchemaSeed::<MultiPrimitive>::new(MultiPrimitive::schema());
                seed.deserialize(deserializer)
            }
        }

        let json = r#"{
            "string_field": "test",
            "int_field": 42,
            "bool_field": true,
            "float_field": 3.14
        }"#;

        let result: MultiPrimitive = serde_json::from_str(json).unwrap();

        assert_eq!(result.string_field, "test");
        assert_eq!(result.int_field, 42);
        assert_eq!(result.bool_field, true);
        assert_eq!(result.float_field, 3.14);
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
}
