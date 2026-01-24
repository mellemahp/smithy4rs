use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DataStruct, Type};

use crate::utils::{
    IdentOrExpr, extract_option_type, get_crate_ident, get_ident, get_inner_type, is_optional,
    is_primitive, parse_default, parse_schema, replace_inner,
};

pub(crate) fn builder_struct(shape_name: &Ident, field_data: &[BuilderFieldData]) -> TokenStream {
    let builder_name = Ident::new(&format!("{shape_name}Builder"), Span::call_site());
    let crate_ident = get_crate_ident();

    // Generate builder struct fields
    let builder_fields = field_data
        .iter()
        .map(|d| d.field_type(&crate_ident))
        .collect::<Vec<_>>();

    // Generate new() initialization
    let new_fields = field_data
        .iter()
        .map(|d| d.initializer(&crate_ident))
        .collect::<Vec<_>>();

    // Generate setter methods - consuming for chaining
    let setters = field_data
        .iter()
        .map(|d| d.setters(&crate_ident))
        .collect::<Vec<_>>();

    quote! {
        #[automatically_derived]
        #[derive(Clone)]
        pub struct #builder_name {
            #(#builder_fields,)*
        }

        #[automatically_derived]
        impl #builder_name {
            pub fn new() -> Self {
                Self {
                    #(#new_fields,)*
                }
            }

            #(#setters)*
        }
    }
}

pub fn builder_impls(shape_name: &Ident, field_data: &[BuilderFieldData]) -> TokenStream {
    let builder_name = Ident::new(&format!("{shape_name}Builder"), Span::call_site());

    // Generate correct() method used to automatically derive `build()` methods
    let build_fields = field_data
        .iter()
        .map(BuilderFieldData::correct)
        .collect::<Vec<_>>();

    quote! {
        #[automatically_derived]
        impl _ErrorCorrection for #builder_name {
            type Value = #shape_name;

            fn correct(self) -> Self::Value {
                #shape_name {
                    #(#build_fields,)*
                }
            }
        }

        #[automatically_derived]
        impl<'de> _ShapeBuilder<'de, #shape_name> for #builder_name {
            fn new() -> Self {
                Self::new()
            }
        }

        #[automatically_derived]
        impl _ErrorCorrectionDefault for #shape_name {
            fn default() -> Self {
                #builder_name::new().correct()
            }
        }
    }
}

pub fn get_builder_fields(schema_ident: &Ident, data: &DataStruct) -> Vec<BuilderFieldData> {
    let mut field_data = Vec::new();
    for field in &data.fields {
        let schema = Ident::new(
            &format!("_{}_MEMBER_{}", schema_ident, parse_schema(&field.attrs)),
            Span::call_site(),
        );
        let field_ident = field.ident.as_ref().unwrap().clone();
        let field_ty = &field.ty;
        let default = parse_default(&field.attrs);
        let optional = is_optional(field_ty) && default.is_none();
        let target = resolve_build_target(field_ty, optional);

        field_data.push(BuilderFieldData {
            schema,
            field_ident,
            default,
            optional,
            target,
        });
    }
    field_data
}

fn resolve_build_target(field_ty: &Type, optional: bool) -> BuildTarget {
    // The target type is the inner type of any optional
    let ty = if optional {
        extract_option_type(field_ty).unwrap_or(field_ty)
    } else {
        field_ty
    };

    // Get the inner type of parametrized types (i.e. `Vec<T>`, `IndexMap<String, T>`)
    let inner_type = get_inner_type(ty);

    // If the inner type is a primitive type, just return that
    if is_primitive(inner_type) {
        return BuildTarget::Primitive(ty.clone());
    }

    // We will create two target types. One with the builder
    // and the other with the "built" type.
    let mut builder_type = ty.clone();
    let type_ident = get_ident(inner_type);
    let builder_ident = Ident::new(&format!("{type_ident}Builder"), Span::call_site());
    replace_inner(&mut builder_type, builder_ident);

    // Create the build target for a `MaybeBuilt<>` impl
    BuildTarget::Builable {
        shape: ty.clone(),
        builder: builder_type.clone(),
    }
}

