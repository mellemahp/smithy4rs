
use std::fmt::Display;
use crate::{
    prelude::RequiredTrait,
    schema::{
        Document, DocumentError, Schema, ShapeType,
        TryFromDocument,
    },
    serde::{
        de::{DeserializeWithSchema, Deserializer, ListReader, MapReader, StructReader},
        deserializers::DeserializableShape,
        se::{ListWriter, MapWriter, Serializer, StructWriter},
        serializers::{Error, SerializeWithSchema},
    },
};
use crate::prelude::{DOCUMENT, STRING};

// ============================================================================
// Errors
// ============================================================================

impl Error for DocumentError {
    fn custom<T: Display>(msg: T) -> Self {
        DocumentError::CustomError(msg.to_string())
    }
}

// ============================================================================
// Serialization of a document
// ============================================================================

impl SerializeWithSchema for Box<dyn Document> {
    fn serialize_with_schema<S: Serializer>(&self, schema: &Schema, serializer: S) -> Result<S::Ok, S::Error> {
        let doc = self.as_ref();
        doc.serialize_with_schema(schema, serializer)
    }
}

impl SerializeWithSchema for dyn Document + '_ {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // Try to coerce into schema requested type if possible
        if self.is_null() {
            return serializer.write_null(schema);
        }
        // Try to coerce into schema requested type if possible
        match schema.shape_type() {
            ShapeType::Blob => {
                serializer.write_blob(schema, self.as_blob().map_err(S::Error::custom)?)
            },
            ShapeType::Boolean => {
                serializer.write_boolean(schema, self.as_boolean().map_err(S::Error::custom)?)
            },
            ShapeType::String | ShapeType::Enum => {
                serializer.write_string(schema, self.as_str().map_err(S::Error::custom)?)
            }
            ShapeType::Timestamp => {
                serializer.write_timestamp(schema, self.as_timestamp().map_err(S::Error::custom)?)
            }
            ShapeType::Byte => {
                serializer.write_byte(schema, self.as_byte().map_err(S::Error::custom)?)
            },
            ShapeType::Short => {
                serializer.write_short(schema, self.as_short().map_err(S::Error::custom)?)
            },
            ShapeType::Integer | ShapeType::IntEnum => {
                serializer.write_integer(schema, self.as_integer().map_err(S::Error::custom)?)
            }
            ShapeType::Long => {
                serializer.write_long(schema, self.as_long().map_err(S::Error::custom)?)
            },
            ShapeType::Float => {
                serializer.write_float(schema, self.as_float().map_err(S::Error::custom)?)
            },
            ShapeType::Double => {
                serializer.write_double(schema, self.as_double().map_err(S::Error::custom)?)
            },
            ShapeType::BigInteger => {
                let b = self.as_big_integer().map_err(S::Error::custom)?;
                serializer.write_big_integer(schema, &b)
            }
            ShapeType::BigDecimal => {
                let d = self.as_big_decimal().map_err(S::Error::custom)?;
                serializer.write_big_decimal(schema, &d)
            }
            ShapeType::Document => serializer.write_document(schema, self),
            ShapeType::List => {
                let cursor = self.as_list().map_err(S::Error::custom)?;
                let mut list = serializer.write_list(schema, cursor.len())?;
                let value_schema = schema
                    .get_list_member()
                    .ok_or_else(|| S::Error::custom("Expected a list schema"))?;
                let mut i = 0;
                while i < cursor.len() {
                    let item = cursor.get(i).ok_or_else(|| S::Error::custom("Expected document"))?;
                    list.write_element(value_schema, item)?;
                    i += 1;
                }
                list.end(schema)
            }
            ShapeType::Map => {
                let cursor = self.as_map().map_err(S::Error::custom)?;
                let (key_schema, value_schema) = schema.get_key_value().ok_or_else(|| S::Error::custom("Expected a map schema"))?;
                let mut map = serializer.write_map(schema, cursor.len())?;

                let mut i = 0;
                while i < cursor.len() {
                    if let Some((key, child_doc)) = cursor.get_entry(i) {
                        map.write_entry(key_schema, value_schema, key, child_doc)?;
                    }
                    i += 1;
                }
                map.end(schema)
            }
            ShapeType::Structure | ShapeType::Union => {
                let document_map = self.as_map().map_err(S::Error::custom)?;
                let mut struct_serializer = serializer.write_struct(&DOCUMENT, self.len())?;
                if let Some(discriminator) = &self.discriminator() {
                    struct_serializer.write_discriminator(discriminator)?;
                }
                for (key, member_schema) in schema.members() {
                    match (
                        member_schema.get_trait_as::<RequiredTrait>(),
                        document_map.get(key),
                    ) {
                        (Some(_), None) => {
                            struct_serializer.write_unknown(member_schema, key)?;
                        }
                        (Some(_) | None, Some(value)) => {
                            struct_serializer.write_member(member_schema, value)?;
                        }
                        (None, None) => {
                            struct_serializer.skip_member(member_schema)?;
                        }
                    }
                }
                struct_serializer.end(schema)
            }
            _ => Err(Error::custom("Unsupported shape type")),
        }
    }
}

