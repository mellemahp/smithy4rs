use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::{
    attr::{EnumShape, Shape, SimpleShape, StructMember, StructShape, UnionShape, UnionVariant},
    utils::member_schema,
};

/// Generates the `SerializeWithSchema` implementation for a shape.
pub(crate) fn expand_serialize_from_schema(
    shape: &Shape,
    crate_ident: &TokenStream,
) -> TokenStream {
    let mut imports = quote! {
        use #crate_ident::schema::Schema as _Schema;
        use #crate_ident::serde::serializers::Serializer as _Serializer;
        use #crate_ident::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    };
    let name = shape.name();
    let body = match shape {
        Shape::Struct(struct_shape) => {
            imports = quote! {
                #imports
                use #crate_ident::serde::serializers::StructWriter as _StructWriter;
            };
            serialize_struct(struct_shape)
        }
        Shape::Simple(simple_shape) => serialize_wrapper(simple_shape),
        Shape::Union(union_shape) => {
            imports = quote! {
                #imports
                use #crate_ident::serde::serializers::StructWriter as _StructWriter;
            };
            let has_unit = union_shape
                .data
                .as_enum()
                .expect("Union variants")
                .iter()
                .any(|v| v.fields.is_empty());
            if has_unit {
                imports = quote! {
                    #imports
                    use #crate_ident::schema::Unit as _Unit;
                };
            }
            serialize_union(union_shape)
        }
        Shape::Enum(enum_shape) => serialize_enum(enum_shape),
    };

    quote! {
        const _: () = {
            #imports

            #[automatically_derived]
            impl _SerializeWithSchema for #name {
                fn serialize_with_schema<S: _Serializer>(
                    &self,
                    schema: &_Schema,
                    serializer: S,
                ) -> Result<S::Ok, S::Error> {
                    #body
                }
            }
        };
    }
}

// ============================================================================
// Structure Serialization
// ============================================================================

/// Generates body of serialization impl for Structures
pub(crate) fn serialize_struct(shape: &StructShape) -> TokenStream {
    let fields = shape.data.as_struct().expect("Struct fields");
    let length = fields.len();

    // Now write the thing
    let method = fields.iter().map(member_method);
    let member_schema = fields
        .iter()
        .map(|v| member_schema(&v.schema, &shape.schema));
    let member_name = fields.iter().map(|d| &d.ident);
    // TODO: This needs to be the exact member name used in the schema. I think it might differ from the field name
    // in some cases
    let member_name_str = fields
        .iter()
        .map(|d| d.ident.as_ref().expect("Member name").to_string());
    quote! {
        let mut ser = serializer.write_struct(schema, #length)?;
        #(ser.#method(#member_name_str, &#member_schema, &self.#member_name)?;)*
        ser.end(schema)
    }
}

fn member_method(member: &StructMember) -> Ident {
    if member.optional() {
        Ident::new("write_optional_member_named", Span::call_site())
    } else {
        Ident::new("write_member_named", Span::call_site())
    }
}

fn serialize_wrapper(_shape: &SimpleShape) -> TokenStream {
    quote! {
        self.0.serialize_with_schema(schema, serializer)
    }
}

// ============================================================================
// Enum Serialization
// ============================================================================

/// Generates body of serialization impl for Enums
fn serialize_enum(shape: &EnumShape) -> TokenStream {
    let value_ident = if shape.is_string() {
        quote! { value.as_str() }
    } else {
        quote! { *value }
    };
    let method = if shape.is_string() {
        quote! { write_string }
    } else {
        quote! { write_integer }
    };
    let unknown = syn::parse_str::<Ident>("Unknown").unwrap();
    let variants = shape
        .data
        .as_enum()
        .expect("Enum variants")
        .iter()
        .filter(|v| v.ident != unknown)
        .map(|v| (&v.ident, &v.value));

    let name = &shape.ident;
    let (variant, value): (Vec<_>, Vec<_>) = variants.into_iter().unzip();
    quote! {
        let value = match self {
            #(#name::#variant => #value,)*
            #name::Unknown(value) => #value_ident
        };
        serializer.#method(schema, value)
    }
}

// ============================================================================
// Unit Serialization
// ============================================================================

// TODO: keep this or remove?
#[allow(unused)]
fn serialize_unit() -> TokenStream {
    quote! {
        serializer.write_struct(schema, 0usize)?.end(schema)
    }
}

// ============================================================================
// Union Serialization
// ============================================================================

/// Generates body of serialization impl for Enums
fn serialize_union(shape: &UnionShape) -> TokenStream {
    let name = &shape.ident;

    let variants = shape
        .data
        .as_enum()
        .expect("Union variants")
        .iter()
        // Unknown variants have no member schema, so skip
        .filter(|v| v.schema.is_some())
        .map(|v| match_arm(shape, v));

    quote! {
        let mut ser = serializer.write_struct(schema, 1)?;
        match self {
            #(#variants,)*
            #name::Unknown(unknown) => ser.write_unknown(schema, unknown)?,
        }
        ser.end(schema)
    }
}

fn match_arm(shape: &UnionShape, variant: &UnionVariant) -> TokenStream {
    let name = &variant.ident;
    let shape_name = &shape.ident;
    let root_schema = &shape.schema;
    let member_name = &variant.ident.to_string().to_lowercase();
    let schema = variant.schema.as_ref().expect("Member");
    let schema = member_schema(schema, root_schema);

    // If unit, use custom route.
    if variant.fields.is_empty() {
        quote! {
            #shape_name::#name => ser.write_member_named(
                #member_name,
                &#schema,
                &_Unit
            )?
        }
    } else {
        quote! {
            #shape_name::#name(value) => ser.write_member_named(
                #member_name,
                &#schema,
                value
            )?
        }
    }
}
