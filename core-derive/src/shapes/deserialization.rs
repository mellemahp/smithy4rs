use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::{
    attr::{
        EnumShape, EnumValue, Shape, SimpleShape, StructMember, StructShape, UnionShape,
        UnionVariant,
    },
    utils::{TargetType, member_schema, resolve_builder_target},
};

/// Generate `DeserializeWithSchema` implementation for Smithy Shapes
pub(crate) fn expand_deserialize_with_schema(
    shape: &Shape,
    crate_ident: &TokenStream,
) -> TokenStream {
    let mut imports = quote! {
        use #crate_ident::schema::Schema as _Schema;
        use #crate_ident::serde::deserializers::Deserializer as _Deserializer;
        use #crate_ident::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    };
    let mut inline_attr: TokenStream = quote! { #[inline] };
    let body = match shape {
        Shape::Struct(struct_shape) => {
            if struct_shape
                .data
                .as_struct()
                .expect("Must be a struct")
                .is_empty()
            {
                deserialize_unit(&struct_shape.ident)
            } else {
                // do not inline builders
                inline_attr = TokenStream::new();
                deserialize_builder(struct_shape, crate_ident, &mut imports)
            }
        }
        Shape::Simple(simple_shape) => deserialize_wrapper(simple_shape),
        Shape::Union(union_shape) => deserialize_union(union_shape, crate_ident, &mut imports),
        Shape::Enum(enum_shape) => deserialize_enum(enum_shape),
    };
    let shape_name = if let Shape::Struct(struct_shape) = &shape {
        struct_shape.builder()
    } else {
        shape.name()
    };

    quote! {
        const _: () = {
            #imports

            #[automatically_derived]
            impl<'de> _DeserializeWithSchema<'de> for #shape_name {
                #inline_attr
                fn deserialize_with_schema<D>(schema: &_Schema, deserializer: D) -> Result<Self, D::Error>
                where
                    D: _Deserializer<'de>,
                {
                    #body
                }
            }
        };
    }
}

// ============================================================================
// Builder (Union & Structure) Deserialization
// ============================================================================

/// Generate deserializer body for structure builder
fn deserialize_builder(
    shape: &StructShape,
    crate_ident: &TokenStream,
    imports: &mut TokenStream,
) -> TokenStream {
    let builder_name = shape.builder();
    let field_data = shape.data.as_struct().expect("Must be a struct");

    // Generate deserialize_member! or deserialize_optional_member! macro calls for each field
    let match_arms = field_data
        .iter()
        .map(|m| deserialize_match_arm(m, crate_ident, &shape.schema));

    // Builder-specific imports
    imports.extend(quote! {
        use #crate_ident::serde::correction::ErrorCorrection as _ErrorCorrection;
        use #crate_ident::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
        use #crate_ident::serde::Buildable as _Buildable;
        use #crate_ident::serde::deserializers::StructReader as _StructReader;
    });

    quote! {
        let mut builder = #builder_name::new();
        let mut reader = deserializer.read_struct(schema)?;

        while let Some(member_schema) = reader.read_member(schema)? {
            #(#match_arms)*
            // Known schema member but unknown to this code version (forward compat)
            reader.skip_value()?;
        }
        Ok(builder)
    }
}

/// Get the corresponding match arm for the builder field
pub(crate) fn deserialize_match_arm(
    member: &StructMember,
    crate_ident: &TokenStream,
    root_schema: &Ident,
) -> TokenStream {
    let field_name = member.ident.as_ref().expect("Should have named member");
    let schema = member_schema(&member.schema, root_schema);

    // Buildable fields use the `_builder` setter for deserialization
    // to take an unbuilt shape as input.
    match (member.optional(), resolve_builder_target(member)) {
        // === Optional types ===
        // For optional fields, use deserialize_optional_member! with inner type
        (true, TargetType::Primitive(ty)) => {
            quote! {
                #crate_ident::deserialize_optional_member!(member_schema, #schema, reader, builder, #field_name, #ty);
            }
        }
        (true, TargetType::Builable { builder, .. }) => {
            let field_builder =
                Ident::new(format!("{field_name}_builder").as_str(), Span::call_site());
            quote! {
                #crate_ident::deserialize_optional_member!(member_schema, #schema, reader, builder, #field_builder, #builder);
            }
        }
        // === Required types ===
        // For required fields, use deserialize_member!
        (false, TargetType::Primitive(ty)) => {
            quote! {
                #crate_ident::deserialize_member!(member_schema, #schema, reader, builder, #field_name, #ty);
            }
        }
        (false, TargetType::Builable { builder, .. }) => {
            let field_builder =
                Ident::new(format!("{field_name}_builder").as_str(), Span::call_site());
            quote! {
                #crate_ident::deserialize_member!(member_schema, #schema, reader, builder, #field_builder, #builder);
            }
        }
    }
}

// ============================================================================
// Tuple (Wrapper) struct Deserialization
// ============================================================================

fn deserialize_wrapper(shape: &SimpleShape) -> TokenStream {
    let inner_type = shape.inner_type();
    let name = &shape.ident;
    quote! {
        let inner = <#inner_type as _DeserializeWithSchema>::deserialize_with_schema(schema, deserializer)?;
        Ok(#name(inner))
    }
}

// ============================================================================
// Unit Deserialization
// ============================================================================

// TODO: Doest this make sense?
fn deserialize_unit(_shape_name: &Ident) -> TokenStream {
    quote! {
        //let _result = deserializer.read_struct(schema)?;
        Ok(Self {})
    }
}

// ============================================================================
// Enum Deserialization
// ============================================================================

fn deserialize_enum(shape: &EnumShape) -> TokenStream {
    let name = &shape.ident;
    let variants = shape.data.as_enum().expect("Must be a enum");
    let first = variants.first().expect("At least one variant");
    let method = match &first.value {
        Some(EnumValue::Str(_)) => Ident::new("read_string", Span::call_site()),
        Some(EnumValue::Int(_)) => Ident::new("read_integer", Span::call_site()),
        _ => panic!("Unexpected enum value"),
    };
    let match_val = match &first.value {
        Some(EnumValue::Str(_)) => quote! { val.as_str() },
        Some(EnumValue::Int(_)) => quote! { val },
        _ => panic!("Unexpected enum value"),
    };
    let unknown = syn::parse_str::<Ident>("Unknown").unwrap();

    let variant = variants.iter().map(|v| &v.ident).filter(|i| **i != unknown);

    let value = variants.iter().map(|v| &v.value);

    quote! {
        let val = deserializer.#method(schema)?;
        let result = match #match_val {
            #(#value => #name::#variant,)*
            _ => #name::Unknown(val)
        };
        Ok(result)
    }
}

// ============================================================================
// Union Deserialization
// ============================================================================

// TODO: Add support for capturing the unknown schema name! (enums already support)
fn deserialize_union(
    shape: &UnionShape,
    crate_ident: &TokenStream,
    imports: &mut TokenStream,
) -> TokenStream {
    imports.extend(quote! {
        use #crate_ident::serde::deserializers::Error as _;
        use #crate_ident::serde::deserializers::StructReader as _StructReader;
    });
    let data = shape.data.as_enum().expect("Union must be enum");
    if data.iter().any(|v| v.fields.is_empty()) {
        imports.extend(quote! {
            use #crate_ident::schema::Unit as _Unit;
        });
    }
    let name = &shape.ident;
    let unknown = syn::parse_str::<Ident>("Unknown").unwrap();
    let variants = data
        .iter()
        .filter(|v| v.ident != unknown)
        .map(|v| matcher(v, shape));

    quote! {
        let mut reader = deserializer.read_struct(schema)?;
        let mut result: Option<#name> = None;

        while let Some(member_schema) = reader.read_member(schema)? {
            if result.is_some() {
                return Err(D::Error::custom("Attempted to set union value twice"));
            }
            #(#variants)*
            // Known schema member but unknown to this code version (forward compat)
            result = Some(#name::Unknown("unknown".to_string()));
            continue;
        }

        result.ok_or(D::Error::custom("Failed to deserialize union"))
    }
}

fn matcher(variant: &UnionVariant, root: &UnionShape) -> TokenStream {
    let variant_name = &variant.ident;
    let member_schema_name = member_schema(variant.schema.as_ref().expect("member"), &root.schema);
    let shape_name = &root.ident;

    if variant.fields.is_empty() {
        quote! {
            if member_schema == *#member_schema_name {
                let _: _Unit = reader.read_value(member_schema)?;
                result = Some(#shape_name::#variant_name);
                continue;
            }
        }
    } else {
        // TODO: Re-raise error if multiple
        // TODO: Also we should probably move these match arms to rules macros
        let ty = variant
            .fields
            .fields
            .first()
            .expect("One field enum variant");
        quote! {
            if member_schema == *#member_schema_name {
                let value: #ty = reader.read_value(member_schema)?;
                result = Some(#shape_name::#variant_name(value));
                continue;
            }
        }
    }
}
