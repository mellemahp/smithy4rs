use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// Generates `StaticSchemaShape` impl for Smithy Shapes.
pub(crate) fn schema_impl(shape_name: &Ident, schema_ident: &Ident) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl _StaticSchemaShape for #shape_name {
            fn schema() -> &'static _SchemaRef {
                &#schema_ident
            }
        }
    }
}
