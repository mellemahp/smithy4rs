use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields};

/// Generates `SmithyTrait` impl for Smithy Shapes.
pub(crate) fn trait_value_impl(trait_ident: &Ident, input: &DeriveInput) -> TokenStream {
    let field = get_field(input);
    quote! {
        #[automatically_derived]
        impl _SmithyTrait for #trait_ident {
            fn id(&self) -> &_ShapeId {
                Self::trait_id()
            }

            fn value(&self) -> &Box<dyn _Document> {
                #field
            }
        }
    }
}

/// Get the field that stores the document value
fn get_field(input: &DeriveInput) -> TokenStream {
    match &input.data {
        Data::Struct(s) =>  match &s.fields {
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() != 2 {
                    panic!("Expected exactly 2 unnamed fields");
                }
                // Tuple struct traits will store in second field
                quote! { &self.1 }
            }
            Fields::Named(_) => {
                quote! { &self._value }
            }
            Fields::Unit => {
                panic!("Unit structs are not supported as traits");
            }
        },
        Data::Enum(_) => todo!(),
        Data::Union(_) => panic!("Union types not supported")
    }
}