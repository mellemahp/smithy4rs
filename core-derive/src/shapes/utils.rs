use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{__private::TokenStream2, Attribute, DataEnum, Expr, Lit, Type};

/// Parses out attribute data for the `smithy_schema` macro attribute from the struct and
/// its fields.
pub(crate) fn parse_schema(attrs: &[Attribute]) -> Ident {
    let mut target_schema = None;
    for attr in attrs {
        if attr.path().is_ident("smithy_schema") {
            target_schema = Some(
                attr.parse_args::<Ident>()
                    .expect("`smithy_schema` attribute should be an identifier"),
            );
        }
    }
    target_schema.expect("Could not find `smithy_schema` attribute")
}

/// Determine if a type is an `Option<T>`
pub(crate) fn is_optional(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            let idents_of_path = path.segments.iter().fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push(':');
                acc
            });
            // Figure out if the type is optional
            // TODO(sparse list): Might erroneously detect optionals in sparse lists or maps
            vec!["Option:", "std:option:Option:", "core:option:Option:"]
                .into_iter()
                .any(|s| idents_of_path == *s)
        }
        _ => panic!("Serde can only be derived for resolvable types"),
    }
}

/// Get the inner type of `Option<T>` if possible.
///
/// If the type is not optional, then `None` is returned.
pub(crate) fn extract_option_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return Some(inner_ty);
    }
    None
}

/// Get the inner type of generic type signature.
///
/// This could be an `Option<T>` or a `Map<String, T>` or `Vec<T>` type or nested versions thereof.
pub(crate) fn get_inner_type(ty: &Type) -> &Type {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.last()
    {
        return get_inner_type(inner_ty);
    }
    ty
}

/// Get references for the core crate.
pub(crate) fn get_crate_info() -> (TokenStream, TokenStream) {
    let found_crate =
        crate_name("smithy4rs-core").expect("smithy4rs-core is present in `Cargo.toml`");
    let extern_import = match &found_crate {
        FoundCrate::Itself => quote!(),
        FoundCrate::Name(name) => {
            let ident = Ident::new(name, Span::call_site());
            quote! {
                extern crate #ident as _smithy4rs;
            }
        }
    };
    let crate_ident = match &found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(_) => {
            let ident = Ident::new("_smithy4rs", Span::call_site());
            quote!( #ident )
        }
    };
    (extern_import, crate_ident)
}

/// Get identifier to use outside `const` block for crate
pub(crate) fn get_crate_ident() -> TokenStream {
    let found_crate =
        crate_name("smithy4rs-core").expect("smithy4rs-core is present in `Cargo.toml`");
    match &found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(_) => {
            let ident = Ident::new("smithy4rs_core", Span::call_site());
            quote!( #ident )
        }
    }
}

/// Checks if a type is a Smithy data model primitive.
pub(crate) fn is_primitive(field_ty: &Type) -> bool {
    if let Type::Path(type_path) = field_ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "String"
            || segment.ident == "bool"
            || segment.ident == "i8"
            || segment.ident == "i16"
            || segment.ident == "i32"
            || segment.ident == "i64"
            || segment.ident == "f32"
            || segment.ident == "f64"
            || segment.ident == "BigInt"
            || segment.ident == "BigDecimal"
            || segment.ident == "Instant"
            || segment.ident == "Document"
            || segment.ident == "ByteBuffer";
    }
    false
}

pub(crate) fn replace_inner(field_ty: &mut Type, replacement: Ident) {
    let inner = get_inner_mut(field_ty);
    if let Type::Path(type_path) = inner
        && let Some(segment) = type_path.path.segments.last_mut()
    {
        segment.ident = replacement;
    }
}
fn get_inner_mut(ty: &mut Type) -> &mut Type {
    if let Type::Path(type_path) = &ty
        && let Some(segment) = type_path.path.segments.last()
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(_)) = args.args.last()
    {
        get_inner_mut(expect_inner_mut(ty))
    } else {
        ty
    }
}

