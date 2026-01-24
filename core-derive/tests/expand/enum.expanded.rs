use smithy4rs_core_derive::{SmithyShape, smithy_enum};
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
