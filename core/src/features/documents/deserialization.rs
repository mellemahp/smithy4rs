//! Generic deserialization support for documents
//!
//! Note: This functionality is used to convert untyped documents
//! into typed Smithy Shapes.

use crate::{
    features::documents::{Document, DocumentError, ListAccessor, MapAccessor},
    schema::{Schema, ShapeType},
    serde::de::{DeserializeWithSchema, Deserializer, ListReader, MapReader, StructReader},
};

/// Reader structure for List document types.
///
/// Note: Reading a list will `take` its values so it will no longer be
/// usable for re-read and should be dropped.
pub struct DocumentListReader<'b> {
    /// Inner document.
    ///
    /// Note: Should always represent a collection (Struct, List, Map)
    inner: &'b mut (dyn ListAccessor + 'b),
    /// Current index of the reader
    idx: usize,
}

impl<'de> ListReader<'de> for DocumentListReader<'_> {
    type Error = DocumentError;

    #[inline]
    fn read_element<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<Option<T>, Self::Error> {
        let Some(elem) = self.inner.get_element_mut(self.idx) else {
            return Ok(None);
        };
        self.idx += 1;
        T::deserialize_with_schema(schema, elem).map(Some)
    }
}

/// Reader structure for objects (i.e. Map, Struct, Unions) document types.
///
/// Note: Reading a collection will `take` its values so it will no longer be
/// usable for re-read and should be dropped.
pub struct DocumentObjectReader<'b> {
    /// Inner document.
    ///
    /// Note: Should always represent a collection (Struct, List, Map)
    inner: &'b mut (dyn MapAccessor + 'b),
    /// Current index of the reader
    idx: usize,
    /// If the object is a union (requires custom handling)
    is_union: bool,
}

impl<'de> MapReader<'de> for DocumentObjectReader<'_> {
    type Error = DocumentError;

    #[inline]
    fn read_key(&mut self) -> Result<Option<String>, Self::Error> {
        let Some(key) = self.inner.get_key_mut(self.idx) else {
            return Ok(None);
        };
        Ok(Some(key.into_owned()))
    }

    #[inline]
    fn read_value<V: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<V, Self::Error> {
        let (_k, elem) = self.inner.get_entry_mut(self.idx)?;
        self.idx += 1;
        V::deserialize_with_schema(schema, elem)
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.idx += 1;
        Ok(())
    }
}

impl<'de> StructReader<'de> for DocumentObjectReader<'_> {
    type Error = DocumentError;

    #[inline]
    fn read_member<'a>(&mut self, schema: &'a Schema) -> Result<Option<&'a Schema>, Self::Error> {
        if self.is_union {
            // Iterate until we find the populated member
            for _ in self.idx..self.inner.len() {
                if self.inner.get_key_mut(self.idx).is_some() {
                    break;
                }
                // No value, keep going.
                self.idx += 1;
            }
        }
        Ok(schema.members().get_index(self.idx).map(|(_, v)| v))
    }

    #[inline]
    fn read_value<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<T, Self::Error> {
        let (_key, doc) = self.inner.get_entry_mut(self.idx)?;
        self.idx += 1;
        T::deserialize_with_schema(schema, doc)
    }

    #[inline]
    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.idx += 1;
        Ok(())
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        if self.is_union {
            // Union maps can only ever have a single member
            return Some(1);
        }
        Some(self.inner.len())
    }
}

impl<'b> Deserializer<'_> for &'b mut (dyn Document + '_) {
    type Error = DocumentError;
    // Assuming your StructReader uses a similar unboxed/stack pattern
    type StructReader = DocumentObjectReader<'b>;
    type ListReader = DocumentListReader<'b>;
    type MapReader = DocumentObjectReader<'b>;

    // Primitives (Bitwise Stack Copies)
    #[inline]
    fn read_bool(self, schema: &Schema) -> Result<bool, Self::Error> {
        self.take_bool(schema)
    }

    #[inline]
    fn read_byte(self, schema: &Schema) -> Result<i8, Self::Error> {
        self.take_byte(schema)
    }

    #[inline]
    fn read_short(self, schema: &Schema) -> Result<i16, Self::Error> {
        self.take_short(schema)
    }

    #[inline]
    fn read_integer(self, schema: &Schema) -> Result<i32, Self::Error> {
        self.take_integer(schema)
    }

    #[inline]
    fn read_long(self, schema: &Schema) -> Result<i64, Self::Error> {
        self.take_long(schema)
    }

    #[inline]
    fn read_float(self, schema: &Schema) -> Result<f32, Self::Error> {
        self.take_float(schema)
    }

    #[inline]
    fn read_double(self, schema: &Schema) -> Result<f64, Self::Error> {
        self.take_double(schema)
    }

    // Heap-Allocated Extractor (In-place std::mem::take)
    #[inline]
    fn read_string(self, schema: &Schema) -> Result<String, Self::Error> {
        self.take_string(schema)
    }

    #[inline]
    fn read_struct(self, schema: &Schema) -> Result<Self::StructReader, Self::Error> {
        Ok(DocumentObjectReader {
            inner: self.take_map()?,
            idx: 0,
            is_union: matches!(schema.shape_type(), ShapeType::Union),
        })
    }

