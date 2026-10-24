use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote};

use crate::attr::{
    EnumShape, EnumValue, Shape, SimpleShape, StructMember, StructShape, UnionShape, UnionVariant,
};

/// Generates an implementation of the [`Document`] trait for generated
/// Smithy shapes.
pub(crate) fn expand_document(shape: &Shape, crate_ident: &TokenStream) -> TokenStream {
    match shape {
        Shape::Struct(s) => struct_document_impl(s, crate_ident),
        Shape::Simple(s) => wrapper_document_impl(s, crate_ident),
        Shape::Union(u) => union_document_impl(u, crate_ident),
        Shape::Enum(e) => enum_document_impl(e, crate_ident),
    }
}

// ============================================================================
// Structure Documents
// ============================================================================

fn struct_document_impl(shape: &StructShape, crate_ident: &TokenStream) -> TokenStream {
    let ident = &shape.ident;
    let field_data = shape.data.as_struct().expect("Must be a struct");

    let get_branch = field_data.iter().map(get_match_arm);
    let entry_branch = field_data.iter().enumerate().map(entry_match_arm);
    let entry_mut_branch = field_data.iter().enumerate().map(entry_mut_match_arm);
    let key_branch = field_data.iter().enumerate().map(key_match_arm);
    let len = field_data.len();
    let mut base = map_base(ident, crate_ident);

    let map_accessor = if len == 0 {
        empty_struct_impl(ident, crate_ident)
    } else {
        quote! {
            const _: () = {
                use #crate_ident::features::documents::MapAccessor as _MapAccessor;
                use #crate_ident::features::documents::Document as _Document;
                use #crate_ident::features::documents::DocumentError as _DocumentError;
                use #crate_ident::schema::ShapeId as _ShapeId;
                use #crate_ident::features::documents::DocumentType as _DocumentType;
                use #crate_ident::Cow as _Cow;

                impl _MapAccessor for #ident {
                    #[inline]
                    #[allow(trivial_casts)]
                    fn get(&self, key: &str) -> Option<&dyn _Document> {
                        match key {
                            #(#get_branch,)*
                            _ => None,
                        }
                    }

                    #[inline]
                    fn get_entry(&self, idx: usize) -> Option<(&str, &dyn _Document)> {
                        match idx {
                            #(#entry_branch,)*
                            _ => None,
                        }
                    }

                    #[inline]
                    fn get_entry_mut(&mut self, index: usize) -> Result<(&str, &mut dyn _Document), _DocumentError> {
                        let out: (&str, &mut dyn _Document) = match index {
                            #(#entry_mut_branch,)*
                            _ => return Err(_DocumentError::CustomError("Invalid index".to_string())),
                        };
                        Ok(out)
                    }

                    #[inline]
                    #[allow(trivial_casts)]
                    fn get_key_mut(&mut self, index: usize) -> Option<_Cow<'_, str>> {
                        match index {
                            #(#key_branch,)*
                            _ => None,
                        }
                    }

                    #[inline]
                    fn len(&self) -> usize {
                        #len
                    }
                }
            };
        }
    };
    base.extend(map_accessor);
    base
}

fn empty_struct_impl(ident: &Ident, crate_ident: &TokenStream) -> TokenStream {
    quote! {
        const _: () = {
            use #crate_ident::features::documents::MapAccessor as _MapAccessor;
            use #crate_ident::features::documents::Document as _Document;
            use #crate_ident::features::documents::DocumentError as _DocumentError;
            use #crate_ident::Cow as _Cow;

            impl _MapAccessor for #ident {
                #[inline]
                #[allow(trivial_casts)]
                fn get(&self, key: &str) -> Option<&dyn _Document> {
                    None
                }

                #[inline]
                fn get_entry(&self, idx: usize) -> Option<(&str, &dyn _Document)> {
                    None
                }

                #[inline]
                fn get_entry_mut(&mut self, index: usize) -> Result<(&str, &mut dyn _Document), _DocumentError> {
                    Err(_DocumentError::CustomError("Invalid index".to_string()))
                }

                #[inline]
                #[allow(trivial_casts)]
                fn get_key_mut(&mut self, index: usize) -> Option<_Cow<'_, str>> {
                    None
                }

                #[inline]
                fn len(&self) -> usize {
                    0
                }
            }
        };
    }
}

