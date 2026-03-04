use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::FieldsUnnamed;

use crate::shapes::utils::{get_crate_name, parse_wrapper_type};

// ============================================================================
// Wrapper (Newtype) constructor
// ============================================================================

pub(crate) fn get_tuple_constructor(
    schema_ident: &Ident,
    shape_name: &Ident,
    fields: &FieldsUnnamed,
) -> TokenStream {
    let inner_type = parse_wrapper_type(fields);
    let crate_name = get_crate_name();
    quote! {
        impl #shape_name {
            #[doc = "Create a new [`"]
            #[doc = stringify!(#shape_name)]
            #[doc = "`] instance"]
            #[automatically_derived]
            #[inline]
            pub fn new<T: Into<#inner_type>>(value: T) -> #crate_name::serde::validation::Validated<#shape_name> {
                let mut validator = #crate_name::serde::validation::DefaultValidator::new();
                let res = #shape_name(value.into());
                #crate_name::serde::validation::Validator::validate(&mut validator, &#schema_ident, &res)?;
                Ok(res)
            }
        }
    }
}
