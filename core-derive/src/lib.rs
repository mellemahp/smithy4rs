//! Provides derive macros for Smithy Shapes
//!
//! These macros are used to generate schema-guided (de)Serialization
//! implementations for generated shapes.
mod builder;
mod debug;
mod deserialization;
mod schema;
mod serialization;
mod utils;

#[cfg(feature = "serde-adapter")]
mod adapter;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, ItemEnum, Lit, Variant, parse, parse_macro_input, parse_quote};

#[cfg(feature = "serde-adapter")]
use crate::adapter::{deser_adapter_impl, ser_adapter_impl};
use crate::{
    builder::{buildable, builder_impls, builder_struct, get_builder_fields},
    debug::debug_impl,
    deserialization::deserialization_impl,
    schema::schema_impl,
    serialization::serialization_impl,
    utils::{get_builder_ident, get_crate_info, parse_enum_value, parse_schema},
};
// TODO(errors): Make error handling use: `syn::Error::into_compile_error`
// TODO(derive): Smithy Struct should automatically derive: PartialEq, and Clone
//               if not already derived on shape.

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

/// Convert discriminants to `[#enum_value]` attributes
fn discriminants_to_attributes(enum_data: &mut ItemEnum) {
    // Change all discriminants to attributes for consistency
    for variant in enum_data.variants.iter_mut() {
        if let Some((_, expr)) = &variant.discriminant {
            variant.attrs.push(parse_quote!(#[enum_value(#expr)]));
            variant.discriminant = None;
        };
    }
}

/// Adds an `Unknown` variant for Enums and Unions.
fn unknown_variant(enum_data: &mut ItemEnum) {
    // Determine if unknown should store string or int. Unions (without `enum_value` attr)
    // will default tousing String as unknown data value.
    let field = if let Some(val) = parse_enum_value(
        &enum_data
            .variants
            .first()
            .expect("Expected at least one variant")
            .attrs,
    ) && let Lit::Int(_) = val
    {
        parse_quote!((i32))
    } else {
        parse_quote!((String))
    };
    enum_data.variants.push(Variant {
        attrs: vec![
            parse_quote!(#[automatically_derived]),
            parse_quote!(#[doc(hidden)]),
        ],
        discriminant: None,
        fields: syn::Fields::Unnamed(field),
        ident: Ident::new("Unknown", Span::call_site()),
    });
}

// ============================================================================
// Derive Macros
// ============================================================================

/// Convenience derive that combines `SchemaShape`, `SerializableShape`, and `DeserializableShape`
/// for Smithy Enums, Structures, and Unions.
#[proc_macro_derive(
    SmithyShape,
    attributes(smithy_schema, enum_value, smithy_union_enum, default)
)]
pub fn smithy_shape_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Generate all three derive expansions
    let schema = schema_shape_derive(input.clone());
    let serializable = serializable_shape_derive(input.clone());
    let deserializable = deserializable_shape_derive(input.clone());

    // Adapters for serde (if enabled)
    #[cfg(feature = "serde-adapter")]
    let serde = smithy_serde_adapter(input.clone());

    // Add additional core derivations
    let debug = smithy_debug(input);

    // Combine all outputs
    let schema_tokens = TokenStream::from(schema);
    let serializable_tokens = TokenStream::from(serializable);
    let deserializable_tokens = TokenStream::from(deserializable);
    let debug_tokens = TokenStream::from(debug);

    #[cfg(feature = "serde-adapter")]
    let serde_tokens = TokenStream::from(serde);

    #[cfg(feature = "serde-adapter")]
    {
        quote! {
            #schema_tokens
            #serializable_tokens
            #deserializable_tokens
            #debug_tokens
            #serde_tokens
        }
        .into()
    }

    #[cfg(not(feature = "serde-adapter"))]
    {
        quote! {
            #schema_tokens
            #serializable_tokens
            #deserializable_tokens
            #debug_tokens
        }
        .into()
    }
}

/// Derives `SchemaShape` for a struct, backed by a static schema (`StaticSchemaShape`)
#[proc_macro_derive(SchemaShape, attributes(smithy_schema))]
pub fn schema_shape_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let schema_ident = parse_schema(&input.attrs);
    let shape_name = &input.ident;
    let (extern_import, crate_ident) = get_crate_info();
    let schema_trait = schema_impl(shape_name, &schema_ident);

    quote! {
        const _: () = {
            #extern_import
            use #crate_ident::schema::SchemaRef as _SchemaRef;
            use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;

            #schema_trait
        };
    }
    .into()
}

