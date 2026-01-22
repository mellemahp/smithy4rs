use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

// ============================================================================
// Serialization
// ============================================================================

/// Generates a serializer adapter impl
pub(crate) fn ser_adapter_impl(
    crate_ident: &TokenStream,
    shape_name: &Ident,
    schema_ident: &Ident,
) -> TokenStream {
    quote! {
        use #crate_ident::serde::serializers::Serializer as _Serializer;
        use #crate_ident::serde::adapters::SerAdapter as _SerAdapter;
        use #crate_ident::serde::serializers::SerializeWithSchema as _SerializeWithSchema;

        #[automatically_derived]
        impl _serde::Serialize for #shape_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: _serde::Serializer,
            {
                self.serialize_with_schema(&#schema_ident, _SerAdapter::new(serializer))
                    .map_err(|wrapper| wrapper.inner())
            }
        }
    }
}

// ============================================================================
// Deserialization
// ============================================================================

/// Generates a deserializer adapter impl
pub(crate) fn deser_adapter_impl(
    crate_ident: &TokenStream,
    shape_name: &Ident,
) -> TokenStream {
    let builder_name = Ident::new(&format!("{shape_name}Builder"), Span::call_site());
    quote! {
        use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;
        use #crate_ident::serde::adapters::SchemaSeed as _SchemaSeed;
        use #crate_ident::serde::ShapeBuilder as _ShapeBuilder;
        use _serde::de::Error as _SerdeDeserError;
        use _serde::de::DeserializeSeed as _DeserializeSeed;

        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for #shape_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: _serde::Deserializer<'de>,
            {
                let seed = _SchemaSeed::<#builder_name>::new(#shape_name::schema());
                seed.deserialize(deserializer)?
                    .build()
                    .map_err(D::Error::custom)
            }
        }
    }
}
