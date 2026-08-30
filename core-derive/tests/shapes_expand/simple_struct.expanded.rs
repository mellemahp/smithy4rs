use smithy4rs_core::{
    prelude::{INTEGER, STRING},
    smithy,
};
use smithy4rs_core_derive::SmithyShape;
#[doc(hidden)]
pub static SIMPLE_SCHEMA_BUILDER: ::smithy4rs_core::LazyLock<
    ::smithy4rs_core::Ref<::smithy4rs_core::schema::SchemaBuilder>,
> = ::smithy4rs_core::LazyLock::new(|| ::smithy4rs_core::Ref::new(
    ::smithy4rs_core::schema::Schema::structure_builder("test#SimpleSchema", Vec::new()),
));
pub static SIMPLE_SCHEMA: ::smithy4rs_core::LazyLock<::smithy4rs_core::schema::Schema> = ::smithy4rs_core::LazyLock::new(||
{
    (&*SIMPLE_SCHEMA_BUILDER)
        .put_member("fieldA", &STRING, Vec::new())
        .put_member("fieldB", &INTEGER, Vec::new())
        .put_member("fieldC", &STRING, Vec::new())
        .build()
});
#[allow(dead_code)]
const SIMPLE_SCHEMA_KEYS: &[&str] = &["fieldA", "fieldB", "fieldC"];
#[schema(schema = SIMPLE_SCHEMA)]
pub struct SimpleStruct {
    pub field_a: String,
    #[schema(default = 0)]
    pub field_b: i32,
    pub field_c: Option<Nested>,
}
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for SimpleStruct {
        #[inline]
        fn schema() -> &'static _Schema {
            &SIMPLE_SCHEMA
        }
    }
};
const _: () = ::smithy4rs_core::assert_contains_all(
    SIMPLE_SCHEMA_KEYS,
    &["fieldA", "fieldB", "fieldC"],
);
static _SIMPLE_SCHEMA_MEMBER_FIELD_A: ::smithy4rs_core::LazyLock<
    &::smithy4rs_core::schema::Schema,
> = ::smithy4rs_core::LazyLock::new(|| SIMPLE_SCHEMA.expect_member("fieldA"));
static _SIMPLE_SCHEMA_MEMBER_FIELD_B: ::smithy4rs_core::LazyLock<
    &::smithy4rs_core::schema::Schema,
> = ::smithy4rs_core::LazyLock::new(|| SIMPLE_SCHEMA.expect_member("fieldB"));
static _SIMPLE_SCHEMA_MEMBER_FIELD_C: ::smithy4rs_core::LazyLock<
    &::smithy4rs_core::schema::Schema,
