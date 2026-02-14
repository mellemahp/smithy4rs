use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Lit, Type, Variant};

use crate::shapes::{
    get_builder_fields,
    utils::{get_builder_ident, is_union, parse_enum_value, parse_schema},
};

/// Generate `DeserializeWithSchema` implementation for Smithy Shapes
pub(crate) fn deserialization_impl(
    crate_ident: &TokenStream,
    shape_name: &Ident,
    schema_ident: &Ident,
    input: &DeriveInput,
) -> TokenStream {
    let deser_impl = match &input.data {
        // Structures are deserialized via builders
        Data::Struct(data) => deserialize_builder(crate_ident, schema_ident, shape_name, data),
        Data::Enum(data) => {
            if is_union(data) {
                deserialize_union(crate_ident, shape_name, schema_ident, data)
            } else {
                deserialize_enum(shape_name, data)
            }
        }
        _ => panic!("SerializableShape can only be derived for structs, enum, or unions"),
    };
    quote! {
        // Base deserialization imports
        use #crate_ident::serde::deserializers::Deserializer as _Deserializer;
        use #crate_ident::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;

        #deser_impl
    }
}

// ============================================================================
// Builder (Union & Structure) Deserialization
// ============================================================================

/// Generate deserializer body for structure builder
fn deserialize_builder(
    crate_ident: &TokenStream,
    schema_ident: &Ident,
    shape_name: &Ident,
    data_struct: &DataStruct,
) -> TokenStream {
    let builder_name = get_builder_ident(shape_name);
    let field_data = get_builder_fields(schema_ident, data_struct);

    // Generate deserialize_member! or deserialize_optional_member! macro calls for each field
    let match_arms = field_data
        .iter()
        .map(|d| d.deserialize_match_arm(crate_ident))
        .collect::<Vec<_>>();

    quote! {
        // builder-specific imports
        use #crate_ident::serde::correction::ErrorCorrection as _ErrorCorrection;
        use #crate_ident::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
        use #crate_ident::serde::ShapeBuilder as _ShapeBuilder;
        use #crate_ident::serde::Buildable as _Buildable;
        use #crate_ident::serde::deserializers::StructReader as _StructReader;

        #[automatically_derived]
        impl<'de> _DeserializeWithSchema<'de> for #builder_name {
            fn deserialize_with_schema<D>(schema: &_Schema, deserializer: &mut D) -> Result<Self, D::Error>
            where
                D: _Deserializer<'de>,
            {
                let mut builder = #builder_name::new();
                let mut reader = deserializer.read_struct()?;

                while let Some(field_name) = reader.read_name()? {
                    if let Some(member_schema) = schema.get_member(&field_name) {
                        #(#match_arms)*
                        // Unknown field
                        reader.skip_value()?;
                    } else {
                        reader.skip_value()?;
                    }
                }

                Ok(builder)
            }
        }
    }
}

// ============================================================================
// Enum Deserialization
// ============================================================================

fn deserialize_enum(shape_name: &Ident, data: &DataEnum) -> TokenStream {
    let method = determine_enum_deser_method(data);
    let match_val = determine_enum_match_method(data);
    let unknown = syn::parse_str::<Ident>("Unknown").unwrap();
    let variant = data
        .variants
        .iter()
        .map(|v| &v.ident)
        .filter(|i| **i != unknown);
    let value = data.variants.iter().map(|v| parse_enum_value(&v.attrs));
    quote! {
        #[automatically_derived]
        impl<'de> _DeserializeWithSchema<'de> for #shape_name {
            fn deserialize_with_schema<D>(schema: &_Schema, deserializer: &mut D) -> Result<Self, D::Error>
            where
                D: _Deserializer<'de>,
            {
                let val = deserializer.#method(schema)?;
                let result = match #match_val {
                    #(#value => #shape_name::#variant,)*
                    _ => #shape_name::Unknown(val)
                };
                Ok(result)
            }
        }
    }
}

