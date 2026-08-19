use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::Type;

use crate::attr::StructMember;

/// Get reference for the core crate.
pub(crate) fn get_crate_ident() -> TokenStream {
    match &crate_name("smithy4rs-core").expect("smithy4rs-core is present in `Cargo.toml`") {
        FoundCrate::Itself => quote! { crate },
        FoundCrate::Name(name) => {
            let ident = Ident::new(name, Span::call_site());
            quote! { ::#ident }
        }
    }
}

/// Get the member schema scoped to its parent schema
pub(crate) fn member_schema(schema: &Ident, root_schema_ident: &Ident) -> Ident {
    Ident::new(
        &format!(
            "_{}_MEMBER_{}",
            root_schema_ident,
            schema.to_string().to_uppercase()
        ),
        Span::call_site(),
    )
}

#[allow(clippy::large_enum_variant)]
pub(crate) enum TargetType {
    /// A type that also implements `ShapeBuilder` and so must be wrapped with `MaybeBuilder<>`.
    Builable { shape: Type, builder: Type },
    /// A simple type (`string`, `i32`, etc.) that needs no additional wrapping.
    Primitive(Type),
}

// TODO: Refactor into a more sensible place once the base refactor is working
pub(crate) fn resolve_builder_target(member: &StructMember) -> TargetType {
    // The target type is the inner type of any optional
    // TODO: Re-assess. Logic seems redundant?
    let ty = if member.optional() {
        extract_option_type(&member.ty).unwrap_or(&member.ty)
    } else {
        &member.ty
    };

    // Get the inner type of parametrized types (i.e. `Vec<T>`, `IndexMap<String, T>`)
    let inner_type = get_inner_type(ty);

    // If the inner type is a primitive type, just return that
    if is_primitive(inner_type) || member.no_builder.is_present() {
        return TargetType::Primitive(ty.clone());
    }

    // We will create two target types. One with the builder
    // and the other with the "built" type.
    let mut builder_type = ty.clone();
    let type_ident = get_ident(inner_type);
    let builder_ident = Ident::new(&format!("{type_ident}Builder"), Span::call_site());
    replace_inner(&mut builder_type, builder_ident);

    // Create the build target for a `MaybeBuilt<>` impl
    TargetType::Builable {
        shape: ty.clone(),
        builder: builder_type.clone(),
    }
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
    panic!("Expected path type")
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