> = ::smithy4rs_core::LazyLock::new(|| SIMPLE_SCHEMA.expect_member("fieldC"));
const _: () = {
    use ::smithy4rs_core::serde::debug::DebugWrapper as _DebugWrapper;
    #[automatically_derived]
    impl std::fmt::Debug for SimpleStruct {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&_DebugWrapper::new(&SIMPLE_SCHEMA, self), f)
        }
    }
};
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::serde::serializers::Serializer as _Serializer;
    use ::smithy4rs_core::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use ::smithy4rs_core::serde::serializers::StructWriter as _StructWriter;
    #[automatically_derived]
    impl _SerializeWithSchema for SimpleStruct {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_Schema,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 3usize)?;
            ser.write_member_named(
                "fieldA",
                &_SIMPLE_SCHEMA_MEMBER_FIELD_A,
                &self.field_a,
            )?;
            ser.write_member_named(
                "fieldB",
                &_SIMPLE_SCHEMA_MEMBER_FIELD_B,
                &self.field_b,
            )?;
            ser.write_optional_member_named(
                "fieldC",
                &_SIMPLE_SCHEMA_MEMBER_FIELD_C,
                &self.field_c,
            )?;
            ser.end(schema)
        }
    }
};
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::serde::deserializers::Deserializer as _Deserializer;
    use ::smithy4rs_core::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    use ::smithy4rs_core::serde::correction::ErrorCorrection as _ErrorCorrection;
    use ::smithy4rs_core::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
    use ::smithy4rs_core::serde::Buildable as _Buildable;
    use ::smithy4rs_core::serde::deserializers::StructReader as _StructReader;
    #[automatically_derived]
    impl<'de> _DeserializeWithSchema<'de> for SimpleStructBuilder {
        fn deserialize_with_schema<D>(
            schema: &_Schema,
            deserializer: D,
        ) -> Result<Self, D::Error>
        where
            D: _Deserializer<'de>,
        {
            let mut builder = SimpleStructBuilder::new();
            let mut reader = deserializer.read_struct(schema)?;
            while let Some(member_schema) = reader.read_member(schema)? {
                if member_schema == *_SIMPLE_SCHEMA_MEMBER_FIELD_A {
                    let value: String = reader.read_value(member_schema)?;
                    builder = builder.field_a(value);
                    continue;
                }
                if member_schema == *_SIMPLE_SCHEMA_MEMBER_FIELD_B {
                    let value: i32 = reader.read_value(member_schema)?;
                    builder = builder.field_b(value);
                    continue;
                }
                if member_schema == *_SIMPLE_SCHEMA_MEMBER_FIELD_C {
                    let value: Option<NestedBuilder> = reader.read_value(member_schema)?;
                    if let Some(v) = value {
                        builder = builder.field_c_builder(v);
                    }
                    continue;
                }
                reader.skip_value()?;
            }
            Ok(builder)
        }
    }
};
const _: () = {
    use ::smithy4rs_core::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
    impl _ErrorCorrectionDefault for SimpleStruct {
        #[inline]
        #[automatically_derived]
        fn default() -> Self {
            SimpleStructBuilder::new().correct()
        }
    }
};
/// Builder for [`SimpleStruct`]
#[automatically_derived]
impl SimpleStruct {
    /// Get a new builder for this shape.
    #[must_use]
    #[inline]
    pub fn builder() -> SimpleStructBuilder {
        <Self as ::smithy4rs_core::serde::Buildable<SimpleStructBuilder>>::builder()
    }
}
/// Builder for [`SimpleStruct`]
#[automatically_derived]
pub struct SimpleStructBuilder {
    field_a: ::smithy4rs_core::serde::Required<String>,
    field_b: ::smithy4rs_core::serde::Required<i32>,
    field_c: Option<::smithy4rs_core::serde::MaybeBuilt<Nested, NestedBuilder>>,
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
    ///Create a new `SimpleStructBuilder` instance
    pub fn new() -> Self {
        SimpleStructBuilder {
            field_a: ::smithy4rs_core::serde::Required::Unset,
            field_b: ::smithy4rs_core::serde::Required::Set(0),
            field_c: None,
        }
    }
    ///Set `field_a`.
    #[inline]
    pub fn field_a<T: Into<String>>(mut self, value: T) -> Self {
        self.field_a = ::smithy4rs_core::serde::Required::Set(value.into());
        self
    }
    ///Set `field_b`.
    #[inline]
    pub fn field_b<T: Into<i32>>(mut self, value: T) -> Self {
        self.field_b = ::smithy4rs_core::serde::Required::Set(value.into());
        self
    }
    ///Set `field_c`.
    #[inline]
    pub fn field_c(mut self, value: Nested) -> Self {
        self.field_c = Some(::smithy4rs_core::serde::MaybeBuilt::Struct(value));
        self
    }
    ///Set `field_c`.
    #[inline]
    pub fn field_c_builder(mut self, value: NestedBuilder) -> Self {
        self.field_c = Some(::smithy4rs_core::serde::MaybeBuilt::Builder(value));
        self
    }
    /// Build the shape, validating with the default validator.
    #[inline]
    pub fn build(self) -> ::smithy4rs_core::serde::validation::Validated<SimpleStruct> {
        ::smithy4rs_core::serde::ShapeBuilder::build(self)
    }
    /// Build the shape using a custom validator.
    #[inline]
    pub fn build_with_validator(
        self,
        validator: impl ::smithy4rs_core::serde::validation::Validator,
    ) -> ::smithy4rs_core::serde::validation::Validated<SimpleStruct> {
        ::smithy4rs_core::serde::ShapeBuilder::build_with_validator(self, validator)
    }
    /// Error correct builder
    pub fn correct(self) -> SimpleStruct {
        <Self as ::smithy4rs_core::serde::correction::ErrorCorrection>::correct(self)
    }
}
const _: () = {
    use ::smithy4rs_core::serde::correction::ErrorCorrection as _ErrorCorrection;
    #[automatically_derived]
    impl _ErrorCorrection for SimpleStructBuilder {
        type Value = SimpleStruct;
        fn correct(self) -> Self::Value {
            Self::Value {
                field_a: self.field_a.get(),
                field_b: self.field_b.get(),
                field_c: self.field_c.correct(),
            }
        }
    }
};
const _: () = {
    use ::smithy4rs_core::serde::ShapeBuilder as _ShapeBuilder;
    #[automatically_derived]
    impl<'de> _ShapeBuilder<'de, SimpleStruct> for SimpleStructBuilder {
        fn new() -> Self {
            SimpleStructBuilder::new()
        }
    }
};
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::serde::serializers::Serializer as _Serializer;
    use ::smithy4rs_core::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use ::smithy4rs_core::serde::serializers::StructWriter as _StructWriter;
    #[automatically_derived]
    impl _SerializeWithSchema for SimpleStructBuilder {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_Schema,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 3usize)?;
            ser.write_member_named(
                "fieldA",
                &_SIMPLE_SCHEMA_MEMBER_FIELD_A,
                &self.field_a,
            )?;
            ser.write_member_named(
                "fieldB",
                &_SIMPLE_SCHEMA_MEMBER_FIELD_B,
                &self.field_b,
            )?;
            ser.write_optional_member_named(
                "fieldC",
                &_SIMPLE_SCHEMA_MEMBER_FIELD_C,
                &self.field_c,
            )?;
            ser.end(schema)
        }
    }
};
const _: () = {
    use ::smithy4rs_core::serde::Buildable as _Buildable;
    impl<'de> _Buildable<'de, SimpleStructBuilder> for SimpleStruct {}
};
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for SimpleStructBuilder {
        #[inline]
        fn schema() -> &'static _Schema {
            &SIMPLE_SCHEMA
        }
    }
};
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
#[automatically_derived]
impl ::core::clone::Clone for SimpleStruct {
    #[inline]
    fn clone(&self) -> SimpleStruct {
        SimpleStruct {
            field_a: ::core::clone::Clone::clone(&self.field_a),
            field_b: ::core::clone::Clone::clone(&self.field_b),
            field_c: ::core::clone::Clone::clone(&self.field_c),
        }
    }
}
#[doc(hidden)]
pub static NESTED_SCHEMA_BUILDER: ::smithy4rs_core::LazyLock<
    ::smithy4rs_core::Ref<::smithy4rs_core::schema::SchemaBuilder>,
