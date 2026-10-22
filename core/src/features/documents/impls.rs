//! Implementations of [`Document`] for core types.
//!
//! Supported types with automatic implementations include:
//! - `bool`
//! - `String`
//! - `ByteBuffer` (`blob`)
//! - `i8` (`byte`)
//! - `i16` (`short`)
//! - `i32` (`integer`)
//! - `i64` (`long`)
//! - `BigInt` (`big_integer`)
//! - `f32` (`float`)
//! - `f64` (`double`)
//! - `Vec` of Documents (`list`)
//! - `IndexMap` of Documents (`map`)
//! - `Option` (supports `null` document members)
//!
//! Wherever possible automatic conversions are implemented to ease the effort
//! of protocol smoothing. For example, i8 documents can be treated a i32
//! automatically.

use std::{borrow::Cow, mem};

use bigdecimal::{FromPrimitive, num_traits};
use indexmap::map::MutableKeys;
use num_traits::ToPrimitive;

use crate::{
    BigDecimal, BigInt, ByteBuffer, IndexMap, Instant,
    features::documents::{
        Document, DocumentError, DocumentType, ListAccessor, MapAccessor, Number,
    },
    schema::{Schema, ShapeId, StaticSchemaShape},
};

impl Document for bool {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Boolean
    }

    #[inline]
    fn as_boolean(&self) -> Option<bool> {
        Some(*self)
    }

    #[inline]
    fn take_bool(&mut self, _schema: &Schema) -> Result<bool, DocumentError> {
        Ok(*self)
    }
}

impl Document for ByteBuffer {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Blob
    }

    #[inline]
    fn as_blob(&self) -> Option<&ByteBuffer> {
        Some(self)
    }

    #[inline]
    fn take_blob(&mut self, _schema: &Schema) -> Result<ByteBuffer, DocumentError> {
        Ok(mem::take(self))
    }
}

impl Document for String {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::String
    }

    #[inline]
    fn as_str(&self) -> Option<&str> {
        Some(self)
    }

    #[inline]
    fn take_string(&mut self, _schema: &Schema) -> Result<String, DocumentError> {
        Ok(mem::take(self))
    }
}

// TODO: How to handle timestamps? They can be both?
impl Document for Instant {
    #[inline]
    fn document_type(&self) -> DocumentType {
        todo!()
    }
}

macro_rules! integer_doc {
    ($ty:ty, $variant:ident) => {
        #[allow(trivial_casts)]
        impl Document for $ty {
            #[inline]
            fn document_type(&self) -> DocumentType {
                DocumentType::Number(Number::$variant)
            }

            #[inline]
            fn as_byte(&self) -> Option<i8> {
                <i8 as TryFrom<$ty>>::try_from(*self).ok()
            }

            #[inline]
            fn as_short(&self) -> Option<i16> {
                <i16 as TryFrom<$ty>>::try_from(*self).ok()
            }

            #[inline]
            fn as_integer(&self) -> Option<i32> {
                <i32 as TryFrom<$ty>>::try_from(*self).ok()
            }

            #[inline]
            fn as_long(&self) -> Option<i64> {
                <i64 as TryFrom<$ty>>::try_from(*self).ok()
            }

            #[inline]
            fn as_big_integer(&self) -> Option<Cow<'_, BigInt>> {
                BigInt::try_from(*self).ok().map(Cow::Owned)
            }

            #[inline]
            fn take_byte(&mut self, _schema: &Schema) -> Result<i8, DocumentError> {
                <i8 as TryFrom<$ty>>::try_from(*self)
                    .map_err(|_| DocumentError::ConversionFailed("i8".to_string()))
            }

            #[inline]
            fn take_short(&mut self, _schema: &Schema) -> Result<i16, DocumentError> {
                <i16 as TryFrom<$ty>>::try_from(*self)
                    .map_err(|_| DocumentError::ConversionFailed("i16".to_string()))
            }

            #[inline]
            #[allow(trivial_numeric_casts)]
            #[allow(clippy::cast_lossless)]
            fn take_integer(&mut self, _schema: &Schema) -> Result<i32, DocumentError> {
                <i32 as TryFrom<$ty>>::try_from(*self)
                    .map_err(|_| DocumentError::ConversionFailed("i32".to_string()))
            }

            #[inline]
            fn take_long(&mut self, _schema: &Schema) -> Result<i64, DocumentError> {
                <i64 as TryFrom<$ty>>::try_from(*self)
                    .map_err(|_| DocumentError::ConversionFailed("i64".to_string()))
            }

            #[inline]
            fn take_big_integer(&mut self, _schema: &Schema) -> Result<BigInt, DocumentError> {
                Ok(BigInt::from(*self))
            }
        }
    };
}

