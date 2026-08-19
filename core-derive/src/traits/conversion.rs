use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use crate::attr::Shape;

/// Implements [`TryFromDocument`] for trait shapes
pub(crate) fn expand_try_from_document(shape: &Shape, crate_ident: &TokenStream) -> TokenStream {
    let Shape::Struct(struct_shape) = shape else {
        // TODO: implement handling for other trait types!
        return "".to_token_stream();
    };
    let builder = struct_shape.builder();
    let name = &struct_shape.ident;

    quote! {
        const _: () = {
            use #crate_ident::schema::Document as _Document;
            use #crate_ident::schema::DocumentError as _DocumentError;
            use #crate_ident::schema::TryFromDocument as _TryFromDocument;

            impl _TryFromDocument for #name {
                fn try_from(document: Box<dyn _Document>) -> Result<Self, _DocumentError> {
                    Ok(<#builder as _TryFromDocument>::try_from(document)?.build()?)
                }
            }
        };
    }
}
