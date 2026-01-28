use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote};
use syn::{Data, DeriveInput, Fields, FieldsNamed, FieldsUnnamed};

/// Adds a builder or constructor to a trait
pub(crate) fn constructor(trait_ident: &Ident, input: &DeriveInput) -> TokenStream {
    match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Unnamed(fields) => tuple_struct_constructor(trait_ident, fields),
                Fields::Named(fields) => struct_builder(trait_ident, fields),
                Fields::Unit => { panic!("Unit struct not supported"); }
            }
        }
        _ => todo!()
    }
}

fn tuple_struct_constructor(trait_ident: &Ident, fields: &FieldsUnnamed) -> TokenStream {
    let value = fields.unnamed.first().cloned().map(|f| f.ty).expect("Expected a field");
    quote! {
        impl #trait_ident {
            #[doc = "Create a new [`"]
            #[doc = stringify!(#trait_ident)]
            #[doc = "`] instance"]
            #[automatically_derived]
            pub fn new(value: #value) -> Self {
                #trait_ident(value.clone(), value.into())
            }
        }
    }
}

fn struct_builder(trait_ident: &Ident, fields: &FieldsNamed) -> TokenStream {
    let builder_ident = Ident::new(&format!("{trait_ident}Builder"), Span::call_site());
    //let builder_impl = builder_impl();
    quote! {
        impl #trait_ident {
            #[doc = "Create a new [`"]
            #[doc = stringify!(#builder_ident)]
            #[doc = "`] instance"]
            #[automatically_derived]
            pub fn builder() -> #builder_ident {
                #builder_ident::new()
            }
        }

        //#builder_impl
    }
}

fn builder_impl(builder_ident: &Ident) -> TokenStream {
    quote! {
        struct #builder_ident {

        }
        impl #builder_ident {}
    }
}