integer_doc!(i8, Byte);
integer_doc!(i16, Short);
integer_doc!(i32, Integer);
integer_doc!(i64, Long);

// TODO: Should we allow integer conversions between float/int?
impl Document for f32 {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Number(Number::Float)
    }

    #[inline]
    fn as_float(&self) -> Option<f32> {
        Some(*self)
    }

    #[inline]
    fn as_double(&self) -> Option<f64> {
        Some(f64::from(*self))
    }

    #[inline]
    fn take_float(&mut self, _schema: &Schema) -> Result<f32, DocumentError> {
        Ok(*self)
    }

    #[inline]
    fn take_double(&mut self, _schema: &Schema) -> Result<f64, DocumentError> {
        Ok(f64::from(*self))
    }

    #[inline]
    fn take_big_decimal(&mut self, _schema: &Schema) -> Result<BigDecimal, DocumentError> {
        BigDecimal::from_f32(*self)
            .ok_or_else(|| DocumentError::ConversionFailed("BigDecimal".to_string()))
    }
}

impl Document for f64 {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Number(Number::Double)
    }

    #[inline]
    fn as_float(&self) -> Option<f32> {
        num_traits::cast(*self)
    }

    #[inline]
    fn as_double(&self) -> Option<f64> {
        Some(*self)
    }

    #[inline]
    fn take_float(&mut self, _schema: &Schema) -> Result<f32, DocumentError> {
        num_traits::cast(*self).ok_or_else(|| DocumentError::ConversionFailed("f32".to_string()))
    }

    #[inline]
    fn take_double(&mut self, _schema: &Schema) -> Result<f64, DocumentError> {
        Ok(*self)
    }

    #[inline]
    fn take_big_decimal(&mut self, _schema: &Schema) -> Result<BigDecimal, DocumentError> {
        BigDecimal::from_f64(*self)
            .ok_or_else(|| DocumentError::ConversionFailed("f64".to_string()))
    }
}

impl Document for BigDecimal {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Number(Number::BigDecimal)
    }

    #[inline]
    fn as_byte(&self) -> Option<i8> {
        self.to_i8()
    }

    #[inline]
    fn as_short(&self) -> Option<i16> {
        self.to_i16()
    }

    #[inline]
    fn as_integer(&self) -> Option<i32> {
        self.to_i32()
    }

    #[inline]
    fn as_long(&self) -> Option<i64> {
        self.to_i64()
    }

    #[inline]
    fn take_float(&mut self, _schema: &Schema) -> Result<f32, DocumentError> {
        self.to_f32()
            .ok_or_else(|| DocumentError::ConversionFailed("f32".to_string()))
    }

    #[inline]
    fn take_double(&mut self, _schema: &Schema) -> Result<f64, DocumentError> {
        self.to_f64()
            .ok_or_else(|| DocumentError::ConversionFailed("f32".to_string()))
    }

    fn take_big_decimal(&mut self, _schema: &Schema) -> Result<BigDecimal, DocumentError> {
        Ok(mem::take(self))
    }
}

