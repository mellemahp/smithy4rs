use std::fmt::Display;

use crate::{
    BigDecimal, BigInt, ByteBuffer, IndexMap, Instant,
    schema::{
        Document, DocumentError, NULL, Schema, ShapeId, ShapeType, StaticSchemaShape,
        default::Value,
    },
    serde::{
        ShapeBuilder,
        de::{DeserializeWithSchema, Deserializer},
        se::{ListSerializer, MapSerializer, Serializer, StructSerializer},
        serializers::{Error, SerializeWithSchema},
        utils::KeySerializer,
    },
};
// ============================================================================
// Serialization
// ============================================================================

impl SerializeWithSchema for Box<dyn Document> {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // TODO(errors): Handle exceptions?
        match self.get_type() {
            Some(ShapeType::Blob) => serializer.write_blob(schema, self.as_blob().unwrap()),
            Some(ShapeType::Boolean) => serializer.write_boolean(schema, self.as_bool().unwrap()),
            Some(ShapeType::String | ShapeType::Enum) => {
                serializer.write_string(schema, self.as_string().unwrap())
            }
            Some(ShapeType::Timestamp) => {
                serializer.write_timestamp(schema, self.as_timestamp().unwrap())
            }
            Some(ShapeType::Byte) => serializer.write_byte(schema, self.as_byte().unwrap()),
            Some(ShapeType::Short) => serializer.write_short(schema, self.as_short().unwrap()),
            Some(ShapeType::Integer | ShapeType::IntEnum) => {
                serializer.write_integer(schema, self.as_integer().unwrap())
            }
            Some(ShapeType::Long) => serializer.write_long(schema, self.as_long().unwrap()),
            Some(ShapeType::Float) => serializer.write_float(schema, self.as_float().unwrap()),
            Some(ShapeType::Double) => serializer.write_double(schema, self.as_double().unwrap()),
            Some(ShapeType::BigInteger) => {
                serializer.write_big_integer(schema, self.as_big_integer().unwrap())
            }
            Some(ShapeType::BigDecimal) => {
                serializer.write_big_decimal(schema, self.as_big_decimal().unwrap())
            }
            Some(ShapeType::List) => self
                .as_list()
                .unwrap()
                .serialize_with_schema(schema, serializer),
            Some(ShapeType::Map) => self
                .as_map()
                .unwrap()
                .serialize_with_schema(schema, serializer),
            Some(ShapeType::Structure | ShapeType::Union) => {
                let document_map = self.as_map().unwrap();
                let mut struct_serializer = serializer.write_struct(schema, self.size())?;
                if let Some(discriminator) = &self.discriminator() {
                    struct_serializer.serialize_discriminator(discriminator)?;
                }
                for (key, value) in document_map {
                    if let Some(member_schema) = schema.get_member(key) {
                        struct_serializer.serialize_member(member_schema, value)?;
                    } else {
                        // TODO(unknown members) Should unknown members be allowed?
                        todo!("Add some logging on unknown members");
                    }
                }
                struct_serializer.end(schema)
            }
            None => serializer.write_null(schema),
            _ => Err(Error::custom("Unsupported shape type")),
        }
    }
}

impl<T> From<T> for Box<dyn Document>
where
    T: StaticSchemaShape + SerializeWithSchema,
{
    fn from(shape: T) -> Self {
        shape
            .serialize_with_schema(T::schema(), DocumentParser)
            .expect(
                "Infallible conversion from StaticSchemaShape to Document failed - this is a bug",
            )
    }
}

// ====== Public Conversion API ========

impl dyn Document {
    /// Convert a document into a [`ShapeBuilder`]
    ///
    /// <div class="note">
    /// **Note**: the returned builder still needs to be built and validated
    /// after conversion from a document.
    /// </div>
    ///
    /// # Errors
    /// Returns `DocumentError` if the document cannot be deserialized into the
    /// shape builder typically due to schema mismatches or failures
    /// such as invalid int -> float conversions.
    #[inline]
    pub(crate) fn into_builder<'de, B: ShapeBuilder<'de, S>, S: StaticSchemaShape>(
        self: Box<Self>,
    ) -> Result<B, DocumentError> {
        B::deserialize_with_schema(S::schema(), &mut DocumentDeserializer::new(self))
    }
}

impl Error for DocumentError {
    fn custom<T: Display>(msg: T) -> Self {
        DocumentError::CustomError(msg.to_string())
    }
}

struct DocumentParser;
// TODO(document validation): Should this have schema type validation?
impl Serializer for DocumentParser {
    type Error = DocumentError;
    type Ok = Box<dyn Document>;
    type SerializeList = DocumentListAccumulator;
    type SerializeMap = DocumentMapAccumulator;
    type SerializeStruct = DocumentMapAccumulator;