// ============================================================================
// Deserialize a generated shape into a document
//
// NOTE: This consumes the shape
// ============================================================================

// impl<'de, T: StaticSchemaShape + DeserializeWithSchema<'de>> From<T> for Box<dyn Document> {
//     fn from(shape: T) -> Self {
//         let schema = <T as StaticSchemaShape>::schema();
//
//     }
// }

// ============================================================================
// Deserialization FROM Document
// ============================================================================

/// Reader structure for collection (i.e. List, Map, Struct) document types.
///
/// Note: Reading a collection will `take` its values so it will no longer be
/// usable for re-read and should be dropped.
pub struct DocumentCollectionReader<'b, 'doc> {
    /// Inner document.
    ///
    /// Note: Should always represent a collection (Struct, List, Map)
    inner: &'b mut (dyn Document + 'doc),
    /// Current index of the reader
    idx: usize,
}
impl <'b, 'doc> DocumentCollectionReader<'b, 'doc> {
    #[inline]
    fn new(inner: &'b mut (dyn Document + 'doc)) -> Self {
        Self { inner, idx: 0 }
    }
}

impl <'de> ListReader<'de> for DocumentCollectionReader<'_, '_>
{
    type Error = DocumentError;

    fn read_element<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<Option<T>, Self::Error> {
        let Some(elem) = self.inner.get_element_mut(self.idx)? else {
            return Ok(None)
        };
        self.idx += 1;
        T::deserialize_with_schema(schema, elem).map(Some)
    }
}

impl <'de> MapReader<'de> for DocumentCollectionReader<'_, '_>{
    type Error = DocumentError;

    fn read_key(&mut self) -> Result<Option<String>, Self::Error> {
        let Some(key) = self.inner.get_key_mut(self.idx)? else {
            return Ok(None)
        };
        Ok(Some(key.take_string(&STRING)?))
    }

    fn read_value<V: DeserializeWithSchema<'de>>(&mut self, schema: &Schema) -> Result<V, Self::Error> {
        // Todo: Seems a bit off to read key if we dont use.
        let (_k, elem) = self.inner.get_entry_mut(self.idx)?;
        self.idx += 1;
        V::deserialize_with_schema(schema, elem)
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl <'de> StructReader<'de> for DocumentCollectionReader<'_, '_>{
    type Error = DocumentError;

    fn read_member<'a>(&mut self, schema: &'a Schema) -> Result<Option<&'a Schema>, Self::Error> {
        Ok(schema.members().get_index(self.idx).map(|(_, v)| v))
    }

    fn read_value<T: DeserializeWithSchema<'de>>(&mut self, schema: &Schema) -> Result<T, Self::Error> {
        let (_key, doc) = self.inner.get_entry_mut(self.idx)?;
        self.idx += 1;
        T::deserialize_with_schema(schema, doc)
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.idx += 1;
        Ok(())
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.inner.len())
    }
}