impl Document for BigInt {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Number(Number::BigInt)
    }

    #[inline]
    fn as_byte(&self) -> Option<i8> {
        self.to_i8()
    }

    #[inline]
    fn as_short(&self) -> Option<i16> {
        self.to_i16()
    }

    #[inline]
    fn as_integer(&self) -> Option<i32> {
        self.to_i32()
    }

    #[inline]
    fn as_long(&self) -> Option<i64> {
        self.to_i64()
    }

    #[inline]
    fn take_byte(&mut self, _schema: &Schema) -> Result<i8, DocumentError> {
        <i8 as TryFrom<BigInt>>::try_from(mem::take(self))
            .map_err(|_| DocumentError::ConversionFailed("i8".to_string()))
    }

    #[inline]
    fn take_short(&mut self, _schema: &Schema) -> Result<i16, DocumentError> {
        <i16 as TryFrom<BigInt>>::try_from(mem::take(self))
            .map_err(|_| DocumentError::ConversionFailed("i16".to_string()))
    }

    #[inline]
    fn take_integer(&mut self, _schema: &Schema) -> Result<i32, DocumentError> {
        <i32 as TryFrom<BigInt>>::try_from(mem::take(self))
            .map_err(|_| DocumentError::ConversionFailed("i32".to_string()))
    }

    #[inline]
    fn take_long(&mut self, _schema: &Schema) -> Result<i64, DocumentError> {
        <i64 as TryFrom<BigInt>>::try_from(mem::take(self))
            .map_err(|_| DocumentError::ConversionFailed("i64".to_string()))
    }

    #[inline]
    fn take_big_integer(&mut self, _schema: &Schema) -> Result<BigInt, DocumentError> {
        Ok(mem::take(self))
    }
}

// Note: necessary for trait instantiators!
impl Document for &'static str {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::String
    }

    #[inline]
    fn as_str(&self) -> Option<&str> {
        Some(self)
    }

    #[inline]
    fn take_string(&mut self, _schema: &Schema) -> Result<String, DocumentError> {
        Ok(self.to_string())
    }
}

// Collections
impl<T: Document + Clone + 'static> Document for Vec<T> {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::List
    }

    #[inline]
    fn as_list(&self) -> Option<&dyn ListAccessor> {
        Some(self)
    }

    #[inline]
    fn take_list(&mut self) -> Result<&mut dyn ListAccessor, DocumentError> {
        Ok(self)
    }
}

impl<T: Document> ListAccessor for Vec<T> {
    #[inline]
    #[allow(trivial_casts)]
    #[allow(clippy::unnecessary_cast)]
    fn get(&self, idx: usize) -> Option<&(dyn Document + '_)> {
        self[..].get(idx).map(|doc| doc as &dyn Document)
    }

    #[inline]
    #[allow(trivial_casts)]
    fn get_element_mut(&mut self, index: usize) -> Option<&mut dyn Document> {
        self.get_mut(index).map(|item| item as &mut dyn Document)
    }

    #[inline]
    fn len(&self) -> usize {
        self[..].len()
    }
}

impl<T: Document + Clone + 'static> Document for IndexMap<String, T> {
    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Map
    }

    #[inline]
    fn as_map(&self) -> Option<&dyn MapAccessor> {
        Some(self)
    }

    #[inline]
    #[allow(trivial_casts)]
    fn take_map(&mut self) -> Result<&mut dyn MapAccessor, DocumentError> {
        Ok(&mut *self as &mut dyn MapAccessor)
    }
}

impl<T: Document> MapAccessor for IndexMap<String, T> {
    #[inline]
    #[allow(trivial_casts)]
    fn get(&self, key: &str) -> Option<&dyn Document> {
        self.get(key).map(|doc| doc as &dyn Document)
    }

    #[inline]
    #[allow(trivial_casts)]
    fn get_entry(&self, idx: usize) -> Option<(&str, &dyn Document)> {
        self.get_index(idx)
            .map(|(k, v)| (k.as_str(), v as &dyn Document))
    }