fn get_match_arm(member: &StructMember) -> TokenStream {
    let member_ident = member.ident.as_ref().expect("member ident");
    let member_name = member.member_name();
    if member.optional() {
        quote! {
            #member_name => self.#member_ident.as_ref().map(|v| v as &dyn _Document)
        }
    } else {
        quote! {
            #member_name => Some(&self.#member_ident)
        }
    }
}

fn entry_match_arm(input: (usize, &StructMember)) -> TokenStream {
    let index = input.0;
    let member = input.1;
    let member_ident = member.ident.as_ref().expect("member ident");
    let member_name = member.member_name();
    quote! {
        #index => Some((#member_name, &self.#member_ident))
    }
}

fn entry_mut_match_arm(input: (usize, &StructMember)) -> TokenStream {
    let index = input.0;
    let member = input.1;
    let member_ident = member.ident.as_ref().expect("member ident");
    let member_name = member.member_name();
    quote! {
        #index => (#member_name, &mut self.#member_ident)
    }
}

fn key_match_arm(input: (usize, &StructMember)) -> TokenStream {
    let index = input.0;
    let member_name = input.1.member_name();
    quote! {
        #index => Some(_Cow::Borrowed(#member_name))
    }
}

// ============================================================================
// Tuple (Wrapper) struct Documents
// ============================================================================

fn wrapper_document_impl(simple_shape: &SimpleShape, crate_ident: &TokenStream) -> TokenStream {
    let ident = &simple_shape.ident;
    quote! {
        const _: () = {
            #crate_ident::wrapped_document!(#ident);
        };
    }
}

// ============================================================================
// Union Documents
// ============================================================================

fn union_document_impl(shape: &UnionShape, crate_ident: &TokenStream) -> TokenStream {
    let ident = &shape.ident;
    let unknown = Ident::new("Unknown", Span::call_site());
    let variants: Vec<&UnionVariant> = shape
        .data
        .as_enum()
        .expect("Union variants")
        .iter()
        .filter(|v| v.ident != unknown)
        .collect();
    let len = variants.len();
    let get_branch = variants
        .iter()
        .map(|v| union_get_branch(ident, v, crate_ident));
    let entry_branch = variants
        .iter()
        .enumerate()
        .map(|(idx, variant)| union_entry_branch(idx, ident, variant, crate_ident));

    let entry_mut_branch = variants
        .iter()
        .enumerate()
        .map(|(idx, variant)| union_entry_mut_branch(idx, ident, variant, crate_ident));
    let get_key_branch = variants
        .iter()
        .enumerate()
        .map(|(idx, variant)| union_key_mut_branch(idx, ident, variant, crate_ident));

    let mut base = map_base(ident, crate_ident);
    let doc_impl = quote! {
        const _: () = {
            use #crate_ident::features::documents::MapAccessor as _MapAccessor;
            use #crate_ident::features::documents::Document as _Document;
            use #crate_ident::features::documents::DocumentError as _DocumentError;
            use #crate_ident::Cow as _Cow;

            impl _MapAccessor for #ident {
                #[inline]
                #[allow(trivial_casts)]
                fn get(&self, key: &str) -> Option<&dyn _Document> {
                    match self {
                        #(#get_branch,)*
                        _ => None,
                    }
                }

                #[inline]
                fn get_entry(&self, idx: usize) -> Option<(&str, &dyn _Document)> {
                    match self {
                        #(#entry_branch,)*
                        _ => None,
                    }
                }

                #[inline]
                fn get_entry_mut(&mut self, index: usize) -> Result<(&str, &mut dyn _Document), _DocumentError> {
                    match self {
                        #(#entry_mut_branch,)*
                        _ => Err(_DocumentError::CustomError("Invalid index".to_string()))
                    }
                }

                #[inline]
                #[allow(trivial_casts)]
                fn get_key_mut(&mut self, index: usize) -> Option<_Cow<'_, str>> {
                    match self {
                        #(#get_key_branch,)*
                        _ => None
                    }
                }

                #[inline]
                fn len(&self) -> usize {
                    #len
                }
            }
        };
    };
    base.extend(doc_impl);
    base
}