    fn write_struct(
        self,
        schema: &Schema,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(DocumentMapAccumulator {
            schema: schema.clone(),
            values: IndexMap::with_capacity(len),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_map(self, schema: &Schema, len: usize) -> Result<Self::SerializeMap, Self::Error> {
        Ok(DocumentMapAccumulator {
            schema: schema.clone(),
            values: IndexMap::with_capacity(len),
            discriminator: Some(schema.id().clone()),
        })
    }

    fn write_list(
        self,
        schema: &Schema,
        len: usize,
    ) -> Result<Self::SerializeList, Self::Error> {
        Ok(DocumentListAccumulator {
            schema: schema.clone(),
            values: Vec::with_capacity(len),
            discriminator: Some(schema.id().clone()),
        })
    }

    #[inline]
    fn write_boolean(self, _schema: &Schema, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(value.into())
    }

    #[inline]
    fn write_byte(self, _schema: &Schema, value: i8) -> Result<Self::Ok, Self::Error> {
        Ok(value.into())
    }

    #[inline]
    fn write_short(self, _schema: &Schema, value: i16) -> Result<Self::Ok, Self::Error> {
        Ok(value.into())
    }

    #[inline]
    fn write_integer(self, _schema: &Schema, value: i32) -> Result<Self::Ok, Self::Error> {
        Ok(value.into())
    }

    #[inline]
    fn write_long(self, _schema: &Schema, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(value.into())
    }

    #[inline]
    fn write_float(self, _schema: &Schema, value: f32) -> Result<Self::Ok, Self::Error> {
        Ok(value.into())
    }

    #[inline]
    fn write_double(self, _schema: &Schema, value: f64) -> Result<Self::Ok, Self::Error> {
        Ok(value.into())
    }

    #[inline]
    fn write_big_integer(
        self,
        _schema: &Schema,
        value: &BigInt,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(value.clone().into())
    }

    #[inline]
    fn write_big_decimal(
        self,
        _schema: &Schema,
        value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(value.clone().into())
    }

    #[inline]
    fn write_string(self, _schema: &Schema, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(value.into())
    }

    #[inline]
    fn write_blob(self, _schema: &Schema, value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        Ok(value.clone().into())
    }

    #[inline]
    fn write_timestamp(
        self,
        _schema: &Schema,
        value: &Instant,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(value.into())
    }

    #[inline]
    fn write_document(
        self,
        _schema: &Schema,
        value: &Box<dyn Document>,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(value.clone())
    }

    #[inline]
    fn write_null(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(NULL.clone())
    }

    #[inline]
    fn skip(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        // When skipping (e.g., for None values), return a null document
        Ok(NULL.clone())
    }
}

#[doc(hidden)]
pub struct DocumentListAccumulator {
    schema: Schema,
    values: Vec<Box<dyn Document>>,
    discriminator: Option<ShapeId>,
}
impl ListSerializer for DocumentListAccumulator {
    type Error = DocumentError;
    type Ok = Box<dyn Document>;

    #[inline]
    fn serialize_element<T>(
        &mut self,
        element_schema: &Schema,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        let el = value.serialize_with_schema(element_schema, DocumentParser)?;
        self.values.push(el);
        Ok(())
    }

    #[inline]
    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(crate::schema::default::Document {
            schema: self.schema,
            value: Value::List(self.values),
            discriminator: self.discriminator,
        }
        .into())
    }
}

#[doc(hidden)]
pub struct DocumentMapAccumulator {
    schema: Schema,
    values: IndexMap<String, Box<dyn Document>>,
    discriminator: Option<ShapeId>,
}
impl MapSerializer for DocumentMapAccumulator {
    type Error = DocumentError;
    type Ok = Box<dyn Document>;

    #[inline]
    fn serialize_entry<K, V>(
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
        let key_str =
            key.serialize_with_schema(key_schema, &mut KeySerializer::<DocumentError>::new())?;
        let val = value.serialize_with_schema(value_schema, DocumentParser)?;
        self.values.insert(key_str, val);
        Ok(())
    }

    #[inline]
    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(crate::schema::default::Document {
            schema: self.schema,
            value: Value::Map(self.values),
            discriminator: self.discriminator,
        }
        .into())
    }
}

impl StructSerializer for DocumentMapAccumulator {
    type Error = DocumentError;
    type Ok = Box<dyn Document>;