/// Derives `SerializableShape` (`SerializeWithSchema` only, no schema)
#[proc_macro_derive(SerializableShape, attributes(smithy_schema, enum_value))]
pub fn serializable_shape_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let shape_name = &input.ident;
    let schema_ident = parse_schema(&input.attrs);
    let (extern_import, crate_ident) = get_crate_info();
    let serialization = serialization_impl(&crate_ident, shape_name, &schema_ident, &input);
    quote! {
        const _: () = {
            #extern_import
            use #crate_ident::schema::SchemaRef as _SchemaRef;

            #serialization
        };
    }
    .into()
}

/// Derives `DeserializeWithSchema` and, implicitly `Deserialize` for a Shape.
#[proc_macro_derive(DeserializableShape, attributes(smithy_schema, default))]
pub fn deserializable_shape_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let shape_name = &input.ident;
    let schema_ident = parse_schema(&input.attrs);
    let (extern_import, crate_ident) = get_crate_info();
    let deser = deserialization_impl(&crate_ident, shape_name, &schema_ident, &input);
    match &input.data {
        // Generate builder for structures
        Data::Struct(data) => {
            let field_data = get_builder_fields(&schema_ident, data);
            let builder = builder_struct(shape_name, &field_data);
            let builder_impls = builder_impls(shape_name, &field_data);
            let builder_name = get_builder_ident(shape_name);
            let builder_serializer =
                serialization_impl(&crate_ident, &builder_name, &schema_ident, &input);
            let buildable = buildable(shape_name, &builder_name);
            // Builder struct is generated outside the const block to make it publicly accessible
            quote! {
                #builder

                const _: () = {
                    #extern_import
                    use #crate_ident::schema::SchemaRef as _SchemaRef;

                    #deser

                    #builder_impls
                    #builder_serializer
                    #buildable
                };
            }
            .into()
        }
        Data::Enum(_) => quote! {
            const _: () = {
                #extern_import
                use #crate_ident::schema::SchemaRef as _SchemaRef;

                #deser
            };
        }
        .into(),
        _ => panic!("SerializableShape can only be derived for structs, enum, or unions"),
    }
}

/// Derives `Debug` for a struct, backed by a static schema (`StaticSchemaShape`)
#[proc_macro_derive(SmithyDebug)]
pub fn smithy_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let shape_name = &input.ident;
    let schema_ident = parse_schema(&input.attrs);
    let (extern_import, crate_ident) = get_crate_info();
    let debug = debug_impl(shape_name, &schema_ident);

    quote! {
        const _: () = {
            #extern_import

            use #crate_ident::serde::debug::DebugWrapper as _DebugWrapper;

            #debug
        };
    }
    .into()
}

/// Derives `serde` adapter implementations for a Smithy shape.
#[cfg(feature = "serde-adapter")]
#[proc_macro_derive(SmithySerdeAdapter)]
pub fn smithy_serde_adapter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let shape_name = &input.ident;
    let schema_ident = parse_schema(&input.attrs);
    let (extern_import, crate_ident) = get_crate_info();
    let ser = ser_adapter_impl(&crate_ident, shape_name, &schema_ident);
    let deser = deser_adapter_impl(&crate_ident, shape_name, &input);

    quote! {
        const _: () = {
            #extern_import
            extern crate serde as _serde;

            #ser
            #deser
        };
    }
    .into()
}
