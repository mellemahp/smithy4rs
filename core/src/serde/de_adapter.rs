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
            ShapeType::Map => {
                // Tell serde we expect a map
                deserializer.deserialize_map(MapVisitor {
                    schema: self.schema,
                    _phantom: PhantomData,
                })
            }
<<<<<<< HEAD
            // TODO(adapter): We probably need a primitiveVistor to handle root json primitives
=======
>>>>>>> d91b0c1 (checkpoint: adapter full worky!!)
            _ => Err(D::Error::custom(format!(
                "Unsupported shape type for deserialization: {:?}",
                get_shape_type(self.schema)
            ))),
        }
    }
}

// Visitor for lists - receives a SeqAccess and creates adapter
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
        schema: &SchemaRef,
        _builder: B,
        _consumer: F2,
    ) -> Result<B, Self::Error>
    where
        B: DeserializeWithSchema<'de>,
        F2: FnMut(B, &SchemaRef, &mut Self) -> Result<B, Self::Error>,
    {
        // When deserializing a nested struct in a list, use next_element_seed
        // to delegate to the underlying serde deserializer
        let seed = SchemaSeed::<B>::new(schema);
        match self.seq_access.next_element_seed(seed)? {
            Some(value) => Ok(value),
            None => {
                self.end_of_sequence = true;
                Err(Self::Error::custom("End of sequence"))
            }
        }
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

// Visitor for maps (IndexMap, etc) - receives MapAccess and creates adapter
struct MapVisitor<'a, T> {
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
            is_top_level: true,
            _phantom: PhantomData,
        };

        // Call our deserialization
        T::deserialize_with_schema(self.schema, &mut adapter)
            .map_err(|e| A::Error::custom(format!("Deserialization error: {}", e)))
    }
}

