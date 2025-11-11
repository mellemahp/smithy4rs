use std::sync::{Arc, LazyLock};
use indexmap::IndexMap;
use smithy4rs_core::{
    ByteBuffer, Instant, prelude::*, schema::{Schema, SchemaBuilder, SchemaRef, ShapeId},
    traits,
};
use smithy4rs_core_derive::{DeserializableStruct, SerializableStruct};
pub static STRING_LIST_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::list_builder("test#StringList", Vec::new())
        .put_member("member", &STRING, Vec::new())
        .build()
});
pub static STRING_MAP_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Schema::map_builder(ShapeId::from("test#StringMap"), Vec::new())
        .put_member("key", &STRING, Vec::new())
        .put_member("value", &STRING, Vec::new())
        .build()
});
pub static ALL_SHAPES_BUILDER: LazyLock<Arc<SchemaBuilder>> = LazyLock::new(|| {
    Arc::new(Schema::structure_builder(ShapeId::from("test#AllShapes"), Vec::new()))
});
pub static ALL_SHAPES_SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_BUILDER
        .put_member("string_field", &STRING, Vec::new())
        .put_member("byte_field", &BYTE, Vec::new())
        .put_member("short_field", &SHORT, Vec::new())
        .put_member("integer_field", &INTEGER, Vec::new())
        .put_member("long_field", &LONG, Vec::new())
        .put_member("float_field", &FLOAT, Vec::new())
        .put_member("double_field", &DOUBLE, Vec::new())
        .put_member("boolean_field", &BOOLEAN, Vec::new())
        .put_member("blob_field", &BLOB, Vec::new())
        .put_member("timestamp_field", &TIMESTAMP, Vec::new())
        .put_member("list_field", &STRING_LIST_SCHEMA, Vec::new())
        .put_member("map_field", &STRING_MAP_SCHEMA, Vec::new())
        .put_member("optional_field", &STRING, Vec::new())
        .put_member("recursive_field", &*ALL_SHAPES_BUILDER, Vec::new())
        .build()
});
static STRING_FIELD: LazyLock<&SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_SCHEMA.expect_member("string_field")
});
static BYTE_FIELD: LazyLock<&SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_SCHEMA.expect_member("byte_field")
});
static SHORT_FIELD: LazyLock<&SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_SCHEMA.expect_member("short_field")
});
static INTEGER_FIELD: LazyLock<&SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_SCHEMA.expect_member("integer_field")
});
static LONG_FIELD: LazyLock<&SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_SCHEMA.expect_member("long_field")
});
static FLOAT_FIELD: LazyLock<&SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_SCHEMA.expect_member("float_field")
});
static DOUBLE_FIELD: LazyLock<&SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_SCHEMA.expect_member("double_field")
});
static BOOLEAN_FIELD: LazyLock<&SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_SCHEMA.expect_member("boolean_field")
});
static BLOB_FIELD: LazyLock<&SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_SCHEMA.expect_member("blob_field")
});
static TIMESTAMP_FIELD: LazyLock<&SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_SCHEMA.expect_member("timestamp_field")
});
static LIST_FIELD: LazyLock<&SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_SCHEMA.expect_member("list_field")
});
static MAP_FIELD: LazyLock<&SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_SCHEMA.expect_member("map_field")
});
static OPTIONAL_FIELD: LazyLock<&SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_SCHEMA.expect_member("optional_field")
});
static RECURSIVE_FIELD: LazyLock<&SchemaRef> = LazyLock::new(|| {
    ALL_SHAPES_SCHEMA.expect_member("recursive_field")
});
#[smithy_schema(ALL_SHAPES_SCHEMA)]
pub struct AllShapes {
    #[smithy_schema(STRING_FIELD)]
    pub string_field: String,
    #[smithy_schema(BYTE_FIELD)]
    pub byte_field: i8,
    #[smithy_schema(SHORT_FIELD)]
    pub short_field: i16,
    #[smithy_schema(INTEGER_FIELD)]
    pub integer_field: i32,
    #[smithy_schema(LONG_FIELD)]
    pub long_field: i64,
    #[smithy_schema(FLOAT_FIELD)]
    pub float_field: f32,
    #[smithy_schema(DOUBLE_FIELD)]
    pub double_field: f64,
    #[smithy_schema(BOOLEAN_FIELD)]
    pub boolean_field: bool,
    #[smithy_schema(BLOB_FIELD)]
    pub blob_field: ByteBuffer,
    #[smithy_schema(TIMESTAMP_FIELD)]
    pub timestamp_field: Instant,
    #[smithy_schema(LIST_FIELD)]
    pub list_field: Vec<String>,
    #[smithy_schema(MAP_FIELD)]
    pub map_field: IndexMap<String, String>,
    #[smithy_schema(OPTIONAL_FIELD)]
    pub optional_field: Option<String>,
    #[smithy_schema(RECURSIVE_FIELD)]
    pub recursive_field: Option<Box<AllShapes>>,
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::serde::documents::SerializableShape as _SerializableShape;
    use _smithy4rs::schema::SchemaShape as _SchemaShape;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::StructSerializer as _StructSerializer;
    #[automatically_derived]
    impl _SerializableShape for AllShapes {}
    #[automatically_derived]
    impl _SchemaShape for AllShapes {
        fn schema(&self) -> &_SchemaRef {
            &ALL_SHAPES_SCHEMA
        }
    }
    #[automatically_derived]
    impl _SerializeWithSchema for AllShapes {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 14usize)?;
            ser.serialize_member(&STRING_FIELD, &self.string_field)?;
            ser.serialize_member(&BYTE_FIELD, &self.byte_field)?;
            ser.serialize_member(&SHORT_FIELD, &self.short_field)?;
            ser.serialize_member(&INTEGER_FIELD, &self.integer_field)?;
            ser.serialize_member(&LONG_FIELD, &self.long_field)?;
            ser.serialize_member(&FLOAT_FIELD, &self.float_field)?;
            ser.serialize_member(&DOUBLE_FIELD, &self.double_field)?;
            ser.serialize_member(&BOOLEAN_FIELD, &self.boolean_field)?;
            ser.serialize_member(&BLOB_FIELD, &self.blob_field)?;
            ser.serialize_member(&TIMESTAMP_FIELD, &self.timestamp_field)?;
            ser.serialize_member(&LIST_FIELD, &self.list_field)?;
            ser.serialize_member(&MAP_FIELD, &self.map_field)?;
            ser.serialize_optional_member(&OPTIONAL_FIELD, &self.optional_field)?;
            ser.serialize_optional_member(&RECURSIVE_FIELD, &self.recursive_field)?;
            ser.end(schema)
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::serde::deserializers::Deserialize as _Deserialize;
    use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
    use _smithy4rs::serde::deserializers::Error as _Error;
    #[automatically_derived]
    pub struct AllShapesBuilder {
        string_field: Option<String>,
        byte_field: Option<i8>,
        short_field: Option<i16>,
        integer_field: Option<i32>,
        long_field: Option<i64>,
        float_field: Option<f32>,
        double_field: Option<f64>,
        boolean_field: Option<bool>,
        blob_field: Option<ByteBuffer>,
        timestamp_field: Option<Instant>,
        list_field: Option<Vec<String>>,
        map_field: Option<IndexMap<String, String>>,
        optional_field: Option<String>,
        recursive_field: Option<Box<AllShapes>>,
    }
    #[automatically_derived]
    impl AllShapesBuilder {
        pub fn new() -> Self {
            Self {
                string_field: None,
                byte_field: None,
                short_field: None,
                integer_field: None,
                long_field: None,
                float_field: None,
                double_field: None,
                boolean_field: None,
                blob_field: None,
                timestamp_field: None,
                list_field: None,
                map_field: None,
                optional_field: None,
                recursive_field: None,
            }
        }
        pub fn string_field(&mut self, value: String) -> &mut Self {
            self.string_field = Some(value);
            self
        }
        pub fn byte_field(&mut self, value: i8) -> &mut Self {
            self.byte_field = Some(value);
            self
        }
        pub fn short_field(&mut self, value: i16) -> &mut Self {
            self.short_field = Some(value);
            self
        }
        pub fn integer_field(&mut self, value: i32) -> &mut Self {
            self.integer_field = Some(value);
            self
        }
        pub fn long_field(&mut self, value: i64) -> &mut Self {
            self.long_field = Some(value);
            self
        }
        pub fn float_field(&mut self, value: f32) -> &mut Self {
            self.float_field = Some(value);
            self
        }
        pub fn double_field(&mut self, value: f64) -> &mut Self {
            self.double_field = Some(value);
            self
        }
        pub fn boolean_field(&mut self, value: bool) -> &mut Self {
            self.boolean_field = Some(value);
            self
        }
        pub fn blob_field(&mut self, value: ByteBuffer) -> &mut Self {
            self.blob_field = Some(value);
            self
        }
        pub fn timestamp_field(&mut self, value: Instant) -> &mut Self {
            self.timestamp_field = Some(value);
            self
        }
        pub fn list_field(&mut self, value: Vec<String>) -> &mut Self {
            self.list_field = Some(value);
            self
        }
        pub fn map_field(&mut self, value: IndexMap<String, String>) -> &mut Self {
            self.map_field = Some(value);
            self
        }
        pub fn optional_field(&mut self, value: String) -> &mut Self {
            self.optional_field = Some(value);
            self
        }
        pub fn recursive_field(&mut self, value: Box<AllShapes>) -> &mut Self {
            self.recursive_field = Some(value);
            self
        }
        pub fn build(self) -> Result<AllShapes, String> {
            Ok(AllShapes {
                string_field: self
                    .string_field
                    .ok_or_else(|| "string_field is required".to_string())?,
                byte_field: self
                    .byte_field
                    .ok_or_else(|| "byte_field is required".to_string())?,
                short_field: self
                    .short_field
                    .ok_or_else(|| "short_field is required".to_string())?,
                integer_field: self
                    .integer_field
                    .ok_or_else(|| "integer_field is required".to_string())?,
                long_field: self
                    .long_field
                    .ok_or_else(|| "long_field is required".to_string())?,
                float_field: self
                    .float_field
                    .ok_or_else(|| "float_field is required".to_string())?,
                double_field: self
                    .double_field
                    .ok_or_else(|| "double_field is required".to_string())?,
                boolean_field: self
                    .boolean_field
                    .ok_or_else(|| "boolean_field is required".to_string())?,
                blob_field: self
                    .blob_field
                    .ok_or_else(|| "blob_field is required".to_string())?,
                timestamp_field: self
                    .timestamp_field
                    .ok_or_else(|| "timestamp_field is required".to_string())?,
                list_field: self
                    .list_field
                    .ok_or_else(|| "list_field is required".to_string())?,
                map_field: self
                    .map_field
                    .ok_or_else(|| "map_field is required".to_string())?,
                optional_field: self.optional_field,
                recursive_field: self.recursive_field,
            })
        }
    }
    #[automatically_derived]
    impl<'de> _Deserialize<'de> for AllShapes {
        fn deserialize<D>(
            schema: &_SchemaRef,
            deserializer: &mut D,
        ) -> Result<Self, D::Error>
        where
            D: _Deserializer<'de>,
        {
            let mut builder = AllShapesBuilder::new();
            deserializer
                .read_struct(
                    schema,
                    &mut builder,
                    |builder, member_schema, de| {
                        if std::sync::Arc::ptr_eq(member_schema, &STRING_FIELD) {
                            let value = <String as _Deserialize>::deserialize(
                                member_schema,
                                de,
                            )?;
                            builder.string_field(value);
                        } else if std::sync::Arc::ptr_eq(member_schema, &BYTE_FIELD) {
                            let value = <i8 as _Deserialize>::deserialize(
                                member_schema,
                                de,
                            )?;
                            builder.byte_field(value);
                        } else if std::sync::Arc::ptr_eq(member_schema, &SHORT_FIELD) {
                            let value = <i16 as _Deserialize>::deserialize(
                                member_schema,
                                de,
                            )?;
                            builder.short_field(value);
                        } else if std::sync::Arc::ptr_eq(member_schema, &INTEGER_FIELD) {
                            let value = <i32 as _Deserialize>::deserialize(
                                member_schema,
                                de,
                            )?;
                            builder.integer_field(value);
                        } else if std::sync::Arc::ptr_eq(member_schema, &LONG_FIELD) {
                            let value = <i64 as _Deserialize>::deserialize(
                                member_schema,
                                de,
                            )?;
                            builder.long_field(value);
                        } else if std::sync::Arc::ptr_eq(member_schema, &FLOAT_FIELD) {
                            let value = <f32 as _Deserialize>::deserialize(
                                member_schema,
                                de,
                            )?;
                            builder.float_field(value);
                        } else if std::sync::Arc::ptr_eq(member_schema, &DOUBLE_FIELD) {
                            let value = <f64 as _Deserialize>::deserialize(
                                member_schema,
                                de,
                            )?;
                            builder.double_field(value);
                        } else if std::sync::Arc::ptr_eq(member_schema, &BOOLEAN_FIELD) {
                            let value = <bool as _Deserialize>::deserialize(
                                member_schema,
                                de,
                            )?;
                            builder.boolean_field(value);
                        } else if std::sync::Arc::ptr_eq(member_schema, &BLOB_FIELD) {
                            let value = <ByteBuffer as _Deserialize>::deserialize(
                                member_schema,
                                de,
                            )?;
                            builder.blob_field(value);
                        } else if std::sync::Arc::ptr_eq(
                            member_schema,
                            &TIMESTAMP_FIELD,
                        ) {
                            let value = <Instant as _Deserialize>::deserialize(
                                member_schema,
                                de,
                            )?;
                            builder.timestamp_field(value);
                        } else if std::sync::Arc::ptr_eq(member_schema, &LIST_FIELD) {
                            let value = <Vec<
                                String,
                            > as _Deserialize>::deserialize(member_schema, de)?;
                            builder.list_field(value);
                        } else if std::sync::Arc::ptr_eq(member_schema, &MAP_FIELD) {
                            let value = <IndexMap<
                                String,
                                String,
                            > as _Deserialize>::deserialize(member_schema, de)?;
                            builder.map_field(value);
                        } else if std::sync::Arc::ptr_eq(
                            member_schema,
                            &OPTIONAL_FIELD,
                        ) {
                            let value = Option::<
                                String,
                            >::deserialize(member_schema, de)?;
                            if let Some(v) = value {
                                builder.optional_field(v);
                            }
                        } else if std::sync::Arc::ptr_eq(
                            member_schema,
                            &RECURSIVE_FIELD,
                        ) {
                            let value = Option::<
                                Box<AllShapes>,
                            >::deserialize(member_schema, de)?;
                            if let Some(v) = value {
                                builder.recursive_field(v);
                            }
                        } else {}
                        Ok(())
                    },
                )?;
            builder.build().map_err(_Error::custom)
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for AllShapes {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "string_field",
            "byte_field",
            "short_field",
            "integer_field",
            "long_field",
            "float_field",
            "double_field",
            "boolean_field",
            "blob_field",
            "timestamp_field",
            "list_field",
            "map_field",
            "optional_field",
            "recursive_field",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.string_field,
            &self.byte_field,
            &self.short_field,
            &self.integer_field,
            &self.long_field,
            &self.float_field,
            &self.double_field,
            &self.boolean_field,
            &self.blob_field,
            &self.timestamp_field,
            &self.list_field,
            &self.map_field,
            &self.optional_field,
            &&self.recursive_field,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(f, "AllShapes", names, values)
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for AllShapes {}
#[automatically_derived]
impl ::core::cmp::PartialEq for AllShapes {
    #[inline]
    fn eq(&self, other: &AllShapes) -> bool {
        self.byte_field == other.byte_field && self.short_field == other.short_field
            && self.integer_field == other.integer_field
            && self.long_field == other.long_field
            && self.float_field == other.float_field
            && self.double_field == other.double_field
            && self.boolean_field == other.boolean_field
            && self.string_field == other.string_field
            && self.blob_field == other.blob_field
            && self.timestamp_field == other.timestamp_field
            && self.list_field == other.list_field && self.map_field == other.map_field
            && self.optional_field == other.optional_field
            && self.recursive_field == other.recursive_field
    }
}
