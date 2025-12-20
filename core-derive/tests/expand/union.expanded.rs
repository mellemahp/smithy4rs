use smithy4rs_core::prelude::{INTEGER, STRING, UNIT};
use smithy4rs_core_derive::{Dummy, smithy_union};
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
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for TestEnum {
        fn schema() -> &'static _SchemaRef {
            &UNION
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::SchemaRef as _SchemaRef;
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::StructSerializer as _StructSerializer;
    use _smithy4rs::serde::unit::Unit as _Unit;
    #[automatically_derived]
    impl _SerializeWithSchema for TestEnum {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_SchemaRef,
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
                TestEnum::C => ser.serialize_member_named("c", &_UNION_MEMBER_C, &_Unit)?,
                TestEnum::Unknown(unknown) => ser.serialize_unknown(schema, unknown)?,
            }
            ser.end(schema)
        }
    }
};
