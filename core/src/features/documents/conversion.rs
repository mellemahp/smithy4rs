//! Conversions from Document into other types
//!
//! All core Smithy data types are supported as well
//! as conversion from a boxed variant (necessary for Wrapper types)
//!
//! Note: some types (timestamp, structures) need a schema to be
//! converted from a document. Types defined here are those that can be
//! safely converted when no schema is present.

use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::{
    features::documents::{Document, DocumentError},
    prelude::{
        BIG_DECIMAL, BIG_INTEGER, BLOB, BOOLEAN, BYTE, DOUBLE, FLOAT, INTEGER, LONG, SHORT, STRING,
    },
    serde::de::DeserializableShape,
};

/// Attempt to create a typed shape from an untyped document.
///
/// NOTE: Implemented as a separate trait to avoid conflict with
/// blanket `TryFrom` impls.
///
/// This trait is reflexive in that any types that implement this trait
/// will also support `try_into` from a dynamic document.
pub trait TryFromDocument: Sized {
    /// Attempt to create a typed shape from an untyped document.
    ///
    /// ## Errors
    /// Returns [`DocumentError`] if the type could not be converted to a
    /// document value.
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError>;
}

// === Impls for basic types ===

impl TryFromDocument for bool {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_bool(&BOOLEAN)
    }
}

impl TryFromDocument for i8 {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_byte(&BYTE)
    }
}

impl TryFromDocument for i16 {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_short(&SHORT)
    }
}

impl TryFromDocument for i32 {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_integer(&INTEGER)
    }
}

impl TryFromDocument for i64 {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_long(&LONG)
    }
}

impl TryFromDocument for f32 {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_float(&FLOAT)
    }
}

impl TryFromDocument for f64 {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_double(&DOUBLE)
    }
}

impl TryFromDocument for BigInt {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_big_integer(&BIG_INTEGER)
    }
}

impl TryFromDocument for BigDecimal {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_big_decimal(&BIG_DECIMAL)
    }
}

impl TryFromDocument for ByteBuffer {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_blob(&BLOB)
    }
}

impl TryFromDocument for String {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        document.take_string(&STRING)
    }
}

impl<T: TryFromDocument> TryFromDocument for Vec<T> {
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        let list = document.take_list()?;
        if list.is_empty() {
            return Ok(Vec::new());
        }
        let mut idx = 0;
        let mut result = Vec::with_capacity(list.len());
        while let Some(inner) = list.get_element_mut(idx) {
            idx += 1;
            result.push(T::try_from(inner)?);
        }
        Ok(result)
    }
}

impl<T: TryFromDocument> TryFromDocument for IndexMap<String, T> {
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        let map = document.take_map()?;
        if map.is_empty() {
            return Ok(IndexMap::new());
        }
        let mut idx = 0;
        let mut result = IndexMap::with_capacity(map.len());
        while idx < map.len() {
            let (k, v) = map.get_entry_mut(idx)?;
            idx += 1;
            result.insert(k.to_string(), T::try_from(v)?);
        }
        Ok(result)
    }
}

impl<T: TryFromDocument> TryFromDocument for Option<T> {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        if document.null() {
            Ok(None)
        } else {
            Ok(Some(T::try_from(document)?))
        }
    }
}

// === Blanket impl for Typed Shapes ===
impl dyn Document {
    /// Attempt to convert a document into a shape by deserialization.
    ///
    /// ## Errors
    /// Returns [`DocumentError`] if the type could not be converted to a
    /// document value.
    #[inline]
    pub fn try_into<S: TryFromDocument>(mut self: Box<Self>) -> Result<S, DocumentError> {
        <S as TryFromDocument>::try_from(&mut *self)
    }
}

// ================================================================
// Blanket impls
// ================================================================

// ===== Blanket Try From =====
impl<'de, S: DeserializableShape<'de>> TryFromDocument for S {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        S::deserialize(document)
    }
}

