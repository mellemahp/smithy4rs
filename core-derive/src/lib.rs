mod deserialization;
mod schema;
mod serialization;
mod utils;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, ItemEnum, Variant, parse, parse_macro_input, parse_quote};

use crate::{
    deserialization::{
        buildable, builder_impls, builder_struct, deserialization_impl, get_builder_fields,
    },
    schema::schema_impl,
    serialization::serialization_impl,
    utils::{get_crate_info, parse_schema},
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
    // Add unknown variants
    let input = unknown_variants(args.clone(), input);
    // process all discriminants
    discriminant_to_attribute(args, input)
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
pub fn unknown_variants(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut enum_struct = parse_macro_input!(input as ItemEnum);
    // Expect NO args
    let _ = parse_macro_input!(args as parse::Nothing);

    enum_struct.variants.push(Variant {
        attrs: vec![],
        discriminant: None,
        fields: syn::Fields::Unnamed(parse_quote!((String))),
        ident: Ident::new("_Unknown", Span::call_site()),
    });

    quote!(#enum_struct).into()
}

/// Derive `SchemaShape`, `SerializableShape` and `DeserializableStruct` for an enum
#[proc_macro_derive(SmithyEnum, attributes(smithy_schema, enum_value))]
pub fn smithy_enum_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let schema = schema_shape_derive(input.clone());
    let serializable = serializable_shape_derive(input);

    let schema_tokens = TokenStream::from(schema);
    let serialize_tokens = TokenStream::from(serializable);

    quote! {
        #schema_tokens
        #serialize_tokens
    }
    .into()
}

/// Convenience derive that combines `SchemaShape`, `SerializableShape`, and `DeserializableShape`
/// for Smithy Enums, Structures, and Unions.
#[proc_macro_derive(SmithyShape, attributes(smithy_schema))]
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

    // Generate builder struct and impl

    let imports = quote! {
        // Base imports
        #extern_import
        use #crate_ident::schema::SchemaRef as _SchemaRef;
        use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;
        // deserialization imports
        use #crate_ident::serde::deserializers::Deserializer as _Deserializer;
        use #crate_ident::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
        use #crate_ident::serde::deserializers::Error as _Error;
        // builder imports
        use #crate_ident::serde::correction::ErrorCorrection as _ErrorCorrection;
        use #crate_ident::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
        use #crate_ident::serde::ShapeBuilder as _ShapeBuilder;
        use #crate_ident::serde::Buildable as _Buildable;
    };
    let field_data = get_builder_fields(&schema_ident, &input);
    let builder = builder_struct(shape_name, &field_data);
    let builder_impls = builder_impls(shape_name, &field_data);
    let builder_name = Ident::new(&format!("{}Builder", shape_name), Span::call_site());
    let builder_serializer = serialization_impl(&crate_ident, &builder_name, &schema_ident, &input);
    let deserialization = deserialization_impl(shape_name, &schema_ident, &input, &crate_ident);
    let buildable = buildable(shape_name, &builder_name);

    // Builder struct is generated outside the const block to make it publicly accessible
    quote! {
        #builder

        const _: () = {
            #imports

            #builder_impls

            #builder_serializer

            #deserialization

            #buildable
        };
    }
    .into()
}
