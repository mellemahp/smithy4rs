use proc_macro2::TokenStream;
use quote::quote;

use crate::attr::Shape;

/// Generates a serializer adapter that bridges between smithy serialization
/// and serde (de)serialization
pub fn expand_serde_adapter(shape: &Shape, crate_ident: &TokenStream) -> TokenStream {
    let ser = expand_ser(shape, crate_ident);
    let deser = expand_deser(shape, crate_ident);

    quote! {
        const _: () = {
            use ::serde as _serde;

            #ser
            #deser
        };
    }
}

pub(crate) fn expand_ser(shape: &Shape, crate_ident: &TokenStream) -> TokenStream {
    let name = shape.name();
    let schema = shape.schema();

    quote! {
        use #crate_ident::serde::serializers::Serializer as _Serializer;
        use #crate_ident::features::adapters::SerAdapter as _SerAdapter;
        use #crate_ident::serde::serializers::SerializeWithSchema as _SerializeWithSchema;

        #[automatically_derived]
        impl _serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: _serde::Serializer,
            {
                self.serialize_with_schema(&#schema, _SerAdapter::new(serializer))
                    .map_err(|wrapper| wrapper.inner())
            }
        }
    }
}

// ============================================================================
// Deserialization
// ============================================================================

/// Generates a deserializer adapter impl
pub(crate) fn expand_deser(shape: &Shape, crate_ident: &TokenStream) -> TokenStream {
    // common imports for all types
    let mut imports = quote! {
        use _serde::de::Error as _SerdeDeserError;
        use _serde::de::DeserializeSeed as _DeserializeSeed;

        use #crate_ident::schema::StaticSchemaShape as _StaticSchemaShape;
        use #crate_ident::features::adapters::SchemaSeed as _SchemaSeed;
    };
    let name = shape.name();

    let body = if let Shape::Struct(struct_shape) = &shape {
        imports = quote! {
            #imports
            use #crate_ident::serde::ShapeBuilder as _ShapeBuilder;
        };
        let builder_name = struct_shape.builder();
        quote! {
            let seed = _SchemaSeed::<#builder_name>::new(#name::schema());
            seed.deserialize(deserializer)?
                .build()
                .map_err(D::Error::custom)
        }
    } else {
        quote! {
            let seed = _SchemaSeed::<#name>::new(#name::schema());
            seed.deserialize(deserializer)
        }
    };

    quote! {
        #imports

        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: _serde::Deserializer<'de>,
            {
                #body
            }
        }
    }
}
