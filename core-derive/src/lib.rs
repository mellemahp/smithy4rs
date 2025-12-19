mod builder;
mod deserialization;
mod schema;
mod serialization;
mod utils;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, ItemEnum, Lit, Variant, parse, parse_macro_input, parse_quote};

use crate::{
    builder::{buildable, builder_impls, builder_struct, get_builder_fields},
    deserialization::deserialization_impl,
    schema::schema_impl,
    serialization::serialization_impl,
    utils::{get_builder_ident, get_crate_info, parse_enum_value, parse_schema},
};

// TODO(errors): Make error handling use: `syn::Error::into_compile_error`
// TODO(macro): Add debug impl using fmt serializer
// TODO(derive): Smithy Struct should automatically derive: Debug, PartialEq, and Clone
//               if not already derived on shape.

/// Modifies an enum to be usable as a Smithy enum
///
/// This macro is used to automatically add an unknown variant for Smithy Enums and Union shapes.
/// It also allows us to use discriminants for both string and int enums
#[proc_macro_attribute]
pub fn smithy_enum(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // process all discriminants.
    // *WARNING*: This must occur _BEFORE_ adding unknown variant
    let input = discriminant_to_attribute(args.clone(), input);
    // Add unknown variants
    unknown_variant(args, input)
}

#[proc_macro_attribute]
pub fn discriminant_to_attribute(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut enum_struct = parse_macro_input!(input as ItemEnum);
    // Expect NO args
    let _ = parse_macro_input!(args as parse::Nothing);

    // Change all discriminants to attributes for consistency
    for variant in enum_struct.variants.iter_mut() {
        if let Some((_, expr)) = &variant.discriminant {
            variant.attrs.push(parse_quote!(#[enum_value(#expr)]));
            variant.discriminant = None;
        };
    }

    quote!(#enum_struct).into()
}

/// This macro is automatically adds an unknown variant for Enums and Unions.
#[proc_macro_attribute]
pub fn unknown_variant(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut enum_struct = parse_macro_input!(input as ItemEnum);
    // Expect NO args
    let _ = parse_macro_input!(args as parse::Nothing);

    // Determine if unknown should store string or int. Unions (without `enum_value` attr)
    // will default tousing String as unknown data value.
    let field = if let Some(val) = parse_enum_value(
        &enum_struct
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
    enum_struct.variants.push(Variant {
        attrs: vec![
            parse_quote!(#[automatically_derived]),
            parse_quote!(#[doc(hidden)]),
        ],
        discriminant: None,
        fields: syn::Fields::Unnamed(field),
        ident: Ident::new("Unknown", Span::call_site()),
    });
    quote!(#enum_struct).into()
}

#[proc_macro_derive(Dummy, attributes(smithy_schema, enum_value))]
pub fn dummy_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let schema = schema_shape_derive(input.clone());
    let serializable = serializable_shape_derive(input.clone());
    let deserializable = deserializable_shape_derive(input);

    let schema_tokens = TokenStream::from(schema);
    let serializable_tokens = TokenStream::from(serializable);
    let deserializable_tokens = TokenStream::from(deserializable);

    quote! {
        #schema_tokens
        #serializable_tokens
        #deserializable_tokens
    }
    .into()
}

/// Convenience derive that combines `SchemaShape`, `SerializableShape`, and `DeserializableShape`
/// for Smithy Enums, Structures, and Unions.
#[proc_macro_derive(SmithyShape, attributes(smithy_schema, enum_value))]
pub fn smithy_shape_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Generate all three derive expansions
    let schema = schema_shape_derive(input.clone());
    let serializable = serializable_shape_derive(input.clone());
    let deserializable = deserializable_shape_derive(input);

    // Combine all outputs
    let schema_tokens = TokenStream::from(schema);
    let serializable_tokens = TokenStream::from(serializable);
    let deserializable_tokens = TokenStream::from(deserializable);

    quote! {
        #schema_tokens
        #serializable_tokens
        #deserializable_tokens
    }
    .into()
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
#[proc_macro_derive(DeserializableShape, attributes(smithy_schema))]
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
