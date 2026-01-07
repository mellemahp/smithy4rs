use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Field, Lit, Variant};

use crate::{
    parse_schema,
    utils::{is_optional, is_union, parse_enum_value},
};

/// Generates the `SerializeWithSchema` implementation for a shape.
pub(crate) fn serialization_impl(
    crate_ident: &TokenStream,
    shape_name: &Ident,
    schema_ident: &Ident,
    input: &DeriveInput,
) -> TokenStream {
    let mut imports = quote! {
        use #crate_ident::serde::serializers::Serializer as _Serializer;
        use #crate_ident::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    };
    // Add structure-specific imports
    // TODO(unions): This should also be added for unions
    if let Data::Struct(_) = &input.data {}
    let body = match &input.data {
        Data::Struct(data) => {
            imports = quote! {
                #imports
                use #crate_ident::serde::serializers::StructSerializer as _StructSerializer;
            };
            serialize_struct(schema_ident, data)
        }
        Data::Enum(data) => {
            if is_union(data) {
                imports = quote! {
                    #imports
                    use #crate_ident::serde::serializers::StructSerializer as _StructSerializer;
                };
                if data.variants.iter().any(|v| v.fields.is_empty()) {
                    imports = quote! {
                        #imports
                        use #crate_ident::schema::Unit as _Unit;
                    };
                }
                serialize_union(shape_name, schema_ident, data)
            } else {
                serialize_enum(shape_name, data)
            }
        }
        _ => panic!("SerializableShape can only be derived for structs, enum, or unions"),
    };

    quote! {
        #imports

        #[automatically_derived]
        impl _SerializeWithSchema for #shape_name {
            fn serialize_with_schema<S: _Serializer>(
                &self,
                schema: &_SchemaRef,
                serializer: S,
            ) -> Result<S::Ok, S::Error> {
                #body
            }
        }
    }
}

// ============================================================================
// Structure Serialization
// ============================================================================

/// Generates body of serialization impl for Structures
fn serialize_struct(schema_ident: &Ident, data_struct: &DataStruct) -> TokenStream {
    let length = data_struct.fields.len();
    let field_data: Vec<FieldData> = data_struct
        .fields
        .iter()
        .map(FieldData::from)
        .collect::<Vec<_>>();
    // Now write the thing
    let method = field_data.iter().map(|d| d.method_call());
    let member_schema = field_data.iter().map(|d| d.member_schema(schema_ident));
    let member_name = field_data.iter().map(|d| &d.field_ident);
    // TODO: This needs to be the exact member name used in the schema. I think it might differ from the field name
    // in some cases
    let member_name_str = field_data.iter().map(|d| d.field_ident.to_string());
    quote! {
        let mut ser = serializer.write_struct(schema, #length)?;
        #(ser.#method(#member_name_str, &#member_schema, &self.#member_name)?;)*
        ser.end(schema)
    }
}

struct FieldData {
    schema: Ident,
    field_ident: Ident,
    optional: bool,
}
impl FieldData {
    fn method_call(&self) -> Ident {
        if self.optional {
            Ident::new("serialize_optional_member_named", Span::call_site())
        } else {
            Ident::new("serialize_member_named", Span::call_site())
        }
    }

    fn member_schema(&self, root_schema_ident: &Ident) -> Ident {
        Ident::new(
            &format!("_{}_MEMBER_{}", root_schema_ident, &self.schema),
            Span::call_site(),
        )
    }

    fn from(field: &Field) -> Self {
        let schema = parse_schema(&field.attrs);
        let field_ident = field.ident.as_ref().unwrap().clone();
        let optional = is_optional(&field.ty);
        FieldData {
            schema,
            field_ident,
            optional,
        }
    }
}

// ============================================================================
// Enum Serialization
// ============================================================================

/// Generates body of serialization impl for Enums
fn serialize_enum(shape_name: &Ident, data: &DataEnum) -> TokenStream {
    let method = determine_enum_ser_method(data);
    let unknown = syn::parse_str::<Ident>("Unknown").unwrap();
    let variant = data
        .variants
        .iter()
        .map(|v| &v.ident)
        .filter(|i| **i != unknown);
    let value = data
        .variants
        .iter()
        .map(|v| parse_enum_value(&v.attrs).expect("parsable #[enum_value] attribute"));
    let is_string = matches!(
        parse_enum_value(&data.variants.first().expect("at least one variant").attrs),
        Some(Lit::Str(_))
    );
    let value_ident = if is_string {
        quote! { value.as_str() }
    } else {
        quote! { *value }
    };
    quote! {
        let value = match self {
            #(#shape_name::#variant => #value,)*
            #shape_name::Unknown(value) => #value_ident
        };
        serializer.#method(schema, value)
    }
}

/// Determines enum method to use for serializing an enum.
fn determine_enum_ser_method(data: &DataEnum) -> Ident {
    let first_var = data
        .variants
        .first()
        .expect("At least one enum variant expected");
    match parse_enum_value(&first_var.attrs) {
        Some(Lit::Str(_)) => Ident::new("write_string", Span::call_site()),
        Some(Lit::Int(_)) => Ident::new("write_integer", Span::call_site()),
        _ => panic!("Unsupported enum value. Expected string or int literal."),
    }
}

// ============================================================================
// Union Serialization
// ============================================================================

/// Generates body of serialization impl for Enums
fn serialize_union(shape_name: &Ident, schema_ident: &Ident, data: &DataEnum) -> TokenStream {
    let unknown = syn::parse_str::<Ident>("Unknown").unwrap();
    let variants = data
        .variants
        .iter()
        .filter(|v| v.ident != unknown)
        .map(UnionVariant::from)
        .collect::<Vec<_>>();
    let match_arm = variants
        .iter()
        .map(|v| v.match_arm(shape_name, schema_ident));
    quote! {
        let mut ser = serializer.write_struct(schema, 1)?;
        match self {
            #(#match_arm,)*
            #shape_name::Unknown(unknown) => ser.serialize_unknown(schema, unknown)?,
        }
        ser.end(schema)
    }
}

struct UnionVariant {
    schema: Ident,
    field_ident: Ident,
    unit: bool,
}

impl UnionVariant {
    fn variant_schema(&self, root_schema_ident: &Ident) -> Ident {
        Ident::new(
            &format!("_{}_MEMBER_{}", root_schema_ident, &self.schema),
            Span::call_site(),
        )
    }

    fn from(variant: &Variant) -> Self {
        let schema = parse_schema(&variant.attrs);
        let field_ident = variant.ident.clone();
        let unit = variant.fields.is_empty();
        UnionVariant {
            schema,
            field_ident,
            unit,
        }
    }

    fn match_arm(&self, shape_name: &Ident, schema_ident: &Ident) -> TokenStream {
        let variant = &self.field_ident;
        let member_name = &self.field_ident.to_string().to_lowercase();
        let schema = &self.variant_schema(schema_ident);
        if self.unit {
            quote! {
                #shape_name::#variant => ser.serialize_member_named(
                    #member_name,
                    &#schema,
                    &_Unit
                )?
            }
        } else {
            quote! {
                #shape_name::#variant(val) => ser.serialize_member_named(
                    #member_name,
                    &#schema,
                    val
                )?
            }
        }
    }
}
