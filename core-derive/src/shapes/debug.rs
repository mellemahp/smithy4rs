use proc_macro2::TokenStream;
use quote::quote;

use crate::attr::Shape;

/// Generates `Debug` impl for Smithy Shapes.
pub(crate) fn expand_debug(shape: &Shape, crate_ident: &TokenStream) -> TokenStream {
    let name = shape.name();
    let schema = shape.schema();

    quote! {
        const _: () = {
            use #crate_ident::serde::debug::DebugWrapper as _DebugWrapper;

            #[automatically_derived]
            impl std::fmt::Debug for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Debug::fmt(&_DebugWrapper::new(&#schema, self), f)
                }
            }
        };
    }
}