fn expect_inner_mut(ty: &mut Type) -> &mut Type {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last_mut()
        && let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.last_mut()
    {
        inner_ty
    } else {
        panic!("Expeccted to be able to extract mutable inner type")
    }
}

pub(crate) fn get_ident(ty: &Type) -> &Ident {
    if let Type::Path(type_path) = ty {
        return &type_path.path.segments.last().unwrap().ident;
    }
    panic!("Expected to get ident")
}

/// Parse an `#[enum_value(...)` attribute
pub(crate) fn parse_enum_value(attrs: &[Attribute]) -> Option<Lit> {
    let mut value = None;
    for attr in attrs {
        if attr.path().is_ident("enum_value") {
            value = Some(
                attr.parse_args::<Lit>()
                    .expect("`enum_value` attribute should be an identifier"),
            );
        }
    }
    value
}

pub(crate) fn get_builder_ident(shape_name: &Ident) -> Ident {
    Ident::new(&format!("{}Builder", shape_name), Span::call_site())
}

/// Determines if the shape should be treated as a regular enum or a union.
///
/// Union's have member schemas for their variants.
pub(crate) fn is_union(data_enum: &DataEnum) -> bool {
    data_enum
        .variants
        .first()
        .expect("Enum must have at least one variant")
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("smithy_schema"))
}

/// Parses out attribute data for the `smithy_schema` macro attribute from the struct and
/// its fields.
pub(crate) fn parse_default(attrs: &[Attribute]) -> Option<IdentOrExpr> {
    let mut default = None;
    for attr in attrs {
        if attr.path().is_ident("default") {
            if let Ok(ident) = attr.parse_args::<Ident>() {
                default = Some(IdentOrExpr::Ident(ident));
            }
            if let Ok(expr) = attr.parse_args::<Expr>() {
                default = Some(IdentOrExpr::Expr(expr));
            }
        }
    }
    default
}

pub(crate) enum IdentOrExpr {
    Ident(Ident),
    Expr(Expr),
}
impl ToTokens for IdentOrExpr {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            IdentOrExpr::Ident(ident) => ident.to_tokens(tokens),
            IdentOrExpr::Expr(expr) => expr.to_tokens(tokens),
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::Type;

    use super::*;

    #[test]
    fn is_optional_test() {
        let optional_simple = syn::parse_str::<Type>("Option<A>").unwrap();
        let not_optional = syn::parse_str::<Type>("Other<B>").unwrap();
        let optional_nested = syn::parse_str::<Type>("Option<Vec<Vec<B>>>").unwrap();
        assert!(is_optional(&optional_simple));
        assert!(!is_optional(&not_optional));
        assert!(is_optional(&optional_nested));
    }

    #[test]
    fn extract_option_simple() {
        let optional_simple = syn::parse_str::<Type>("Option<A>").unwrap();
        let expected_type = syn::parse_str::<Type>("A").unwrap();
        assert_eq!(extract_option_type(&optional_simple), Some(&expected_type));
    }

    #[test]
    fn extract_non_option() {
        let ty = syn::parse_str::<Type>("A").unwrap();
        assert_eq!(extract_option_type(&ty), None);
    }

    #[test]
    fn extract_option_vec() {
        let optional_simple = syn::parse_str::<Type>("Option<Vec<A>>").unwrap();
        let expected_type = syn::parse_str::<Type>("Vec<A>").unwrap();
        assert_eq!(extract_option_type(&optional_simple), Some(&expected_type));
    }

    #[test]
    fn extract_option_map() {
        let optional_simple = syn::parse_str::<Type>("Option<IndexMap<String, A>>").unwrap();
        let expected_type = syn::parse_str::<Type>("IndexMap<String, A>").unwrap();
        assert_eq!(extract_option_type(&optional_simple), Some(&expected_type));
    }

    #[test]
    fn extract_nested_collections() {
        let optional_simple = syn::parse_str::<Type>("Option<Vec<IndexMap<String, A>>>").unwrap();
        let expected_type = syn::parse_str::<Type>("Vec<IndexMap<String, A>>").unwrap();
        assert_eq!(extract_option_type(&optional_simple), Some(&expected_type));
    }