    #[inline]
    fn serialize_member<T>(
        &mut self,
        member_schema: &Schema,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        let Some(me) = member_schema.as_member() else {
            return Err(DocumentError::DocumentConversion(
                "Expected member schema!".to_string(),
            ));
        };
        let val = value.serialize_with_schema(member_schema, DocumentParser)?;
        self.values.insert(me.name().to_string(), val);
        Ok(())
    }

    #[inline]
    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(crate::schema::default::Document {
            schema: self.schema,
            value: Value::Map(self.values),
            discriminator: self.discriminator,
        }
        .into())
    }
}

// ============================================================================
// Deserialization
// ============================================================================

/// A deserializer that reads from a `Document`.
struct DocumentDeserializer {
    document: Option<Box<dyn Document>>,
}

impl DocumentDeserializer {
    pub fn new(document: Box<dyn Document>) -> Self {
        Self {
            document: Some(document),
        }
    }

    #[inline]
    fn get_inner<T: TryFrom<Box<dyn Document>, Error = DocumentError>>(
        &mut self,
    ) -> Result<T, DocumentError> {
        self.document
            .take()
            .ok_or_else(|| {
                DocumentError::DocumentConversion(
                    "Encountered empty document deserializer".to_string(),
                )
            })?
            .try_into()
    }
}

impl<'de> Deserializer<'de> for DocumentDeserializer {
    type Error = DocumentError;

    #[inline]
    fn read_bool(&mut self, _schema: &Schema) -> Result<bool, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_byte(&mut self, _schema: &Schema) -> Result<i8, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_short(&mut self, _schema: &Schema) -> Result<i16, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_integer(&mut self, _schema: &Schema) -> Result<i32, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_long(&mut self, _schema: &Schema) -> Result<i64, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_float(&mut self, _schema: &Schema) -> Result<f32, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_double(&mut self, _schema: &Schema) -> Result<f64, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_big_integer(&mut self, _schema: &Schema) -> Result<BigInt, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_big_decimal(&mut self, _schema: &Schema) -> Result<BigDecimal, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_string(&mut self, _schema: &Schema) -> Result<String, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_blob(&mut self, _schema: &Schema) -> Result<ByteBuffer, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_timestamp(&mut self, _schema: &Schema) -> Result<Instant, Self::Error> {
        self.get_inner()
    }

    #[inline]
    fn read_document(&mut self, _schema: &Schema) -> Result<Box<dyn Document>, Self::Error> {
        self.document.take().ok_or_else(|| {
            DocumentError::DocumentConversion("Encountered empty document deserializer".to_string())
        })
    }

    fn read_struct<B, F>(
        &mut self,
        schema: &Schema,
        mut builder: B,
        consumer: F,
    ) -> Result<B, Self::Error>
    where
        B: DeserializeWithSchema<'de>,
        F: Fn(B, &Schema, &mut Self) -> Result<B, Self::Error>,
    {
        let map: IndexMap<String, Box<dyn Document>> = self.get_inner()?;

        // Iterate through members in the document map so we have owned values.
        // Add only values that match the provided schema.
        for (key, value) in map {
            if let Some(member_schema) = schema.members().get(&key) {
                builder = consumer(
                    builder,
                    member_schema,
                    &mut DocumentDeserializer::new(value),
                )?;
            }
        }

        Ok(builder)
    }

    fn read_list<T, F>(
        &mut self,
        schema: &Schema,
        state: &mut T,
        consumer: F,
    ) -> Result<(), Self::Error>
    where
        T: DeserializeWithSchema<'de>,
        F: Fn(&mut T, &Schema, &mut Self) -> Result<(), Self::Error>,
    {
        let list: Vec<Box<dyn Document>> = self.get_inner()?;

        let member_schema = schema.get_member("member").ok_or_else(|| {
            DocumentError::DocumentConversion("List missing member schema".to_string())
        })?;

        for element_doc in list {
            consumer(
                state,
                member_schema,
                &mut DocumentDeserializer::new(element_doc),
            )?;
        }

        Ok(())
    }

    fn read_map<T, F>(
        &mut self,
        _schema: &Schema,
        state: &mut T,
        consumer: F,
    ) -> Result<(), Self::Error>
    where
        T: DeserializeWithSchema<'de>,
        F: Fn(&mut T, String, &mut Self) -> Result<(), Self::Error>,
    {
        let map: IndexMap<String, Box<dyn Document>> = self.get_inner()?;
        for (key_str, value_doc) in map {
            // Key is already a String, create deserializer for the value
            consumer(
                state,
                key_str.clone(),
                &mut DocumentDeserializer::new(value_doc),
            )?;
        }
        Ok(())
    }