> = ::smithy4rs_core::LazyLock::new(|| ::smithy4rs_core::Ref::new(
    ::smithy4rs_core::schema::Schema::structure_builder("test#NestedSchema", Vec::new()),
));
pub static NESTED_SCHEMA: ::smithy4rs_core::LazyLock<::smithy4rs_core::schema::Schema> = ::smithy4rs_core::LazyLock::new(||
{ (&*NESTED_SCHEMA_BUILDER).put_member("fieldD", &STRING, Vec::new()).build() });
#[allow(dead_code)]
const NESTED_SCHEMA_KEYS: &[&str] = &["fieldD"];
#[schema(schema = NESTED_SCHEMA)]
pub struct Nested {
    pub field_d: String,
}
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for Nested {
        #[inline]
        fn schema() -> &'static _Schema {
            &NESTED_SCHEMA
        }
    }
};
const _: () = ::smithy4rs_core::assert_contains_all(NESTED_SCHEMA_KEYS, &["fieldD"]);
static _NESTED_SCHEMA_MEMBER_FIELD_D: ::smithy4rs_core::LazyLock<
    &::smithy4rs_core::schema::Schema,
> = ::smithy4rs_core::LazyLock::new(|| NESTED_SCHEMA.expect_member("fieldD"));
const _: () = {
    use ::smithy4rs_core::serde::debug::DebugWrapper as _DebugWrapper;
    #[automatically_derived]
    impl std::fmt::Debug for Nested {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&_DebugWrapper::new(&NESTED_SCHEMA, self), f)
        }
    }
};
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::serde::serializers::Serializer as _Serializer;
    use ::smithy4rs_core::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use ::smithy4rs_core::serde::serializers::StructWriter as _StructWriter;
    #[automatically_derived]
    impl _SerializeWithSchema for Nested {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_Schema,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 1usize)?;
            ser.write_member_named(
                "fieldD",
                &_NESTED_SCHEMA_MEMBER_FIELD_D,
                &self.field_d,
            )?;
            ser.end(schema)
        }
    }
};
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::serde::deserializers::Deserializer as _Deserializer;
    use ::smithy4rs_core::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    use ::smithy4rs_core::serde::correction::ErrorCorrection as _ErrorCorrection;
    use ::smithy4rs_core::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
    use ::smithy4rs_core::serde::Buildable as _Buildable;
    use ::smithy4rs_core::serde::deserializers::StructReader as _StructReader;
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
                if member_schema == *_NESTED_SCHEMA_MEMBER_FIELD_D {
                    let value: String = reader.read_value(member_schema)?;
                    builder = builder.field_d(value);
                    continue;
                }
                reader.skip_value()?;
            }
            Ok(builder)
        }
    }
};
const _: () = {
    use ::smithy4rs_core::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
    impl _ErrorCorrectionDefault for Nested {
        #[inline]
        #[automatically_derived]
        fn default() -> Self {
            NestedBuilder::new().correct()
        }
    }
};
/// Builder for [`Nested`]
#[automatically_derived]
impl Nested {
    /// Get a new builder for this shape.
    #[must_use]
    #[inline]
    pub fn builder() -> NestedBuilder {
        <Self as ::smithy4rs_core::serde::Buildable<NestedBuilder>>::builder()
    }
}
/// Builder for [`Nested`]
#[automatically_derived]
pub struct NestedBuilder {
    field_d: ::smithy4rs_core::serde::Required<String>,
}
#[automatically_derived]
impl ::core::clone::Clone for NestedBuilder {
    #[inline]
    fn clone(&self) -> NestedBuilder {
        NestedBuilder {
            field_d: ::core::clone::Clone::clone(&self.field_d),
        }
    }
}
#[automatically_derived]
impl NestedBuilder {
    ///Create a new `NestedBuilder` instance
    pub fn new() -> Self {
        NestedBuilder {
            field_d: ::smithy4rs_core::serde::Required::Unset,
        }
    }
    ///Set `field_d`.
    #[inline]
    pub fn field_d<T: Into<String>>(mut self, value: T) -> Self {
        self.field_d = ::smithy4rs_core::serde::Required::Set(value.into());
        self
    }
    /// Build the shape, validating with the default validator.
    #[inline]
    pub fn build(self) -> ::smithy4rs_core::serde::validation::Validated<Nested> {
        ::smithy4rs_core::serde::ShapeBuilder::build(self)
    }
    /// Build the shape using a custom validator.
    #[inline]
    pub fn build_with_validator(
        self,
        validator: impl ::smithy4rs_core::serde::validation::Validator,
    ) -> ::smithy4rs_core::serde::validation::Validated<Nested> {
        ::smithy4rs_core::serde::ShapeBuilder::build_with_validator(self, validator)
    }
    /// Error correct builder
    pub fn correct(self) -> Nested {
        <Self as ::smithy4rs_core::serde::correction::ErrorCorrection>::correct(self)
    }
}
const _: () = {
    use ::smithy4rs_core::serde::correction::ErrorCorrection as _ErrorCorrection;
    #[automatically_derived]
    impl _ErrorCorrection for NestedBuilder {
        type Value = Nested;
        fn correct(self) -> Self::Value {
            Self::Value {
                field_d: self.field_d.get(),
            }
        }
    }
};
const _: () = {
    use ::smithy4rs_core::serde::ShapeBuilder as _ShapeBuilder;
    #[automatically_derived]
    impl<'de> _ShapeBuilder<'de, Nested> for NestedBuilder {
        fn new() -> Self {
            NestedBuilder::new()
        }
    }
};
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::serde::serializers::Serializer as _Serializer;
    use ::smithy4rs_core::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use ::smithy4rs_core::serde::serializers::StructWriter as _StructWriter;
    #[automatically_derived]
    impl _SerializeWithSchema for NestedBuilder {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_Schema,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 1usize)?;
            ser.write_member_named(
                "fieldD",
                &_NESTED_SCHEMA_MEMBER_FIELD_D,
                &self.field_d,
            )?;
            ser.end(schema)
        }
    }
};
const _: () = {
    use ::smithy4rs_core::serde::Buildable as _Buildable;
    impl<'de> _Buildable<'de, NestedBuilder> for Nested {}
};
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for NestedBuilder {
        #[inline]
        fn schema() -> &'static _Schema {
            &NESTED_SCHEMA
        }
    }
};
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Nested {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Nested {
    #[inline]
    fn eq(&self, other: &Nested) -> bool {
        self.field_d == other.field_d
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Nested {
    #[inline]
    fn clone(&self) -> Nested {
        Nested {
            field_d: ::core::clone::Clone::clone(&self.field_d),
        }
    }
}
