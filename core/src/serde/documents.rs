use std::borrow::Cow;
use std::convert::Infallible;
use std::fmt::Display;
use crate::{
    BigDecimal, BigInt, ByteBuffer, IndexMap, Instant,
    prelude::RequiredTrait,
    schema::{
        Document, DocumentError, Schema, ShapeType, StaticSchemaShape,
        TryFromDocument, Value,
    },
    serde::{
        de::{DeserializeWithSchema, Deserializer, ListReader, MapReader, StructReader},
        deserializers::DeserializableShape,
        se::{ListWriter, MapWriter, Serializer, StructWriter},
        serializers::{Error, SerializeWithSchema},
        utils::KeySerializer,
    },
};
use crate::schema::{DocumentVariant, Number, StaticDocument};
use crate::serde::serializers::SerializableShape;

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

impl SerializeWithSchema for &dyn Document {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // Try to coerce into schema requested type if possible
        let variant = self.as_variant(); // TODO: is schema right here?
        if matches!(variant, DocumentVariant::Null) {
            return serializer.write_null(schema);
        }

        match schema.shape_type() {
            ShapeType::Blob => {
                serializer.write_blob(schema, variant.to_blob().map_err(S::Error::custom)?)
            },
            ShapeType::Boolean => {
                serializer.write_boolean(schema, variant.to_bool().map_err(S::Error::custom)?)
            },
            ShapeType::String | ShapeType::Enum => {
                serializer.write_string(schema, variant.to_str().map_err(S::Error::custom)?)
            }
            ShapeType::Timestamp => {
                serializer.write_timestamp(schema, variant.to_timestamp().map_err(S::Error::custom)?)
            }
            ShapeType::Byte => {
                serializer.write_byte(schema, variant.to_byte().map_err(S::Error::custom)?)
            },
            ShapeType::Short => {
                serializer.write_short(schema, variant.to_short().map_err(S::Error::custom)?)
            },
            ShapeType::Integer | ShapeType::IntEnum => {
                serializer.write_integer(schema, variant.to_integer().map_err(S::Error::custom)?)
            }
            ShapeType::Long => {
                serializer.write_long(schema, variant.to_long().map_err(S::Error::custom)?)
            },
            ShapeType::Float => {
                serializer.write_float(schema, variant.to_float().map_err(S::Error::custom)?)
            },
            ShapeType::Double => {
                serializer.write_double(schema, variant.to_double().map_err(S::Error::custom)?)
            },
            ShapeType::BigInteger => {
                let b = variant.to_big_integer().map_err(S::Error::custom)?;
                serializer.write_big_integer(schema, &b)
            }
            ShapeType::BigDecimal => {
                let d = variant.to_big_decimal().map_err(S::Error::custom)?;
                serializer.write_big_decimal(schema, &d)
            }
            ShapeType::Document => serializer.write_document(schema, &**self),
            ShapeType::List => {
                // TODO: Maybe just impl serialize for slices of serializable?
                let slice = variant.to_list().map_err(S::Error::custom)?;
                let mut list = serializer.write_list(schema, slice.len())?;
                let value_schema = schema
                    .get_list_member()
                    .ok_or_else(|| S::Error::custom("Expected a list schema"))?;
                for element in slice {
                    list.write_element(value_schema, element)?;
                }
                list.end(schema)
            }
            ShapeType::Map => {
                let map = variant.to_map().map_err(S::Error::custom)?;
                map.serialize_with_schema(schema, serializer)
            }
            ShapeType::Structure | ShapeType::Union => {
                let document_map = variant.to_map().map_err(S::Error::custom)?;
                let mut struct_serializer = serializer.write_struct(schema, self.size())?;
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

impl<T: StaticSchemaShape + SerializableShape> From<T> for Box<dyn Document> {
    fn from(shape: T) -> Self {
        let schema = <T as StaticSchemaShape>::schema();
        shape
            .serialize_with_schema(schema, StaticDocumentParser::new(schema))
            .expect("Shape failed to serialize to document. This should never happen.")
    }
}

struct StaticDocumentParser {
    schema: &'static Schema,
}
impl StaticDocumentParser {
    fn new(schema: &'static Schema) -> Self {
        StaticDocumentParser { schema }
    }
}
impl Serializer for StaticDocumentParser {
    type Error = Infallible;
    type Ok = Box<dyn Document>;
    type ListWriter = DocumentListAccumulator;
    type MapWriter = DocumentMapAccumulator;
    type StructWriter = DocumentStructAccumulator;

    #[inline]
    fn write_struct(self, _schema: &Schema, len: usize) -> Result<Self::StructWriter, Self::Error> {
        Ok(DocumentStructAccumulator {
            root_schema: self.schema,
            values: IndexMap::with_capacity(len),
        })
    }

    #[inline]
    fn write_map(self, _schema: &Schema, len: usize) -> Result<Self::MapWriter, Self::Error> {
        let value_schema = self.schema.get_member("value").expect("must be map schema");
        Ok(DocumentMapAccumulator {
            root_schema: self.schema,
            value_schema,
            values: IndexMap::with_capacity(len),
        })
    }

    #[inline]
    fn write_list(self, _schema: &Schema, len: usize) -> Result<Self::ListWriter, Self::Error> {

        let item_schema = self.schema.get_member("member")
            .expect("must be list");
        Ok(DocumentListAccumulator {
            root_schema: self.schema,
            item_schema,
            values: Vec::with_capacity(len),
        })
    }

    #[inline]
    fn write_boolean(self, _schema: &Schema, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            value: Value::Boolean(value),
            schema: self.schema
        }.into())
    }

    #[inline]
    fn write_byte(self, _schema: &Schema, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            value: Value::Number(Number::Byte(value)),
            schema: self.schema
        }.into())
    }

    #[inline]
    fn write_short(self, _schema: &Schema, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            value: Value::Number(Number::Short(value)),
            schema: self.schema
        }.into())
    }

    #[inline]
    fn write_integer(self, _schema: &Schema, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            value: Value::Number(Number::Integer(value)),
            schema: self.schema
        }.into())
    }

    #[inline]
    fn write_long(self, _schema: &Schema, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            value: Value::Number(Number::Long(value)),
            schema: self.schema
        }.into())
    }

    #[inline]
    fn write_float(self, _schema: &Schema, value: f32) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            value: Value::Number(Number::Float(value)),
            schema: self.schema
        }.into())
    }

    #[inline]
    fn write_double(self, _schema: &Schema, value: f64) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            value: Value::Number(Number::Double(value)),
            schema: self.schema
        }.into())
    }

    #[inline]
    fn write_big_integer(self, _schema: &Schema, value: &BigInt) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            value: Value::Number(Number::BigInt(value.clone())),
            schema: self.schema
        }.into())
    }

    #[inline]
    fn write_big_decimal(
        self,
        _schema: &Schema,
        value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            value: Value::Number(Number::BigDecimal(value.clone())),
            schema: self.schema
        }.into())
    }

    #[inline]
    fn write_string(self, _schema: &Schema, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            value: Value::String(value.to_string()),
            schema: self.schema
        }.into())
    }

    #[inline]
    fn write_blob(self, _schema: &Schema, value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            value: Value::Blob(value.clone()),
            schema: self.schema
        }.into())
    }

    #[inline]
    fn write_timestamp(self, _schema: &Schema, value: Instant) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            value: Value::Timestamp(value),
            schema: self.schema
        }.into())
    }

    #[inline]
    fn write_document(
        self,
        _schema: &Schema,
        value: &dyn Document,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(value.box_clone())
    }

    #[inline]
    fn write_null(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            value: Value::Null,
            schema: self.schema
        }.into())
    }

    #[inline]
    fn skip(self, schema: &Schema) -> Result<Self::Ok, Self::Error> {
        // When skipping (e.g., for None values), return a null document
        self.write_null(schema)
    }
}