    #[test]
    fn inner_type_of_simple() {
        let simple = syn::parse_str::<Type>("A").unwrap();
        assert_eq!(get_inner_type(&simple), &simple);
    }

    #[test]
    fn inner_type_of_vec() {
        let vec_simple = syn::parse_str::<Type>("Vec<A>").unwrap();
        let expected_type = syn::parse_str::<Type>("A").unwrap();
        assert_eq!(get_inner_type(&vec_simple), &expected_type);
    }

    #[test]
    fn inner_type_of_map() {
        let map_simple = syn::parse_str::<Type>("IndexMap<String, A>").unwrap();
        let expected_type = syn::parse_str::<Type>("A").unwrap();
        assert_eq!(get_inner_type(&map_simple), &expected_type);
    }

    #[test]
    fn inner_type_of_nested_list() {
        let vec_nested = syn::parse_str::<Type>("Vec<Vec<Vec<A>>>").unwrap();
        let expected_type = syn::parse_str::<Type>("A").unwrap();
        assert_eq!(get_inner_type(&vec_nested), &expected_type);
    }

    #[test]
    fn inner_type_of_nested_map_of_list() {
        let vec_nested = syn::parse_str::<Type>("Map<String, Vec<Vec<A>>>").unwrap();
        let expected_type = syn::parse_str::<Type>("A").unwrap();
        assert_eq!(get_inner_type(&vec_nested), &expected_type);
    }

    #[test]
    fn inner_type_of_nested_map_of_maps() {
        let vec_nested =
            syn::parse_str::<Type>("Map<String, Map<String, Map<String, A>>>").unwrap();
        let expected_type = syn::parse_str::<Type>("A").unwrap();
        assert_eq!(get_inner_type(&vec_nested), &expected_type);
    }

    #[test]
    fn is_primitive_test() {
        let primitive = syn::parse_str::<Type>("String").unwrap();
        let not_primitive = syn::parse_str::<Type>("B").unwrap();
        let primitive_with_qualified_type = syn::parse_str::<Type>("smithy4rs::Instant").unwrap();
        assert!(is_primitive(&primitive));
        assert!(!is_primitive(&not_primitive));
        assert!(is_primitive(&primitive_with_qualified_type));
    }

    #[test]
    fn replaces_inner_list() {
        let mut list = syn::parse_str::<Type>("Vec<A>").unwrap();
        let replacement = syn::parse_str::<Ident>("B").unwrap();
        let expected = syn::parse_str::<Type>("Vec<B>").unwrap();
        replace_inner(&mut list, replacement);
        assert_eq!(list, expected);
    }

    #[test]
    fn replaces_inner_nested_list() {
        let mut list = syn::parse_str::<Type>("Vec<Vec<A>>").unwrap();
        let replacement = syn::parse_str::<Ident>("B").unwrap();
        let expected = syn::parse_str::<Type>("Vec<Vec<B>>").unwrap();
        replace_inner(&mut list, replacement);
        assert_eq!(list, expected);
    }

    #[test]
    fn replaces_inner_map() {
        let mut list = syn::parse_str::<Type>("IndexMap<String, A>").unwrap();
        let replacement = syn::parse_str::<Ident>("B").unwrap();
        let expected = syn::parse_str::<Type>("IndexMap<String, B>").unwrap();
        replace_inner(&mut list, replacement);
        assert_eq!(list, expected);
    }

    #[test]
    fn replaces_nested_map() {
        let mut list = syn::parse_str::<Type>("IndexMap<String, IndexMap<String, A>>").unwrap();
        let replacement = syn::parse_str::<Ident>("B").unwrap();
        let expected = syn::parse_str::<Type>("IndexMap<String, IndexMap<String, B>>").unwrap();
        replace_inner(&mut list, replacement);
        assert_eq!(list, expected);
    }
}
