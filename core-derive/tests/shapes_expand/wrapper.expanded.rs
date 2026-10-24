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
    impl ::smithy4rs_core::features::documents::Document for SimpleTrait {
        #[inline]
        fn discriminant(&self) -> Option<&::smithy4rs_core::schema::ShapeId> {
            Some(<Self as ::smithy4rs_core::schema::StaticSchemaShape>::schema().id())
        }
        #[inline]
        fn document_type(&self) -> ::smithy4rs_core::features::documents::DocumentType {
            self.0.document_type()
        }
        #[inline]
        fn as_boolean(&self) -> Option<bool> {
            self.0.as_boolean()
        }
        #[inline]
        fn as_blob(&self) -> Option<&::smithy4rs_core::ByteBuffer> {
            self.0.as_blob()
        }
        #[inline]
        fn as_string(&self) -> Option<&str> {
            self.0.as_string()
        }
        #[inline]
        fn as_timestamp(&self) -> Option<::smithy4rs_core::Instant> {
            self.0.as_timestamp()
        }
        #[inline]
        fn as_byte(&self) -> Option<i8> {
            self.0.as_byte()
        }
        #[inline]
        fn as_short(&self) -> Option<i16> {
            self.0.as_short()
        }
        #[inline]
        fn as_integer(&self) -> Option<i32> {
            self.0.as_integer()
        }
        #[inline]
        fn as_long(&self) -> Option<i64> {
            self.0.as_long()
        }
        #[inline]
        fn as_big_integer(
            &self,
        ) -> Option<std::borrow::Cow<'_, ::smithy4rs_core::BigInt>> {
            self.0.as_big_integer()
        }
        #[inline]
        fn as_float(&self) -> Option<f32> {
            self.0.as_float()
        }
        #[inline]
        fn as_double(&self) -> Option<f64> {
            self.0.as_double()
        }
        #[inline]
        fn as_big_decimal(
            &self,
        ) -> Option<std::borrow::Cow<'_, ::smithy4rs_core::BigDecimal>> {
            self.0.as_big_decimal()
        }
        #[inline]
        fn as_list(
            &self,
        ) -> Option<&dyn ::smithy4rs_core::features::documents::ListAccessor> {
            self.0.as_list()
        }
        #[inline]
        fn as_map(
            &self,
        ) -> Option<&dyn ::smithy4rs_core::features::documents::MapAccessor> {
            self.0.as_map()
        }
        #[inline]
        fn take_bool(
            &mut self,
            schema: &::smithy4rs_core::schema::Schema,
        ) -> Result<bool, ::smithy4rs_core::features::documents::DocumentError> {
            self.0.take_bool(schema)
        }
        #[inline]
        fn take_blob(
            &mut self,
            schema: &::smithy4rs_core::schema::Schema,
        ) -> Result<
            ::smithy4rs_core::ByteBuffer,
            ::smithy4rs_core::features::documents::DocumentError,
        > {
            self.0.take_blob(schema)
        }
        #[inline]
        fn take_string(
            &mut self,
            schema: &::smithy4rs_core::schema::Schema,
        ) -> Result<String, ::smithy4rs_core::features::documents::DocumentError> {
            self.0.take_string(schema)
        }
        #[inline]
        fn take_byte(
            &mut self,
            schema: &::smithy4rs_core::schema::Schema,
        ) -> Result<i8, ::smithy4rs_core::features::documents::DocumentError> {
            self.0.take_byte(schema)
        }
        #[inline]
        fn take_short(
            &mut self,
            schema: &::smithy4rs_core::schema::Schema,
        ) -> Result<i16, ::smithy4rs_core::features::documents::DocumentError> {
            self.0.take_short(schema)
        }
        #[inline]
        fn take_integer(
            &mut self,
            schema: &::smithy4rs_core::schema::Schema,
        ) -> Result<i32, ::smithy4rs_core::features::documents::DocumentError> {
            self.0.take_integer(schema)
        }
        #[inline]
        fn take_long(
            &mut self,
            schema: &::smithy4rs_core::schema::Schema,
        ) -> Result<i64, ::smithy4rs_core::features::documents::DocumentError> {
            self.0.take_long(schema)
        }
        #[inline]
        fn take_big_integer(
            &mut self,
            schema: &::smithy4rs_core::schema::Schema,
        ) -> Result<
            ::smithy4rs_core::BigInt,
            ::smithy4rs_core::features::documents::DocumentError,
        > {
            self.0.take_big_integer(schema)
        }
        #[inline]
        fn take_float(
            &mut self,
            schema: &::smithy4rs_core::schema::Schema,
        ) -> Result<f32, ::smithy4rs_core::features::documents::DocumentError> {
            self.0.take_float(schema)
        }
        #[inline]
        fn take_double(
            &mut self,
            schema: &::smithy4rs_core::schema::Schema,
        ) -> Result<f64, ::smithy4rs_core::features::documents::DocumentError> {
            self.0.take_double(schema)
        }
        #[inline]
        fn take_big_decimal(
            &mut self,
            schema: &::smithy4rs_core::schema::Schema,
        ) -> Result<
            ::smithy4rs_core::BigDecimal,
            ::smithy4rs_core::features::documents::DocumentError,
        > {
            self.0.take_big_decimal(schema)
        }
        #[inline]
        fn take_list(
            &mut self,
        ) -> Result<
            &mut dyn ::smithy4rs_core::features::documents::ListAccessor,
            ::smithy4rs_core::features::documents::DocumentError,
        > {
            self.0.take_list()
        }
        #[inline]
        fn take_map(
            &mut self,
        ) -> Result<
            &mut dyn ::smithy4rs_core::features::documents::MapAccessor,
            ::smithy4rs_core::features::documents::DocumentError,
        > {
            self.0.take_map()
        }
    }
};
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
