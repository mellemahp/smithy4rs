use smithy4rs_core_derive::{SmithyEnum, smithy_enum};
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
    _Unknown(i32),
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
            match self {
                TestIntEnum::A => serializer.write_integer(schema, 1),
                TestIntEnum::B => serializer.write_integer(schema, 2),
                TestIntEnum::C => serializer.write_integer(schema, 3),
                TestIntEnum::_Unknown(value) => serializer.write_integer(schema, value),
            }
        }
    }
};
