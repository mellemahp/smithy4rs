//! Provides derive macros for Smithy Shapes
//!
//! These macros are used to generate schema-guided (de)Serialization
//! implementations for generated shapes.

mod shapes;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use shapes::utils::{get_builder_ident, get_crate_info, parse_enum_value, parse_schema};
use syn::{
    Data, DeriveInput, Fields, ItemEnum, Lit, Variant, parse, parse_macro_input, parse_quote,
};

#[cfg(feature = "serde-adapter")]
use crate::shapes::adapter::{deser_adapter_impl, ser_adapter_impl};
#[cfg(feature = "arbitrary")]
use crate::shapes::arbitrary::arbitrary_impl;
use crate::shapes::{
    buildable, builder_impls, builder_struct, debug_impl, deref_impl, deserialization_impl,
    enum_error_correction_impl, get_builder_fields, get_static_trait_id_impl,
    get_try_from_document_impl, get_tuple_constructor, schema_impl, serialization_impl,
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
        fields: Fields::Unnamed(field),
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
    attributes(smithy_schema, enum_value, smithy_union_enum, default, no_builder)
)]
pub fn smithy_shape_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Generate all three derive expansions
    let schema = schema_shape_derive(input.clone());
    let serializable = serializable_shape_derive(input.clone());
    let deserializable = deserializable_shape_derive(input.clone());

    // Adapters for serde (if enabled)
    #[cfg(feature = "serde-adapter")]
    let serde = smithy_serde_adapter(input.clone());

    #[cfg(feature = "arbitrary")]
    let arbitrary = smithy_arbitrary(input.clone());

    // Add additional core derivations
    let debug = smithy_debug(input);

    // Combine all outputs
    let schema_tokens = TokenStream::from(schema);
    let serializable_tokens = TokenStream::from(serializable);
    let deserializable_tokens = TokenStream::from(deserializable);
    let debug_tokens = TokenStream::from(debug);

    #[cfg(any(feature = "arbitrary", feature = "serde-adapter"))]
    let mut output = quote! {
        #schema_tokens
        #serializable_tokens
        #deserializable_tokens
        #debug_tokens
    };
    #[cfg(not(any(feature = "arbitrary", feature = "serde-adapter")))]
    let output = quote! {
        #schema_tokens
        #serializable_tokens
        #deserializable_tokens
        #debug_tokens
    };

    #[cfg(feature = "serde-adapter")]
    {
        let serde_tokens = TokenStream::from(serde);
        output.extend(serde_tokens);
    }

    #[cfg(feature = "arbitrary")]
    {
        let arbitrary_tokens = TokenStream::from(arbitrary);
        output.extend(arbitrary_tokens);
    }

    output.into()
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
            use #crate_ident::schema::Schema as _Schema;
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
            use #crate_ident::schema::Schema as _Schema;

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
        Data::Struct(data) => {
            match &data.fields {
                // Generate builder for structures with named fields
                Fields::Named(fields) => {
                    let field_data = get_builder_fields(&schema_ident, fields);
                    let builder = builder_struct(shape_name, &field_data);
                    let builder_impls = builder_impls(shape_name, &field_data);
                    let builder_name = get_builder_ident(shape_name);
                    let builder_serializer =
                        serialization_impl(&crate_ident, &builder_name, &schema_ident, &input);
                    let buildable = buildable(shape_name, &builder_name);
                    let builder_schema = schema_impl(&builder_name, &schema_ident);
                    // Builder struct is generated outside the const block to make it publicly accessible
                    quote! {
                        #builder

                        const _: () = {
                            #extern_import
                            use #crate_ident::schema::Schema as _Schema;
                            use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;

                            #builder_schema
                        };

                        const _: () = {
                            #extern_import
                            use #crate_ident::schema::Schema as _Schema;

                            #deser

                            #builder_impls
                            #builder_serializer
                            #buildable
                        };
                    }
                    .into()
                }
                // Generate deser for wrappers
                Fields::Unnamed(_) | Fields::Unit => quote! {
                    const _: () = {
                        #extern_import
                        use #crate_ident::schema::Schema as _Schema;

                        #deser
                    };
                }
                .into(),
            }
        }
        Data::Enum(data) => {
            let error_correction = enum_error_correction_impl(&crate_ident, shape_name, data);
            quote! {
                const _: () = {
                    #extern_import
                    use #crate_ident::schema::Schema as _Schema;

                    #deser
                };

                const _: () = {
                    #extern_import

                    #error_correction
                };
            }.into()
        }
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

// ============================================================================
// Serde Adapter
// ============================================================================

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

// ============================================================================
// Serde Adapter
// ============================================================================

/// Derives `serde` adapter implementations for a Smithy shape.
#[cfg(feature = "arbitrary")]
#[proc_macro_derive(SmithyArbitrary, attributes(smithy_schema))]
pub fn smithy_arbitrary(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let shape_name = &input.ident;
    let schema_ident = parse_schema(&input.attrs);
    let (extern_import, crate_ident) = get_crate_info();
    let arbitrary_tokens = arbitrary_impl(&crate_ident, shape_name, &schema_ident, &input);

    quote! {
        const _: () = {
            #extern_import
            extern crate arbitrary as _arbitrary;

            #arbitrary_tokens
        };
    }
    .into()
}

// ============================================================================
// Smithy Trait Implementations
// ============================================================================

#[proc_macro_derive(SmithyTraitImpl, attributes(smithy_schema))]
pub fn smithy_trait_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let shape_name = &input.ident;
    let schema_ident = parse_schema(&input.attrs);
    let (extern_import, crate_ident) = get_crate_info();
    let static_id = get_static_trait_id_impl(&crate_ident, shape_name);
    // SmithyTrait implementations
    let contents = quote! {
        const _: () = {
            #extern_import

            #static_id
        };
    };
    // Generate a constructor for wrapper types
    if let Data::Struct(data_struct) = input.data {
        match &data_struct.fields {
            Fields::Unnamed(fields) => {
                let constructor = get_tuple_constructor(&schema_ident, shape_name, fields);
                let deref = deref_impl(shape_name, fields);
                // TODO: Re-evaluate partialEq location
                quote! {
                    #constructor

                    const _: () = {
                        #deref
                    };

                    #contents

                    impl PartialEq for #shape_name {
                        fn eq(&self, other: &Self) -> bool {
                            &self.0 == &other.0
                        }
                    }
                }
                .into()
            }
            Fields::Named(_) => {
                let try_from = get_try_from_document_impl(&crate_ident, shape_name);
                quote! {
                    const _: () = {
                        #extern_import

                        #try_from
                    };

                    #contents
                }
                .into()
            }
            Fields::Unit => contents.into(),
        }
    } else {
        contents.into()
    }
}
