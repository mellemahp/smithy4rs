use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, LitStr};

/// Generates `StaticTraitId` impl for Smithy Shapes.
pub(crate) fn static_trait_id_impl(trait_name: &Ident, attrs: &[Attribute]) -> TokenStream {
    let id = parse_trait_id(attrs);
    quote! {
        static ID: _LazyLock<_ShapeId> = _LazyLock::new(|| _ShapeId::from(#id));
        
        #[automatically_derived]
        impl _StaticTraitId for #trait_name {
            #[inline]
            fn trait_id() -> &'static _ShapeId {
                &ID
            }
        }
    }
}

/// Parses out attribute data for the `trait_id` macro attribute
fn parse_trait_id(attrs: &[Attribute]) -> LitStr {
    let mut target_schema = None;
    for attr in attrs {
        if attr.path().is_ident("smithy_trait_id") {
            target_schema = Some(
                attr.parse_args::<LitStr>()
                    .expect("`smithy_trait_id` attribute should be a string literal"),
            );
        }
    }
    target_schema.expect("Could not find `smithy_trait_id` attribute")
}