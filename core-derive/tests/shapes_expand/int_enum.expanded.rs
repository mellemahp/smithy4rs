use smithy4rs_core::smithy;
use smithy4rs_core_derive::{SmithyShape, smithy_enum};
pub static SIMPLE_INT_ENUM: ::smithy4rs_core::LazyLock<
    ::smithy4rs_core::schema::Schema,
> = ::smithy4rs_core::LazyLock::new(|| {
    ::smithy4rs_core::schema::Schema::create_int_enum(
        "test#SimpleStruct",
        Box::new([1, 2, 3]),
        Vec::new(),
    )
});
#[schema(schema = SIMPLE_INT_ENUM)]
pub enum TestIntEnum {
    #[schema(value = 1)]
    A,
    #[schema(value = 2)]
    B,
    #[schema(value = 3)]
    C,
    #[automatically_derived]
    #[doc(hidden)]
    Unknown(i32),
}
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for TestIntEnum {
        #[inline]
        fn schema() -> &'static _Schema {
            &SIMPLE_INT_ENUM
        }
    }
};
const _: () = {
    use ::smithy4rs_core::serde::debug::DebugWrapper as _DebugWrapper;
    #[automatically_derived]
    impl std::fmt::Debug for TestIntEnum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&_DebugWrapper::new(&SIMPLE_INT_ENUM, self), f)
        }
    }
};
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::serde::serializers::Serializer as _Serializer;
    use ::smithy4rs_core::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
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
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::serde::deserializers::Deserializer as _Deserializer;
    use ::smithy4rs_core::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    #[automatically_derived]
    impl<'de> _DeserializeWithSchema<'de> for TestIntEnum {
        #[inline]
        fn deserialize_with_schema<D>(
            schema: &_Schema,
            deserializer: D,
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
    use ::smithy4rs_core::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
    impl _ErrorCorrectionDefault for TestIntEnum {
        #[inline]
        #[automatically_derived]
        fn default() -> Self {
            TestIntEnum::Unknown(0i32)
        }
    }
};
