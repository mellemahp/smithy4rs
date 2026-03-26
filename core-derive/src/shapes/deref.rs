use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::FieldsUnnamed;

use crate::shapes::utils::parse_wrapper_type;

/// Implement deref for wrapper types
pub(crate) fn deref_impl(shape_name: &Ident, fields: &FieldsUnnamed) -> TokenStream {
    let inner_type = parse_wrapper_type(fields);
    quote! {
        use std::ops::Deref as _Deref;

        impl _Deref for #shape_name {
            type Target = #inner_type;

            #[automatically_derived]
            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    }
}
