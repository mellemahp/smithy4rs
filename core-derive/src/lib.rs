use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Type, parse_macro_input};

// TODO: Enable feature to generate serde adapters (maybe not needed?)
// TODO: Add some unit tests!
// TODO: Make error handling use: `syn::Error::into_compile_error`
// TODO(macro): Add debug impl using fmt serializer

/// Derives `SchemaShape` for a struct, backed by a static schema (`StaticSchemaShape`)
#[proc_macro_derive(SchemaShape, attributes(smithy_schema))]
pub fn schema_shape_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let schema_ident = parse_schema(&input.attrs);
    let shape_name = &input.ident;

    let found_crate =
        crate_name("smithy4rs-core").expect("smithy4rs-core is present in `Cargo.toml`");
    let extern_import = match &found_crate {
        FoundCrate::Itself => quote!(),
        FoundCrate::Name(name) => {
            let ident = Ident::new(name, Span::call_site());
            quote! {
                extern crate #ident as _smithy4rs;
            }
        }
    };
    let crate_ident = match &found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(_) => {
            let ident = Ident::new("_smithy4rs", Span::call_site());
            quote!( #ident )
        }
    };

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

    // Allows us to use these within the smithy4rs tests
    let found_crate =
        crate_name("smithy4rs-core").expect("smithy4rs-core is present in `Cargo.toml`");
    let extern_import = match &found_crate {
        FoundCrate::Itself => quote!(),
        FoundCrate::Name(name) => {
            let ident = Ident::new(name, Span::call_site());
            quote! {
                extern crate #ident as _smithy4rs;
            }
        }
    };
    let crate_ident = match &found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(_) => {
            let ident = Ident::new("_smithy4rs", Span::call_site());
            quote!( #ident )
        }
    };
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

#[proc_macro_derive(DeserializableStruct, attributes(smithy_schema))]
pub fn deserializable_struct_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let schema_ident = parse_schema(&input.attrs);
    let shape_name = &input.ident;

    // Generate builder struct and impl
    let builder = builder_impl(shape_name, &input);

    // Get crate reference
    let found_crate =
        crate_name("smithy4rs-core").expect("smithy4rs-core is present in `Cargo.toml`");
    let crate_ident = match &found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(_) => {
            let ident = Ident::new("_smithy4rs", Span::call_site());
            quote!( #ident )
        }
    };
    // `Deserialize` is implemented implicitly
    // Generate DeserializeWithSchema impl
    let deserialization = deserialization_impl(shape_name, &schema_ident, &input, &crate_ident);

    let extern_import = match &found_crate {
        FoundCrate::Itself => quote!(),
        FoundCrate::Name(name) => {
            let ident = Ident::new(name, Span::call_site());
            quote! {
                extern crate #ident as _smithy4rs;
            }
        }
    };

    let imports = quote! {
        #extern_import
        use #crate_ident::schema::SchemaRef as _SchemaRef;
        use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;
        use #crate_ident::serde::deserializers::Deserializer as _Deserializer;
        use #crate_ident::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
        use #crate_ident::serde::deserializers::Error as _Error;
    };

    quote! {
        // Builder is generated outside the const block to make it publicly accessible
        #builder

        const _: () = {
            #imports

            #deserialization
        };
    }
    .into()
}

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

fn parse_schema(attrs: &[Attribute]) -> Ident {
    let mut target_schema = None;
    for attr in attrs {
        if attr.path().is_ident("smithy_schema") {
            target_schema = Some(
                attr.parse_args::<Ident>()
                    .expect("`smithy_schema` attribute should be an identifier"),
            );
        }
    }
    target_schema.expect("Could not find `smithy_schema` attribute")
}

fn schema_impl(shape_name: &Ident, schema_ident: &Ident) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl _StaticSchemaShape for #shape_name {
            fn schema() -> &'static _SchemaRef {
                &#schema_ident
            }
        }
    }
}

fn serialization_impl(shape_name: &Ident, input: &DeriveInput) -> TokenStream {
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("SerializableStruct can only be derived for structs"),
    };
    let mut field_data = Vec::new();
    let length = fields.len();
    for field in fields {
        let schema = parse_schema(&field.attrs);
        let field_ident = field.ident.as_ref().unwrap().clone();
        let optional = is_optional(&field.ty);
        field_data.push(FieldData {
            schema,
            field_ident,
            optional,
        });
    }
    // Now write the thing
    let method = field_data.iter().map(|d| d.method_call());
    let member_schema = field_data.iter().map(|d| &d.schema);
    let member_name = field_data.iter().map(|d| &d.field_ident);
    let member_name_str = field_data.iter().map(|d| d.field_ident.to_string());
    quote! {
        #[automatically_derived]
        impl _SerializeWithSchema for #shape_name {
            fn serialize_with_schema<S: _Serializer>(
                &self,
                schema: &_SchemaRef,
                serializer: S,
            ) -> Result<S::Ok, S::Error> {
                let mut ser = serializer.write_struct(schema, #length)?;
                #(ser.#method(#member_name_str, &#member_schema, &self.#member_name)?;)*
                ser.end(schema)
            }
        }
    }
}

struct FieldData {
    schema: Ident,
    field_ident: Ident,
    optional: bool,
}
impl FieldData {
    fn method_call(&self) -> Ident {
        if self.optional {
            Ident::new("serialize_optional_member_named", Span::call_site())
        } else {
            Ident::new("serialize_member_named", Span::call_site())
        }
    }
}

