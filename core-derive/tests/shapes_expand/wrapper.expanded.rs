use smithy4rs_core::smithy;
use smithy4rs_core_derive::SmithyShape;
pub static STRING_TRAIT: ::smithy4rs_core::LazyLock<::smithy4rs_core::schema::Schema> = ::smithy4rs_core::LazyLock::new(||
{ ::smithy4rs_core::schema::Schema::create_string("test#SimpleTrait", Vec::new()) });
#[smithy_schema(STRING_TRAIT)]
pub struct SimpleTrait(String);
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for SimpleTrait {
        fn schema() -> &'static _Schema {
            &STRING_TRAIT
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::StructWriter as _StructWriter;
    #[automatically_derived]
    impl _SerializeWithSchema for SimpleTrait {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_Schema,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            self.0.serialize_with_schema(schema, serializer)
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
    use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    #[automatically_derived]
    impl<'de> _DeserializeWithSchema<'de> for SimpleTrait {
        #[inline]
        fn deserialize_with_schema<D>(
            schema: &_Schema,
            deserializer: D,
        ) -> Result<Self, D::Error>
        where
            D: _Deserializer<'de>,
        {
            let inner = <String as _DeserializeWithSchema>::deserialize_with_schema(
                schema,
                deserializer,
            )?;
            Ok(Self(inner))
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::serde::debug::DebugWrapper as _DebugWrapper;
    #[automatically_derived]
    impl std::fmt::Debug for SimpleTrait {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&_DebugWrapper::new(&STRING_TRAIT, self), f)
        }
    }
};
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for SimpleTrait {}
#[automatically_derived]
impl ::core::cmp::PartialEq for SimpleTrait {
    #[inline]
    fn eq(&self, other: &SimpleTrait) -> bool {
        self.0 == other.0
    }
}
#[automatically_derived]
impl ::core::clone::Clone for SimpleTrait {
    #[inline]
    fn clone(&self) -> SimpleTrait {
        SimpleTrait(::core::clone::Clone::clone(&self.0))
    }
}