#[doc(hidden)]
pub struct DocumentListAccumulator {
    root_schema: &'static Schema,
    item_schema: &'static Schema,
    values: Vec<Box<dyn Document>>,
}
impl ListWriter for DocumentListAccumulator {
    type Error = Infallible;
    type Ok = Box<dyn Document>;

    #[inline]
    fn write_element<T>(&mut self, element_schema: &Schema, value: &T) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        let el = value.serialize_with_schema(element_schema, StaticDocumentParser {
            schema: self.item_schema,
        })?;
        self.values.push(el);
        Ok(())
    }

    #[inline]
    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            schema: self.root_schema,
            value: Value::List(self.values),
        }
        .into())
    }
}

#[doc(hidden)]
pub struct DocumentMapAccumulator {
    root_schema: &'static Schema,
    value_schema: &'static Schema,
    values: IndexMap<Cow<'static, str>, Box<dyn Document>>,
}
impl MapWriter for DocumentMapAccumulator {
    type Error = Infallible;
    type Ok = Box<dyn Document>;

    #[inline]
    fn write_entry<K, V>(
        &mut self,
        key_schema: &Schema,
        value_schema: &Schema,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: SerializeWithSchema,
        V: SerializeWithSchema,
    {
        // Serialize the key to get its string representation
        let key_str = key.serialize_with_schema(key_schema, &mut KeySerializer::<Infallible>::new())?;
        let val = value.serialize_with_schema(value_schema, StaticDocumentParser { schema: self.value_schema })?;
        self.values.insert(Cow::Owned(key_str), val);
        Ok(())
    }

    #[inline]
    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            schema: self.root_schema,
            value: Value::Map(self.values),
        }
        .into())
    }
}