    #[inline]
    #[allow(trivial_casts)]
    fn get_entry_mut(&mut self, index: usize) -> Result<(&str, &mut dyn Document), DocumentError> {
        let Some((k, v)) = self.get_index_mut(index) else {
            return Err(DocumentError::InvalidIndex(index));
        };
        Ok((k.as_str(), v as &mut dyn Document))
    }

    #[inline]
    #[allow(trivial_casts)]
    fn get_key_mut(&mut self, index: usize) -> Option<Cow<'_, str>> {
        self.get_index_mut2(index)
            .map(|(k, _)| Cow::Owned(mem::take(k)))
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T: Document + Clone + 'static> Document for Option<T> {
    #[inline]
    fn document_type(&self) -> DocumentType {
        self.as_ref()
            .map_or(DocumentType::Null, Document::document_type)
    }

    // ==== Delegation of As-ers to inner document type ====

    #[inline]
    fn as_boolean(&self) -> Option<bool> {
        self.as_ref().and_then(Document::as_boolean)
    }

    #[inline]
    fn as_blob(&self) -> Option<&ByteBuffer> {
        self.as_ref().and_then(Document::as_blob)
    }

    #[inline]
    fn as_str(&self) -> Option<&str> {
        self.as_ref().and_then(Document::as_str)
    }

    #[inline]
    fn as_byte(&self) -> Option<i8> {
        self.as_ref().and_then(Document::as_byte)
    }

    #[inline]
    fn as_short(&self) -> Option<i16> {
        self.as_ref().and_then(Document::as_short)
    }

    #[inline]
    fn as_integer(&self) -> Option<i32> {
        self.as_ref().and_then(Document::as_integer)
    }

    #[inline]
    fn as_long(&self) -> Option<i64> {
        self.as_ref().and_then(Document::as_long)
    }

    #[inline]
    fn as_big_integer(&self) -> Option<Cow<'_, BigInt>> {
        self.as_ref().and_then(Document::as_big_integer)
    }

    #[inline]
    fn as_float(&self) -> Option<f32> {
        self.as_ref().and_then(Document::as_float)
    }

    #[inline]
    fn as_double(&self) -> Option<f64> {
        self.as_ref().and_then(Document::as_double)
    }

    #[inline]
    fn as_big_decimal(&self) -> Option<Cow<'_, BigDecimal>> {
        self.as_ref().and_then(Document::as_big_decimal)
    }

    #[inline]
    fn as_list(&self) -> Option<&dyn ListAccessor> {
        self.as_ref().and_then(Document::as_list)
    }

    #[inline]
    fn as_map(&self) -> Option<&dyn MapAccessor> {
        self.as_ref().and_then(Document::as_map)
    }

    #[inline]
    fn null(&self) -> bool {
        self.as_ref().is_none_or(Document::null)
    }

    // ==== Delegation of takers to inner document type ====

    // TODO: Errors could be clearer I think?

    #[inline]
    fn take_bool(&mut self, schema: &Schema) -> Result<bool, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::NullRead)?
            .take_bool(schema)
    }

    #[inline]
    fn take_blob(&mut self, schema: &Schema) -> Result<ByteBuffer, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::NullRead)?
            .take_blob(schema)
    }

    #[inline]
    fn take_string(&mut self, schema: &Schema) -> Result<String, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::NullRead)?
            .take_string(schema)
    }

    #[inline]
    fn take_byte(&mut self, schema: &Schema) -> Result<i8, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::NullRead)?
            .take_byte(schema)
    }

    #[inline]
    fn take_short(&mut self, schema: &Schema) -> Result<i16, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::NullRead)?
            .take_short(schema)
    }

    #[inline]
    fn take_integer(&mut self, schema: &Schema) -> Result<i32, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::NullRead)?
            .take_integer(schema)
    }

    #[inline]
    fn take_long(&mut self, schema: &Schema) -> Result<i64, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::NullRead)?
            .take_long(schema)
    }

    #[inline]
    fn take_big_integer(&mut self, schema: &Schema) -> Result<BigInt, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::NullRead)?
            .take_big_integer(schema)
    }

    #[inline]
    fn take_float(&mut self, schema: &Schema) -> Result<f32, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::NullRead)?
            .take_float(schema)
    }

    #[inline]
    fn take_double(&mut self, schema: &Schema) -> Result<f64, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::NullRead)?
            .take_double(schema)
    }

    #[inline]
    fn take_big_decimal(&mut self, schema: &Schema) -> Result<BigDecimal, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::NullRead)?
            .take_big_decimal(schema)
    }

    #[inline]
    fn take_list(&mut self) -> Result<&mut dyn ListAccessor, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::NullRead)?
            .take_list()
    }

    #[inline]
    fn take_map(&mut self) -> Result<&mut dyn MapAccessor, DocumentError> {
        self.as_mut()
            .ok_or_else(|| DocumentError::NullRead)?
            .take_map()
    }
}

