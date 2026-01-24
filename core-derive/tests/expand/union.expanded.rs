use smithy4rs_core::prelude::{INTEGER, STRING, UNIT};
use smithy4rs_core_derive::{SmithyShape, smithy_union};
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
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for TestEnum {
        fn schema() -> &'static _Schema {
            &UNION
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::StructSerializer as _StructSerializer;
    use _smithy4rs::schema::Unit as _Unit;
    #[automatically_derived]
    impl _SerializeWithSchema for TestEnum {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_Schema,
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
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
    use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    use _smithy4rs::serde::deserializers::Error as _DeserializerError;
    use _smithy4rs::schema::Unit as _Unit;
    #[automatically_derived]
    impl<'de> _DeserializeWithSchema<'de> for TestEnum {
        fn deserialize_with_schema<D>(
            schema: &_Schema,
            deserializer: &mut D,
        ) -> Result<Self, D::Error>
        where
            D: _Deserializer<'de>,
        {
            deserializer
                .read_struct(
                    schema,
                    None,
                    |option, member_schema, de| {
                        if option.is_some() {
                            return Err(
                                D::Error::custom("Attempted to set union value twice"),
                            );
                        }
                        if &member_schema == &*_UNION_MEMBER_A {
                            let value = String::deserialize_with_schema(
                                member_schema,
                                de,
                            )?;
                            return Ok(Some(TestEnum::A(value)));
                        }
                        if &member_schema == &*_UNION_MEMBER_B {
                            let value = i32::deserialize_with_schema(member_schema, de)?;
                            return Ok(Some(TestEnum::B(value)));
                        }
                        if &member_schema == &*_UNION_MEMBER_C {
                            let _ = _Unit::deserialize_with_schema(member_schema, de)?;
                            return Ok(Some(TestEnum::C));
                        }
                        Ok(Some(TestEnum::Unknown("unknown".to_string())))
                    },
                )?
                .ok_or(D::Error::custom("Failed to deserialize union"))
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::serde::debug::DebugWrapper as _DebugWrapper;
    #[automatically_derived]
    impl std::fmt::Debug for TestEnum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&_DebugWrapper::new(&UNION, self), f)
        }
    }
};
