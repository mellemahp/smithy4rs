#![allow(unused_qualifications, clippy::option_if_let_else)]

//! Definition of the `schema` attribute
//!
//! `darling` is used to auto-derive and parse the attribute.

use std::cell::OnceCell;

use darling::{
    Error, FromDeriveInput, FromField, FromMeta, FromVariant,
    ast::{Data, Fields},
    util::{Flag, Ignored, Override},
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::{DeriveInput, Expr, Lit, LitInt, LitStr};

use crate::utils::is_optional;

/// Entry point for the `SmithyShape` derive macro
///
/// Target shapes MUST have the `schema` attribute applied.
#[derive(Debug)]
pub enum Shape {
    Struct(StructShape),
    Simple(SimpleShape),
    Union(UnionShape),
    Enum(EnumShape),
}
impl Shape {
    pub fn name(&self) -> &Ident {
        match &self {
            Shape::Struct(StructShape { ident, .. }) => ident,
            Shape::Simple(SimpleShape { ident, .. }) => ident,
            Shape::Union(UnionShape { ident, .. }) => ident,
            Shape::Enum(EnumShape { ident, .. }) => ident,
        }
    }

    pub fn schema(&self) -> &Ident {
        match &self {
            Shape::Struct(StructShape { schema, .. }) => schema,
            Shape::Simple(SimpleShape { schema, .. }) => schema,
            Shape::Union(UnionShape { schema, .. }) => schema,
            Shape::Enum(EnumShape { schema, .. }) => schema,
        }
    }
}

impl FromDeriveInput for Shape {
    fn from_derive_input(input: &DeriveInput) -> darling::Result<Self> {
        match &input.data {
            syn::Data::Struct(data_struct) => match &data_struct.fields {
                syn::Fields::Named(_) => StructShape::from_derive_input(input).map(Shape::Struct),
                syn::Fields::Unnamed(tuple_fields) => {
                    if tuple_fields.unnamed.len() != 1 {
                        return Err(darling::Error::unsupported_shape(
                            "Wrappers must have exactly one unnamed field",
                        )
                        .with_span(&input.ident));
                    }
                    SimpleShape::from_derive_input(input).map(Shape::Simple)
                }
                syn::Fields::Unit => Err(darling::Error::unsupported_shape(
                    "Unit structs are not supported",
                )
                .with_span(&input.ident)),
            },
            syn::Data::Enum(_) => {
                let has_union_attr = input
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("smithy_union_enum"));
                if has_union_attr {
                    UnionShape::from_derive_input(input).map(Shape::Union)
                } else {
                    EnumShape::from_derive_input(input).map(Shape::Enum)
                }
            }
            syn::Data::Union(_) => Err(darling::Error::unsupported_shape(
                "rust unions are not supported",
            )
            .with_span(&input.ident)),
        }
    }
}

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(schema), supports(struct_named))]
pub struct StructShape {
    pub ident: Ident,
    pub schema: Ident,
    pub data: Data<Ignored, StructMember>,
    #[darling(skip)]
    builder_ident: OnceCell<Ident>,
}

impl StructShape {
    pub fn builder(&self) -> &Ident {
        self.builder_ident
            .get_or_init(|| Ident::new(&format!("{}Builder", self.ident), Span::call_site()))
    }
}

/// Handle a Smithy Struct memeber field
#[derive(FromField, Debug)]
#[darling(attributes(schema))]
pub struct StructMember {
    pub(crate) ident: Option<Ident>,
    pub(crate) ty: syn::Type,

    /// Member schema to use
    pub(crate) schema: Ident,

    /// Optional expression that sets the default for the field
    #[darling(default)]
    pub(crate) default: Option<Override<Expr>>,

    /// Optional flag to indicate that type has no builder
    /// TODO: We should be able to completely remove this
    pub(crate) no_builder: Flag,
}

impl StructMember {
    pub fn optional(&self) -> bool {
        is_optional(&self.ty)
    }
}

/// Simple shape types that are represented with a transparent wrapper types.
///
/// For example, string, documents, maps, lists, numbers, etc
#[derive(FromDeriveInput, Debug)]
#[darling(attributes(schema), supports(struct_tuple))]
pub struct SimpleShape {
    pub ident: Ident,
    pub schema: Ident,
    pub data: Data<Ignored, WrappedField>,
}
impl SimpleShape {
    /// Gets the wrapped inner type
    pub fn inner_type(&self) -> &WrappedField {
        self.data
            .as_struct()
            .expect("Simple shapes should only ever be tuple structs")
            .fields
            .first()
            .expect("Wrapper types should only ever have one field")
    }
}

#[derive(FromField, Debug)]
pub struct WrappedField {
    ty: syn::Type,
}
impl ToTokens for WrappedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ty.to_tokens(tokens)
    }
}

/// A Smithy Union shape definition
///
/// NOTE: Unions are differentiated from other enum types by a `#[smithy_union_enum]`
/// attribute.
#[derive(FromDeriveInput, Debug)]
#[darling(attributes(schema, union), supports(enum_any))]
pub struct UnionShape {
    pub ident: Ident,
    #[darling(rename = "schema")]
    pub schema: Ident,

    pub data: Data<UnionVariant, Ignored>,
}

#[derive(FromVariant, Debug)]
#[darling(attributes(schema), supports(newtype, unit))]
pub struct UnionVariant {
    pub ident: Ident,
    #[darling(default)]
    pub schema: Option<Ident>,
    pub fields: Fields<WrappedField>,
}

/// A smithy Enum or `IntEnum` definition
///
/// Note: Users should make sure all enums have the `smithy_enum` attribute
/// macro applied. This will automatically add the `enum_value` attr to variants.
#[derive(FromDeriveInput, Debug)]
#[darling(supports(enum_any), attributes(schema))]
pub struct EnumShape {
    pub ident: Ident,
    pub schema: Ident,

    pub data: Data<EnumVariant, Ignored>,
}
impl EnumShape {
    pub fn is_string(&self) -> bool {
        let first = &self
            .data
            .as_enum()
            .expect("Enum variant")
            .first()
            .expect("At least one enum variant")
            .value;
        matches!(first, Some(EnumValue::Str(_)))
    }
}

#[derive(FromVariant, Debug)]
#[darling(attributes(schema), supports(unit, newtype))]
pub struct EnumVariant {
    pub ident: Ident,
    #[darling(default)]
    pub value: Option<EnumValue>,

    #[allow(unused)]
    fields: Fields<Ignored>,
}

/// Value used for the enum variant.
///
/// This allows us to support smithy string and int enums
#[derive(Debug)]
pub enum EnumValue {
    /// String Enum value
    Str(LitStr),
    /// Int Enum value
    Int(LitInt),
}
impl ToTokens for EnumValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            EnumValue::Str(str) => str.to_tokens(tokens),
            EnumValue::Int(int) => int.to_tokens(tokens),
        }
    }
}
impl FromMeta for EnumValue {
    fn from_value(value: &Lit) -> darling::Result<Self> {
        match value {
            Lit::Str(s) => Ok(EnumValue::Str(s.clone())),
            Lit::Int(i) => Ok(EnumValue::Int(i.clone())),
            _ => Err(Error::custom(
                "enum_value must be either a string or integer literal",
            )),
        }
    }
}
