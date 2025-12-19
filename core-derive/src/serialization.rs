use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, Data, DataEnum, DataStruct, DeriveInput, Field, Lit};

use crate::{parse_schema, utils::is_optional};

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
    if let Data::Struct(_) = &input.data {
        imports = quote! {
            #imports
            use #crate_ident::serde::serializers::StructSerializer as _StructSerializer;
        }
    }
    let body = match &input.data {
        Data::Struct(data) => serialize_struct(schema_ident, data),
        Data::Enum(data) => serialize_enum(shape_name, data),
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
    let unknown = syn::parse_str::<Ident>("_Unknown").unwrap();
    let variant = data
        .variants
        .iter()
        .map(|v| &v.ident)
        .filter(|i| **i != unknown);
    let value = data.variants.iter().map(|v| parse_enum_value(&v.attrs));
    quote! {
        match self {
            #(#shape_name::#variant => serializer.#method(schema, #value)),*
            #shape_name::_Unknown(value) =>  serializer.write_string(schema, value)
        }
    }
}

/// Determines enum method to use for serializing an enum.
pub(crate) fn determine_enum_ser_method(data: &DataEnum) -> Ident {
    let first_var = data
        .variants
        .first()
        .expect("At least one enum variant expected");
    let value = parse_enum_value(&first_var.attrs);
    match value {
        Lit::Str(_) => Ident::new("write_string", Span::call_site()),
        Lit::Int(_) => Ident::new("write_integer", Span::call_site()),
        _ => panic!("Unsupported enum value. Expected string or int literal."),
    }
}

/// Parse an `#[enum_value(...)` attribute
pub(crate) fn parse_enum_value(attrs: &[Attribute]) -> Lit {
    let mut target_schema = None;
    for attr in attrs {
        if attr.path().is_ident("enum_value") {
            target_schema = Some(
                attr.parse_args::<Lit>()
                    .expect("`enum_value` attribute should be an identifier"),
            );
        }
    }
    target_schema.expect("Could not find `enum_value` attribute")
}
