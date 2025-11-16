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
        // Dispatch based on schema type to appropriate serde deserialize method
        match get_shape_type(self.schema).map_err(D::Error::custom)? {
            ShapeType::List => {
                // Tell serde we expect a sequence
                deserializer.deserialize_seq(ListVisitor {
                    schema: self.schema,
                    _phantom: PhantomData,
                })
            }
            ShapeType::Structure => {
                // Tell serde we expect a map/object
                deserializer.deserialize_map(StructVisitor {
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

// Visitor for lists - receives SeqAccess and creates adapter
struct ListVisitor<'a, T> {
    schema: &'a SchemaRef,
    _phantom: PhantomData<T>,
}

impl<'a, 'de, T: DeserializeWithSchema<'de>> Visitor<'de> for ListVisitor<'a, T> {
    type Value = T;

    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "a list")
    }

    fn visit_seq<A>(self, seq: A) -> Result<T, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Create adapter from SeqAccess
        let mut adapter = SeqAccessAdapter {
            seq_access: seq,
            end_of_sequence: false,
            _phantom: PhantomData,
        };

        // Call our deserialization
        T::deserialize_with_schema(self.schema, &mut adapter)
            .map_err(|e| A::Error::custom(format!("Deserialization error: {}", e)))
    }
}

// SeqAccessAdapter wraps serde's SeqAccess and implements our Deserializer trait
struct SeqAccessAdapter<'de, S: SeqAccess<'de>> {
    seq_access: S,
    /// Flag to track when next_element() returns None (end of sequence)
    end_of_sequence: bool,
    _phantom: PhantomData<&'de ()>,
}

impl<'de, S: SeqAccess<'de>> crate::serde::deserializers::Deserializer<'de>
    for SeqAccessAdapter<'de, S>
{
    type Error = DeserdeErrorWrapper<S::Error>;

    fn read_list<T, F>(
        &mut self,
        schema: &SchemaRef,
        state: &mut T,
        mut consumer: F,
    ) -> Result<(), Self::Error>
    where
        F: FnMut(&mut T, &SchemaRef, &mut Self) -> Result<(), Self::Error>,
    {
        // Get the element schema
        let member_schema = schema
            .get_member("member")
            .ok_or_else(|| Self::Error::custom("List schema missing member"))?;

        // Iterate through all elements
        loop {
            // Reset the flag before each iteration
            self.end_of_sequence = false;

            // Call the consumer to deserialize one element
            // The consumer will call back into our read_* methods
            let result = consumer(state, member_schema, self);

            // Check if we reached the end of the sequence
            // This flag is set by read_* methods when next_element() returns None
            if self.end_of_sequence {
                // End of sequence is a normal termination, not an error
                break;
            }

            // If there was an actual error (not end-of-sequence), propagate it
            result?;
        }

        Ok(())
    }

    // Primitives - call next_element on SeqAccess
    fn read_string(&mut self, _schema: &SchemaRef) -> Result<String, Self::Error> {
        match self.seq_access.next_element()? {
            Some(value) => Ok(value),
            None => {
                self.end_of_sequence = true;
                Err(Self::Error::custom("End of sequence"))
            }
        }
    }

    fn read_integer(&mut self, _schema: &SchemaRef) -> Result<i32, Self::Error> {
        match self.seq_access.next_element()? {
            Some(value) => Ok(value),
            None => {
                self.end_of_sequence = true;
                Err(Self::Error::custom("End of sequence"))
            }
        }
    }

    fn read_bool(&mut self, _schema: &SchemaRef) -> Result<bool, Self::Error> {
        match self.seq_access.next_element()? {
            Some(value) => Ok(value),
            None => {
                self.end_of_sequence = true;
                Err(Self::Error::custom("End of sequence"))
            }
        }
    }

    fn read_byte(&mut self, _schema: &SchemaRef) -> Result<i8, Self::Error> {
        match self.seq_access.next_element()? {
            Some(value) => Ok(value),
            None => {
                self.end_of_sequence = true;
                Err(Self::Error::custom("End of sequence"))
            }
        }
    }

    fn read_short(&mut self, _schema: &SchemaRef) -> Result<i16, Self::Error> {
        match self.seq_access.next_element()? {
            Some(value) => Ok(value),
            None => {
                self.end_of_sequence = true;
                Err(Self::Error::custom("End of sequence"))
            }
        }
    }

    fn read_long(&mut self, _schema: &SchemaRef) -> Result<i64, Self::Error> {
        match self.seq_access.next_element()? {
            Some(value) => Ok(value),
            None => {
                self.end_of_sequence = true;
                Err(Self::Error::custom("End of sequence"))
            }
        }
    }

    fn read_float(&mut self, _schema: &SchemaRef) -> Result<f32, Self::Error> {
        match self.seq_access.next_element()? {
            Some(value) => Ok(value),
            None => {
                self.end_of_sequence = true;
                Err(Self::Error::custom("End of sequence"))
            }
        }
    }

    fn read_double(&mut self, _schema: &SchemaRef) -> Result<f64, Self::Error> {
        match self.seq_access.next_element()? {
            Some(value) => Ok(value),
            None => {
                self.end_of_sequence = true;
                Err(Self::Error::custom("End of sequence"))
            }
        }
    }

    fn read_big_integer(&mut self, _schema: &SchemaRef) -> Result<crate::BigInt, Self::Error> {
        Err(Self::Error::custom("BigInteger not yet supported"))
    }

    fn read_big_decimal(&mut self, _schema: &SchemaRef) -> Result<crate::BigDecimal, Self::Error> {
        Err(Self::Error::custom("BigDecimal not yet supported"))
    }

    fn read_blob(&mut self, _schema: &SchemaRef) -> Result<crate::ByteBuffer, Self::Error> {
        Err(Self::Error::custom("Blob not yet supported"))
    }

    fn read_timestamp(&mut self, _schema: &SchemaRef) -> Result<crate::Instant, Self::Error> {
        Err(Self::Error::custom("Timestamp not yet supported"))
    }

    fn read_document(
        &mut self,
        _schema: &SchemaRef,
    ) -> Result<crate::schema::Document, Self::Error> {
        Err(Self::Error::custom("Document not yet supported"))
    }

    fn read_struct<B, F2>(
        &mut self,
        _schema: &SchemaRef,
        _builder: B,
        _consumer: F2,
    ) -> Result<B, Self::Error>
    where
        F2: FnMut(B, &SchemaRef, &mut Self) -> Result<B, Self::Error>,
    {
        Err(Self::Error::custom("Nested structs not yet supported"))
    }

    fn read_map<T2, F2>(
        &mut self,
        _schema: &SchemaRef,
        _state: &mut T2,
        _consumer: F2,
    ) -> Result<(), Self::Error>
    where
        F2: FnMut(&mut T2, String, &mut Self) -> Result<(), Self::Error>,
    {
        Err(Self::Error::custom("Maps not yet supported"))
    }

    fn is_null(&mut self) -> bool {
        false
    }

    fn read_null(&mut self) -> Result<(), Self::Error> {
        Err(Self::Error::custom("Null not supported in sequences"))
    }
}

