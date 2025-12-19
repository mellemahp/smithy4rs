use smithy4rs_core::{
    prelude::{INTEGER, STRING},
    schema::ShapeId, traits,
};
use smithy4rs_core_derive::SmithyShape;
#[smithy_schema(SIMPLE_SCHEMA)]
pub struct SimpleStruct {
    #[smithy_schema(A)]
    pub field_a: String,
    #[smithy_schema(B)]
    pub field_b: i32,
    #[smithy_schema(C)]
    pub field_c: Option<Nested>,
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for SimpleStruct {
        fn schema() -> &'static _SchemaRef {
            &SIMPLE_SCHEMA
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::StructSerializer as _StructSerializer;
    #[automatically_derived]
    impl _SerializeWithSchema for SimpleStruct {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 3usize)?;
            ser.serialize_member_named(
                "field_a",
                &_SIMPLE_SCHEMA_MEMBER_A,
                &self.field_a,
            )?;
            ser.serialize_member_named(
                "field_b",
                &_SIMPLE_SCHEMA_MEMBER_B,
                &self.field_b,
            )?;
            ser.serialize_optional_member_named(
                "field_c",
                &_SIMPLE_SCHEMA_MEMBER_C,
                &self.field_c,
            )?;
            ser.end(schema)
        }
    }
};
#[automatically_derived]
pub struct SimpleStructBuilder {
    field_a: smithy4rs_core::serde::builders::Required<String>,
    field_b: smithy4rs_core::serde::builders::Required<i32>,
    field_c: Option<smithy4rs_core::serde::builders::MaybeBuilt<Nested, NestedBuilder>>,
}
#[automatically_derived]
impl ::core::clone::Clone for SimpleStructBuilder {
    #[inline]
    fn clone(&self) -> SimpleStructBuilder {
        SimpleStructBuilder {
            field_a: ::core::clone::Clone::clone(&self.field_a),
            field_b: ::core::clone::Clone::clone(&self.field_b),
            field_c: ::core::clone::Clone::clone(&self.field_c),
        }
    }
}
#[automatically_derived]
impl SimpleStructBuilder {
    pub fn new() -> Self {
        Self {
            field_a: smithy4rs_core::serde::builders::Required::Unset,
            field_b: smithy4rs_core::serde::builders::Required::Unset,
            field_c: None,
        }
    }
    pub fn field_a(mut self, value: String) -> Self {
        self.field_a = smithy4rs_core::serde::builders::Required::Set(value);
        self
    }
    pub fn field_b(mut self, value: i32) -> Self {
        self.field_b = smithy4rs_core::serde::builders::Required::Set(value);
        self
    }
    pub fn field_c(mut self, value: Nested) -> Self {
        self.field_c = Some(smithy4rs_core::serde::builders::MaybeBuilt::Struct(value));
        self
    }
    pub fn field_c_builder(mut self, value: NestedBuilder) -> Self {
        self.field_c = Some(smithy4rs_core::serde::builders::MaybeBuilt::Builder(value));
        self
    }
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
    use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    use _smithy4rs::serde::deserializers::Error as _Error;
    use _smithy4rs::serde::correction::ErrorCorrection as _ErrorCorrection;
    use _smithy4rs::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
    use _smithy4rs::serde::ShapeBuilder as _ShapeBuilder;
    use _smithy4rs::serde::Buildable as _Buildable;
    #[automatically_derived]
    impl _ErrorCorrection for SimpleStructBuilder {
        type Value = SimpleStruct;
        fn correct(self) -> Self::Value {
            SimpleStruct {
                field_a: self.field_a.get(),
                field_b: self.field_b.get(),
                field_c: self.field_c.correct(),
            }
        }
    }
    #[automatically_derived]
    impl<'de> _ShapeBuilder<'de, SimpleStruct> for SimpleStructBuilder {
        fn new() -> Self {
            Self::new()
        }
    }
    #[automatically_derived]
    impl _ErrorCorrectionDefault for SimpleStruct {
        fn default() -> Self {
            SimpleStructBuilder::new().correct()
        }
    }
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::StructSerializer as _StructSerializer;
    #[automatically_derived]
    impl _SerializeWithSchema for SimpleStructBuilder {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 3usize)?;
            ser.serialize_member_named(
                "field_a",
                &_SIMPLE_SCHEMA_MEMBER_A,
                &self.field_a,
            )?;
            ser.serialize_member_named(
                "field_b",
                &_SIMPLE_SCHEMA_MEMBER_B,
                &self.field_b,
            )?;
            ser.serialize_optional_member_named(
                "field_c",
                &_SIMPLE_SCHEMA_MEMBER_C,
                &self.field_c,
            )?;
            ser.end(schema)
        }
    }
    #[automatically_derived]
    impl<'de> _DeserializeWithSchema<'de> for SimpleStructBuilder {
        fn deserialize_with_schema<D>(
            schema: &_SchemaRef,
            deserializer: &mut D,
        ) -> Result<Self, D::Error>
        where
            D: _Deserializer<'de>,
        {
            let builder = SimpleStructBuilder::new();
            deserializer
                .read_struct(
                    schema,
                    builder,
                    |builder, member_schema, de| {
                        if std::sync::Arc::ptr_eq(
                            member_schema,
                            &_SIMPLE_SCHEMA_MEMBER_A,
                        ) {
                            let value = <String as ::smithy4rs_core::serde::deserializers::DeserializeWithSchema>::deserialize_with_schema(
                                member_schema,
                                de,
                            )?;
                            return Ok(builder.field_a(value));
                        }
                        if std::sync::Arc::ptr_eq(
                            member_schema,
                            &_SIMPLE_SCHEMA_MEMBER_B,
                        ) {
                            let value = <i32 as ::smithy4rs_core::serde::deserializers::DeserializeWithSchema>::deserialize_with_schema(
                                member_schema,
                                de,
                            )?;
                            return Ok(builder.field_b(value));
                        }
                        if std::sync::Arc::ptr_eq(
                            member_schema,
                            &_SIMPLE_SCHEMA_MEMBER_C,
                        ) {
                            let value = <Option<
                                NestedBuilder,
                            > as ::smithy4rs_core::serde::deserializers::DeserializeWithSchema>::deserialize_with_schema(
                                member_schema,
                                de,
                            )?;
                            if let Some(v) = value {
                                return Ok(builder.field_c_builder(v));
                            }
                            return Ok(builder);
                        }
                        Ok(builder)
                    },
                )
        }
    }
    impl<'de> _Buildable<'de, SimpleStructBuilder> for SimpleStruct {}
};
#[automatically_derived]
impl ::core::fmt::Debug for SimpleStruct {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "SimpleStruct",
            "field_a",
            &self.field_a,
            "field_b",
            &self.field_b,
            "field_c",
            &&self.field_c,
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
            && self.field_c == other.field_c
    }
}
#[smithy_schema(NESTED_SCHEMA)]
pub struct Nested {
    #[smithy_schema(D)]
    pub field_a: String,
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for Nested {
        fn schema() -> &'static _SchemaRef {
            &NESTED_SCHEMA
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::StructSerializer as _StructSerializer;
    #[automatically_derived]
    impl _SerializeWithSchema for Nested {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 1usize)?;
            ser.serialize_member_named(
                "field_a",
                &_NESTED_SCHEMA_MEMBER_D,
                &self.field_a,
            )?;
            ser.end(schema)
        }
    }
};
#[automatically_derived]
pub struct NestedBuilder {
    field_a: smithy4rs_core::serde::builders::Required<String>,
}
#[automatically_derived]
impl ::core::clone::Clone for NestedBuilder {
    #[inline]
    fn clone(&self) -> NestedBuilder {
        NestedBuilder {
            field_a: ::core::clone::Clone::clone(&self.field_a),
        }
    }
}
#[automatically_derived]
impl NestedBuilder {
    pub fn new() -> Self {
        Self {
            field_a: smithy4rs_core::serde::builders::Required::Unset,
        }
    }
    pub fn field_a(mut self, value: String) -> Self {
        self.field_a = smithy4rs_core::serde::builders::Required::Set(value);
        self
    }
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
    use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    use _smithy4rs::serde::deserializers::Error as _Error;
    use _smithy4rs::serde::correction::ErrorCorrection as _ErrorCorrection;
    use _smithy4rs::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
    use _smithy4rs::serde::ShapeBuilder as _ShapeBuilder;
    use _smithy4rs::serde::Buildable as _Buildable;
    #[automatically_derived]
    impl _ErrorCorrection for NestedBuilder {
        type Value = Nested;
        fn correct(self) -> Self::Value {
            Nested {
                field_a: self.field_a.get(),
            }
        }
    }
    #[automatically_derived]
    impl<'de> _ShapeBuilder<'de, Nested> for NestedBuilder {
        fn new() -> Self {
            Self::new()
        }
    }
    #[automatically_derived]
    impl _ErrorCorrectionDefault for Nested {
        fn default() -> Self {
            NestedBuilder::new().correct()
        }
    }
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::StructSerializer as _StructSerializer;
    #[automatically_derived]
    impl _SerializeWithSchema for NestedBuilder {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 1usize)?;
            ser.serialize_member_named(
                "field_a",
                &_NESTED_SCHEMA_MEMBER_D,
                &self.field_a,
            )?;
            ser.end(schema)
        }
    }
    #[automatically_derived]
    impl<'de> _DeserializeWithSchema<'de> for NestedBuilder {
        fn deserialize_with_schema<D>(
            schema: &_SchemaRef,
            deserializer: &mut D,
        ) -> Result<Self, D::Error>
        where
            D: _Deserializer<'de>,
        {
            let builder = NestedBuilder::new();
            deserializer
                .read_struct(
                    schema,
                    builder,
                    |builder, member_schema, de| {
                        if std::sync::Arc::ptr_eq(
                            member_schema,
                            &_NESTED_SCHEMA_MEMBER_D,
                        ) {
                            let value = <String as ::smithy4rs_core::serde::deserializers::DeserializeWithSchema>::deserialize_with_schema(
                                member_schema,
                                de,
                            )?;
                            return Ok(builder.field_a(value));
                        }
                        Ok(builder)
                    },
                )
        }
    }
    impl<'de> _Buildable<'de, NestedBuilder> for Nested {}
};
#[automatically_derived]
impl ::core::fmt::Debug for Nested {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "Nested",
            "field_a",
            &&self.field_a,
        )
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Nested {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Nested {
    #[inline]
    fn eq(&self, other: &Nested) -> bool {
        self.field_a == other.field_a
    }
}