fn is_optional(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            let idents_of_path = path.segments.iter().fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push(':');
                acc
            });
            // Figure out if the type is optional
            // TODO: Might erroneously detect optionals in sparse lists or maps
            vec!["Option:", "std:option:Option:", "core:option:Option:"]
                .into_iter()
                .any(|s| idents_of_path == *s)
        }
        _ => panic!("Serde can only be derived for resolvable types"),
    }
}

fn extract_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return Some(inner_ty);
    }
    None
}

fn builder_impl(shape_name: &Ident, input: &DeriveInput) -> TokenStream {
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("DeserializableStruct can only be derived for structs"),
    };

    let builder_name = Ident::new(&format!("{}Builder", shape_name), Span::call_site());

    let mut field_data = Vec::new();
    for field in fields {
        let field_ident = field.ident.as_ref().unwrap().clone();
        let field_ty = &field.ty;
        let optional = is_optional(field_ty);

        // For the setter, we need the inner type
        let setter_ty = if optional {
            extract_inner_type(field_ty).unwrap_or(field_ty)
        } else {
            field_ty
        };

        field_data.push(DeserFieldData {
            field_ident,
            setter_ty: setter_ty.clone(),
            optional,
        });
    }

    // Generate builder struct fields: all fields are Option<T> where T is the setter type
    let builder_fields = field_data.iter().map(|d| {
        let field_name = &d.field_ident;
        let setter_ty = &d.setter_ty;
        quote! {
            #field_name: Option<#setter_ty>
        }
    });

    // Generate new() initialization: all fields are None
    let new_fields = field_data.iter().map(|d| {
        let field_name = &d.field_ident;
        quote! { #field_name: None }
    });

    // Generate setter methods - consuming for chaining
    let setters = field_data.iter().map(|d| {
        let field_name = &d.field_ident;
        let setter_ty = &d.setter_ty;

        quote! {
            pub fn #field_name(mut self, value: #setter_ty) -> Self {
                self.#field_name = Some(value);
                self
            }
        }
    });

    // Generate build() method
    let build_fields = field_data.iter().map(|d| {
        let field_name = &d.field_ident;
        if d.optional {
            // For optional fields, just pass the Option through
            quote! {
                #field_name: self.#field_name
            }
        } else {
            // For required fields, unwrap or return error
            let error_msg = format!("{} is required", field_name);
            quote! {
                #field_name: self.#field_name.ok_or_else(|| #error_msg.to_string())?
            }
        }
    });

    quote! {
        #[automatically_derived]
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

            pub fn build(self) -> Result<#shape_name, String> {
                Ok(#shape_name {
                    #(#build_fields,)*
                })
            }
        }
    }
}

struct DeserFieldData {
    field_ident: Ident,
    setter_ty: Type,
    optional: bool,
}

fn deserialization_impl(
    shape_name: &Ident,
    _schema_ident: &Ident,
    input: &DeriveInput,
    crate_ident: &TokenStream,
) -> TokenStream {
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("DeserializableStruct can only be derived for structs"),
    };

    let builder_name = Ident::new(&format!("{}Builder", shape_name), Span::call_site());

    let mut field_data = Vec::new();
    for (index, field) in fields.iter().enumerate() {
        let schema = parse_schema(&field.attrs);
        let field_ident = field.ident.as_ref().unwrap().clone();
        let field_ty = &field.ty;
        let optional = is_optional(field_ty);

        let inner_ty = if optional {
            extract_inner_type(field_ty).unwrap_or(field_ty)
        } else {
            field_ty
        };

        field_data.push((index, schema, field_ident, inner_ty.clone(), optional));
    }

    // Generate deserialize_member! or deserialize_optional_member! macro calls for each field
    let match_arms = field_data
        .iter()
        .map(|(_index, schema, field_ident, inner_ty, optional)| {
            if *optional {
                // For optional fields, use deserialize_optional_member! with inner type
                quote! {
                    #crate_ident::deserialize_optional_member!(member_schema, &#schema, de, builder, #field_ident, #inner_ty);
                }
            } else {
                // For required fields, use deserialize_member!
                quote! {
                    #crate_ident::deserialize_member!(member_schema, &#schema, de, builder, #field_ident, #inner_ty);
                }
            }
        });

    quote! {
       // Builder implements DeserializeWithSchema
        #[automatically_derived]
        impl<'de> _DeserializeWithSchema<'de> for #builder_name {
            fn deserialize_with_schema<D>(schema: &_SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
            where
                D: _Deserializer<'de>,
            {
                let builder = #builder_name::new();
                deserializer.read_struct(schema, builder, |builder, member_schema, de| {
                    #(#match_arms)*
                    Ok(builder) // Unknown field
                })
            }
        }

        // Builder implements ShapeBuilder
        #[automatically_derived]
        impl<'de> #crate_ident::serde::ShapeBuilder<'de, #shape_name> for #builder_name {
            type Error = String;

            fn new() -> Self {
                Self::new()
            }

            fn build(self) -> Result<#shape_name, Self::Error> {
                self.build()
            }
        }

        // Shape implements DeserializeWithSchema by delegating to builder
        #[automatically_derived]
        impl<'de> _DeserializeWithSchema<'de> for #shape_name {
            fn deserialize_with_schema<D>(schema: &_SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
            where
                D: _Deserializer<'de>,
            {
                let builder = #builder_name::deserialize_with_schema(schema, deserializer)?;
                builder.build().map_err(_Error::custom)
            }
        }
    }
}
