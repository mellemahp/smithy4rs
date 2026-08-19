use proc_macro2::{Ident, Span};
use syn::{Attribute, Expr, Fields, ItemEnum, Lit, MetaNameValue, Variant, parse_quote};

/// Convert discriminants to `[#enum_value]` attributes
pub(crate) fn discriminants_to_attributes(enum_data: &mut ItemEnum) {
    // Change all discriminants to attributes for consistency
    for variant in enum_data.variants.iter_mut() {
        if let Some((_, expr)) = &variant.discriminant {
            variant
                .attrs
                .push(parse_quote!(#[enum_value(value = #expr)]));
            variant.discriminant = None;
        };
    }
}

/// Adds an `Unknown` variant for enum-like shapes (enums, int-enums and unions).
pub(crate) fn unknown_variant(enum_data: &mut ItemEnum) {
    // Determine if unknown should store string or int. Unions (without `enum_value` attr)
    // will default tousing String as unknown data value.
    let value = parse_enum_value(
        enum_data
            .variants
            .first()
            .expect("Expected at least one variant")
            .attrs
            .as_slice(),
    );

    let field = match &value {
        Some(Lit::Str(_)) => parse_quote!((String)),
        Some(Lit::Int(_)) => parse_quote!((i32)),
        // Default to String for unions!
        _ => parse_quote!((String)),
    };

    enum_data.variants.push(Variant {
        attrs: vec![
            parse_quote!(#[automatically_derived]),
            parse_quote!(#[doc(hidden)]),
        ],
        discriminant: None,
        fields: Fields::Unnamed(field),
        ident: Ident::new("Unknown", Span::call_site()),
    });
}

/// Parse an `#[enum_value(...)` attribute
fn parse_enum_value(attrs: &[Attribute]) -> Option<Lit> {
    for attr in attrs {
        if attr.path().is_ident("enum_value")
            && let Ok(nv) = attr.parse_args::<MetaNameValue>()
            && nv.path.is_ident("value")
            && let Expr::Lit(expr_lit) = nv.value
        {
            return Some(expr_lit.lit);
        }
    }
    None
}
