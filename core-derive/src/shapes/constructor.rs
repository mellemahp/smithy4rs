use proc_macro2::TokenStream;
use quote::quote;

use crate::attr::SimpleShape;

/// Adds a validated [`new`] constructor for wrapper types
pub(crate) fn expand_tuple_constructor(
    shape: &SimpleShape,
    crate_ident: &TokenStream,
) -> TokenStream {
    let name = &shape.ident;
    let schema = &shape.schema;
    let inner_type = shape.inner_type();
    quote! {
        impl #name {
            #[allow(clippy::new_without_default)]
            #[doc = concat!("Create a new [`", stringify!(#name), "`] instance")]
            #[automatically_derived]
            #[inline]
            pub fn new<T: Into<#inner_type>>(value: T) -> #crate_ident::serde::validation::Validated<#name> {
                let mut validator = #crate_ident::serde::validation::DefaultValidator::new();
                let res = #name(value.into());
                #crate_ident::serde::validation::Validator::validate(&mut validator, &#schema, &res)?;
                Ok(res)
            }
        }
    }
}
