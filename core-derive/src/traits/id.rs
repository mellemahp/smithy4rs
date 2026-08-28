use proc_macro2::TokenStream;
use quote::quote;

use crate::attr::Shape;

/// Adds a [`StaticTraitId` implementation for a Smithy Shape
pub(crate) fn expand_trait_id(shape: &Shape, crate_ident: &TokenStream) -> TokenStream {
    let name = shape.name();

    quote! {
        const _: () = {
            use #crate_ident::schema::StaticTraitId as _StaticTraitId;
            use #crate_ident::schema::ShapeId as _ShapeId;
            use #crate_ident::LazyLock as _LazyLock;
            use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;

            impl _StaticTraitId for #name {
                #[inline]
                #[automatically_derived]
                fn trait_id() -> &'static _ShapeId {
                    static ID: _LazyLock<&_ShapeId> = _LazyLock::new(||
                        &<#name as _StaticSchemaShape>::schema().id()
                    );
                    *ID
                }
            }
        };
    }
}
