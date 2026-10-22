//! Generic deserialization support for documents
//!
//! Note: This functionality is used to convert untyped documents
//! into typed Smithy Shapes.

use crate::{
    features::documents::{Document, DocumentError, ListAccessor, MapAccessor},
    schema::Schema,
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
}

impl<'de> MapReader<'de> for DocumentObjectReader<'_> {
    type Error = DocumentError;

    // TODO: How to handle numeric keys?
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
        // Todo: Seems a bit off to read key if we dont use.
        let (_k, elem) = self.inner.get_entry_mut(self.idx)?;
        self.idx += 1;
        V::deserialize_with_schema(schema, elem)
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl<'de> StructReader<'de> for DocumentObjectReader<'_> {
    type Error = DocumentError;

    #[inline]
    fn read_member<'a>(&mut self, schema: &'a Schema) -> Result<Option<&'a Schema>, Self::Error> {
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
    fn read_struct(self, _schema: &Schema) -> Result<Self::StructReader, Self::Error> {
        Ok(DocumentObjectReader {
            inner: self.take_map()?,
            idx: 0,
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
