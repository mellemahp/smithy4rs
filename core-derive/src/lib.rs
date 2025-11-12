use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Type, parse_macro_input};

// TODO: Enable feature to generate serde adapters
// TODO: Add some unit tests!
// TODO: Make error handling use: `syn::Error::into_compile_error`
#[proc_macro_derive(SerializableStruct, attributes(smithy_schema))]
pub fn serializable_struct_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let input = parse_macro_input!(input as DeriveInput);
    let schema_ident = parse_schema(&input.attrs);
    let shape_name = &input.ident;
    // Add the base SerializableShape trait
    let base_trait = base_trait_impl(shape_name);
    // Now add the SchemaShape trait that returns the base schema
    let schema_trait = schema_impl(shape_name, &schema_ident);
    // And now the serializer implementation
    let serialization = serialization_impl(shape_name, &input);

    // TODO: Deserialization impl

    // Allows us to use these within the smithy4rs tests
    let found_crate =
        crate_name("smithy4rs-core").expect("smithy4rs-core is present in `Cargo.toml`");
    let extern_import = match &found_crate {
        FoundCrate::Itself => quote!(),
        FoundCrate::Name(name) => {
            let ident = Ident::new(name, Span::call_site());
            quote! {
                // #[allow(unused_extern_crates, clippy::useless_attribute)]
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
        use #crate_ident::serde::documents::SerializableShape as _SerializableShape;
        use #crate_ident::schema::SchemaShape as _SchemaShape;
        use #crate_ident::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
        use #crate_ident::serde::serializers::Serializer as _Serializer;
        use #crate_ident::serde::serializers::StructSerializer as _StructSerializer;
    };
    quote! {
        // #[doc(hidden)]
        // #[allow(non_upper_case_globals, unused_attributes, unused_qualifications, clippy::absolute_paths)]
        const _: () = {
            #imports

            #base_trait

            #schema_trait

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

    // Generate Deserialize impl
    let deserialization = deserialization_impl(shape_name, &schema_ident, &input);

    // Get crate reference
    let found_crate =
        crate_name("smithy4rs-core").expect("smithy4rs-core is present in `Cargo.toml`");
    let extern_import = match &found_crate {
        FoundCrate::Itself => quote!(),
        FoundCrate::Name(name) => {
            let ident = Ident::new(name, Span::call_site());
            quote! {
                // #[allow(unused_extern_crates, clippy::useless_attribute)]
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
        use #crate_ident::serde::deserializers::Deserialize as _Deserialize;
        use #crate_ident::serde::deserializers::Deserializer as _Deserializer;
        use #crate_ident::serde::deserializers::Error as _Error;
    };

    quote! {
        // #[doc(hidden)]
        // #[allow(non_upper_case_globals, unused_attributes, unused_qualifications, clippy::absolute_paths)]
        const _: () = {
            #imports

            #builder

            #deserialization
        };
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

fn base_trait_impl(shape_name: &Ident) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl _SerializableShape for #shape_name {}
    }
}

fn schema_impl(shape_name: &Ident, schema_ident: &Ident) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl _SchemaShape for #shape_name {
            fn schema(&self) -> &_SchemaRef {
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

    // Generate setter methods
    let setters = field_data.iter().map(|d| {
        let field_name = &d.field_ident;
        let setter_ty = &d.setter_ty;

        if d.optional {
            // For Option<T> fields, setter takes T and wraps in Some
            quote! {
                pub fn #field_name(&mut self, value: #setter_ty) -> &mut Self {
                    self.#field_name = Some(value);
                    self
                }
            }
        } else {
            // For non-optional fields, setter takes T and wraps in Some
            quote! {
                pub fn #field_name(&mut self, value: #setter_ty) -> &mut Self {
                    self.#field_name = Some(value);
                    self
                }
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

    // Generate if-else chain for each field, comparing member_schema Arc pointer
    let match_arms = field_data
        .iter()
        .map(|(_index, schema, field_ident, inner_ty, optional)| {
            if *optional {
                // For optional fields, deserialize as Option<T>
                quote! {
                    if std::sync::Arc::ptr_eq(member_schema, &#schema) {
                        let value = Option::<#inner_ty>::deserialize(member_schema, de)?;
                        if let Some(v) = value {
                            builder.#field_ident(v);
                        }
                    }
                }
            } else {
                // For required fields, deserialize as T
                quote! {
                    if std::sync::Arc::ptr_eq(member_schema, &#schema) {
                        let value = <#inner_ty as _Deserialize>::deserialize(member_schema, de)?;
                        builder.#field_ident(value);
                    }
                }
            }
        });

    quote! {
        #[automatically_derived]
        impl<'de> _Deserialize<'de> for #shape_name {
            fn deserialize<D>(schema: &_SchemaRef, deserializer: &mut D) -> Result<Self, D::Error>
            where
                D: _Deserializer<'de>,
            {
                let mut builder = #builder_name::new();
                deserializer.read_struct(schema, &mut builder, |builder, member_schema, de| {
                    #(#match_arms else)*
                    { /* field not recognized */ }
                    Ok(())
                })?;
                builder.build().map_err(_Error::custom)
            }
        }
    }
}
