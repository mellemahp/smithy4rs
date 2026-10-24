use smithy4rs_core::{
    prelude::{INTEGER, STRING},
    schema::UNIT, smithy,
};
use smithy4rs_core_derive::{SmithyShape, smithy_union};
#[doc(hidden)]
pub static UNION_BUILDER: ::smithy4rs_core::LazyLock<
    ::smithy4rs_core::Ref<::smithy4rs_core::schema::SchemaBuilder>,
> = ::smithy4rs_core::LazyLock::new(|| ::smithy4rs_core::Ref::new(
    ::smithy4rs_core::schema::Schema::union_builder("test#Union", Vec::new()),
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
            TestEnum::Unknown(String::new())
        }
    }
};
const _: () = {
    use ::smithy4rs_core::features::documents::MapAccessor as _MapAccessor;
    use ::smithy4rs_core::features::documents::Document as _Document;
    use ::smithy4rs_core::features::documents::DocumentError as _DocumentError;
    use ::smithy4rs_core::schema::ShapeId as _ShapeId;
    use ::smithy4rs_core::features::documents::DocumentType as _DocumentType;
    use ::smithy4rs_core::schema::StaticSchemaShape as _StaticSchemaShape;
    impl _Document for TestEnum {
        #[inline]
        fn discriminant(&self) -> Option<&_ShapeId> {
            Some(<Self as _StaticSchemaShape>::schema().id())
        }
        #[inline]
        fn document_type(&self) -> _DocumentType {
            _DocumentType::Map
        }
        #[inline]
        fn as_map(&self) -> Option<&dyn _MapAccessor> {
            Some(self)
        }
        #[inline]
        fn take_map(&mut self) -> Result<&mut dyn _MapAccessor, _DocumentError> {
            Ok(self)
        }
    }
};
const _: () = {
    use ::smithy4rs_core::features::documents::MapAccessor as _MapAccessor;
    use ::smithy4rs_core::features::documents::Document as _Document;
    use ::smithy4rs_core::features::documents::DocumentError as _DocumentError;
    use ::smithy4rs_core::Cow as _Cow;
    impl _MapAccessor for TestEnum {
        #[inline]
        #[allow(trivial_casts)]
        fn get(&self, key: &str) -> Option<&dyn _Document> {
            match self {
                TestEnum::A(value) if key == "a" => Some(value),
                TestEnum::B(value) if key == "b" => Some(value),
                TestEnum::C if key == "c" => Some(&::smithy4rs_core::schema::STATIC_UNIT),
                _ => None,
            }
        }
        #[inline]
        fn get_entry(&self, idx: usize) -> Option<(&str, &dyn _Document)> {
            match self {
                TestEnum::A(value) if idx == 0usize => Some(("a", value)),
                TestEnum::B(value) if idx == 1usize => Some(("b", value)),
                TestEnum::C if idx == 2usize => {
                    Some(("c", &::smithy4rs_core::schema::STATIC_UNIT))
                }
                _ => None,
            }
        }
        #[inline]
        fn get_entry_mut(
            &mut self,
            index: usize,
        ) -> Result<(&str, &mut dyn _Document), _DocumentError> {
            match self {
                TestEnum::A(value) if index == 0usize => Ok(("a", value)),
                TestEnum::B(value) if index == 1usize => Ok(("b", value)),
                TestEnum::C if index == 2usize => {
                    Ok(("c", Box::leak(Box::new(::smithy4rs_core::schema::Unit))))
                }
                _ => Err(_DocumentError::CustomError("Invalid index".to_string())),
            }
        }
        #[inline]
        #[allow(trivial_casts)]
        fn get_key_mut(&mut self, index: usize) -> Option<_Cow<'_, str>> {
            match self {
                TestEnum::A(_) if index == 0usize => {
                    Some(::smithy4rs_core::Cow::Borrowed("a"))
                }
                TestEnum::B(_) if index == 1usize => {
                    Some(::smithy4rs_core::Cow::Borrowed("b"))
                }
                TestEnum::C if index == 2usize => {
                    Some(::smithy4rs_core::Cow::Borrowed("c"))
                }
                _ => None,
            }
        }
        #[inline]
        fn len(&self) -> usize {
            3usize
        }
    }
};
#[automatically_derived]
impl ::core::clone::Clone for TestEnum {
    #[inline]
    fn clone(&self) -> TestEnum {
        match self {
            TestEnum::A(__self_0) => TestEnum::A(::core::clone::Clone::clone(__self_0)),
            TestEnum::B(__self_0) => TestEnum::B(::core::clone::Clone::clone(__self_0)),
            TestEnum::C => TestEnum::C,
            TestEnum::Unknown(__self_0) => {
                TestEnum::Unknown(::core::clone::Clone::clone(__self_0))
            }
        }
    }
}