fn union_get_branch(
    root: &Ident,
    variant: &UnionVariant,
    crate_ident: &TokenStream,
) -> TokenStream {
    let member_name = variant.member_name();
    let var = &variant.ident;
    if variant.fields.is_empty() {
        // Empty variants return Unit type (empty object)
        quote! {
            #root::#var if key == #member_name => Some(&#crate_ident::schema::STATIC_UNIT)
        }
    } else {
        quote! {
            #root::#var(value) if key == #member_name => Some(value)
        }
    }
}

fn union_entry_branch(
    idx: usize,
    root: &Ident,
    variant: &UnionVariant,
    crate_ident: &TokenStream,
) -> TokenStream {
    let member_name = variant.member_name();
    let var = &variant.ident;
    if variant.fields.is_empty() {
        // Empty variants return Unit type (empty object)
        quote! {
            #root::#var if idx == #idx => Some((#member_name, &#crate_ident::schema::STATIC_UNIT))
        }
    } else {
        quote! {
            #root::#var(value) if idx == #idx => Some((#member_name, value))
        }
    }
}

fn union_entry_mut_branch(
    idx: usize,
    root: &Ident,
    variant: &UnionVariant,
    crate_ident: &TokenStream,
) -> TokenStream {
    let member_name = variant.member_name();
    let var = &variant.ident;
    if variant.fields.is_empty() {
        // Empty variants return Unit type (empty object)
        quote! {
            #root::#var if index == #idx => Ok((#member_name, Box::leak(Box::new(#crate_ident::schema::Unit))))
        }
    } else {
        quote! {
            #root::#var(value) if index == #idx => Ok((#member_name, value))
        }
    }
}

fn union_key_mut_branch(
    idx: usize,
    root: &Ident,
    variant: &UnionVariant,
    crate_ident: &TokenStream,
) -> TokenStream {
    let member_name = variant.member_name();
    let var = &variant.ident;
    if variant.fields.is_empty() {
        quote! {
            #root::#var if index == #idx => Some(#crate_ident::Cow::Borrowed(#member_name))
        }
    } else {
        quote! {
            #root::#var(_) if index == #idx => Some(#crate_ident::Cow::Borrowed(#member_name))
        }
    }
}

// ============================================================================
// Enum and IntEnum Documents
// ============================================================================

fn enum_document_impl(enum_shape: &EnumShape, crate_ident: &TokenStream) -> TokenStream {
    let ident = &enum_shape.ident;
    let variants = enum_shape.data.as_enum().expect("Must be a enum");
    let first = variants.first().expect("At least one variant");
    let impls = match &first.value {
        Some(EnumValue::Str(_)) => enum_impls(enum_shape),
        Some(EnumValue::Int(_)) => int_enum_impls(enum_shape, crate_ident),
        _ => panic!("Unexpected enum value"),
    };

    quote! {
        const _: () = {
            use #crate_ident::features::documents::Document as _Document;
            use #crate_ident::features::documents::DocumentError as _DocumentError;
            use #crate_ident::features::documents::DocumentType as _DocumentType;
            use #crate_ident::schema::Schema as _Schema;
            use #crate_ident::schema::ShapeId as _ShapeId;
            use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;

            impl _Document for #ident {
                #[inline]
                fn discriminant(&self) -> Option<&_ShapeId> {
                    Some(<#ident as _StaticSchemaShape>::schema().id())
                }

                #impls
            }
        };
    }
}

