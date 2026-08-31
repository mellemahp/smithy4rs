use proc_macro2::TokenStream;
use quote::quote;

use crate::attr::{EnumShape, EnumValue, Shape};

// TODO: Should we just be using "default" for these?
pub(crate) fn expand_error_correction(shape: &Shape, crate_ident: &TokenStream) -> TokenStream {
    let name = shape.name();
    let default = match shape {
        Shape::Struct(struct_shape) => {
            let builder_name = struct_shape.builder();
            quote! {
                #builder_name::new().correct()
            }
        }
        Shape::Enum(simple_shape) => {
            let filler = determine_enum_filler_value(simple_shape);
            quote! {
                #name::Unknown(#filler)
            }
        }
        Shape::Simple(_) => {
            // TODO: Add for simple types!
            quote! {
                todo!("TODO!")
            }
        }
        Shape::Union(_) => {
            quote! {
                #name::Unknown(String::new())
            }
        }
    };

    quote! {
        const _: () = {
            use #crate_ident::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
            impl _ErrorCorrectionDefault for #name {
                #[inline]
                #[automatically_derived]
                fn default() -> Self {
                    #default
                }
            }
        };
    }
}

/// Determines how to correctly match on String vs Int enums
fn determine_enum_filler_value(data: &EnumShape) -> TokenStream {
    let variant = data
        .data
        .as_enum()
        .expect("Enum variants")
        .first()
        .expect("Should have at least one variant");

    match variant.value {
        Some(EnumValue::Str(_)) => quote! { String::new() },
        Some(EnumValue::Int(_)) => quote! { 0i32 },
        _ => panic!("Unexpected enum value"),
    }
}