impl<'b, 'doc> Deserializer<'_> for &'b mut (dyn Document + 'doc)
{
    type Error = DocumentError;
    // Assuming your StructReader uses a similar unboxed/stack pattern
    type StructReader = DocumentCollectionReader<'b, 'doc>;
    type ListReader = DocumentCollectionReader<'b, 'doc>;
    type MapReader = DocumentCollectionReader<'b, 'doc>;

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
        Ok(DocumentCollectionReader::new(self))
    }

    #[inline]
    fn read_list(self, _schema: &Schema) -> Result<Self::ListReader, Self::Error> {
        Ok(DocumentCollectionReader::new(self))
    }

    #[inline]
    fn read_map(self, _schema: &Schema) -> Result<Self::MapReader, Self::Error> {
        Ok(DocumentCollectionReader::new(self))
    }

    fn is_null(&mut self) -> bool {
        // TODO: on top-level?
        self.value().is_null()
    }

    fn read_null(self) -> Result<(), Self::Error> {
        if self.value().is_null() {
            Ok(())
        } else {
            // TODO: More appropriate error?
            Err(DocumentError::DocumentConversion("Document is not null".into()))
        }
    }
}

// ===== Blanket Try From =====
impl<'de, S: DeserializableShape<'de>> TryFromDocument for S {
    #[inline]
    fn try_from<'a>(document: &mut (dyn Document + 'a)) -> Result<Self, DocumentError> {
        S::deserialize(document)
    }
}

// ============================================================================
// Document Reader Implementations
// ============================================================================

    //
    // fn read_member<'a>(&mut self, schema: &'a Schema) -> Result<Option<&'a Schema>, Self::Error> {
    //     loop {
    //         match self.iter.next() {
    //             Some((key, value)) => {
    //                 if let Some(member_schema) = schema.get_member(&key) {
    //                     self.current_value = Some(value);
    //                     return Ok(Some(member_schema));
    //                 }
    //                 // Unknown key — skip internally and continue
    //             }
    //             None => return Ok(None),
    //         }
    //     }
    // }

    // fn read_value<T: DeserializeWithSchema<'de>>(
    //     &mut self,
    //     schema: &Schema,
    // ) -> Result<T, Self::Error> {
    //     let doc = self.current_value.take().ok_or_else(|| {
    //         DocumentError::DocumentConversion("No current member to read".to_string())
    //     })?;
    //     let de = DocumentDeserializer::new(doc);
    //     T::deserialize_with_schema(schema, de)
    // }

