use proc_macro2::TokenStream;
use quote::quote;

use crate::attr::SimpleShape;

/// Implement deref for wrapper types
pub(crate) fn expand_deref(shape: &SimpleShape) -> TokenStream {
    let name = &shape.ident;
    let inner_type = shape.inner_type();

    quote! {
        const _: () = {
            use std::ops::Deref as _Deref;

            impl _Deref for #name {
                type Target = #inner_type;

                #[automatically_derived]
                #[inline]
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        };
    }
}
