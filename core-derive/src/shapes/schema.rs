use proc_macro2::TokenStream;
use quote::quote;

use crate::attr::Shape;

/// Generates `StaticSchemaShape` impl for Smithy Shapes.
pub(crate) fn expand_schema(shape: &Shape, crate_ident: &TokenStream) -> TokenStream {
    let name = shape.name();
    let schema = shape.schema();

    quote! {
        const _: () = {
            use #crate_ident::schema::Schema as _Schema;
            use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;

            #[automatically_derived]
            impl _StaticSchemaShape for #name {
                #[inline]
                fn schema() -> &'static _Schema {
                    &#schema
                }
            }
        };
    }
}
