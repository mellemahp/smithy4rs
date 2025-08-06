// extern crate proc_macro;
// use proc_macro2::TokenStream;
// use proc_macro2::Ident;
// use quote::quote;
// use serde::Serialize;
// use syn::Attribute;
//
// /// EXAMPLE DESIRED OUTPUT:
// ///     #[smithy_schema(SCHEMA)]
// ///     pub(crate) struct SerializeMe {
// //         // #[smithy_schema(MEMBER_A)]
// //         pub member_a: String,
// //         // #[schema(MEMBER_B)]
// //         pub member_b: String,
// //         // #[schema(MEMBER_C)]
// //         pub member_optional: Option<String>,
// //         pub member_list: Vec<String>,
// //         pub member_map: IndexMap<String, String>,
// //     }
// /// Derives:
// /// 1. impl SerializableShape for SerializeMe {}
// /// 2.    impl SchemaShape for SerializeMe {
// //         fn schema(&self) -> &SchemaRef {
// //             &SCHEMA
// //         }
// //     }
// /// 3.     impl SerializeWithSchema for SerializeMe {
// //         fn serialize_with_schema<S: Serializer>(
// //             &self,
// //             schema: &SchemaRef,
// //             serializer: S,
// //         ) -> Result<S::Ok, S::Error> {
// //             let mut ser = serializer.write_struct(schema, 2)?;
// //             ser.serialize_member(&MEMBER_A, &self.member_a)?;
// //             ser.serialize_member(&MEMBER_B, &self.member_b)?;
// //             ser.serialize_optional_member(&MEMBER_C, &self.member_optional)?;
// //             ser.serialize_member(&MEMBER_LIST, &self.member_list)?;
// //             ser.serialize_member(&MEMBER_MAP, &self.member_map)?;
// //             ser.end(schema)
// //         }
// //     }
// // Based on derive(Serialize, Deserialize) we should
// //
//
//
// #[derive(Serialize)]
// struct X {
//     member: String
// }
//
// #[proc_macro_derive(SerializableStruct, attributes(smithy_schema))]
// pub fn serializable_struct_derive(input: TokenStream) -> TokenStream {
//     // Construct a representation of Rust code as a syntax tree
//     // that we can manipulate
//     let ast = syn::parse(input).unwrap();
//     // Build the trait implementation
//     impl_serializable_struct_derive(&ast)
// }
//
// fn impl_serializable_struct_derive(ast: &syn::DeriveInput) -> TokenStream {
//     let name = &ast.ident;
//     let schema_ident = parse_schema(&ast.attrs);
//     // First, generate the basic implementation of the structure serializer.
//     // TODO: How to handle enums and unions?
//     let mut output = quote! {
//         impl Serializable for #name {
//             fn serialize<S: Serializer>(self, serializer: &mut S) -> Result<S::Ok<'_>, S::Error> {
//                 SerializableStruct::serialize(self, serializer)
//             }
//         }
//     };
//
//     // Now implement the member-specific serialization
//     output.extend(quote! {
//         impl SerializableStruct for #name {
//             fn schema() -> &'static Schema {
//                 &#schema_ident
//             }
//
//             fn serialize_members<T: Serializer>(&self, serializer: &mut T) {
//                 serializer.write_string(&MEMBER_A, &self.member_a);
//             }
//         }
//     });
//     output.into()
// }
//
// fn parse_schema(attrs: &[Attribute]) -> Ident {
//     let mut target_schema = None;
//     for attr in attrs {
//         if attr.path.is_ident("schema") {
//             target_schema = Some(
//                 attr.parse_args::<Ident>()
//                     .expect("schema attribute should be an identifier"),
//             );
//         }
//     }
//     target_schema.expect("Could not find schema attribute")
// }