fn enum_impls(shape: &EnumShape) -> TokenStream {
    let ident = &shape.ident;
    let variants = shape.data.as_enum().expect("Must be a enum");
    let unknown = syn::parse_str::<Ident>("Unknown").unwrap();
    let variant = variants.iter().map(|v| &v.ident).filter(|i| **i != unknown);
    let value = variants.iter().map(|v| &v.value);
    let value_take = value.clone();
    let variant_take = variant.clone();

    quote! {
        #[inline]
        fn document_type(&self) -> _DocumentType {
            _DocumentType::String
        }

        #[inline]
        fn as_string(&self) -> Option<&str> {
            let out = match self {
                #(#ident::#variant => #value,)*
                #ident::Unknown(value) => value.as_str()
            };
            Some(out)
        }

        #[inline]
        fn take_string(&mut self, _schema: &_Schema) -> Result<String, _DocumentError> {
            let out = match self {
                #(#ident::#variant_take => #value_take.to_string(),)*
                #ident::Unknown(value) => std::mem::take(value)
            };
            Ok(out)
        }
    }
}

fn int_enum_impls(shape: &EnumShape, crate_ident: &TokenStream) -> TokenStream {
    let ident = &shape.ident;
    let variants = shape.data.as_enum().expect("Must be a enum");
    let unknown = syn::parse_str::<Ident>("Unknown").unwrap();
    let variant = variants.iter().map(|v| &v.ident).filter(|i| **i != unknown);
    let value = variants.iter().map(|v| &v.value);
    let value_take = value.clone();
    let variant_take = variant.clone();

    quote! {
        #[inline]
        fn document_type(&self) -> _DocumentType {
            _DocumentType::Number(
                #crate_ident::features::documents::Number::Integer
            )
        }

        #[inline]
        fn as_integer(&self) -> Option<i32> {
            let out = match self {
                #(#ident::#variant => #value,)*
                #ident::Unknown(value) => *value
            };
            Some(out)
        }

        #[inline]
        fn take_integer(&mut self, _schema: &_Schema) -> Result<i32, _DocumentError> {
            let out = match self {
                #(#ident::#variant_take => #value_take,)*
                #ident::Unknown(value) => std::mem::take(value)
            };
            Ok(out)
        }
    }
}

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
            use #crate_ident::features::documents::Document as _Document;
            use #crate_ident::features::documents::DocumentError as _DocumentError;
            use #crate_ident::features::documents::TryFromDocument as _TryFromDocument;

            impl _TryFromDocument for #name {
                #[inline]
                fn try_from<'a>(document: &mut (dyn _Document + 'a)) -> Result<Self, _DocumentError> {
                    Ok(<#builder as _TryFromDocument>::try_from(document)?.build()?)
                }
            }
        };
    }
}

fn map_base(ident: &Ident, crate_ident: &TokenStream) -> TokenStream {
    quote! {
        const _: () = {
            use #crate_ident::features::documents::MapAccessor as _MapAccessor;
            use #crate_ident::features::documents::Document as _Document;
            use #crate_ident::features::documents::DocumentError as _DocumentError;
            use #crate_ident::schema::ShapeId as _ShapeId;
            use #crate_ident::features::documents::DocumentType as _DocumentType;
            use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;

            impl _Document for #ident {
                #[inline]
                fn discriminant(&self) -> Option<&_ShapeId> {
                    Some(<Self as _StaticSchemaShape>::schema().id())
                }

                #[inline]
                fn document_type(&self) -> _DocumentType {
                    _DocumentType::Map
                }

                #[inline]
                fn as_map(&self) -> Option<&dyn _MapAccessor> {
                    Some(self)
                }

                #[inline]
                fn take_map(&mut self) -> Result<&mut dyn _MapAccessor, _DocumentError> {
                    Ok(self)
                }
            }
        };
    }
}