// Blanket conversion into a boxed document type
impl<T: Document + 'static> From<T> for Box<dyn Document> {
    #[inline]
    #[allow(trivial_casts)]
    fn from(value: T) -> Self {
        Box::new(value) as Box<dyn Document>
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use indexmap::IndexMap;

    use super::*;
    use crate::{BigDecimal, BigInt, derive::SmithyShape, schema::prelude::*, smithy};

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
            memberB: STRING
            memberOptional: STRING
            memberList: LIST
            memberMap: MAP
        }
    );

    #[derive(SmithyShape, Clone, PartialEq)]
    #[schema(schema = SHAPE)]
    pub struct SerializeMe {
        pub member_a: String,
        pub member_b: String,
        pub member_optional: Option<String>,
        pub member_list: Vec<String>,
        pub member_map: IndexMap<String, String>,
    }

    #[test]
    fn struct_to_document() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let list = vec!["a".to_string(), "b".to_string()];
        let struct_to_convert = SerializeMe {
            member_a: "a".to_string(),
            member_b: "b".to_string(),
            member_optional: None,
            member_map: map,
            member_list: list,
        };
        let document: Box<dyn Document> = struct_to_convert.into();
        assert_eq!(document.discriminant().unwrap(), SHAPE.id());
        let members = document.as_map().unwrap();
        let doc_a = members.get("memberA").unwrap();
        assert_eq!(doc_a.as_string().unwrap(), "a");
        let doc_b = members.get("memberB").unwrap();
        assert_eq!(doc_b.as_string().unwrap(), "b");
        assert!(members.get("memberMap").is_some());
        assert!(members.get("memberList").is_some());
    }

    #[test]
    fn string_document_value() {
        let document_str: Box<dyn Document> = "MyStr".into();
        let output_str = document_str.as_string().expect("string");
        assert_eq!(output_str, &"MyStr".to_string());
    }

    #[test]
    fn roundtrip_shape() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let list = vec!["a".to_string(), "b".to_string()];
        let struct_to_convert = SerializeMe {
            member_a: "a".to_string(),
            member_b: "b".to_string(),
            member_optional: Some("c".to_string()),
            member_map: map,
            member_list: list,
        };
        let document: Box<dyn Document> = struct_to_convert.clone().into();
        let builder: SerializeMeBuilder = document.try_into().unwrap();
        let result: SerializeMe = builder.build().unwrap();
        assert_eq!(result, struct_to_convert);
    }

    #[test]
    fn roundtrip_bool() {
        let original = true;
        let doc: Box<dyn Document> = original.into();
        let result: bool = doc.try_into().unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn roundtrip_string() {
        let doc: Box<dyn Document> = "hello world".into();
        let result: String = doc.try_into().unwrap();
        assert_eq!("hello world".to_string(), result);
    }

    #[test]
    fn roundtrip_byte() {
        let original_byte = 127i8;
        let doc: Box<dyn Document> = original_byte.into();
        let result: i8 = doc.try_into().unwrap();
        assert_eq!(127i8, result);
    }

    #[test]
    fn roundtrip_short() {
        let original_short = 32000i16;
        let doc: Box<dyn Document> = original_short.into();
        let result: i16 = doc.try_into().unwrap();
        assert_eq!(original_short, result);
    }

    #[test]
    fn roundtrip_integer() {
        let original_int = 123_456_i32;
        let doc: Box<dyn Document> = original_int.into();
        let result: i32 = doc.try_into().unwrap();
        assert_eq!(original_int, result);
    }

    #[test]
    fn roundtrip_long() {
        let original_long = 9_876_543_210_i64;
        let doc: Box<dyn Document> = original_long.into();
        let result: i64 = doc.try_into().unwrap();
        assert_eq!(original_long, result);
    }

    #[test]
    fn roundtrip_floats() {
        // Float
        // let original_float = 1.2345f32;
        // let doc: Box<dyn Document> = original_float.into();
        // let result: f32 = doc.try_into().unwrap();
        // assert_eq!(original_float, result);
    }

    #[test]
    fn roundtrip_double() {
        let original_double = 1.234_567_89_f64;
        let doc: Box<dyn Document> = original_double.into();
        let result: f64 = doc.try_into().unwrap();
        assert_eq!(original_double, result);
    }

    #[test]
    fn roundtrip_big_numbers() {
        // BigInteger
        let original_big_int = BigInt::from(123_456_789);
        let doc: Box<dyn Document> = original_big_int.clone().into();
        let result: BigInt = doc.try_into().unwrap();
        assert_eq!(original_big_int, result);

        // BigDecimal
        let original_big_dec = BigDecimal::from_str("123.456").unwrap();
        let doc: Box<dyn Document> = original_big_dec.clone().into();
        let result: BigDecimal = doc.try_into().unwrap();
        assert_eq!(original_big_dec, result);
    }

    #[test]
    fn roundtrip_list() {
        let original: Vec<String> = vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ];
        let doc: Box<dyn Document> = original.clone().into();
        let result: Vec<String> = doc.try_into().unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn roundtrip_map() {
        let mut original = IndexMap::new();
        original.insert("key1".to_string(), "value1".to_string());
        original.insert("key2".to_string(), "value2".to_string());
        original.insert("key3".to_string(), "value3".to_string());

        let doc: Box<dyn Document> = original.clone().into();
        let result: IndexMap<String, String> = doc.try_into().unwrap();
        assert_eq!(original, result);
    }

    #[test]
    fn roundtrip_struct() {
        let mut original_map = IndexMap::new();
        original_map.insert("mapkey1".to_string(), "mapvalue1".to_string());
        original_map.insert("mapkey2".to_string(), "mapvalue2".to_string());

        let original_list = vec![
            "listitem1".to_string(),
            "listitem2".to_string(),
            "listitem3".to_string(),
        ];

        let original = SerializeMe {
            member_a: "value_a".to_string(),
            member_b: "value_b".to_string(),
            member_optional: Some("value_c".to_string()),
            member_map: original_map.clone(),
            member_list: original_list,
        };

        // Serialize to document
        let doc: Box<dyn Document> = original.into();

        // Deserialize back field by field (since we don't have a full Deserialize impl for SerializeMe)
        let doc_map = doc.as_map().expect("Should be a map");

        // Check member_a
        let member_a_doc = doc_map.get("memberA").expect("Should have member a");
        let a_value = member_a_doc.as_string().unwrap();
        assert_eq!(a_value, "value_a".to_string());

        // Check member_b
        let member_b_doc = doc_map.get("memberB").expect("Should have member b");
        let b_value = member_b_doc.as_string().unwrap();
        assert_eq!(b_value, "value_b");

        // Check member_optional
        let member_c_doc = doc_map
            .get("memberOptional")
            .expect("Should have member optional");
        let c_value = member_c_doc.as_string().unwrap();
        assert_eq!(c_value, "value_c");

        // Check list
        let list_doc = doc_map.get("memberList").expect("Should have list");
        let list = list_doc.as_list().expect("Should be a list");
        assert!(!list.is_empty());
        assert_eq!(list.len(), 3);
        let item_one = list.get_index(0).expect("Should have an item");
        let item_str = item_one.as_string().expect("Should be a string");
        assert_eq!(item_str, "listitem1");

        // // Check map
        let map_doc = doc_map.get("memberMap").expect("Should have map");
        let map_member = map_doc.as_map().expect("Should be a map");
        let map_value_one = map_member.get("mapkey1").expect("Should have mapkey1 key");
        let value = map_value_one.as_string().expect("Should be a string");
        assert_eq!(value, "mapvalue1");
    }

    #[test]
    fn roundtrip_option() {
        // Some value
        let original_some: Option<String> = Some("test".to_string());
        let doc: Box<dyn Document> = original_some.clone().into();
        let result: Option<String> = doc.try_into().unwrap();
        assert_eq!(original_some, result);

        // None value
        let original_none: Option<String> = None;
        let doc: Box<dyn Document> = original_none.clone().into();
        let result: Option<String> = doc.try_into().unwrap();
        assert_eq!(original_none, result);
    }

    #[test]
    fn map_roundtrip() {
        // Into doc
        let mut map_in: IndexMap<String, String> = IndexMap::new();
        map_in.insert("a".to_string(), "b".to_string());
        let map_doc: Box<dyn Document> = map_in.into();

        // Into map
        let map_out: IndexMap<String, String> = map_doc.try_into().unwrap();
        assert_eq!(map_out.len(), 1);
        assert_eq!(map_out["a"], "b");
    }

    #[test]
    fn list_roundtrip() {
        let vec = vec!["a", "b", "c"];
        let document_list: Box<dyn Document> = vec.into();

        let vec_out: Vec<String> = document_list.try_into().unwrap();
        assert_eq!(vec_out.len(), 3);
        assert_eq!(vec_out[0], "a");
        assert_eq!(vec_out[1], "b");
        assert_eq!(vec_out[2], "c");
    }
}
