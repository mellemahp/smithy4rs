use darling::util::Override;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::{
    attr::{StructMember, StructShape},
    shapes::serialization::serialize_struct,
    utils::{TargetType, resolve_builder_target},
};

pub(crate) fn expand_builder(shape: &StructShape, crate_ident: &TokenStream) -> TokenStream {
    let struct_impls = expand_struct_impl(shape, crate_ident);
    let builder_struct = expand_builder_struct(shape, crate_ident);
    let builder_impls = expand_builder_impls(shape, crate_ident);

    quote! {
        #struct_impls

        #builder_struct

        #builder_impls
    }
}

fn expand_struct_impl(shape: &StructShape, crate_ident: &TokenStream) -> TokenStream {
    let shape_name = &shape.ident;
    let builder_name = shape.builder();

    quote! {
        #[doc = concat!(" Builder for [`", stringify!(#shape_name), "`]")]
        #[automatically_derived]
        impl #shape_name {
            /// Get a new builder for this shape.
            #[must_use]
            #[inline]
            pub fn builder() -> #builder_name {
                <Self as #crate_ident::serde::Buildable<#builder_name>>::builder()
            }
        }
    }
}

fn expand_builder_struct(shape: &StructShape, crate_ident: &TokenStream) -> TokenStream {
    let shape_name = &shape.ident;
    let builder_name = shape.builder();
    let fields = shape.data.as_struct().expect("Not a struct");

    let builder_fields = fields.iter().map(|m| field_type(m, crate_ident));

    // Generate new() initialization
    let new_fields = fields.iter().map(|m| initializer(m, crate_ident));

    // Generate setter methods - consuming for chaining
    let setters = fields.iter().map(|m| setter(m, crate_ident));

    quote! {
        #[doc = concat!(" Builder for [`", stringify!(#shape_name), "`]")]
        #[automatically_derived]
        #[derive(Clone)]
        pub struct #builder_name {
            #(#builder_fields,)*
        }

        #[automatically_derived]
        impl #builder_name {
            #[doc = concat!("Create a new `", stringify!(#builder_name), "` instance")]
            pub fn new() -> Self {
                #builder_name {
                    #(#new_fields,)*
                }
            }

            #(#setters)*

            /// Build the shape, validating with the default validator.
            #[inline]
            pub fn build(self) -> #crate_ident::serde::validation::Validated<#shape_name> {
                #crate_ident::serde::ShapeBuilder::build(self)
            }

            /// Build the shape using a custom validator.
            #[inline]
            pub fn build_with_validator(self, validator: impl #crate_ident::serde::validation::Validator) -> #crate_ident::serde::validation::Validated<#shape_name> {
                #crate_ident::serde::ShapeBuilder::build_with_validator(self, validator)
            }

            /// Error correct builder
            pub fn correct(self) -> #shape_name {
                <Self as #crate_ident::serde::correction::ErrorCorrection>::correct(self)
            }
        }
    }
}

fn expand_builder_impls(shape: &StructShape, crate_ident: &TokenStream) -> TokenStream {
    let shape_name = &shape.ident;
    let schema = &shape.schema;
    let builder_name = shape.builder();
    let fields = shape.data.as_struct().expect("Not a struct");
    let ser = serialize_struct(shape);

    // Generate correct() method used to automatically derive `build()` methods
    let build_fields = fields.iter().map(correct);

    // TODO: Update so we can reuse same impls as other shapes.
    quote! {
        const _: () = {
            use #crate_ident::serde::correction::ErrorCorrection as _ErrorCorrection;

            #[automatically_derived]
            impl _ErrorCorrection for #builder_name {
                type Value = #shape_name;

                fn correct(self) -> Self::Value {
                    Self::Value {
                        #(#build_fields,)*
                    }
                }
            }
        };

        const _: () = {
            use #crate_ident::serde::ShapeBuilder as _ShapeBuilder;

            #[automatically_derived]
            impl<'de> _ShapeBuilder<'de, #shape_name> for #builder_name {
                fn new() -> Self {
                    #builder_name::new()
                }
            }
        };

        const _: () = {
            use #crate_ident::schema::Schema as _Schema;
            use #crate_ident::serde::serializers::Serializer as _Serializer;
            use #crate_ident::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
            use #crate_ident::serde::serializers::StructWriter as _StructWriter;

            #[automatically_derived]
            impl _SerializeWithSchema for #builder_name {
                fn serialize_with_schema<S: _Serializer>(
                    &self,
                    schema: &_Schema,
                    serializer: S,
                ) -> Result<S::Ok, S::Error> {
                    #ser
                }
            }
        };

        const _: () = {
            use #crate_ident::serde::Buildable as _Buildable;

            impl <'de> _Buildable<'de, #builder_name> for #shape_name {}
        };

        const _: () = {
            use #crate_ident::schema::Schema as _Schema;
            use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;

            #[automatically_derived]
            impl _StaticSchemaShape for #builder_name {
                #[inline]
                fn schema() -> &'static _Schema {
                    &#schema
                }
            }
        };
    }
}

