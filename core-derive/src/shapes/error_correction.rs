use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{DataEnum, Lit};

use crate::shapes::utils::parse_enum_value;

pub(crate) fn enum_error_correction_impl(
    crate_ident: &TokenStream,
    shape_name: &Ident,
    data: &DataEnum,
) -> TokenStream {
    let filler = determine_enum_filler_value(data);
    quote! {
        use #crate_ident::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;

        impl _ErrorCorrectionDefault for #shape_name {
            #[inline]
            #[automatically_derived]
            fn default() -> Self {
                #shape_name::Unknown(#filler)
            }
        }
    }
}

/// Determines how to correctly match on value
fn determine_enum_filler_value(data: &DataEnum) -> TokenStream {
    let first_var = data
        .variants
        .first()
        .expect("At least one enum variant expected");
    if matches!(parse_enum_value(&first_var.attrs), Some(Lit::Int(_))) {
        quote! { 0i32 }
    } else {
        quote! { "".to_string() }
    }
}