struct DocumentStructAccumulator {
    root_schema: &'static Schema,
    values: IndexMap<Cow<'static, str>, Box<dyn Document>>,
}
impl StructWriter for DocumentStructAccumulator {
    type Error = Infallible;
    type Ok = Box<dyn Document>;

    
    #[inline]
    fn write_member<T>(&mut self, member_schema: &Schema, value: &T) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        let Some(me) = member_schema.as_member() else {
            unreachable!()
        };
        let static_member = self.root_schema.get_member(me.name()).expect("must exist");
        let val = value.serialize_with_schema(member_schema, StaticDocumentParser { schema: static_member })?;
        self.values.insert(Cow::Owned(me.name().to_string()), val);
        Ok(())
    }

    fn write_member_named<T>(&mut self, member_name: &'static str, member_schema: &Schema, value: &T) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema
    {
        let Some(me) = member_schema.as_member() else {
            unreachable!()
        };
        let static_member = self.root_schema.get_member(me.name()).expect("must exist");
        let val = value.serialize_with_schema(member_schema, StaticDocumentParser { schema: static_member })?;
        self.values.insert(Cow::Borrowed(member_name), val);
        Ok(())
    }

    #[inline]
    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(StaticDocument {
            schema: self.root_schema,
            value: Value::Map(self.values),
        }
        .into())
    }
}

// ============================================================================
// Deserialization FROM Document
// ============================================================================

impl<'de, S: DeserializableShape<'de>> TryFromDocument for S {
    #[inline]
    fn try_from(document: Box<dyn Document>) -> Result<Self, DocumentError> {
        S::deserialize(DocumentDeserializer::new(document))
    }
}

/// A deserializer that reads from a `Document`.
struct DocumentDeserializer {
    document: Box<dyn Document>,
}

impl DocumentDeserializer {
    pub fn new(document: Box<dyn Document>) -> Self {
        Self { document }
    }
}

/// Reader for struct members from a Document.
///
/// Callers must alternate `read_member()` and `read_value()`/`skip_value()` calls.
struct DocumentStructReader {
    iter: indexmap::map::IntoIter<Cow<'static, str>, Box<dyn Document>>,
    current_value: Option<Box<dyn Document>>,
}

/// Reader for list elements from a Document.
struct DocumentListReader {
    iter: std::vec::IntoIter<Box<dyn Document>>,
}

/// Reader for map entries from a Document.
///
/// Callers must alternate `read_key()` and `read_value()`/`skip_value()` calls.
struct DocumentMapReader {
    iter: indexmap::map::IntoIter<Cow<'static, str>, Box<dyn Document>>,
    current_value: Option<Box<dyn Document>>,
}