    #[inline]
    fn read_list(self, _schema: &Schema) -> Result<Self::ListReader, Self::Error> {
        Ok(DocumentListReader {
            inner: self.take_list()?,
            idx: 0,
        })
    }

    #[inline]
    fn read_map(self, _schema: &Schema) -> Result<Self::MapReader, Self::Error> {
        Ok(DocumentObjectReader {
            inner: self.take_map()?,
            idx: 0,
            is_union: false,
        })
    }

    #[inline]
    fn is_null(&mut self) -> bool {
        self.null()
    }

    #[inline]
    fn read_null(self) -> Result<(), Self::Error> {
        if self.null() {
            Ok(())
        } else {
            Err(DocumentError::NullRead)
        }
    }
}

// ================================================================
// Boxed Variant impl
//
// Note: Required to support trait and wrapper type deser
// ================================================================

impl<'de> DeserializeWithSchema<'de> for Box<dyn Document> {
    #[inline]
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_document(schema)
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use smithy4rs_core_derive::{SmithyShape, SmithyTrait, smithy_enum, smithy_union};

    use super::*;
    use crate::{
        prelude::{INTEGER, STRING},
        schema::UNIT,
        smithy,
    };

    smithy!(
        map com::example::Map {
            key: STRING
            value: STRING
        }
    );
    smithy!(
        list com::example::List {
            member: STRING
        }
    );
    smithy!(
        structure com::example::Shape {
            memberA: STRING
            memberB: INTEGER
            memberOptional: STRING
            memberList: LIST
            memberMap: MAP
        }
    );

    #[derive(SmithyShape, Clone, PartialEq)]
    #[schema(schema = SHAPE)]
    pub struct SerializeMe {
        pub member_a: String,
        pub member_b: i32,
        pub member_optional: Option<String>,
        pub member_list: Vec<String>,
        pub member_map: IndexMap<String, String>,
    }

    #[test]
    fn roundtrip_deserialize() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let struct_to_convert = SerializeMeBuilder::new()
            .member_a(String::from("a"))
            .member_b(1)
            .member_list(vec!["a".to_string(), "b".to_string()])
            .member_map(map)
            .build()
            .expect("Should build document");
        let document: Box<dyn Document> = struct_to_convert.into();
        let builder: SerializeMeBuilder = document.try_into().expect("Should build document");
        let out = builder.build().expect("Should build document");
        assert_eq!(out.member_a, String::from("a"));
        assert_eq!(out.member_b, 1);
        assert_eq!(out.member_list, vec![String::from("a"), String::from("b")]);
    }

    smithy!(
        string test::StringTrait
    );

    #[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
    #[schema(schema = STRING_TRAIT)]
    pub struct SimpleTrait(String);

    #[test]
    fn simple_wrapper_roundtrip() {
        let my_string: SimpleTrait = SimpleTrait(String::from("a"));
        let document: Box<dyn Document> = my_string.into();
        let out: SimpleTrait = document.try_into().expect("Should build document");
        assert_eq!(*out, "a");
    }

    smithy!(
        list test::ListTrait {
            member: STRING
        }
    );

    #[derive(SmithyShape, SmithyTrait, PartialEq, Clone)]
    #[schema(schema = LIST_TRAIT)]
    #[repr(transparent)]
    pub struct SimpleList(Vec<String>);

    #[test]
    fn list_wrapper_roundtrip() {
        let my_list: SimpleList = SimpleList(vec![String::from("a"), String::from("b")]);
        let document: Box<dyn Document> = my_list.into();
        let out: SimpleList = document.try_into().expect("Should build document");
        let first = out.get_index(0).unwrap().as_string().unwrap();
        let second = out.get_index(1).unwrap().as_string().unwrap();
        assert_eq!(first, "a");
        assert_eq!(second, "b");
    }

    smithy!(
        enum test::SimpleEnum {
            A = "a"
            B = "b"
            C = "c"
        }
    );

    #[smithy_enum]
    #[derive(SmithyShape, Clone, PartialEq)]
    #[schema(schema = SIMPLE_ENUM)]
    pub enum TestEnum {
        A = "a",
        B = "b",
        C = "c",
    }

    #[test]
    fn enum_roundtrip() {
        let my_enum = TestEnum::A;
        let document: Box<dyn Document> = my_enum.into();
        assert_eq!(document.as_string().expect("string"), "a");
        let out: TestEnum = document.try_into().expect("Should build document");
        assert_eq!(TestEnum::A, out);
    }

    smithy!(
        union test::Union {
            a: STRING
            b: INTEGER
            c: UNIT
        }
    );

    #[smithy_union]
    #[derive(SmithyShape, PartialEq, Clone)]
    #[schema(schema = UNION)]
    pub enum TestUnion {
        A(String),
        B(i32),
        C,
    }

    #[test]
    fn union_roundtrip() {
        let my_enum = TestUnion::A("a".to_string());
        let document: Box<dyn Document> = my_enum.into();
        let map = document.as_map().expect("expected map document");
        let a = map.get("a").expect("expected a");
        assert_eq!(a.as_string().expect("str"), "a");
        let out: TestUnion = document.try_into().expect("Should build union");
        assert_eq!(out, TestUnion::A(String::from("a")));
    }
}
