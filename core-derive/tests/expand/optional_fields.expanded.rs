use smithy4rs_core::{
    lazy_schema, prelude::{INTEGER, STRING},
    schema::{Schema, ShapeId},
    traits,
};
use smithy4rs_core_derive::{DeserializableStruct, SerializableStruct};
pub static SCHEMA_WITH_OPTIONAL: ::smithy4rs_core::LazyLock<
    ::smithy4rs_core::schema::SchemaRef,
> = ::smithy4rs_core::LazyLock::new(|| {
    Schema::structure_builder(ShapeId::from("test#StructWithOptional"), Vec::new())
        .put_member("required", &STRING, Vec::new())
        .put_member("optional", &INTEGER, Vec::new())
        .build()
});
static REQUIRED_FIELD: ::smithy4rs_core::LazyLock<
    &::smithy4rs_core::schema::SchemaRef,
> = ::smithy4rs_core::LazyLock::new(|| SCHEMA_WITH_OPTIONAL.expect_member("required"));
static OPTIONAL_FIELD: ::smithy4rs_core::LazyLock<
    &::smithy4rs_core::schema::SchemaRef,
> = ::smithy4rs_core::LazyLock::new(|| SCHEMA_WITH_OPTIONAL.expect_member("optional"));
#[smithy_schema(SCHEMA_WITH_OPTIONAL)]
pub struct StructWithOptional {
    #[smithy_schema(REQUIRED_FIELD)]
    pub required: String,
    #[smithy_schema(OPTIONAL_FIELD)]
    pub optional: Option<i32>,
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
    impl _SerializableShape for StructWithOptional {}
    #[automatically_derived]
    impl _SchemaShape for StructWithOptional {
        fn schema(&self) -> &_SchemaRef {
            &SCHEMA_WITH_OPTIONAL
        }
    }
    #[automatically_derived]
    impl _SerializeWithSchema for StructWithOptional {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 2usize)?;
            ser.serialize_member(&REQUIRED_FIELD, &self.required)?;
            ser.serialize_optional_member(&OPTIONAL_FIELD, &self.optional)?;
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
    pub struct StructWithOptionalBuilder {
        required: Option<String>,
        optional: Option<i32>,
    }
    #[automatically_derived]
    impl StructWithOptionalBuilder {
        pub fn new() -> Self {
            Self {
                required: None,
                optional: None,
            }
        }
        pub fn required(&mut self, value: String) -> &mut Self {
            self.required = Some(value);
            self
        }
        pub fn optional(&mut self, value: i32) -> &mut Self {
            self.optional = Some(value);
            self
        }
        pub fn build(self) -> Result<StructWithOptional, String> {
            Ok(StructWithOptional {
                required: self
                    .required
                    .ok_or_else(|| "required is required".to_string())?,
                optional: self.optional,
            })
        }
    }
    #[automatically_derived]
    impl<'de> _Deserialize<'de> for StructWithOptional {
        fn deserialize<D>(
            schema: &_SchemaRef,
            deserializer: &mut D,
        ) -> Result<Self, D::Error>
        where
            D: _Deserializer<'de>,
        {
            let mut builder = StructWithOptionalBuilder::new();
            deserializer
                .read_struct(
                    schema,
                    &mut builder,
                    |builder, member_schema, de| {
                        if std::sync::Arc::ptr_eq(member_schema, &REQUIRED_FIELD) {
                            let value = <String as _Deserialize>::deserialize(
                                member_schema,
                                de,
                            )?;
                            builder.required(value);
                        } else if std::sync::Arc::ptr_eq(
                            member_schema,
                            &OPTIONAL_FIELD,
                        ) {
                            let value = Option::<i32>::deserialize(member_schema, de)?;
                            if let Some(v) = value {
                                builder.optional(v);
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
impl ::core::fmt::Debug for StructWithOptional {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "StructWithOptional",
            "required",
            &self.required,
            "optional",
            &&self.optional,
        )
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for StructWithOptional {}
#[automatically_derived]
impl ::core::cmp::PartialEq for StructWithOptional {
    #[inline]
    fn eq(&self, other: &StructWithOptional) -> bool {
        self.required == other.required && self.optional == other.optional
    }
}
