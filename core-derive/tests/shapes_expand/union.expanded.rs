use smithy4rs_core::{
    prelude::{INTEGER, STRING},
    schema::UNIT, smithy,
};
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
        .put_member("a", &STRING, Vec::new())
        .put_member("b", &INTEGER, Vec::new())
        .put_member("c", &UNIT, Vec::new())
        .build()
});
#[allow(dead_code)]
const UNION_KEYS: &[&str] = &["a", "b", "c"];
#[schema(schema = UNION)]
#[smithy_union_enum]
pub enum TestEnum {
    A(String),
    B(i32),
    C,
    #[automatically_derived]
    #[doc(hidden)]
    Unknown(String),
}
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for TestEnum {
        #[inline]
        fn schema() -> &'static _Schema {
            &UNION
        }
    }
};
const _: () = ::smithy4rs_core::assert_contains_all(UNION_KEYS, &["a", "b", "c"]);
static _UNION_MEMBER_A: ::smithy4rs_core::LazyLock<&::smithy4rs_core::schema::Schema> = ::smithy4rs_core::LazyLock::new(||
UNION.expect_member("a"));
static _UNION_MEMBER_B: ::smithy4rs_core::LazyLock<&::smithy4rs_core::schema::Schema> = ::smithy4rs_core::LazyLock::new(||
UNION.expect_member("b"));
static _UNION_MEMBER_C: ::smithy4rs_core::LazyLock<&::smithy4rs_core::schema::Schema> = ::smithy4rs_core::LazyLock::new(||
UNION.expect_member("c"));
const _: () = {
    use ::smithy4rs_core::serde::debug::DebugWrapper as _DebugWrapper;
    #[automatically_derived]
    impl std::fmt::Debug for TestEnum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&_DebugWrapper::new(&UNION, self), f)
        }
    }
};
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::serde::serializers::Serializer as _Serializer;
    use ::smithy4rs_core::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use ::smithy4rs_core::serde::serializers::StructWriter as _StructWriter;
    use ::smithy4rs_core::schema::Unit as _Unit;
    #[automatically_derived]
    impl _SerializeWithSchema for TestEnum {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_Schema,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 1)?;
            match self {
                TestEnum::A(value) => {
                    ser.write_member_named("a", &_UNION_MEMBER_A, value)?
                }
                TestEnum::B(value) => {
                    ser.write_member_named("b", &_UNION_MEMBER_B, value)?
                }
                TestEnum::C => ser.write_member_named("c", &_UNION_MEMBER_C, &_Unit)?,
                TestEnum::Unknown(unknown) => ser.write_unknown(schema, unknown)?,
            }
            ser.end(schema)
        }
    }
};
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::serde::deserializers::Deserializer as _Deserializer;
    use ::smithy4rs_core::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    use ::smithy4rs_core::serde::deserializers::Error as _;
    use ::smithy4rs_core::serde::deserializers::StructReader as _StructReader;
    use ::smithy4rs_core::schema::Unit as _Unit;
    #[automatically_derived]
    impl<'de> _DeserializeWithSchema<'de> for TestEnum {
        #[inline]
        fn deserialize_with_schema<D>(
            schema: &_Schema,
            deserializer: D,
        ) -> Result<Self, D::Error>
        where
            D: _Deserializer<'de>,
        {
            let mut reader = deserializer.read_struct(schema)?;
            let mut result: Option<TestEnum> = None;
            while let Some(member_schema) = reader.read_member(schema)? {
                if result.is_some() {
                    return Err(D::Error::custom("Attempted to set union value twice"));
                }
                if member_schema == *_UNION_MEMBER_A {
                    let value: String = reader.read_value(member_schema)?;
                    result = Some(TestEnum::A(value));
                    continue;
                }
                if member_schema == *_UNION_MEMBER_B {
                    let value: i32 = reader.read_value(member_schema)?;
                    result = Some(TestEnum::B(value));
                    continue;
                }
                if member_schema == *_UNION_MEMBER_C {
                    let _: _Unit = reader.read_value(member_schema)?;
                    result = Some(TestEnum::C);
                    continue;
                }
                result = Some(TestEnum::Unknown("unknown".to_string()));
                continue;
            }
            result.ok_or(D::Error::custom("Failed to deserialize union"))
        }
    }
};
const _: () = {
    use ::smithy4rs_core::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
    impl _ErrorCorrectionDefault for TestEnum {
        #[inline]
        #[automatically_derived]
        fn default() -> Self {
            TestEnum::Unknown("".to_string())
        }
    }
};
