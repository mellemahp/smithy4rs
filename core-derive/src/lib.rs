extern crate proc_macro;
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
                #[allow(unused_extern_crates, clippy::useless_attribute)]
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
    };
    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications, clippy::absolute_paths)]
        const _: () = {
            #imports

            #base_trait

            #schema_trait

            #serialization
        };
    }.into()
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
    quote! {
        #[automatically_derived]
        impl _SerializeWithSchema for #shape_name {
            fn serialize_with_schema<S: _Serializer>(
                &self,
                schema: &_SchemaRef,
                serializer: S,
            ) -> Result<S::Ok, S::Error> {
                let mut ser = serializer.write_struct(schema, #length)?;
                #(ser.#method(&#member_schema, &self.#member_name)?;)*
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
            Ident::new("serialize_optional_member", Span::call_site())
        } else {
            Ident::new("serialize_member", Span::call_site())
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
                .find(|s| idents_of_path == *s)
                .is_some()
        }
        _ => panic!("Serde can only be derived for resolvable types"),
    }
}
