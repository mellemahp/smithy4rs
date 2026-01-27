#![allow(dead_code)]
mod r#enum {
    use smithy4rs_core_derive::{SmithyShape, smithy_enum};
    use smithy4rs_core::smithy;
    pub static SIMPLE_ENUM: ::smithy4rs_core::LazyLock<
        ::smithy4rs_core::schema::Schema,
    > = ::smithy4rs_core::LazyLock::new(|| {
        ::smithy4rs_core::schema::Schema::create_enum(
            "test#SimpleStruct",
            Box::new(["a", "b", "c"]),
            Vec::new(),
        )
    });
    #[smithy_schema(SIMPLE_ENUM)]
    pub enum TestEnum {
        #[enum_value("a")]
        A,
        #[enum_value("b")]
        B,
        #[enum_value("c")]
        C,
        #[automatically_derived]
        #[doc(hidden)]
        Unknown(String),
    }
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::schema::Schema as _Schema;
        use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
        #[automatically_derived]
        impl _StaticSchemaShape for TestEnum {
            fn schema() -> &'static _Schema {
                &SIMPLE_ENUM
            }
        }
    };
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::schema::Schema as _Schema;
        use _smithy4rs::serde::serializers::Serializer as _Serializer;
        use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
        #[automatically_derived]
        impl _SerializeWithSchema for TestEnum {
            fn serialize_with_schema<S: _Serializer>(
                &self,
                schema: &_Schema,
                serializer: S,
            ) -> Result<S::Ok, S::Error> {
                let value = match self {
                    TestEnum::A => "a",
                    TestEnum::B => "b",
                    TestEnum::C => "c",
                    TestEnum::Unknown(value) => value.as_str(),
                };
                serializer.write_string(schema, value)
            }
        }
    };
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::schema::Schema as _Schema;
        use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
        use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
        #[automatically_derived]
        impl<'de> _DeserializeWithSchema<'de> for TestEnum {
            fn deserialize_with_schema<D>(
                schema: &_Schema,
                deserializer: &mut D,
            ) -> Result<Self, D::Error>
            where
                D: _Deserializer<'de>,
            {
                let val = deserializer.read_string(schema)?;
                let result = match val.as_str() {
                    "a" => TestEnum::A,
                    "b" => TestEnum::B,
                    "c" => TestEnum::C,
                    _ => TestEnum::Unknown(val),
                };
                Ok(result)
            }
        }
    };
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::serde::debug::DebugWrapper as _DebugWrapper;
        #[automatically_derived]
        impl std::fmt::Debug for TestEnum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&_DebugWrapper::new(&SIMPLE_ENUM, self), f)
            }
        }
    };
}
mod int_enum {
    use smithy4rs_core_derive::{SmithyShape, smithy_enum};
    use smithy4rs_core::smithy;
    pub static SIMPLE_INT_ENUM: ::smithy4rs_core::LazyLock<
        ::smithy4rs_core::schema::Schema,
    > = ::smithy4rs_core::LazyLock::new(|| {
        ::smithy4rs_core::schema::Schema::create_int_enum(
            "test#SimpleStruct",
            Box::new([1, 2, 3]),
            Vec::new(),
        )
    });
    #[smithy_schema(SIMPLE_INT_ENUM)]
    pub enum TestIntEnum {
        #[enum_value(1)]
        A,
        #[enum_value(2)]
        B,
        #[enum_value(3)]
        C,
        #[automatically_derived]
        #[doc(hidden)]
        Unknown(i32),
    }
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::schema::Schema as _Schema;
        use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
        #[automatically_derived]
        impl _StaticSchemaShape for TestIntEnum {
            fn schema() -> &'static _Schema {
                &SIMPLE_INT_ENUM
            }
        }
    };
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::schema::Schema as _Schema;
        use _smithy4rs::serde::serializers::Serializer as _Serializer;
        use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
        #[automatically_derived]
        impl _SerializeWithSchema for TestIntEnum {
            fn serialize_with_schema<S: _Serializer>(
                &self,
                schema: &_Schema,
                serializer: S,
            ) -> Result<S::Ok, S::Error> {
                let value = match self {
                    TestIntEnum::A => 1,
                    TestIntEnum::B => 2,
                    TestIntEnum::C => 3,
                    TestIntEnum::Unknown(value) => *value,
                };
                serializer.write_integer(schema, value)
            }
        }
    };
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::schema::Schema as _Schema;
        use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
        use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
        #[automatically_derived]
        impl<'de> _DeserializeWithSchema<'de> for TestIntEnum {
            fn deserialize_with_schema<D>(
                schema: &_Schema,
                deserializer: &mut D,
            ) -> Result<Self, D::Error>
            where
                D: _Deserializer<'de>,
            {
                let val = deserializer.read_integer(schema)?;
                let result = match val {
                    1 => TestIntEnum::A,
                    2 => TestIntEnum::B,
                    3 => TestIntEnum::C,
                    _ => TestIntEnum::Unknown(val),
                };
                Ok(result)
            }
        }
    };
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::serde::debug::DebugWrapper as _DebugWrapper;
        #[automatically_derived]
        impl std::fmt::Debug for TestIntEnum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&_DebugWrapper::new(&SIMPLE_INT_ENUM, self), f)
            }
        }
    };
}
mod simple_struct {
    use smithy4rs_core::{
        prelude::{INTEGER, STRING},
        smithy,
    };
    use smithy4rs_core_derive::SmithyShape;
    #[doc(hidden)]
    pub static SIMPLE_SCHEMA_BUILDER: ::smithy4rs_core::LazyLock<
        ::smithy4rs_core::Ref<::smithy4rs_core::schema::SchemaBuilder>,
    > = ::smithy4rs_core::LazyLock::new(|| ::smithy4rs_core::Ref::new(
        ::smithy4rs_core::schema::Schema::structure_builder(
            "test#SimpleStruct",
            Vec::new(),
        ),
    ));
    pub static SIMPLE_SCHEMA: ::smithy4rs_core::LazyLock<
        ::smithy4rs_core::schema::Schema,
    > = ::smithy4rs_core::LazyLock::new(|| {
        (&*SIMPLE_SCHEMA_BUILDER)
            .put_member("field_a", &STRING, Vec::new())
            .put_member("field_b", &INTEGER, Vec::new())
            .put_member("field_c", &STRING, Vec::new())
            .build()
    });
    static _SIMPLE_SCHEMA_MEMBER_A: ::smithy4rs_core::LazyLock<
        &::smithy4rs_core::schema::Schema,
    > = ::smithy4rs_core::LazyLock::new(|| SIMPLE_SCHEMA.expect_member("field_a"));
    static _SIMPLE_SCHEMA_MEMBER_B: ::smithy4rs_core::LazyLock<
        &::smithy4rs_core::schema::Schema,
    > = ::smithy4rs_core::LazyLock::new(|| SIMPLE_SCHEMA.expect_member("field_b"));
    static _SIMPLE_SCHEMA_MEMBER_C: ::smithy4rs_core::LazyLock<
        &::smithy4rs_core::schema::Schema,
    > = ::smithy4rs_core::LazyLock::new(|| SIMPLE_SCHEMA.expect_member("field_c"));
    #[smithy_schema(SIMPLE_SCHEMA)]
    pub struct SimpleStruct {
        #[smithy_schema(A)]
        pub field_a: String,
        #[smithy_schema(B)]
        #[default(0)]
        pub field_b: i32,
        #[smithy_schema(C)]
        pub field_c: Option<Nested>,
    }
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::schema::Schema as _Schema;
        use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
        #[automatically_derived]
        impl _StaticSchemaShape for SimpleStruct {
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
        use _smithy4rs::serde::serializers::StructSerializer as _StructSerializer;
        #[automatically_derived]
        impl _SerializeWithSchema for SimpleStruct {
            fn serialize_with_schema<S: _Serializer>(
                &self,
                schema: &_Schema,
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
        field_a: smithy4rs_core::serde::Required<String>,
        field_b: smithy4rs_core::serde::Required<i32>,
        field_c: Option<smithy4rs_core::serde::MaybeBuilt<Nested, NestedBuilder>>,
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
                field_a: smithy4rs_core::serde::Required::Unset,
                field_b: smithy4rs_core::serde::Required::Set(0),
                field_c: None,
            }
        }
        pub fn field_a<T: Into<String>>(mut self, value: T) -> Self {
            self.field_a = smithy4rs_core::serde::Required::Set(value.into());
            self
        }
        pub fn field_b<T: Into<i32>>(mut self, value: T) -> Self {
            self.field_b = smithy4rs_core::serde::Required::Set(value.into());
            self
        }
        pub fn field_c(mut self, value: Nested) -> Self {
            self.field_c = Some(smithy4rs_core::serde::MaybeBuilt::Struct(value));
            self
        }
        pub fn field_c_builder(mut self, value: NestedBuilder) -> Self {
            self.field_c = Some(smithy4rs_core::serde::MaybeBuilt::Builder(value));
            self
        }
    }
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::schema::Schema as _Schema;
        use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
        use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
        use _smithy4rs::serde::correction::ErrorCorrection as _ErrorCorrection;
        use _smithy4rs::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
        use _smithy4rs::serde::ShapeBuilder as _ShapeBuilder;
        use _smithy4rs::serde::Buildable as _Buildable;
        #[automatically_derived]
        impl<'de> _DeserializeWithSchema<'de> for SimpleStructBuilder {
            fn deserialize_with_schema<D>(
                schema: &_Schema,
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
                            if &member_schema == &*_SIMPLE_SCHEMA_MEMBER_A {
                                let value = <String as ::smithy4rs_core::serde::deserializers::DeserializeWithSchema>::deserialize_with_schema(
                                    member_schema,
                                    de,
                                )?;
                                return Ok(builder.field_a(value));
                            }
                            if &member_schema == &*_SIMPLE_SCHEMA_MEMBER_B {
                                let value = <i32 as ::smithy4rs_core::serde::deserializers::DeserializeWithSchema>::deserialize_with_schema(
                                    member_schema,
                                    de,
                                )?;
                                return Ok(builder.field_b(value));
                            }
                            if &member_schema == &*_SIMPLE_SCHEMA_MEMBER_C {
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
                schema: &_Schema,
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
        impl<'de> _Buildable<'de, SimpleStructBuilder> for SimpleStruct {}
    };
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::serde::debug::DebugWrapper as _DebugWrapper;
        #[automatically_derived]
        impl std::fmt::Debug for SimpleStruct {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&_DebugWrapper::new(&SIMPLE_SCHEMA, self), f)
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
        ::smithy4rs_core::schema::Schema::structure_builder(
            "test#NESTED_STRUCT",
            Vec::new(),
        ),
    ));
    pub static NESTED_SCHEMA: ::smithy4rs_core::LazyLock<
        ::smithy4rs_core::schema::Schema,
    > = ::smithy4rs_core::LazyLock::new(|| {
        (&*NESTED_SCHEMA_BUILDER).put_member("field_d", &STRING, Vec::new()).build()
    });
    static _NESTED_SCHEMA_MEMBER_D: ::smithy4rs_core::LazyLock<
        &::smithy4rs_core::schema::Schema,
    > = ::smithy4rs_core::LazyLock::new(|| NESTED_SCHEMA.expect_member("field_d"));
    #[smithy_schema(NESTED_SCHEMA)]
    pub struct Nested {
        #[smithy_schema(D)]
        pub field_a: String,
    }
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::schema::Schema as _Schema;
        use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
        #[automatically_derived]
        impl _StaticSchemaShape for Nested {
            fn schema() -> &'static _Schema {
                &NESTED_SCHEMA
            }
        }
    };
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::schema::Schema as _Schema;
        use _smithy4rs::serde::serializers::Serializer as _Serializer;
        use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
        use _smithy4rs::serde::serializers::StructSerializer as _StructSerializer;
        #[automatically_derived]
        impl _SerializeWithSchema for Nested {
            fn serialize_with_schema<S: _Serializer>(
                &self,
                schema: &_Schema,
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
        pub fn new() -> Self {
            Self {
                field_a: smithy4rs_core::serde::Required::Unset,
            }
        }
        pub fn field_a<T: Into<String>>(mut self, value: T) -> Self {
            self.field_a = smithy4rs_core::serde::Required::Set(value.into());
            self
        }
    }
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::schema::Schema as _Schema;
        use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
        use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
        use _smithy4rs::serde::correction::ErrorCorrection as _ErrorCorrection;
        use _smithy4rs::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
        use _smithy4rs::serde::ShapeBuilder as _ShapeBuilder;
        use _smithy4rs::serde::Buildable as _Buildable;
        #[automatically_derived]
        impl<'de> _DeserializeWithSchema<'de> for NestedBuilder {
            fn deserialize_with_schema<D>(
                schema: &_Schema,
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
                            if &member_schema == &*_NESTED_SCHEMA_MEMBER_D {
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
                schema: &_Schema,
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
        impl<'de> _Buildable<'de, NestedBuilder> for Nested {}
    };
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::serde::debug::DebugWrapper as _DebugWrapper;
        #[automatically_derived]
        impl std::fmt::Debug for Nested {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&_DebugWrapper::new(&NESTED_SCHEMA, self), f)
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
}
mod union {
    use smithy4rs_core::prelude::{INTEGER, STRING};
    use smithy4rs_core::schema::UNIT;
    use smithy4rs_core::smithy;
    use smithy4rs_core_derive::{SmithyShape, smithy_union};
    #[doc(hidden)]
    pub static UNION_BUILDER: ::smithy4rs_core::LazyLock<
        ::smithy4rs_core::Ref<::smithy4rs_core::schema::SchemaBuilder>,
    > = ::smithy4rs_core::LazyLock::new(|| ::smithy4rs_core::Ref::new(
        ::smithy4rs_core::schema::Schema::union_builder("test#SimpleUnion", Vec::new()),
    ));
    pub static UNION: ::smithy4rs_core::LazyLock<::smithy4rs_core::schema::Schema> = ::smithy4rs_core::LazyLock::new(||
    {
        (&*UNION_BUILDER)
            .put_member("field_a", &STRING, Vec::new())
            .put_member("field_b", &INTEGER, Vec::new())
            .put_member("field_c", &UNIT, Vec::new())
            .build()
    });
    static _UNION_MEMBER_A: ::smithy4rs_core::LazyLock<
        &::smithy4rs_core::schema::Schema,
    > = ::smithy4rs_core::LazyLock::new(|| UNION.expect_member("field_a"));
    static _UNION_MEMBER_B: ::smithy4rs_core::LazyLock<
        &::smithy4rs_core::schema::Schema,
    > = ::smithy4rs_core::LazyLock::new(|| UNION.expect_member("field_b"));
    static _UNION_MEMBER_C: ::smithy4rs_core::LazyLock<
        &::smithy4rs_core::schema::Schema,
    > = ::smithy4rs_core::LazyLock::new(|| UNION.expect_member("field_c"));
    #[smithy_schema(UNION)]
    #[smithy_union_enum]
    pub enum TestEnum {
        #[smithy_schema(A)]
        A(String),
        #[smithy_schema(B)]
        B(i32),
        #[smithy_schema(C)]
        C,
        #[automatically_derived]
        #[doc(hidden)]
        Unknown(String),
    }
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::schema::Schema as _Schema;
        use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
        #[automatically_derived]
        impl _StaticSchemaShape for TestEnum {
            fn schema() -> &'static _Schema {
                &UNION
            }
        }
    };
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::schema::Schema as _Schema;
        use _smithy4rs::serde::serializers::Serializer as _Serializer;
        use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
        use _smithy4rs::serde::serializers::StructSerializer as _StructSerializer;
        use _smithy4rs::schema::Unit as _Unit;
        #[automatically_derived]
        impl _SerializeWithSchema for TestEnum {
            fn serialize_with_schema<S: _Serializer>(
                &self,
                schema: &_Schema,
                serializer: S,
            ) -> Result<S::Ok, S::Error> {
                let mut ser = serializer.write_struct(schema, 1)?;
                match self {
                    TestEnum::A(val) => {
                        ser.serialize_member_named("a", &_UNION_MEMBER_A, val)?
                    }
                    TestEnum::B(val) => {
                        ser.serialize_member_named("b", &_UNION_MEMBER_B, val)?
                    }
                    TestEnum::C => {
                        ser.serialize_member_named("c", &_UNION_MEMBER_C, &_Unit)?
                    }
                    TestEnum::Unknown(unknown) => ser.serialize_unknown(schema, unknown)?,
                }
                ser.end(schema)
            }
        }
    };
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::schema::Schema as _Schema;
        use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
        use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
        use _smithy4rs::serde::deserializers::Error as _DeserializerError;
        use _smithy4rs::schema::Unit as _Unit;
        #[automatically_derived]
        impl<'de> _DeserializeWithSchema<'de> for TestEnum {
            fn deserialize_with_schema<D>(
                schema: &_Schema,
                deserializer: &mut D,
            ) -> Result<Self, D::Error>
            where
                D: _Deserializer<'de>,
            {
                deserializer
                    .read_struct(
                        schema,
                        None,
                        |option, member_schema, de| {
                            if option.is_some() {
                                return Err(
                                    D::Error::custom("Attempted to set union value twice"),
                                );
                            }
                            if &member_schema == &*_UNION_MEMBER_A {
                                let value = String::deserialize_with_schema(
                                    member_schema,
                                    de,
                                )?;
                                return Ok(Some(TestEnum::A(value)));
                            }
                            if &member_schema == &*_UNION_MEMBER_B {
                                let value = i32::deserialize_with_schema(
                                    member_schema,
                                    de,
                                )?;
                                return Ok(Some(TestEnum::B(value)));
                            }
                            if &member_schema == &*_UNION_MEMBER_C {
                                let _ = _Unit::deserialize_with_schema(member_schema, de)?;
                                return Ok(Some(TestEnum::C));
                            }
                            Ok(Some(TestEnum::Unknown("unknown".to_string())))
                        },
                    )?
                    .ok_or(D::Error::custom("Failed to deserialize union"))
            }
        }
    };
    const _: () = {
        extern crate smithy4rs_core as _smithy4rs;
        use _smithy4rs::serde::debug::DebugWrapper as _DebugWrapper;
        #[automatically_derived]
        impl std::fmt::Debug for TestEnum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&_DebugWrapper::new(&UNION, self), f)
            }
        }
    };
}
