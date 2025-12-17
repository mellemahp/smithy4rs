use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput};

use crate::{parse_schema, utils::is_optional};

/// Generates the `SerializableStruct` implementation for a shape.
pub(crate) fn serialization_impl(shape_name: &Ident, input: &DeriveInput) -> TokenStream {
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("SerializableStruct can only be derived for structs"),
    };
    let mut field_data = Vec::new();
    let length = fields.len();
    for field in fields {
        let schema = parse_schema(&field.attrs);
        let field_ident = field.ident.as_ref().unwrap().clone();
        let optional = is_optional(&field.ty);
        field_data.push(FieldData {
            schema,
            field_ident,
            optional,
        });
    }
    // Now write the thing
    let method = field_data.iter().map(|d| d.method_call());
    let member_schema = field_data.iter().map(|d| &d.schema);
    let member_name = field_data.iter().map(|d| &d.field_ident);
    let member_name_str = field_data.iter().map(|d| d.field_ident.to_string());
    quote! {
        #[automatically_derived]
        impl _SerializeWithSchema for #shape_name {
            fn serialize_with_schema<S: _Serializer>(
                &self,
                schema: &_SchemaRef,
                serializer: S,
            ) -> Result<S::Ok, S::Error> {
                let mut ser = serializer.write_struct(schema, #length)?;
                #(ser.#method(#member_name_str, &#member_schema, &self.#member_name)?;)*
                ser.end(schema)
            }
        }
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
}