// Visitor for structs - receives MapAccess and creates adapter
struct StructVisitor<'a, T> {
    schema: &'a SchemaRef,
    _phantom: PhantomData<T>,
}

impl<'a, 'de, T: DeserializeWithSchema<'de>> Visitor<'de> for StructVisitor<'a, T> {
    type Value = T;

    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "a struct/map")
    }

    fn visit_map<A>(self, map: A) -> Result<T, A::Error>
    where
        A: MapAccess<'de>,
    {
        // Create adapter from MapAccess
        let mut adapter = MapAccessAdapter {
            map_access: map,
            _phantom: PhantomData,
        };

        // Call our deserialization
        T::deserialize_with_schema(self.schema, &mut adapter)
            .map_err(|e| A::Error::custom(format!("Deserialization error: {}", e)))
    }
}

// MapAccessAdapter wraps serde's MapAccess and implements our Deserializer trait
struct MapAccessAdapter<'de, M: MapAccess<'de>> {
    map_access: M,
    _phantom: PhantomData<&'de ()>,
}

impl<'de, M: MapAccess<'de>> crate::serde::deserializers::Deserializer<'de>
    for MapAccessAdapter<'de, M>
{
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
        // Iterate through all map entries
        while let Some(key) = self.map_access.next_key::<String>()? {
            // Look up the member schema by field name
            if let Some(member_schema) = schema.get_member(&key) {
                // Call the consumer with the member schema
                // The consumer will call back into our read_* methods to deserialize the value
                builder = consumer(builder, member_schema, self)?;
            } else {
                // Unknown field - skip the value
                self.map_access.next_value::<serde::de::IgnoredAny>()?;
            }
        }

        Ok(builder)
    }

    // Primitives - call next_value on MapAccess
    fn read_string(&mut self, _schema: &SchemaRef) -> Result<String, Self::Error> {
        Ok(self.map_access.next_value()?)
    }

    fn read_integer(&mut self, _schema: &SchemaRef) -> Result<i32, Self::Error> {
        Ok(self.map_access.next_value()?)
    }

    fn read_bool(&mut self, _schema: &SchemaRef) -> Result<bool, Self::Error> {
        Ok(self.map_access.next_value()?)
    }

    fn read_byte(&mut self, _schema: &SchemaRef) -> Result<i8, Self::Error> {
        Ok(self.map_access.next_value()?)
    }

    fn read_short(&mut self, _schema: &SchemaRef) -> Result<i16, Self::Error> {
        Ok(self.map_access.next_value()?)
    }

    fn read_long(&mut self, _schema: &SchemaRef) -> Result<i64, Self::Error> {
        Ok(self.map_access.next_value()?)
    }

    fn read_float(&mut self, _schema: &SchemaRef) -> Result<f32, Self::Error> {
        Ok(self.map_access.next_value()?)
    }

    fn read_double(&mut self, _schema: &SchemaRef) -> Result<f64, Self::Error> {
        Ok(self.map_access.next_value()?)
    }

    fn read_big_integer(&mut self, _schema: &SchemaRef) -> Result<crate::BigInt, Self::Error> {
        Err(Self::Error::custom("BigInteger not yet supported"))
    }

    fn read_big_decimal(&mut self, _schema: &SchemaRef) -> Result<crate::BigDecimal, Self::Error> {
        Err(Self::Error::custom("BigDecimal not yet supported"))
    }

    fn read_blob(&mut self, _schema: &SchemaRef) -> Result<crate::ByteBuffer, Self::Error> {
        Err(Self::Error::custom("Blob not yet supported"))
    }

    fn read_timestamp(&mut self, _schema: &SchemaRef) -> Result<crate::Instant, Self::Error> {
        Err(Self::Error::custom("Timestamp not yet supported"))
    }

    fn read_document(
        &mut self,
        _schema: &SchemaRef,
    ) -> Result<crate::schema::Document, Self::Error> {
        Err(Self::Error::custom("Document not yet supported"))
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
        // When deserializing a struct field that is a list, we need to use next_value_seed
        // to get a proper list deserializer
        Err(Self::Error::custom(
            "Nested lists not yet supported in struct fields",
        ))
    }

    fn read_map<T2, F2>(
        &mut self,
        _schema: &SchemaRef,
        _state: &mut T2,
        _consumer: F2,
    ) -> Result<(), Self::Error>
    where
        F2: FnMut(&mut T2, String, &mut Self) -> Result<(), Self::Error>,
    {
        Err(Self::Error::custom("Nested maps not yet supported"))
    }

    fn is_null(&mut self) -> bool {
        false
    }

    fn read_null(&mut self) -> Result<(), Self::Error> {
        Err(Self::Error::custom("Null not supported in map values"))
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

    // Test list schema
    lazy_schema!(
        STRING_LIST_SCHEMA,
        Schema::list_builder("test#StringList", traits![]),
        ("member", STRING, traits![])
    );

    #[test]
    fn test_list_of_strings() {
        let json = r#"["hello", "world", "test"]"#;

        let seed = SchemaSeed::<Vec<String>>::new(&STRING_LIST_SCHEMA);
        let result: Vec<String> = seed
            .deserialize(&mut serde_json::Deserializer::from_str(json))
            .unwrap();

        assert_eq!(result, vec!["hello", "world", "test"]);
    }

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

    // List schema for tags
    lazy_schema!(
        TAGS_LIST_SCHEMA,
        Schema::list_builder(ShapeId::from("test#TagsList"), traits![]),
        ("member", STRING, traits![])
    );

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
        ),
        (PARENT_TAGS, "tags", TAGS_LIST_SCHEMA, traits![])
    );

    #[derive(SchemaShape, Debug, PartialEq)]
    #[smithy_schema(PARENT_STRUCT_SCHEMA)]
    pub struct ParentStruct {
        #[smithy_schema(PARENT_NAME)]
        name: String,
        #[smithy_schema(PARENT_NESTED)]
        nested: NestedStruct,
        #[smithy_schema(PARENT_OPTIONAL_NESTED)]
        optional_nested: Option<NestedStruct>,
        #[smithy_schema(PARENT_TAGS)]
        tags: Vec<String>,
    }

    // TODO: This is a future API pattern using deserialize_field which doesn't exist yet
    // Commenting out for now to focus on getting basic list deserialization working
    /*
    impl<'de> DeserializeWithSchema<'de> for ParentStruct {
        fn deserialize_with_schema<D>(
            schema: &SchemaRef,
            deserializer: &mut D,
        ) -> Result<Self, D::Error>
        where
            D: crate::serde::deserializers::Deserializer<'de>,
        {
            struct Builder {
                name: Option<String>,
                nested: Option<NestedStruct>,
                optional_nested: Option<Option<NestedStruct>>,
                tags: Option<Vec<String>>,
            }

            impl Builder {
                fn new() -> Self {
                    Self {
                        name: None,
                        nested: None,
                        optional_nested: None,
                        tags: None,
                    }
                }

                fn name(mut self, value: String) -> Self {
                    self.name = Some(value);
                    self
                }

                fn nested(mut self, value: NestedStruct) -> Self {
                    self.nested = Some(value);
                    self
                }

                fn optional_nested(mut self, value: NestedStruct) -> Self {
                    self.optional_nested = Some(Some(value));
                    self
                }

                fn tags(mut self, value: Vec<String>) -> Self {
                    self.tags = Some(value);
                    self
                }

                fn build(self) -> Result<ParentStruct, String> {
                    Ok(ParentStruct {
                        name: self.name.ok_or_else(|| "name is required".to_string())?,
                        nested: self
                            .nested
                            .ok_or_else(|| "nested is required".to_string())?,
                        optional_nested: self.optional_nested.unwrap_or(None),
                        tags: self.tags.ok_or_else(|| "tags is required".to_string())?,
                    })
                }
            }

            let builder = Builder::new();

            // NEW PATTERN: Use deserialize_field for ALL field types (primitives AND nested structs AND lists)
            let builder =
                deserializer.read_struct(schema, builder, |builder, member_schema, de| {
                    // Primitive field - use deserialize_field
                    if std::sync::Arc::ptr_eq(member_schema, &PARENT_NAME) {
                        let value = de.deserialize_field::<String>(member_schema)?;
                        return Ok(builder.name(value));
                    }

                    // Nested struct field - ALSO use deserialize_field!
                    if std::sync::Arc::ptr_eq(member_schema, &PARENT_NESTED) {
                        let value = de.deserialize_field::<NestedStruct>(member_schema)?;
                        return Ok(builder.nested(value));
                    }

                    // Optional nested struct - ALSO use deserialize_field!
                    if std::sync::Arc::ptr_eq(member_schema, &PARENT_OPTIONAL_NESTED) {
                        let value = de.deserialize_field::<Option<NestedStruct>>(member_schema)?;
                        if let Some(v) = value {
                            return Ok(builder.optional_nested(v));
                        }
                    }

                    // List field - ALSO use deserialize_field!
                    if std::sync::Arc::ptr_eq(member_schema, &PARENT_TAGS) {
                        let value = de.deserialize_field::<Vec<String>>(member_schema)?;
                        return Ok(builder.tags(value));
                    }

                    Ok(builder)
                })?;

            builder.build().map_err(DeserError::custom)
        }
    }
    */

    // TODO: Re-enable when ParentStruct has proper implementation
    /*
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
            },
            "tags": ["tag1", "tag2", "tag3"]
        }"#;

        let result: ParentStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.name, "parent");
        assert_eq!(result.nested.field_a, "value_a");
        assert_eq!(result.nested.field_b, "value_b");
        assert!(result.optional_nested.is_some());
        let optional = result.optional_nested.unwrap();
        assert_eq!(optional.field_a, "optional_a");
        assert_eq!(optional.field_b, "optional_b");
        assert_eq!(result.tags, vec!["tag1", "tag2", "tag3"]);
    }

    #[test]
    fn test_nested_struct_with_optional_none() {
        let json = r#"{
            "name": "parent",
            "nested": {
                "field_a": "value_a",
                "field_b": "value_b"
            },
            "tags": ["tag1", "tag2"]
        }"#;

        let result: ParentStruct = serde_json::from_str(json).unwrap();

        assert_eq!(result.name, "parent");
        assert_eq!(result.nested.field_a, "value_a");
        assert_eq!(result.nested.field_b, "value_b");
        assert!(result.optional_nested.is_none());
        assert_eq!(result.tags, vec!["tag1", "tag2"]);
    }
    */

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