    fn is_null(&mut self) -> bool {
        self.document
            .as_ref()
            .expect("Empty document deserializer")
            .is_null()
    }

    fn read_null(&mut self) -> Result<(), Self::Error> {
        if self.is_null() {
            Ok(())
        } else {
            Err(DocumentError::DocumentConversion(
                "Expected null document".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::{derive::SmithyShape, schema::prelude::*, smithy};

    smithy!("com.example#Map": {
        map MAP_SCHEMA {
            key: STRING
            value: STRING
        }
    });
    smithy!("com.example#List": {
        list LIST_SCHEMA {
            member: STRING
        }
    });
    smithy!("com.example#Shape": {
        structure SCHEMA {
            A: STRING = "a"
            B: STRING = "b"
            C: STRING = "c"
            LIST: LIST_SCHEMA = "list"
            MAP: MAP_SCHEMA = "map"
        }
    });

    #[derive(SmithyShape, Clone, PartialEq)]
    #[smithy_schema(SCHEMA)]
    pub struct SerializeMe {
        #[smithy_schema(A)]
        pub member_a: String,
        #[smithy_schema(B)]
        pub member_b: String,
        #[smithy_schema(C)]
        pub member_optional: Option<String>,
        #[smithy_schema(LIST)]
        pub member_list: Vec<String>,
        #[smithy_schema(MAP)]
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
        assert_eq!(document.discriminator().unwrap(), SCHEMA.id());
        if let Some(members) = document.as_map() {
            let doc_a = &members.get("a").unwrap();
            assert_eq!(doc_a.as_string().unwrap(), "a");
            let doc_b = &members.get("b").unwrap();
            assert_eq!(doc_b.as_string().unwrap(), "b");
            let doc_c = &members.get("c").unwrap();
            assert_eq!(doc_c.as_string().unwrap(), "c");
            assert!(members.contains_key("map"));
            assert!(members.contains_key("list"));
        } else {
            panic!("Expected document");
        }
    }

    #[test]
    fn string_document_value() {
        let document_str: Box<dyn Document> = "MyStr".into();
        let output_str = document_str.as_string().expect("string");
        assert_eq!(output_str, &"MyStr".to_string());
        let val: &Schema = &STRING;
        assert_eq!(document_str.schema(), val);
    }

    #[test]
    #[ignore]
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
        let builder: SerializeMeBuilder = document.into_builder().unwrap();
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
        // Byte
        let original_byte = 127i8;
        let doc: Box<dyn Document> = original_byte.into();
        let result: i8 = doc.try_into().unwrap();
        assert_eq!(127i8, result);

        // Short
        let original_short = 32000i16;
        let doc: Box<dyn Document> = original_short.into();
        let result: i16 = doc.try_into().unwrap();
        assert_eq!(original_short, result);

        // Integer
        let original_int = 123_456_i32;
        let doc: Box<dyn Document> = original_int.into();
        let result: i32 = doc.try_into().unwrap();
        assert_eq!(original_int, result);

        // Long
        let original_long = 9_876_543_210_i64;
        let doc: Box<dyn Document> = original_long.into();
        let result: i64 = doc.try_into().unwrap();
        assert_eq!(original_long, result);
    }

    #[test]
    fn roundtrip_floats() {
        // Float
        let original_float = 1.2345f32;
        let doc: Box<dyn Document> = original_float.into();
        let result: f32 = doc.try_into().unwrap();
        assert_eq!(original_float, result);

        // Double
        let original_double = 1.23456789f64;
        let doc: Box<dyn Document> = original_double.into();
        let result: f64 = doc.try_into().unwrap();
        assert_eq!(original_double, result);
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
        let doc_map = doc.as_map().expect("Should be a map");

        // Check member_a
        let member_a_doc = doc_map.get("a").expect("Should have member a").clone();
        let a_value: String = member_a_doc.try_into().unwrap();
        assert_eq!(a_value, "value_a".to_string());

        // Check member_b
        let member_b_doc = doc_map.get("b").expect("Should have member b").clone();
        let b_value: String = member_b_doc.try_into().unwrap();
        assert_eq!(b_value, "value_b");

        // Check member_optional
        let member_c_doc = doc_map.get("c").expect("Should have member c").clone();
        let c_value: String = member_c_doc.try_into().unwrap();
        assert_eq!(c_value, "value_c");

        // Check list
        let list_doc = doc_map.get("list").expect("Should have list").clone();
        let list_value: Vec<String> = list_doc.try_into().unwrap();
        assert_eq!(list_value, original_list);

        // Check map
        let map_doc = doc_map.get("map").expect("Should have map").clone();
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
}