/// The fully resolved field type to set on builders
fn field_type(member: &StructMember, crate_ident: &TokenStream) -> TokenStream {
    let ty = match resolve_builder_target(member) {
        TargetType::Builable { shape, builder } => {
            quote! { #crate_ident::serde::MaybeBuilt<#shape, #builder> }
        }
        TargetType::Primitive(ty) => quote! { #ty },
    };
    let field_name = member
        .ident
        .as_ref()
        .expect("struct memebers should be named");
    if member.optional() {
        quote! {
            #field_name: Option<#ty>
        }
    } else {
        quote! {
            #field_name: #crate_ident::serde::Required<#ty>
        }
    }
}

/// Initializer to use for setting a builder field in `new()` method
/// - all optional fields are `None`.
/// - All required fields are `Required::Unset`
fn initializer(member: &StructMember, crate_ident: &TokenStream) -> TokenStream {
    let field_name = &member.ident;
    if member.optional() {
        quote! { #field_name: None }
    } else if let Some(default_override) = member.default.as_ref() {
        if let Override::Explicit(default) = default_override {
            quote! { #field_name: #crate_ident::serde::Required::Set(#default) }
        } else {
            let ty = &member.ty;
            // TODO: How to handle with wrappers?
            quote! { #field_name: #crate_ident::serde::Required::Set(#ty::default()) }
        }
    } else {
        quote! { #field_name: #crate_ident::serde::Required::Unset }
    }
}

/// Generate builder setters.
///
/// Setters consume `self` to allow for chaining.
fn setter(member: &StructMember, crate_ident: &TokenStream) -> TokenStream {
    let field_name = member
        .ident
        .as_ref()
        .expect("struct members should be named");

    let wrapper = if member.optional() {
        quote! { Some }
    } else {
        quote! { #crate_ident::serde::Required::Set }
    };

    match resolve_builder_target(member) {
        TargetType::Builable { shape, builder } => {
            let builder_fn = Ident::new(&format!("{field_name}_builder"), Span::call_site());

            quote! {
                #[doc = concat!("Set `", stringify!(#field_name), "`.")]
                pub fn #field_name(mut self, value: #shape) -> Self {
                    self.#field_name = #wrapper(#crate_ident::serde::MaybeBuilt::Struct(value));
                    self
                }

                #[doc = concat!("Set `", stringify!(#field_name), "`.")]
                pub fn #builder_fn(mut self, value: #builder) -> Self {
                    self.#field_name = #wrapper(#crate_ident::serde::MaybeBuilt::Builder(value));
                    self
                }
            }
        }
        TargetType::Primitive(ty) => {
            quote! {
                #[doc = concat!("Set `", stringify!(#field_name), "`.")]
                pub fn #field_name<T: Into<#ty>>(mut self, value: T) -> Self {
                    self.#field_name = #wrapper(value.into());
                    self
                }
            }
        }
    }
}

/// Get the `correct`/`build` methods that extract value out of builder.
fn correct(member: &StructMember) -> TokenStream {
    let field_name = &member.ident;
    let target = resolve_builder_target(member);
    match (member.optional(), &target) {
        // === Optional types ===
        (true, TargetType::Primitive(_)) => {
            // simply pass through
            quote! {
                #field_name: self.#field_name
            }
        }
        (true, TargetType::Builable { .. }) => {
            // Unwrap the `MaybeBuilt`
            quote! {
                #field_name: self.#field_name.correct()
            }
        }
        // === Required types ===
        (false, TargetType::Primitive(_)) => {
            // Resolve value from `Required` wrapper
            quote! {
                #field_name: self.#field_name.get()
            }
        }
        (false, TargetType::Builable { .. }) => {
            // Resolve value from `Required` wrapper and then unwrap from `MaybeBuilt`
            quote! {
                #field_name: self.#field_name.get().correct()
            }
        }
    }
}
