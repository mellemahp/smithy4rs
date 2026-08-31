//! Provides derive macros for Smithy Shapes
//!
//! These macros are used to generate schema-guided (de)Serialization
//! implementations for generated shapes.

mod attr;
mod r#enum;
mod features;
mod shapes;
mod traits;
mod utils;

use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemEnum, parse, parse_macro_input, parse_quote};

#[cfg(feature = "arbitrary")]
use crate::features::expand_arbitrary;
#[cfg(feature = "serde-adapter")]
use crate::features::expand_serde_adapter;
use crate::{
    attr::Shape,
    r#enum::{discriminants_to_attributes, unknown_variant},
    shapes::{
        expand_builder, expand_debug, expand_deref, expand_deserialize_with_schema,
        expand_error_correction, expand_schema, expand_serialize_from_schema,
        expand_tuple_constructor,
    },
    traits::{expand_trait_id, expand_try_from_document},
};

// ============================================================================
// Attribute Macros
// ============================================================================

/// Modifies an enum to be usable as a Smithy Union
///
/// This macro is used to automatically add an unknown variant for Union shapes.
#[proc_macro_attribute]
pub fn smithy_union(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut enum_struct = parse_macro_input!(input as ItemEnum);
    // Expect NO args
    let _ = parse_macro_input!(args as parse::Nothing);

    // Add a marker attribute to help us differentiate unions from regular enums
    // NOTE: We cannot use field presence as an indicator b/c unions may have
    // empty (i.e. `UNIT`) values
    enum_struct.attrs.push(parse_quote!(#[smithy_union_enum]));

    // Add unknown variants
    unknown_variant(&mut enum_struct);

    // Re-write structure with changes
    quote!(#enum_struct).into()
}

/// Modifies an enum to be usable as a Smithy enum
///
/// This macro is used to automatically add an unknown variant for Smithy Enums.
/// It also allows us to use discriminants for both string and int enum definitions.
#[proc_macro_attribute]
pub fn smithy_enum(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut enum_struct = parse_macro_input!(input as ItemEnum);
    // Expect NO args
    let _ = parse_macro_input!(args as parse::Nothing);
    // process all discriminants.
    // *WARNING*: This must occur _BEFORE_ adding unknown variant
    discriminants_to_attributes(&mut enum_struct);
    // Add unknown variants
    unknown_variant(&mut enum_struct);

    // Re-write structure with changes
    quote!(#enum_struct).into()
}

// ============================================================================
// Derive Macros
// ============================================================================

/// Convenience derive that generates full Smithy Enums, Structures, and Unions,
/// and Simple types
#[proc_macro_derive(SmithyShape, attributes(schema, enum_value, smithy_union_enum))]
pub fn smithy_shape_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let parsed = match Shape::from_derive_input(&derive_input) {
        Ok(val) => val,
        Err(err) => return err.write_errors().into(),
    };
    let crate_ident = utils::get_crate_ident();

    // TODO: Incorporate error handling into expansions
    #[allow(unused)]
    let mut errors: Vec<darling::Error> = Vec::new();

    // // ==== Core Common ===
    let mut tokens: Vec<TokenStream> = vec![
        expand_schema(&parsed, &crate_ident),
        expand_debug(&parsed, &crate_ident),
        expand_serialize_from_schema(&parsed, &crate_ident),
        expand_deserialize_with_schema(&parsed, &crate_ident),
        expand_error_correction(&parsed, &crate_ident),
    ];

    // ==== Shape Specific ====
    match &parsed {
        Shape::Simple(simple_shape) => {
            tokens.push(expand_deref(simple_shape));
            tokens.push(expand_tuple_constructor(simple_shape, &crate_ident));
        }
        Shape::Struct(struct_shape) => {
            tokens.push(expand_builder(struct_shape, &crate_ident));
        }
        Shape::Union(_) | Shape::Enum(_) => {
            // TODO: Any enum specific?
        }
    }

    // ==== Optional features ====
    #[cfg(feature = "serde-adapter")]
    tokens.push(expand_serde_adapter(&parsed, &crate_ident));

    #[cfg(feature = "arbitrary")]
    tokens.push(expand_arbitrary(&parsed, &crate_ident));

    // We write tokens still b/c we expect each impl to emit a dummy implementation on errors.
    if !errors.is_empty() {
        let error_tokens = darling::Error::multiple(errors).write_errors();
        return quote! {
            #(#tokens)*
            #error_tokens
        }
        .into();
    }

    quote! { #(#tokens)* }.into()
}

// ============================================================================
// Smithy Trait Implementations
// ============================================================================

/// Derives Smithy trait specific implementations
///
/// NOTE: Expects shape to already have `SmithyShape` applied.
#[proc_macro_derive(SmithyTrait, attributes(smithy_schema))]
pub fn smithy_trait_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let parsed = match Shape::from_derive_input(&derive_input) {
        Ok(val) => val,
        Err(err) => return err.write_errors().into(),
    };
    let crate_ident = utils::get_crate_ident();

    let mut tokens: Vec<TokenStream> = Vec::new();

    tokens.push(expand_trait_id(&parsed, &crate_ident));
    tokens.push(expand_try_from_document(&parsed, &crate_ident));

    quote! { #(#tokens)* }.into()
}
