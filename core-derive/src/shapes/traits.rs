use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::shapes::utils::get_builder_ident;

pub(crate) fn get_static_trait_id_impl(
    crate_ident: &TokenStream,
    shape_name: &Ident,
) -> TokenStream {
    quote! {
        use #crate_ident::schema::StaticTraitId as _StaticTraitId;
        use #crate_ident::schema::ShapeId as _ShapeId;
        use #crate_ident::LazyLock as _LazyLock;
        use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;

        impl _StaticTraitId for #shape_name {
            #[inline]
            #[automatically_derived]
            fn trait_id() -> &'static _ShapeId {
                &<#shape_name as _StaticSchemaShape>::schema().id()
            }
        }
    }
}

pub(crate) fn get_try_from_document_impl(
    crate_ident: &TokenStream,
    shape_name: &Ident,
) -> TokenStream {
    let builder = get_builder_ident(shape_name);
    quote! {
        use #crate_ident::schema::Document as _Document;
        use #crate_ident::schema::DocumentError as _DocumentError;
        use #crate_ident::schema::TryFromDocument as _TryFromDocument;

        impl _TryFromDocument for #shape_name {
            fn try_from(document: Box<dyn _Document>) -> Result<Self, _DocumentError> {
                Ok(<#builder as _TryFromDocument>::try_from(document)?.build()?)
            }
        }
    }
}
