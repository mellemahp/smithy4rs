use smithy4rs_core_derive::{SmithyEnum, smithy_enum};
#[smithy_schema(SIMPLE_ENUM)]
pub enum TestEnum {
    #[enum_value("a")]
    A,
    #[enum_value("b")]
    B,
    #[enum_value("c")]
    C,
    _Unknown(String),
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for TestEnum {
        fn schema() -> &'static _SchemaRef {
            &SIMPLE_ENUM
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    #[automatically_derived]
    impl _SerializeWithSchema for TestEnum {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_SchemaRef,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            match self {
                TestEnum::A => serializer.write_string(schema, "a"),
                TestEnum::B => serializer.write_string(schema, "b"),
                TestEnum::C => serializer.write_string(schema, "c"),
                TestEnum::_Unknown(value) => serializer.write_string(schema, value),
            }
        }
    }
};