impl Deserializer<'_> for DocumentDeserializer {
    type Error = DocumentError;
    type StructReader = DocumentStructReader;
    type ListReader = DocumentListReader;
    type MapReader = DocumentMapReader;

    #[inline]
    fn read_bool(self, _schema: &Schema) -> Result<bool, Self::Error> {
        self.document.as_variant().to_bool()
    }

    #[inline]
    fn read_byte(self, _schema: &Schema) -> Result<i8, Self::Error> {
        self.document.as_variant().to_byte()
    }

    #[inline]
    fn read_short(self, _schema: &Schema) -> Result<i16, Self::Error> {
        self.document.as_variant().to_short()
    }

    #[inline]
    fn read_integer(self, _schema: &Schema) -> Result<i32, Self::Error> {
        self.document.as_variant().to_integer()
    }

    #[inline]
    fn read_long(self, _schema: &Schema) -> Result<i64, Self::Error> {
        self.document.as_variant().to_long()
    }

    #[inline]
    fn read_float(self, _schema: &Schema) -> Result<f32, Self::Error> {
        self.document.as_variant().to_float()
    }

    #[inline]
    fn read_double(self, _schema: &Schema) -> Result<f64, Self::Error> {
        self.document.as_variant().to_double()
    }

    #[inline]
    fn read_big_integer(self, _schema: &Schema) -> Result<BigInt, Self::Error> {
        let cow_data = self.document.as_variant().to_big_integer()?;
        Ok(cow_data.into_owned())
    }

    #[inline]
    fn read_big_decimal(self, _schema: &Schema) -> Result<BigDecimal, Self::Error> {
        let cow_data = self.document.as_variant().to_big_decimal()?;
        Ok(cow_data.into_owned())
    }

    #[inline]
    fn read_string(self, _schema: &Schema) -> Result<String, Self::Error> {
        let str = self.document.as_variant().to_str()?;
        Ok(str.to_string())
    }

    #[inline]
    fn read_blob(self, _schema: &Schema) -> Result<ByteBuffer, Self::Error> {
        let b = self.document.as_variant().to_blob()?;
        Ok(b.clone())
    }

    #[inline]
    fn read_timestamp(self, _schema: &Schema) -> Result<Instant, Self::Error> {
        self.document.as_variant().to_timestamp()
    }

    #[inline]
    fn read_document(self, _schema: &Schema) -> Result<Box<dyn Document>, Self::Error> {
        Ok(self.document)
    }

    #[inline]
    fn read_struct(self, _schema: &Schema) -> Result<Self::StructReader, Self::Error> {
        Ok(DocumentStructReader {
            iter: self.document.try_into_map()?.into_iter(),
            current_value: None,
        })
    }

    #[inline]
    fn read_list(self, _schema: &Schema) -> Result<Self::ListReader, Self::Error> {
        Ok(DocumentListReader {
            iter: self.document.try_into_list()?.into_iter(),
        })
    }

    #[inline]
    fn read_map(self, _schema: &Schema) -> Result<Self::MapReader, Self::Error> {
        Ok(DocumentMapReader {
            iter: self.document.try_into_map()?.into_iter(),
            current_value: None,
        })
    }

    #[inline]
    fn is_null(&mut self) -> bool {
        matches!(self.document.as_variant(), DocumentVariant::Null)
    }

    fn read_null(mut self) -> Result<(), Self::Error> {
        if self.is_null() {
            Ok(())
        } else {
            Err(DocumentError::DocumentConversion(
                "Expected null document".to_string(),
            ))
        }
    }
}

// ============================================================================
// Document Reader Implementations
// ============================================================================

impl<'de> StructReader<'de> for DocumentStructReader {
    type Error = DocumentError;

