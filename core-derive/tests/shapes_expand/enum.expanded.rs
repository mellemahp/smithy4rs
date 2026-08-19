use smithy4rs_core::smithy;
use smithy4rs_core_derive::{SmithyShape, smithy_enum};
pub static SIMPLE_ENUM: ::smithy4rs_core::LazyLock<::smithy4rs_core::schema::Schema> = ::smithy4rs_core::LazyLock::new(||
{
    ::smithy4rs_core::schema::Schema::create_enum(
        "test#SimpleStruct",
        Box::new(["a", "b", "c"]),
        Vec::new(),
    )
});
#[schema(schema = SIMPLE_ENUM)]
pub enum TestEnum {
    #[enum_value(value = "a")]
    A,
    #[enum_value(value = "b")]
    B,
    #[enum_value(value = "c")]
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
            &SIMPLE_ENUM
        }
    }
};
const _: () = {
    use ::smithy4rs_core::serde::debug::DebugWrapper as _DebugWrapper;
    #[automatically_derived]
    impl std::fmt::Debug for TestEnum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&_DebugWrapper::new(&SIMPLE_ENUM, self), f)
        }
    }
};
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::serde::serializers::Serializer as _Serializer;
    use ::smithy4rs_core::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
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
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::serde::deserializers::Deserializer as _Deserializer;
    use ::smithy4rs_core::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
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
    use ::smithy4rs_core::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
    impl _ErrorCorrectionDefault for TestEnum {
        #[inline]
        #[automatically_derived]
        fn default() -> Self {
            TestEnum::Unknown("".to_string())
        }
    }
};
