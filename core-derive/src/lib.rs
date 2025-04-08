
extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::Attribute;

#[proc_macro_derive(SerializableStruct, attributes(schema))]
pub fn serializable_struct_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    println!("{:?}", input);
    let ast = syn::parse(input).unwrap();
    // Build the trait implementation
    impl_serializable_struct_derive(&ast)
}

fn impl_serializable_struct_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let schema_ident = parse_schema(&ast.attrs);
    // First, generate
    let output = quote! {
        impl SerializableStruct for #name {
            fn schema() -> &'static Schema {
                &*#schema_ident
            }

            fn serialize_members<T: Serializer>(&self, serializer: &mut T) {
                serializer.write_string(&*MEMBER_A, &self.member_a);
            }
        }
    };
    output.into()
    // let gener = quote! {
    //     impl SerializableStruct for #name {
    //         fn hello_macro() {
    //             println!("Hello, Macro! My name is {}!", stringify!(#name));
    //         }
    //     }
    // };
}

fn parse_schema(attrs: &[Attribute]) -> Ident  {
    let mut target_schema = None;
    for attr in attrs {
        if attr.path.is_ident("schema") {
            target_schema = Some(attr.parse_args::<Ident>().expect("schema attribute should be an identifier"));
        }
    }
    target_schema.expect("Could not find schema attribute")
}