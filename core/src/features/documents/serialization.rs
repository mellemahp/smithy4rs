//! Document Serialization support
//!
//! Documents will always try to serialize using the provided schema
//! as the template. However, if a `document` schema is passed in then
//! serialization will follow the document's own `document_type()` as
//! the guide for which serialization methods to call.

use core::fmt;

use crate::{
    features::documents::{
        Document, DocumentError, DocumentType, ListAccessor, MapAccessor, Number,
    },
    prelude::RequiredTrait,
    schema::{Schema, ShapeType},
    serde::se::{Error, ListWriter, MapWriter, SerializeWithSchema, Serializer, StructWriter},
};
// ============================================================================
// Document Errors
// ============================================================================

impl crate::serde::de::Error for DocumentError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DocumentError::CustomError(msg.to_string())
    }
}

// ============================================================================
// Serialization of a document
// ============================================================================

impl SerializeWithSchema for Box<dyn Document> {
    #[inline]
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let doc = self.as_ref();
        doc.serialize_with_schema(schema, serializer)
    }
}

macro_rules! try_as {
    ($self:ident, $method:ident, $ty:literal) => {
        $self.$method().ok_or_else(|| {
            S::Error::custom(concat!("could not be represented as ", $ty).to_string())
        })?
    };
}

impl SerializeWithSchema for dyn Document + '_ {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        if self.null() {
            return serializer.write_null(schema);
        }
        // Try to coerce into schema requested type if possible
        match schema.shape_type() {
            ShapeType::Blob => serializer.write_blob(schema, try_as!(self, as_blob, "blob")),
            ShapeType::Boolean => {
                serializer.write_boolean(schema, try_as!(self, as_boolean, "boolean"))
            }
            ShapeType::String | ShapeType::Enum => {
                serializer.write_string(schema, try_as!(self, as_string, "string"))
            }
            ShapeType::Timestamp => {
                serializer.write_timestamp(schema, try_as!(self, as_timestamp, "timestamp"))
            }
            ShapeType::Byte => serializer.write_byte(schema, try_as!(self, as_byte, "byte")),
            ShapeType::Short => serializer.write_short(schema, try_as!(self, as_short, "short")),
            ShapeType::Integer | ShapeType::IntEnum => {
                serializer.write_integer(schema, try_as!(self, as_integer, "integer"))
            }
            ShapeType::Long => serializer.write_long(schema, try_as!(self, as_long, "long")),
            ShapeType::Float => serializer.write_float(schema, try_as!(self, as_float, "float")),
            ShapeType::Double => {
                serializer.write_double(schema, try_as!(self, as_double, "double"))
            }
            ShapeType::BigInteger => {
                let b = try_as!(self, as_big_integer, "big_integer");
                serializer.write_big_integer(schema, &b)
            }
            ShapeType::BigDecimal => {
                let d = try_as!(self, as_big_decimal, "big_decimal");
                serializer.write_big_decimal(schema, &d)
            }
            ShapeType::Document => serialize_document(schema, self, serializer),
            ShapeType::List => {
                serialize_list_document(schema, try_as!(self, as_list, "list"), serializer)
            }
            ShapeType::Map => {
                serialize_map_document(schema, try_as!(self, as_map, "map"), serializer)
            }
            ShapeType::Structure | ShapeType::Union => {
                let document_map = try_as!(self, as_map, "map");
                let mut struct_serializer = serializer.write_struct(schema, document_map.len())?;
                if let Some(discriminator) = &self.discriminant() {
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

/// Infer serialization method by document value type rather than schema
///
/// This
fn serialize_document<S: Serializer>(
    schema: &Schema,
    value: &dyn Document,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match value.document_type() {
        DocumentType::Null => serializer.write_null(schema),
        DocumentType::Boolean => {
            serializer.write_boolean(schema, try_as!(value, as_boolean, "boolean"))
        }
        DocumentType::Timestamp => {
            serializer.write_timestamp(schema, try_as!(value, as_timestamp, "timestamp"))
        }
        DocumentType::Number(number) => match number {
            Number::Byte => serializer.write_byte(schema, try_as!(value, as_byte, "byte")),
            Number::Short => serializer.write_short(schema, try_as!(value, as_short, "short")),
            Number::Integer => {
                serializer.write_integer(schema, try_as!(value, as_integer, "integer"))
            }
            Number::Long => serializer.write_long(schema, try_as!(value, as_long, "long")),
            Number::BigInt => {
                let i = try_as!(value, as_big_decimal, "big_decimal");
                serializer.write_big_decimal(schema, &i)
            }
            Number::Float => serializer.write_float(schema, try_as!(value, as_float, "float")),
            Number::Double => serializer.write_double(schema, try_as!(value, as_double, "double")),
            Number::BigDecimal => {
                let d = try_as!(value, as_big_decimal, "big_decimal");
                serializer.write_big_decimal(schema, &d)
            }
        },
        DocumentType::String => {
            serializer.write_string(schema, try_as!(value, as_string, "string"))
        }
        DocumentType::Blob => serializer.write_blob(schema, try_as!(value, as_blob, "string")),
        DocumentType::List => {
            serialize_list_document(schema, try_as!(value, as_list, "list"), serializer)
        }
        DocumentType::Map => {
            let map = try_as!(value, as_map, "map");
            serialize_map_document(schema, map, serializer)
        }
    }
}

fn serialize_list_document<S: Serializer>(
    schema: &Schema,
    accessor: &dyn ListAccessor,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut list = serializer.write_list(schema, accessor.len())?;
    let value_schema = schema
        .get_list_member()
        .ok_or_else(|| S::Error::custom("Expected a list schema"))?;
    let mut i = 0;
    while i < accessor.len() {
        let item = accessor
            .get_index(i)
            .ok_or_else(|| S::Error::custom("Expected document"))?;
        list.write_element(value_schema, item)?;
        i += 1;
    }
    list.end(schema)
}

fn serialize_map_document<S: Serializer>(
    schema: &Schema,
    accessor: &dyn MapAccessor,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    // Documents just use themselves as the schema for keys and values
    let (key_schema, value_schema) = if matches!(schema.shape_type(), ShapeType::Document) {
        (schema, schema)
    } else {
        schema
            .get_key_value()
            .ok_or_else(|| S::Error::custom("Expected a map schema"))?
    };
    let mut map = serializer.write_map(schema, accessor.len())?;
    let mut i = 0;
    while i < accessor.len() {
        if let Some((key, child_doc)) = accessor.get_entry(i) {
            map.write_entry(key_schema, value_schema, key, child_doc)?;
        }
        i += 1;
    }
    map.end(schema)
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use crate::{
        features::documents::Document,
        prelude::{DOCUMENT, INTEGER, PatternTrait, RequiredTrait, STRING, SensitiveTrait},
        serde::debug::DebugWrapper,
        smithy,
    };

    smithy!(
        @PatternTrait::new("^[a-z]*$");
        string com::example::Lowercase
    );

    #[test]
    fn simple_document_serialization() {
        let x: Box<dyn Document> = "document".into();
        let debug = DebugWrapper::new(&LOWERCASE, &x);
        assert_eq!(format!("{debug:?}"), "\"document\"");
    }

    smithy!(
        @SensitiveTrait::builder().build();
        string com::example::Password
    );

    #[test]
    fn simple_document_respects_serialization_traits() {
        let x: Box<dyn Document> = "document".into();
        let debug = DebugWrapper::new(&PASSWORD, &x);
        assert_eq!(format!("{debug:?}"), "**REDACTED**");
    }

    smithy!(
        integer com::example::NumberValue
    );

    #[test]
    fn serializes_number() {
        let x: Box<dyn Document> = 5i32.into();
        let debug = DebugWrapper::new(&NUMBER_VALUE, &x);
        assert_eq!(format!("{debug:?}"), "5");
    }

    smithy!(
        list com::example::ListValue {
            member: STRING
        }
    );

    #[test]
    fn serializes_list() {
        let list = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let x: Box<dyn Document> = list.into();
        let debug = DebugWrapper::new(&LIST_VALUE, &x);
        assert_eq!(format!("{debug:?}"), "[\"a\", \"b\", \"c\", \"d\"]");
    }

    smithy!(
        map com::example::MapOfNumber {
            key: STRING
            value: INTEGER
        }
    );

    #[test]
    fn serializes_map() {
        let mut map = IndexMap::new();
        map.insert("Stuff".to_string(), 1);
        map.insert("Things".to_string(), 2);
        map.insert("Quux".to_string(), 3);
        map.insert("Baz".to_string(), 4);
        let x: Box<dyn Document> = map.into();
        let debug = DebugWrapper::new(&MAP_OF_NUMBER, &x);
        assert_eq!(
            format!("{debug:?}"),
            "{\"Stuff\": 1, \"Things\": 2, \"Quux\": 3, \"Baz\": 4}"
        );
    }

    smithy!(
        structure test::Nested {
            @RequiredTrait::builder().build();
            @PatternTrait::new("^[a-z]*$");
            c: STRING
        }
    );

    smithy!(
        structure test::StructWithNested {
            nested: NESTED
            required: NESTED
        }
    );

    #[test]
    fn serializes_struct() {
        let mut nested: IndexMap<String, String> = IndexMap::new();
        nested.insert("c".to_string(), "data".to_string());

        let mut required: IndexMap<String, String> = IndexMap::new();
        required.insert("c".to_string(), "more".to_string());

        let mut root = IndexMap::new();
        root.insert("nested".to_string(), nested);
        root.insert("required".to_string(), required);

        let document: Box<dyn Document> = root.into();
        let debug = DebugWrapper::new(&STRUCT_WITH_NESTED, &document);
        assert_eq!(
            format!("{debug:?}"),
            "StructWithNested { nested: StructWithNested { c: \"data\" }, required: StructWithNested { c: \"more\" } }"
        );
        let debug_doc = DebugWrapper::new(&DOCUMENT, &document);
        assert_eq!(
            format!("{debug_doc:?}"),
            "{\"nested\": {\"c\": \"data\"}, \"required\": {\"c\": \"more\"}}"
        );
    }
}