// ============================================================================
// Boxed (owned) variant support
// ============================================================================

impl Document for Vec<Box<dyn Document>> {
    fn document_type(&self) -> DocumentType {
        DocumentType::List
    }

    fn as_list(&self) -> Option<&dyn ListAccessor> {
        todo!()
    }

    fn take_list(&mut self) -> Result<&mut dyn ListAccessor, DocumentError> {
        todo!()
    }
}

impl Document for IndexMap<String, Box<dyn Document>> {
    fn document_type(&self) -> DocumentType {
        DocumentType::Map
    }

    fn as_map(&self) -> Option<&dyn MapAccessor> {
        todo!()
    }

    fn take_map(&mut self) -> Result<&mut dyn MapAccessor, DocumentError> {
        todo!()
    }
}

// ================================================================
// Blanket convenience impls for generated shapes
// ================================================================

impl<T: StaticSchemaShape + MapAccessor + Send + Sync + Clone + 'static> Document for T {
    #[inline]
    fn discriminant(&self) -> Option<&ShapeId> {
        Some(Self::schema().id())
    }

    #[inline]
    fn document_type(&self) -> DocumentType {
        DocumentType::Map
    }

    #[inline]
    fn as_map(&self) -> Option<&dyn MapAccessor> {
        Some(self)
    }

    #[inline]
    fn take_map(&mut self) -> Result<&mut dyn MapAccessor, DocumentError> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_document_value() {
        let document_str: Box<dyn Document> = "MyStr".into();
        let output_str = document_str
            .as_str()
            .expect("Could not get document as str");
        assert_eq!(output_str, "MyStr");
    }

    #[test]
    fn list_document_value() {
        let vec = vec!["a", "b", "c"];
        let document_list: Box<dyn Document> = vec.into();
        assert_eq!(document_list.as_list().expect("list").len(), 3);
        let vec_out: Vec<String> = document_list.try_into().unwrap();
        assert_eq!(vec_out.len(), 3);
        assert_eq!(vec_out[0], "a");
        assert_eq!(vec_out[1], "b");
        assert_eq!(vec_out[2], "c");
    }

    #[test]
    fn map_document_value() {
        let mut map_in: IndexMap<String, String> = IndexMap::new();
        map_in.insert("a".to_string(), "b".to_string());
        let map_doc: Box<dyn Document> = map_in.into();
        assert_eq!(map_doc.as_map().expect("map").len(), 1);

        let map_out: IndexMap<String, String> =
            map_doc.try_into().expect("Failed to convert document");
        assert_eq!(map_out.len(), 1);
        assert_eq!(map_out["a"], "b");
    }

    // TODO(numeric comparisons): Add comparison checks

    #[test]
    fn integer_document_values() {
        // TODO: Update
    }

    #[test]
    fn float_document_values() {
        let float: Box<dyn Document> = 1f32.into();
        let float_value: f32 = float.try_into().expect("Failed to convert float");
        assert_eq!(float_value, 1f32);
    }

    #[test]
    fn double_document_value() {
        let double: Box<dyn Document> = 1f64.into();
        let double_value: f64 = double.try_into().expect("Failed to convert double");
        assert_eq!(double_value, 1f64);
    }
}
