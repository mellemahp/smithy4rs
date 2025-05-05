extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::Attribute;

#[proc_macro_derive(SerializableStruct, attributes(schema))]
pub fn serializable_struct_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();
    // Build the trait implementation
    impl_serializable_struct_derive(&ast)
}

fn impl_serializable_struct_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let schema_ident = parse_schema(&ast.attrs);
    // First, generate the basic implementation of the structure serializer.
    // TODO: How to handle enums and unions?
    let mut output = quote! {
        impl Serializable for #name {
            fn serialize<S: Serializer>(self, serializer: &mut S) -> Result<(), S::Error> {
                SerializableStruct::serialize(self, serializer)
            }
        }
    };

    // Now implement the member-specific serialization
    output.extend(quote! {
        impl SerializableStruct for #name {
            fn schema() -> &'static Schema {
                &#schema_ident
            }

            fn serialize_members<T: Serializer>(&self, serializer: &mut T) {
                serializer.write_string(&MEMBER_A, &self.member_a);
            }
        }
    });
    output.into()
}

fn parse_schema(attrs: &[Attribute]) -> Ident {
    let mut target_schema = None;
    for attr in attrs {
        if attr.path.is_ident("schema") {
            target_schema = Some(
                attr.parse_args::<Ident>()
                    .expect("schema attribute should be an identifier"),
            );
        }
    }
    target_schema.expect("Could not find schema attribute")
}
