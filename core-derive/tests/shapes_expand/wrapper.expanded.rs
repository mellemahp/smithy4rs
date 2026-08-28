use smithy4rs_core::smithy;
use smithy4rs_core_derive::{SmithyShape, SmithyTrait};
pub static STRING_TRAIT: ::smithy4rs_core::LazyLock<::smithy4rs_core::schema::Schema> = ::smithy4rs_core::LazyLock::new(||
{ ::smithy4rs_core::schema::Schema::create_string("test#StringTrait", Vec::new()) });
#[schema(schema = STRING_TRAIT)]
pub struct SimpleTrait(String);
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for SimpleTrait {
        #[inline]
        fn schema() -> &'static _Schema {
            &STRING_TRAIT
        }
    }
};
const _: () = {
    use ::smithy4rs_core::serde::debug::DebugWrapper as _DebugWrapper;
    #[automatically_derived]
    impl std::fmt::Debug for SimpleTrait {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&_DebugWrapper::new(&STRING_TRAIT, self), f)
        }
    }
};
const _: () = {
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::serde::serializers::Serializer as _Serializer;
    use ::smithy4rs_core::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
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
    use ::smithy4rs_core::schema::Schema as _Schema;
    use ::smithy4rs_core::serde::deserializers::Deserializer as _Deserializer;
    use ::smithy4rs_core::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
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
            Ok(SimpleTrait(inner))
        }
    }
};
const _: () = {
    use ::smithy4rs_core::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
    impl _ErrorCorrectionDefault for SimpleTrait {
        #[inline]
        #[automatically_derived]
        fn default() -> Self {
            {
                ::core::panicking::panic_fmt(
                    format_args!("not yet implemented: {0}", format_args!("TODO!")),
                );
            }
        }
    }
};
const _: () = {
    use std::ops::Deref as _Deref;
    impl _Deref for SimpleTrait {
        type Target = String;
        #[automatically_derived]
        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
};
impl SimpleTrait {
    #[allow(clippy::new_without_default)]
    ///Create a new [`SimpleTrait`] instance
    #[automatically_derived]
    #[inline]
    pub fn new<T: Into<String>>(
        value: T,
    ) -> ::smithy4rs_core::serde::validation::Validated<SimpleTrait> {
        let mut validator = ::smithy4rs_core::serde::validation::DefaultValidator::new();
        let res = SimpleTrait(value.into());
        ::smithy4rs_core::serde::validation::Validator::validate(
            &mut validator,
            &STRING_TRAIT,
            &res,
        )?;
        Ok(res)
    }
}
const _: () = {
    use ::smithy4rs_core::schema::StaticTraitId as _StaticTraitId;
    use ::smithy4rs_core::schema::ShapeId as _ShapeId;
    use ::smithy4rs_core::LazyLock as _LazyLock;
    use ::smithy4rs_core::schema::StaticSchemaShape as _StaticSchemaShape;
    impl _StaticTraitId for SimpleTrait {
        #[inline]
        #[automatically_derived]
        fn trait_id() -> &'static _ShapeId {
            static ID: _LazyLock<&_ShapeId> = _LazyLock::new(|| {
                &<SimpleTrait as _StaticSchemaShape>::schema().id()
            });
            *ID
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