    fn read_member<'a>(&mut self, schema: &'a Schema) -> Result<Option<&'a Schema>, Self::Error> {
        loop {
            match self.iter.next() {
                Some((key, value)) => {
                    if let Some(member_schema) = schema.get_member(&key) {
                        self.current_value = Some(value);
                        return Ok(Some(member_schema));
                    }
                    // Unknown key — skip internally and continue
                }
                None => return Ok(None),
            }
        }
    }

    fn read_value<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<T, Self::Error> {
        let doc = self.current_value.take().ok_or_else(|| {
            DocumentError::DocumentConversion("No current member to read".to_string())
        })?;
        let de = DocumentDeserializer::new(doc);
        T::deserialize_with_schema(schema, de)
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.current_value = None;
        Ok(())
    }
}

impl<'de> ListReader<'de> for DocumentListReader {
    type Error = DocumentError;

    fn read_element<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<Option<T>, Self::Error> {
        match self.iter.next() {
            Some(doc) => {
                let de = DocumentDeserializer::new(doc);
                let value = T::deserialize_with_schema(schema, de)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

impl<'de> MapReader<'de> for DocumentMapReader {
    type Error = DocumentError;

    fn read_key(&mut self) -> Result<Option<String>, Self::Error> {
        match self.iter.next() {
            Some((key, value)) => {
                self.current_value = Some(value);
                Ok(Some(key.to_string()))
            }
            None => Ok(None),
        }
    }

    fn read_value<V: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<V, Self::Error> {
        let doc = self.current_value.take().ok_or_else(|| {
            DocumentError::DocumentConversion("No current value to read".to_string())
        })?;
        V::deserialize_with_schema(schema, DocumentDeserializer::new(doc))
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.current_value = None;
        Ok(())
    }
}

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

    use super::*;
    use crate::{derive::SmithyShape, schema::prelude::*, smithy};

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
            member_optional: Some("c".to_string()),
            member_map: map,
            member_list: list,
        };
        let document: Box<dyn Document> = struct_to_convert.into();
        assert_eq!(document.discriminator().unwrap(), SHAPE.id());
        let members = document.try_into_map().unwrap();
        let doc_a = &members.get("memberA").unwrap();
        assert_eq!(doc_a.as_variant().to_str().unwrap(), "a");
        let doc_b = &members.get("memberB").unwrap();
        assert_eq!(doc_b.as_variant().to_str().unwrap(), "b");
        let doc_c = &members.get("memberOptional").unwrap();
        assert_eq!(doc_c.as_variant().to_str().unwrap(), "c");
        assert!(members.contains_key("memberMap"));
        assert!(members.contains_key("memberList"));
    }

    #[test]
    fn string_document_value() {
        let document_str: Box<dyn Document> = "MyStr".into();
        let output_str = document_str.as_variant().to_str().expect("string");
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
            member_optional: Some("value_c".to_string()),
            member_map: original_map.clone(),
            member_list: original_list.clone(),
        };

        // Serialize to document
        let doc: Box<dyn Document> = original.into();

        // Deserialize back field by field (since we don't have a full Deserialize impl for SerializeMe)
        let doc_map = doc.try_into_map().expect("Should be a map");

        // Check member_a
        let member_a_doc = doc_map
            .get("memberA")
            .expect("Should have member a")
            .clone();
        let a_value: String = member_a_doc.try_into().unwrap();
        assert_eq!(a_value, "value_a".to_string());

        // Check member_b
        let member_b_doc = doc_map
            .get("memberB")
            .expect("Should have member b")
            .clone();
        let b_value: String = member_b_doc.try_into().unwrap();
        assert_eq!(b_value, "value_b");

        // Check member_optional
        let member_c_doc = doc_map
            .get("memberOptional")
            .expect("Should have member optional")
            .clone();
        let c_value: String = member_c_doc.try_into().unwrap();
        assert_eq!(c_value, "value_c");

        // Check list
        let list_doc = doc_map.get("memberList").expect("Should have list").clone();
        let list_value: Vec<String> = list_doc.try_into().unwrap();
        assert_eq!(list_value, original_list);

        // Check map
        let map_doc = doc_map.get("memberMap").expect("Should have map").clone();
        let map_value: IndexMap<String, String> = map_doc.try_into().unwrap();
        assert_eq!(map_value, original_map);
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
