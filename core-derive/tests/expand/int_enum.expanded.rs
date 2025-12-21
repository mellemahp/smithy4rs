use smithy4rs_core_derive::{SmithyShape, smithy_enum};
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
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for TestIntEnum {
        fn schema() -> &'static _SchemaRef {
            &SIMPLE_INT_ENUM
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    #[automatically_derived]
    impl _SerializeWithSchema for TestIntEnum {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_SchemaRef,
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
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
    use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    #[automatically_derived]
    impl<'de> _DeserializeWithSchema<'de> for TestIntEnum {
        fn deserialize_with_schema<D>(
            schema: &_SchemaRef,
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
