use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// Generates `Debug` impl for Smithy Shapes.
pub(crate) fn debug_impl(shape_name: &Ident, schema_ident: &Ident) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl std::fmt::Debug for #shape_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&_DebugWrapper::new(&#schema_ident, self), f)
            }
        }
    }
}
