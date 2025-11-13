use smithy4rs_core::{
    lazy_schema, prelude::{INTEGER, STRING},
    schema::{Schema, ShapeId},
    traits,
};
use smithy4rs_core_derive::{DeserializableStruct, SerializableStruct};
pub static SIMPLE_SCHEMA_BUILDER: ::smithy4rs_core::LazyLock<
    std::sync::Arc<::smithy4rs_core::schema::SchemaBuilder>,
> = ::smithy4rs_core::LazyLock::new(|| std::sync::Arc::new(
    Schema::structure_builder(ShapeId::from("test#SimpleStruct"), Vec::new()),
));
pub static SIMPLE_SCHEMA: ::smithy4rs_core::LazyLock<
    ::smithy4rs_core::schema::SchemaRef,
> = ::smithy4rs_core::LazyLock::new(|| {
    (&*SIMPLE_SCHEMA_BUILDER)
        .put_member("field_a", &STRING, Vec::new())
        .put_member("field_b", &INTEGER, Vec::new())
        .build()
});
pub static FIELD_A: ::smithy4rs_core::LazyLock<&::smithy4rs_core::schema::SchemaRef> = ::smithy4rs_core::LazyLock::new(||
SIMPLE_SCHEMA.expect_member("field_a"));
pub static FIELD_B: ::smithy4rs_core::LazyLock<&::smithy4rs_core::schema::SchemaRef> = ::smithy4rs_core::LazyLock::new(||
SIMPLE_SCHEMA.expect_member("field_b"));
#[smithy_schema(SIMPLE_SCHEMA)]
pub struct SimpleStruct {
    #[smithy_schema(FIELD_A)]
    pub field_a: String,
    #[smithy_schema(FIELD_B)]
    pub field_b: i32,
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::serde::documents::SerializableShape as _SerializableShape;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::StructSerializer as _StructSerializer;
    #[automatically_derived]
    impl _SerializableShape for SimpleStruct {}
    #[automatically_derived]
    impl _SerializeWithSchema for SimpleStruct {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 2usize)?;
            ser.serialize_member_named("field_a", &FIELD_A, &self.field_a)?;
            ser.serialize_member_named("field_b", &FIELD_B, &self.field_b)?;
            ser.end(schema)
        }
    }
};
#[automatically_derived]
pub struct SimpleStructBuilder {
    field_a: Option<String>,
    field_b: Option<i32>,
}
#[automatically_derived]
impl SimpleStructBuilder {
    pub fn new() -> Self {
        Self {
            field_a: None,
            field_b: None,
        }
    }
    pub fn field_a(mut self, value: String) -> Self {
        self.field_a = Some(value);
        self
    }
    pub fn field_b(mut self, value: i32) -> Self {
        self.field_b = Some(value);
        self
    }
    pub fn build(self) -> Result<SimpleStruct, String> {
        Ok(SimpleStruct {
            field_a: self.field_a.ok_or_else(|| "field_a is required".to_string())?,
            field_b: self.field_b.ok_or_else(|| "field_b is required".to_string())?,
        })
    }
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
    use _smithy4rs::serde::deserializers::Error as _Error;
    #[automatically_derived]
    impl<'de> _DeserializeWithSchema<'de> for SimpleStruct {
        fn deserialize_with_schema<D>(
            schema: &_SchemaRef,
            deserializer: &mut D,
        ) -> Result<Self, D::Error>
        where
            D: _Deserializer<'de>,
        {
            let builder = SimpleStructBuilder::new();
            let builder = deserializer
                .read_struct(
                    schema,
                    builder,
                    |builder, member_schema, de| {
                        if std::sync::Arc::ptr_eq(member_schema, &FIELD_A) {
                            let value = <String as ::smithy4rs_core::serde::deserializers::DeserializeWithSchema>::deserialize_with_schema(
                                member_schema,
                                de,
                            )?;
                            return Ok(builder.field_a(value));
                        }
                        if std::sync::Arc::ptr_eq(member_schema, &FIELD_B) {
                            let value = <i32 as ::smithy4rs_core::serde::deserializers::DeserializeWithSchema>::deserialize_with_schema(
                                member_schema,
                                de,
                            )?;
                            return Ok(builder.field_b(value));
                        }
                        Ok(builder)
                    },
                )?;
            builder.build().map_err(_Error::custom)
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for SimpleStruct {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "SimpleStruct",
            "field_a",
            &self.field_a,
            "field_b",
            &&self.field_b,
        )
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for SimpleStruct {}
#[automatically_derived]
impl ::core::cmp::PartialEq for SimpleStruct {
    #[inline]
    fn eq(&self, other: &SimpleStruct) -> bool {
        self.field_b == other.field_b && self.field_a == other.field_a
    }
}