// === Needed for Trait deser ===
impl<'de> DeserializeWithSchema<'de> for Box<dyn Document> {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.read_document(schema)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use indexmap::IndexMap;
    use super::*;
    use crate::{derive::SmithyShape, schema::prelude::*, smithy, BigInt, BigDecimal};
    use crate::schema::{DocumentValue, MapCursor, ShapeId};

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
            memberList: LIST
            memberMap: MAP
        }
    );
    //          memberOptional: STRING


    #[derive(SmithyShape, Clone, PartialEq)]
    #[schema(schema = SHAPE)]
    pub struct SerializeMe {
        pub member_a: String,
        pub member_b: String,
        // pub member_optional: Option<String>,
        pub member_list: Vec<String>,
        pub member_map: IndexMap<String, String>,
    }

    impl Document for SerializeMe {
        #[inline]
        fn discriminator(&self) -> Option<&ShapeId> {
            Some(SHAPE.id())
        }

        #[inline]
        fn value(&self) -> DocumentValue<'_> {
            DocumentValue::Map(self)
        }

        fn len(&self) -> usize {
            3
        }

        fn get_entry_mut(&mut self, index: usize) -> Result<(&str, &mut dyn Document), DocumentError> {
            let out: (&str, &mut dyn Document) = match index {
                0 => ("memberA", &mut self.member_a),
                1 => ("memberB", &mut self.member_b),
                // 2 => ("memberOptional", &mut self.member_optional),
                2 => ("memberList", &mut self.member_list),
                3 => ("memberMap", &mut self.member_map),
                _ => {
                    return Err(DocumentError::CustomError("Invalid index".to_string()));
                }
            };
            Ok(out)
        }
    }

    impl MapCursor for SerializeMe {
        #[inline]
        fn get_key(&self, idx: usize) -> Option<&str> {
            match idx {
                0 => Some("memberA"),
                1 => Some("memberB"),
                // 2 => Some("memberOptional"),
                2 => Some("memberList"),
                3 => Some("memberMap"),
                _ => None,
            }
        }

        #[inline]
        #[allow(trivial_casts)]
        fn get(&self, key: &str) -> Option<&dyn Document> {
            match key {
                "memberA" => Some(&self.member_a),
                "memberB" => Some(&self.member_b),
                // "memberOptional" => self.member_optional.as_ref().map(|v| v as &dyn Document),
                "memberList" => Some(&self.member_list),
                "memberMap" => Some(&self.member_map),
                _ => None
            }
        }

        fn get_entry(&self, idx: usize) -> Option<(&str, &dyn Document)> {
            match idx {
                0 => Some(("memberA", &self.member_a)),
                1 => Some(("memberB", &self.member_b)),
                // 2 => Some(("memberOptional", &self.member_optional)),
                3 => Some(("memberList", &self.member_list)),
                4 => Some(("memberMap", &self.member_map)),
                _ => None,
            }
        }

        fn len(&self) -> usize {
            5
        }
    }

    #[test]
    fn struct_to_document() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let list = vec!["a".to_string(), "b".to_string()];
        let struct_to_convert = SerializeMe {
            member_a: "a".to_string(),
            member_b: "b".to_string(),
            // member_optional: None,
            member_map: map,
            member_list: list,
        };
        let document: Box<dyn Document> = struct_to_convert.into();
        assert_eq!(document.discriminator().unwrap(), SHAPE.id());
        let members = document.as_map().unwrap();
        let doc_a = members.get("memberA").unwrap();
        assert_eq!(doc_a.as_str().unwrap(), "a");
        let doc_b = members.get("memberB").unwrap();
        assert_eq!(doc_b.as_str().unwrap(), "b");
        // TODO: Add back in

        // let doc_c = &members.get("memberOptional").unwrap();
        // assert_eq!(doc_c.as_str().unwrap(), "c");
        // TODO: check values?
        assert!(members.get("memberMap").is_some());
        assert!(members.get("memberList").is_some());
    }

    #[test]
    fn string_document_value() {
        let document_str: Box<dyn Document> = "MyStr".into();
        let output_str = document_str.as_str().expect("string");
        assert_eq!(output_str, &"MyStr".to_string());
    }

    #[test]
    #[ignore = "todo"]
    fn number_document_values() {
        // TODO: Add number document tests.
    }

    // Roundtrip tests: value -> serialize to Document -> deserialize back to value

    #[test]
    fn roundtrip_shape() {
        let mut map = IndexMap::new();
        map.insert(String::from("a"), String::from("b"));
        let list = vec!["a".to_string(), "b".to_string()];
        let struct_to_convert = SerializeMe {
            member_a: "a".to_string(),
            member_b: "b".to_string(),
            // member_optional: Some("c".to_string()),
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
    fn roundtrip_integers() {
        // TODO: CORRECT
        // // Byte
        // let original_byte = 127i8;
        // let doc: Box<dyn Document> = original_byte.into();
        // let result: i8 = doc.try_into().unwrap();
        // assert_eq!(127i8, result);
        //
        // // Short
        // let original_short = 32000i16;
        // let doc: Box<dyn Document> = original_short.into();
        // let result: i16 = doc.try_into().unwrap();
        // assert_eq!(original_short, result);
        //
        // // Integer
        // let original_int = 123_456_i32;
        // let doc: Box<dyn Document> = original_int.into();
        // let result: i32 = doc.try_into().unwrap();
        // assert_eq!(original_int, result);
        //
        // // Long
        // let original_long = 9_876_543_210_i64;
        // let doc: Box<dyn Document> = original_long.into();
        // let result: i64 = doc.try_into().unwrap();
        // assert_eq!(original_long, result);
    }

    #[test]

    fn roundtrip_floats() {
        // Float
        // let original_float = 1.2345f32;
        // let doc: Box<dyn Document> = original_float.into();
        // let result: f32 = doc.try_into().unwrap();
        // assert_eq!(original_float, result);
        //
        // // Double
        // let original_double = 1.234_567_89_f64;
        // let doc: Box<dyn Document> = original_double.into();
        // let result: f64 = doc.try_into().unwrap();
        // assert_eq!(original_double, result);
    }

    #[test]
    #[ignore = "BigDecimal/BigInteger serialization not yet implemented"]
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
            // member_optional: Some("value_c".to_string()),
            member_map: original_map.clone(),
            member_list: original_list,
        };

        // Serialize to document
        let doc: Box<dyn Document> = original.into();

        // Deserialize back field by field (since we don't have a full Deserialize impl for SerializeMe)
        let doc_map = doc.as_map().expect("Should be a map");

        // Check member_a
        let member_a_doc = doc_map
            .get("memberA")
            .expect("Should have member a");
        let a_value = member_a_doc.as_str().unwrap();
        assert_eq!(a_value, "value_a".to_string());

        // Check member_b
        let member_b_doc = doc_map
            .get("memberB")
            .expect("Should have member b");
        // TODO: Use try_into impl
        let b_value = member_b_doc.as_str().unwrap();
        assert_eq!(b_value, "value_b");

        // Check member_optional
        // let member_c_doc = doc_map
        //     .get("memberOptional")
        //     .expect("Should have member optional");
        // let c_value = member_c_doc.as_str().unwrap();
        // assert_eq!(c_value, "value_c");

        // TODO: RE-ADD

        // Check list
        // let list_doc = doc_map.get("memberList").expect("Should have list").clone();
        // let list_value: Vec<String> = list_doc.try_into().unwrap();
        // assert_eq!(list_value, original_list);
        //
        // // Check map
        // let map_doc = doc_map.get("memberMap").expect("Should have map").clone();
        // let map_value: IndexMap<String, String> = map_doc.try_into().unwrap();
        // assert_eq!(map_value, original_map);
    }

    #[test]
    fn roundtrip_option() {
        // TODO: FIX!
        // Some value
        // let original_some: Option<String> = Some("test".to_string());
        // let doc: Box<dyn Document> = original_some.clone().into();
        // let result: Option<String> = doc.try_into().unwrap();
        // assert_eq!(original_some, result);
        //
        // // None value
        // let original_none: Option<String> = None;
        // let doc: Box<dyn Document> = original_none.clone().into();
        // let result: Option<String> = doc.try_into().unwrap();
        // assert_eq!(original_none, result);
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

    #[test]

    fn float_roundtrip() {
        // let float: Box<dyn Document> = 1f32.into();
        // let float_value: f32 = float.try_into().unwrap();
        // assert_eq!(float_value, 1f32);
    }

    #[test]

    fn double_roundtrip() {
        // let double: Box<dyn Document> = 1f64.into();
        // let double_value: f64 = double.try_into().unwrap();
        // assert_eq!(double_value, 1f64);
    }
}
