use smithy4rs_core::smithy;
use smithy4rs_core_derive::{SmithyShape, smithy_enum};
pub static SIMPLE_INT_ENUM: ::smithy4rs_core::LazyLock<
    ::smithy4rs_core::schema::Schema,
> = ::smithy4rs_core::LazyLock::new(|| {
    ::smithy4rs_core::schema::Schema::create_int_enum(
        "test#SimpleIntEnum",
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
const _: () = {
    use ::smithy4rs_core::features::documents::Document as _Document;
    use ::smithy4rs_core::features::documents::DocumentError as _DocumentError;
    use ::smithy4rs_core::features::documents::DocumentType as _DocumentType;
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::schema::ShapeId as _ShapeId;
    use ::smithy4rs_core::schema::StaticSchemaShape as _StaticSchemaShape;
    impl _Document for TestIntEnum {
        #[inline]
        fn discriminant(&self) -> Option<&_ShapeId> {
            Some(<TestIntEnum as _StaticSchemaShape>::schema().id())
        }
        #[inline]
        fn document_type(&self) -> _DocumentType {
            _DocumentType::Number(::smithy4rs_core::features::documents::Number::Integer)
        }
        #[inline]
        fn as_integer(&self) -> Option<i32> {
            let out = match self {
                TestIntEnum::A => 1,
                TestIntEnum::B => 2,
                TestIntEnum::C => 3,
                TestIntEnum::Unknown(value) => *value,
            };
            Some(out)
        }
        #[inline]
        fn take_integer(&mut self, _schema: &_Schema) -> Result<i32, _DocumentError> {
            let out = match self {
                TestIntEnum::A => 1,
                TestIntEnum::B => 2,
                TestIntEnum::C => 3,
                TestIntEnum::Unknown(value) => std::mem::take(value),
            };
            Ok(out)
        }
    }
};
#[automatically_derived]
impl ::core::clone::Clone for TestIntEnum {
    #[inline]
    fn clone(&self) -> TestIntEnum {
        match self {
            TestIntEnum::A => TestIntEnum::A,
            TestIntEnum::B => TestIntEnum::B,
            TestIntEnum::C => TestIntEnum::C,
            TestIntEnum::Unknown(__self_0) => {
                TestIntEnum::Unknown(::core::clone::Clone::clone(__self_0))
            }
        }
    }
}