pub(crate) struct BuilderFieldData {
    schema: Ident,
    field_ident: Ident,
    default: Option<IdentOrExpr>,
    optional: bool,
    target: BuildTarget,
}
#[allow(clippy::large_enum_variant)]
enum BuildTarget {
    /// A type that also implements `ShapeBuilder` and so must be wrapped with `MaybeBuilder<>`.
    Builable { shape: Type, builder: Type },
    /// A simple type (`string`, `i32`, etc.) that needs no additional wrapping.
    Primitive(Type),
}
impl BuilderFieldData {
    /// Type to use when representing this type as a field in a builder struct definition
    fn field_type(&self, crate_ident: &TokenStream) -> TokenStream {
        let ty = match &self.target {
            BuildTarget::Builable { shape, builder } => {
                quote! { #crate_ident::serde::MaybeBuilt<#shape, #builder> }
            }
            BuildTarget::Primitive(ty) => quote! { #ty },
        };
        let field_name = &self.field_ident;
        if self.optional {
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
    fn initializer(&self, crate_ident: &TokenStream) -> TokenStream {
        let field_name = &self.field_ident;
        if self.optional {
            quote! { #field_name: None }
        } else if let Some(default) = self.default.as_ref() {
            quote! { #field_name: #crate_ident::serde::Required::Set(#default) }
        } else {
            quote! { #field_name: #crate_ident::serde::Required::Unset }
        }
    }

    /// Generate builder setters.
    ///
    /// Setters consume `self` to allow for chaining.
    fn setters(&self, crate_ident: &TokenStream) -> TokenStream {
        let field_name = &self.field_ident;
        let wrapper = if self.optional {
            quote! { Some }
        } else {
            quote! { #crate_ident::serde::Required::Set }
        };

        match &self.target {
            BuildTarget::Builable { shape, builder } => {
                let builder_fn = Ident::new(&format!("{field_name}_builder"), Span::call_site());
                quote! {
                    pub fn #field_name(mut self, value: #shape) -> Self {
                        self.#field_name = #wrapper(#crate_ident::serde::MaybeBuilt::Struct(value));
                        self
                    }

                    pub fn #builder_fn(mut self, value: #builder) -> Self {
                        self.#field_name = #wrapper(#crate_ident::serde::MaybeBuilt::Builder(value));
                        self
                    }
                }
            }
            BuildTarget::Primitive(ty) => {
                quote! {
                    pub fn #field_name<T: Into<#ty>>(mut self, value: T) -> Self {
                        self.#field_name = #wrapper(value.into());
                        self
                    }
                }
            }
        }
    }

    /// Get the `correct`/`build` methods that extract value out of builder.
    fn correct(&self) -> TokenStream {
        let field_name = &self.field_ident;
        match (self.optional, &self.target) {
            // === Optional types ===
            (true, BuildTarget::Primitive(_)) => {
                // simply pass through
                quote! {
                    #field_name: self.#field_name
                }
            }
            (true, BuildTarget::Builable { .. }) => {
                // Unwrap the `MaybeBuilt`
                quote! {
                    #field_name: self.#field_name.correct()
                }
            }
            // === Required types ===
            (false, BuildTarget::Primitive(_)) => {
                // Resolve value from `Required` wrapper
                quote! {
                    #field_name: self.#field_name.get()
                }
            }
            (false, BuildTarget::Builable { .. }) => {
                // Resolve value from `Required` wrapper and then unwrap from `MaybeBuilt`
                quote! {
                    #field_name: self.#field_name.get().correct()
                }
            }
        }
    }

    /// Get the corresponding match arm for the builder field
    pub(crate) fn deserialize_match_arm(&self, crate_ident: &TokenStream) -> TokenStream {
        let field_name = &self.field_ident;
        let schema = &self.schema;
        // Buildable fields use the `_builder` setter for deserialization
        // to take an unbuilt shape as input.
        match (self.optional, &self.target) {
            // === Optional types ===
            // For optional fields, use deserialize_optional_member! with inner type
            (true, BuildTarget::Primitive(ty)) => {
                quote! {
                    #crate_ident::deserialize_optional_member!(member_schema, #schema, de, builder, #field_name, #ty);
                }
            }
            (true, BuildTarget::Builable { builder, .. }) => {
                let field_builder =
                    Ident::new(format!("{field_name}_builder").as_str(), Span::call_site());
                quote! {
                    #crate_ident::deserialize_optional_member!(member_schema, #schema, de, builder, #field_builder, #builder);
                }
            }
            // === Required types ===
            // For required fields, use deserialize_member!
            (false, BuildTarget::Primitive(ty)) => {
                quote! {
                    #crate_ident::deserialize_member!(member_schema, #schema, de, builder, #field_name, #ty);
                }
            }
            (false, BuildTarget::Builable { builder, .. }) => {
                let field_builder =
                    Ident::new(format!("{field_name}_builder").as_str(), Span::call_site());
                quote! {
                    #crate_ident::deserialize_member!(member_schema, #schema, de, builder, #field_builder, #builder);
                }
            }
        }
    }
}

pub(crate) fn buildable(shape_name: &Ident, builder_name: &Ident) -> TokenStream {
    quote! {
       impl <'de> _Buildable<'de, #builder_name> for #shape_name {}
    }
}