/// Determines enum method to use for deserializing an enum.
fn determine_enum_deser_method(data: &DataEnum) -> Ident {
    let first_var = data
        .variants
        .first()
        .expect("At least one enum variant expected");
    match parse_enum_value(&first_var.attrs) {
        Some(Lit::Str(_)) => Ident::new("read_string", Span::call_site()),
        Some(Lit::Int(_)) => Ident::new("read_integer", Span::call_site()),
        _ => panic!("Unsupported enum value. Expected string or int literal."),
    }
}

/// Determines how to correctly match on value
fn determine_enum_match_method(data: &DataEnum) -> TokenStream {
    let first_var = data
        .variants
        .first()
        .expect("At least one enum variant expected");
    match parse_enum_value(&first_var.attrs) {
        Some(Lit::Str(_)) => quote! { val.as_str() },
        Some(Lit::Int(_)) => quote! { val },
        _ => panic!("Unsupported enum value. Expected string or int literal."),
    }
}

// ============================================================================
// Union Deserialization
// ============================================================================

fn deserialize_union(
    crate_ident: &TokenStream,
    shape_name: &Ident,
    schema_ident: &Ident,
    data: &DataEnum,
) -> TokenStream {
    let mut imports = quote! {
        use #crate_ident::serde::deserializers::Error as _DeserializerError;
        use #crate_ident::serde::deserializers::StructReader as _StructReader;
    };
    if data.variants.iter().any(|v| v.fields.is_empty()) {
        imports = quote! {
            #imports
            use #crate_ident::schema::Unit as _Unit;
        }
    }
    let unknown = syn::parse_str::<Ident>("Unknown").unwrap();
    let variants = data
        .variants
        .iter()
        .filter(|v| v.ident != unknown)
        .map(UnionDeserVariant::from)
        .map(|udv| udv.matcher(shape_name, schema_ident));

    quote! {
        #imports

        #[automatically_derived]
        impl<'de> _DeserializeWithSchema<'de> for #shape_name {
            fn deserialize_with_schema<D>(schema: &_Schema, deserializer: &mut D) -> Result<Self, D::Error>
            where
                D: _Deserializer<'de>,
            {
                let mut reader = deserializer.read_struct()?;
                let mut result: Option<#shape_name> = None;

                while let Some(field_name) = reader.read_name()? {
                    if result.is_some() {
                        return Err(_DeserializerError::custom("Attempted to set union value twice"));
                    }
                    if let Some(member_schema) = schema.get_member(&field_name) {
                        #(#variants)*
                        // Member did not match an expected value
                        result = Some(#shape_name::Unknown("unknown".to_string()));
                        continue;
                    } else {
                        reader.skip_value()?;
                    }
                }

                result.ok_or(_DeserializerError::custom("Failed to deserialize union"))
            }
        }
    }
}

struct UnionDeserVariant {
    schema: Ident,
    var_ident: Ident,
    ty: Option<Type>,
    unit: bool,
}

impl UnionDeserVariant {
    fn from(variant: &Variant) -> Self {
        let schema = parse_schema(&variant.attrs);
        let var_ident = variant.ident.clone();
        let unit = variant.fields.is_empty();
        let ty = variant
            .fields
            .iter()
            .map(|f| f.ty.clone())
            .collect::<Vec<_>>()
            .first()
            .cloned();
        UnionDeserVariant {
            schema,
            var_ident,
            ty,
            unit,
        }
    }

    fn matcher(&self, shape_name: &Ident, schema_ident: &Ident) -> TokenStream {
        let variant_name = &self.var_ident;
        let member_schema_const = Ident::new(
            &format!("_{}_MEMBER_{}", schema_ident, &self.schema),
            Span::call_site(),
        );
        if self.unit {
            quote! {
                if &member_schema == &*#member_schema_const {
                    let _: _Unit = reader.read_value(member_schema)?;
                    result = Some(#shape_name::#variant_name);
                    continue;
                }
            }
        } else {
            let ty = self.ty.as_ref().expect("Expected a type");
            quote! {
                if &member_schema == &*#member_schema_const {
                    let value: #ty = reader.read_value(member_schema)?;
                    result = Some(#shape_name::#variant_name(value));
                    continue;
                }
            }
        }
    }
}
