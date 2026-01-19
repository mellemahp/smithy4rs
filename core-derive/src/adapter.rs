use proc_macro2::{Ident, TokenStream};
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

// TODO(deser-adapter)