impl<'a, 'de, T: DeserializeWithSchema<'de>> Visitor<'de> for MapVisitor<'a, T> {
    type Value = T;

    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "a map")
    }

    fn visit_map<A>(self, map: A) -> Result<T, A::Error>
    where
        A: MapAccess<'de>,
    {
        // Create adapter from MapAccess
        let mut adapter = MapAccessAdapter {
            map_access: map,
            is_top_level: true,
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
    /// Track if we're currently in the top-level iteration
    /// When false, we're deserializing a nested value and should use next_value_seed
    is_top_level: bool,
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
        B: DeserializeWithSchema<'de>,
    {
        // If we're not at the top level, we're deserializing a nested struct
        // Use next_value_seed to delegate to serde
        if !self.is_top_level {
            let seed = SchemaSeed::<B>::new(schema);
            return Ok(self.map_access.next_value_seed(seed)?);
        }

        // Mark that we're now in nested context for any further calls
        self.is_top_level = false;

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
        schema: &SchemaRef,
        state: &mut T,
        _consumer: F,
    ) -> Result<(), Self::Error>
    where
        F: FnMut(&mut T, &SchemaRef, &mut Self) -> Result<(), Self::Error>,
        T: DeserializeWithSchema<'de>,
    {
        // When deserializing a nested list in a struct field, we use next_value_seed
        // to delegate to the underlying serde deserializer (e.g., serde_json).
        //
        // This tells serde_json to deserialize the array value, which will:
        // 1. Create a SeqAccess for the array
        // 2. Call our ListVisitor with it
        // 3. Wrap it in SeqAccessAdapter
        // 4. Call Vec::deserialize_with_schema with the SeqAccessAdapter
        // 5. Which can properly iterate through elements using read_list
        //
        // The result is a fully deserialized T (e.g., Vec<String>) which we assign to state.
        let seed = SchemaSeed::<T>::new(schema);
        let result = self.map_access.next_value_seed(seed)?;
        *state = result;
        Ok(())
    }

    fn read_map<T2, F2>(
        &mut self,
        schema: &SchemaRef,
        state: &mut T2,
        mut consumer: F2,
    ) -> Result<(), Self::Error>
    where
        F2: FnMut(&mut T2, String, &mut Self) -> Result<(), Self::Error>,
        T2: DeserializeWithSchema<'de>,
    {
        // If we're not at the top level, we're deserializing a nested map
        // Use next_value_seed to delegate to serde
        if !self.is_top_level {
            let seed = SchemaSeed::<T2>::new(schema);
            let result = self.map_access.next_value_seed(seed)?;
            *state = result;
            return Ok(());
        }

        // Iterate through all map entries
        while let Some(key) = self.map_access.next_key::<String>()? {
            // Call the consumer with the key
            // The consumer will call back into our read_* methods to deserialize the value
            consumer(state, key, self)?;
        }

        Ok(())
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
    use indexmap::IndexMap;
    use smithy4rs_core_derive::{DeserializableStruct, SchemaShape};
    use indexmap::IndexMap;

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

    // Test nested list in struct
    lazy_schema!(
        STRUCT_WITH_LIST_SCHEMA,
        Schema::structure_builder(ShapeId::from("test#StructWithList"), traits![]),
        (SWL_NAME, "name", STRING, traits![]),
        (SWL_TAGS, "tags", STRING_LIST_SCHEMA, traits![])
    );

    #[derive(SchemaShape, DeserializableStruct, Debug, PartialEq)]
    #[smithy_schema(STRUCT_WITH_LIST_SCHEMA)]
    struct StructWithList {
        #[smithy_schema(SWL_NAME)]
        name: String,
        #[smithy_schema(SWL_TAGS)]
        tags: Vec<String>,
    }

    impl<'de> serde::Deserialize<'de> for StructWithList {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let seed = SchemaSeed::<StructWithList>::new(StructWithList::schema());
            seed.deserialize(deserializer)
        }
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
    lazy_schema!(
        ADDRESS_SCHEMA,
        Schema::structure_builder(ShapeId::from("test#Address"), traits![]),
        (ADDR_STREET, "street", STRING, traits![]),
        (ADDR_CITY, "city", STRING, traits![]),
        (ADDR_ZIP, "zipCode", INTEGER, traits![])
    );

    #[derive(SchemaShape, DeserializableStruct, Debug, PartialEq)]
    #[smithy_schema(ADDRESS_SCHEMA)]
    struct Address {
        #[smithy_schema(ADDR_STREET)]
        street: String,
        #[smithy_schema(ADDR_CITY)]
        city: String,
        #[smithy_schema(ADDR_ZIP)]
        zip_code: i32,
    }

    impl<'de> serde::Deserialize<'de> for Address {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let seed = SchemaSeed::<Address>::new(Address::schema());
            seed.deserialize(deserializer)
        }
    }

    lazy_schema!(
        PHONE_LIST_SCHEMA,
        Schema::list_builder(ShapeId::from("test#PhoneList"), traits![]),
        ("member", STRING, traits![])
    );

    lazy_schema!(
        CONTACT_SCHEMA,
        Schema::structure_builder(ShapeId::from("test#Contact"), traits![]),
        (CONTACT_EMAIL, "email", STRING, traits![]),
        (CONTACT_PHONES, "phones", PHONE_LIST_SCHEMA, traits![]),
        (CONTACT_ADDRESS, "address", ADDRESS_SCHEMA, traits![]),
        (CONTACT_BACKUP, "backupAddress", ADDRESS_SCHEMA, traits![])
    );

    #[derive(SchemaShape, DeserializableStruct, Debug, PartialEq)]
    #[smithy_schema(CONTACT_SCHEMA)]
    struct Contact {
        #[smithy_schema(CONTACT_EMAIL)]
        email: String,
        #[smithy_schema(CONTACT_PHONES)]
        phones: Vec<String>,
        #[smithy_schema(CONTACT_ADDRESS)]
        address: Address,
        #[smithy_schema(CONTACT_BACKUP)]
        backup_address: Option<Address>,
    }

    impl<'de> serde::Deserialize<'de> for Contact {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let seed = SchemaSeed::<Contact>::new(Contact::schema());
            seed.deserialize(deserializer)
        }
    }

    lazy_schema!(
        HOBBY_SCHEMA,
        Schema::structure_builder(ShapeId::from("test#Hobby"), traits![]),
        (HOBBY_NAME, "name", STRING, traits![]),
        (HOBBY_YEARS, "yearsOfExperience", INTEGER, traits![])
    );

    #[derive(SchemaShape, DeserializableStruct, Debug, PartialEq)]
    #[smithy_schema(HOBBY_SCHEMA)]
    struct Hobby {
        #[smithy_schema(HOBBY_NAME)]
        name: String,
        #[smithy_schema(HOBBY_YEARS)]
        years_of_experience: i32,
    }

    impl<'de> serde::Deserialize<'de> for Hobby {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let seed = SchemaSeed::<Hobby>::new(Hobby::schema());
            seed.deserialize(deserializer)
        }
    }

    lazy_schema!(
        HOBBY_LIST_SCHEMA,
        Schema::list_builder(ShapeId::from("test#HobbyList"), traits![]),
        ("member", HOBBY_SCHEMA, traits![])
    );

    lazy_schema!(
        STRING_MAP_SCHEMA,
        Schema::map_builder(ShapeId::from("test#StringMap"), traits![]),
        ("key", STRING, traits![]),
        ("value", STRING, traits![])
    );

    lazy_schema!(
        PERSON_SCHEMA,
        Schema::structure_builder(ShapeId::from("test#Person"), traits![]),
        (PERSON_NAME, "name", STRING, traits![]),
        (PERSON_AGE, "age", INTEGER, traits![]),
        (PERSON_ACTIVE, "isActive", BOOLEAN, traits![]),
        (PERSON_SCORE, "score", FLOAT, traits![]),
        (PERSON_CONTACT, "contact", CONTACT_SCHEMA, traits![]),
        (PERSON_HOBBIES, "hobbies", HOBBY_LIST_SCHEMA, traits![]),
        (PERSON_METADATA, "metadata", STRING_MAP_SCHEMA, traits![]),
        (PERSON_NOTES, "notes", STRING, traits![])
    );

    #[derive(SchemaShape, DeserializableStruct, Debug, PartialEq)]
    #[smithy_schema(PERSON_SCHEMA)]
    struct Person {
        #[smithy_schema(PERSON_NAME)]
        name: String,
        #[smithy_schema(PERSON_AGE)]
        age: i32,
        #[smithy_schema(PERSON_ACTIVE)]
        is_active: bool,
        #[smithy_schema(PERSON_SCORE)]
        score: f32,
        #[smithy_schema(PERSON_CONTACT)]
        contact: Contact,
        #[smithy_schema(PERSON_HOBBIES)]
        hobbies: Vec<Hobby>,
        #[smithy_schema(PERSON_METADATA)]
        metadata: IndexMap<String, String>,
        #[smithy_schema(PERSON_NOTES)]
        notes: Option<String>,
    }

    impl<'de> serde::Deserialize<'de> for Person {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let seed = SchemaSeed::<Person>::new(Person::schema());
            seed.deserialize(deserializer)
        }
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
        assert_eq!(result.is_active, true);
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
        assert_eq!(result.is_active, false);
        assert_eq!(result.score, 87.3);
        assert_eq!(result.contact.email, "bob@example.com");
        assert_eq!(result.contact.phones, Vec::<String>::new());
        assert!(result.contact.backup_address.is_none());
        assert_eq!(result.hobbies, Vec::<Hobby>::new());
        assert_eq!(result.metadata, IndexMap::<String, String>::new());
        assert!(result.notes.is_none());
    }
}
