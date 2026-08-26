use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use crate::attr::{Shape, StructMember, StructShape, UnionShape, UnionVariant};
use crate::utils::member_schema;

/// Generates `StaticSchemaShape` impl for Smithy Shapes.
pub(crate) fn expand_schema(shape: &Shape, crate_ident: &TokenStream) -> TokenStream {
    let name = shape.name();
    let schema = shape.schema();
    let members = expand_member_schemas(shape, crate_ident);
    quote! {
        const _: () = {
            use #crate_ident::schema::Schema as _Schema;
            use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;

            #[automatically_derived]
            impl _StaticSchemaShape for #name {
                #[inline]
                fn schema() -> &'static _Schema {
                    &#schema
                }
            }
        };
        #members
    }
}

fn expand_member_schemas(shape: &Shape, crate_ident: &TokenStream) -> TokenStream {
    let schema = shape.schema();
    // TODO: This is a bit clunky. Clean up
    let (member_schemas, member_names) = match shape {
        Shape::Struct(struct_shape) => resolve_struct_member_schemas(struct_shape),
        Shape::Union(union_shape) => resolve_union_member_schemas(union_shape),
        _ => return quote!{ }, // all others dont need named members
    };
    // Ignore annotation-ish structs
    if member_names.is_empty() {
        return quote!{};
    }
    let schema_keys = Ident::new(&format!("{schema}_KEYS"), Span::call_site());

    // TODO: Use hybrid macro approach here.
    quote! {
        const _: () = #crate_ident::assert_contains_all(#schema_keys, &[#(#member_names),*]);
        #(static #member_schemas: #crate_ident::LazyLock<&#crate_ident::schema::Schema> =
                    #crate_ident::LazyLock::new(|| #schema.expect_member(#member_names));
        )*
    }
}

fn resolve_struct_member_schemas(shape: &StructShape) -> (Vec<Ident>, Vec<TokenStream>) {
    let root_schema = &shape.schema;
    let members = shape.data
        .as_struct()
        .expect("Expected Shape");
    let member_schema_idents = members.fields.iter()
        .map(|f| &f.schema)
        .map(|m| member_schema(m, root_schema))
        .collect::<Vec<_>>();
    let member_names = members.fields.iter()
        .map(StructMember::member_name)
        .collect::<Vec<_>>();
    (member_schema_idents, member_names)
}

fn resolve_union_member_schemas(shape: &UnionShape) -> (Vec<Ident>, Vec<TokenStream>) {
    let root_schema = &shape.schema;
    let variants = shape.data
        .as_enum()
        .expect("Expected Shape");
    let member_idents = variants.iter()
        .map(|f| &f.schema)
        .map(|m| member_schema(m.as_ref().expect("expect_schema"), root_schema))
        .collect::<Vec<_>>();
    let member_names = variants.iter()
        .map(UnionVariant::member_name)
        .collect::<Vec<_>>();
    (member_idents, member_names)
}
