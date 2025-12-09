mod utils;
mod schema;
mod serialization;
mod deserialization;

use proc_macro2::{TokenStream};
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

use crate::deserialization::{builder_impl, deserialization_impl};
use crate::schema::schema_impl;
use crate::serialization::serialization_impl;
use crate::utils::{get_crate_info, parse_schema};

// TODO(errors): Make error handling use: `syn::Error::into_compile_error`
// TODO(macro): Add debug impl using fmt serializer

/// Convenience derive that combines `SchemaShape`, `SerializableStruct`, and `DeserializableStruct`
#[proc_macro_derive(SmithyStruct, attributes(smithy_schema))]
pub fn smithy_struct_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Generate all three derive expansions
    let schema = schema_shape_derive(input.clone());
    let serializable = serializable_struct_derive(input.clone());
    let deserializable = deserializable_struct_derive(input);

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
    let imports = quote! {
        #extern_import
        use #crate_ident::schema::SchemaRef as _SchemaRef;
        use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;
    };
    let schema_trait = schema_impl(shape_name, &schema_ident);

    quote! {
        const _: () = {
            #imports

            #schema_trait
        };
    }
    .into()
}

/// Derives `SerializableStruct` (`SerializeWithSchema` only, no schema)
#[proc_macro_derive(SerializableStruct, attributes(smithy_schema))]
pub fn serializable_struct_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let shape_name = &input.ident;

    // `Deserialize` is implemented implicitly
    // Generate the  SerializeWithSchema implementation
    let serialization = serialization_impl(shape_name, &input);

    let (extern_import, crate_ident) = get_crate_info();
    // Dont include imports if tests
    let imports = quote! {
        #extern_import
        use #crate_ident::schema::SchemaRef as _SchemaRef;
        use #crate_ident::serde::serializers::Serializer as _Serializer;
        use #crate_ident::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
        use #crate_ident::serde::serializers::StructSerializer as _StructSerializer;
    };
    quote! {
        const _: () = {
            #imports

            #serialization
        };
    }
    .into()
}

/// Derives `DeserializeWithSchema` and, implicitly `Deserialize` for a Shape.
#[proc_macro_derive(DeserializableStruct, attributes(smithy_schema))]
pub fn deserializable_struct_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let schema_ident = parse_schema(&input.attrs);
    let shape_name = &input.ident;
    let (extern_import, crate_ident) = get_crate_info();

    // Generate builder struct and impl
    let builder = builder_impl(shape_name, &input, &crate_ident);

    let deserialization = deserialization_impl(shape_name, &schema_ident, &input, &crate_ident);
    let imports = quote! {
        #extern_import
        use #crate_ident::schema::SchemaRef as _SchemaRef;
        use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;
        use #crate_ident::serde::deserializers::Deserializer as _Deserializer;
        use #crate_ident::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
        use #crate_ident::serde::deserializers::Error as _Error;
        use #crate_ident::serde::correction::ErrorCorrection as _ErrorCorrection;
        use #crate_ident::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
        use #crate_ident::serde::ShapeBuilder as _ShapeBuilder;
    };

    // (?) Builder is generated outside the const block to make it publicly accessible
    quote! {
        const _: () = {
            #imports

            #builder

            #deserialization
        };
    }
    .into()
}
