use smithy4rs_core::prelude::STRING;
use smithy4rs_core::smithy;
use smithy4rs_core_derive::{SmithyShape, SmithyTraitImpl};
#[doc(hidden)]
pub static NESTED_BUILDER: ::smithy4rs_core::LazyLock<
    ::smithy4rs_core::Ref<::smithy4rs_core::schema::SchemaBuilder>,
> = ::smithy4rs_core::LazyLock::new(|| ::smithy4rs_core::Ref::new(
    ::smithy4rs_core::schema::Schema::list_builder("test#SimpleTrait", Vec::new()),
));
pub static NESTED: ::smithy4rs_core::LazyLock<::smithy4rs_core::schema::Schema> = ::smithy4rs_core::LazyLock::new(||
{ (&*NESTED_BUILDER).put_member("member", &SIMPLE_SCHEMA, Vec::new()).build() });
#[smithy_schema(NESTED)]
#[repr(transparent)]
pub struct NestedWrapper(Vec<Nested>);
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for NestedWrapper {
        #[inline]
        fn schema() -> &'static _Schema {
            &NESTED
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::StructWriter as _StructWriter;
    #[automatically_derived]
    impl _SerializeWithSchema for NestedWrapper {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_Schema,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            self.0.serialize_with_schema(schema, serializer)
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
    use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    #[automatically_derived]
    impl<'de> _DeserializeWithSchema<'de> for NestedWrapper {
        #[inline]
        fn deserialize_with_schema<D>(
            schema: &_Schema,
            deserializer: D,
        ) -> Result<Self, D::Error>
        where
            D: _Deserializer<'de>,
        {
            let inner = <Vec<
                Nested,
            > as _DeserializeWithSchema>::deserialize_with_schema(schema, deserializer)?;
            Ok(Self(inner))
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::serde::debug::DebugWrapper as _DebugWrapper;
    #[automatically_derived]
    impl std::fmt::Debug for NestedWrapper {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&_DebugWrapper::new(&NESTED, self), f)
        }
    }
};
impl NestedWrapper {
    ///Create a new [`NestedWrapper`] instance
    #[automatically_derived]
    #[inline]
    pub fn new<T: Into<Vec<Nested>>>(
        value: T,
    ) -> smithy4rs_core::serde::validation::Validated<NestedWrapper> {
        let mut validator = smithy4rs_core::serde::validation::DefaultValidator::new();
        let res = NestedWrapper(value.into());
        smithy4rs_core::serde::validation::Validator::validate(
            &mut validator,
            &NESTED,
            &res,
        )?;
        Ok(res)
    }
}
const _: () = {
    use std::ops::Deref as _Deref;
    impl _Deref for NestedWrapper {
        type Target = Vec<Nested>;
        #[automatically_derived]
        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::StaticTraitId as _StaticTraitId;
    use _smithy4rs::schema::ShapeId as _ShapeId;
    use _smithy4rs::LazyLock as _LazyLock;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    impl _StaticTraitId for NestedWrapper {
        #[inline]
        #[automatically_derived]
        fn trait_id() -> &'static _ShapeId {
            static ID: _LazyLock<&_ShapeId> = _LazyLock::new(|| {
                &<NestedWrapper as _StaticSchemaShape>::schema().id()
            });
            *ID
        }
    }
};
impl PartialEq for NestedWrapper {
    fn eq(&self, other: &Self) -> bool {
        &self.0 == &other.0
    }
}
#[automatically_derived]
impl ::core::clone::Clone for NestedWrapper {
    #[inline]
    fn clone(&self) -> NestedWrapper {
        NestedWrapper(::core::clone::Clone::clone(&self.0))
    }
}
#[doc(hidden)]
pub static SIMPLE_SCHEMA_BUILDER: ::smithy4rs_core::LazyLock<
    ::smithy4rs_core::Ref<::smithy4rs_core::schema::SchemaBuilder>,
> = ::smithy4rs_core::LazyLock::new(|| ::smithy4rs_core::Ref::new(
    ::smithy4rs_core::schema::Schema::structure_builder("test#SimpleStruct", Vec::new()),
));
pub static SIMPLE_SCHEMA: ::smithy4rs_core::LazyLock<::smithy4rs_core::schema::Schema> = ::smithy4rs_core::LazyLock::new(||
{ (&*SIMPLE_SCHEMA_BUILDER).put_member("field_a", &STRING, Vec::new()).build() });
static _SIMPLE_SCHEMA_MEMBER_A: ::smithy4rs_core::LazyLock<
    &::smithy4rs_core::schema::Schema,
> = ::smithy4rs_core::LazyLock::new(|| SIMPLE_SCHEMA.expect_member("field_a"));
#[smithy_schema(SIMPLE_SCHEMA)]
pub struct Nested {
    #[smithy_schema(A)]
    pub field_a: String,
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for Nested {
        #[inline]
        fn schema() -> &'static _Schema {
            &SIMPLE_SCHEMA
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::StructWriter as _StructWriter;
    #[automatically_derived]
    impl _SerializeWithSchema for Nested {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_Schema,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 1usize)?;
            ser.write_member_named("field_a", &_SIMPLE_SCHEMA_MEMBER_A, &self.field_a)?;
            ser.end(schema)
        }
    }
};
#[automatically_derived]
impl Nested {
    /// Get a new builder for this shape.
    #[must_use]
    #[inline]
    pub fn builder() -> NestedBuilder {
        <Self as smithy4rs_core::serde::Buildable<NestedBuilder>>::builder()
    }
}
///Builder for [`Nested`]
#[automatically_derived]
pub struct NestedBuilder {
    field_a: smithy4rs_core::serde::Required<String>,
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
    ///Create a new `NestedBuilder` instance
    pub fn new() -> Self {
        Self {
            field_a: smithy4rs_core::serde::Required::Unset,
        }
    }
    ///Set `field_a`.
    pub fn field_a<T: Into<String>>(mut self, value: T) -> Self {
        self.field_a = smithy4rs_core::serde::Required::Set(value.into());
        self
    }
    /// Build the shape, validating with the default validator.
    #[inline]
    pub fn build(self) -> smithy4rs_core::serde::validation::Validated<Nested> {
        smithy4rs_core::serde::ShapeBuilder::build(self)
    }
    /// Build the shape using a custom validator.
    #[inline]
    pub fn build_with_validator(
        self,
        validator: impl smithy4rs_core::serde::validation::Validator,
    ) -> smithy4rs_core::serde::validation::Validated<Nested> {
        smithy4rs_core::serde::ShapeBuilder::build_with_validator(self, validator)
    }
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for NestedBuilder {
        #[inline]
        fn schema() -> &'static _Schema {
            &SIMPLE_SCHEMA
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
    use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    use _smithy4rs::serde::correction::ErrorCorrection as _ErrorCorrection;
    use _smithy4rs::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
    use _smithy4rs::serde::ShapeBuilder as _ShapeBuilder;
    use _smithy4rs::serde::Buildable as _Buildable;
    use _smithy4rs::serde::deserializers::StructReader as _StructReader;
    #[automatically_derived]
    impl<'de> _DeserializeWithSchema<'de> for NestedBuilder {
        fn deserialize_with_schema<D>(
            schema: &_Schema,
            deserializer: D,
        ) -> Result<Self, D::Error>
        where
            D: _Deserializer<'de>,
        {
            let mut builder = NestedBuilder::new();
            let mut reader = deserializer.read_struct(schema)?;
            while let Some(member_schema) = reader.read_member(schema)? {
                if member_schema == *_SIMPLE_SCHEMA_MEMBER_A {
                    let value: String = reader.read_value(member_schema)?;
                    builder = builder.field_a(value);
                    continue;
                }
                reader.skip_value()?;
            }
            Ok(builder)
        }
    }
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
    use _smithy4rs::serde::serializers::StructWriter as _StructWriter;
    #[automatically_derived]
    impl _SerializeWithSchema for NestedBuilder {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_Schema,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 1usize)?;
            ser.write_member_named("field_a", &_SIMPLE_SCHEMA_MEMBER_A, &self.field_a)?;
            ser.end(schema)
        }
    }
    impl<'de> _Buildable<'de, NestedBuilder> for Nested {}
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::serde::debug::DebugWrapper as _DebugWrapper;
    #[automatically_derived]
    impl std::fmt::Debug for Nested {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&_DebugWrapper::new(&SIMPLE_SCHEMA, self), f)
        }
    }
};
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Nested {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Nested {
    #[inline]
    fn eq(&self, other: &Nested) -> bool {
        self.field_a == other.field_a
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Nested {
    #[inline]
    fn clone(&self) -> Nested {
        Nested {
            field_a: ::core::clone::Clone::clone(&self.field_a),
        }
    }
}
