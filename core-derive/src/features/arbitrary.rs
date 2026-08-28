use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::attr::{Shape, StructShape};

/// `Arbitrary` implementation for generated shapes
pub(crate) fn expand_arbitrary(shape: &Shape, crate_ident: &TokenStream) -> TokenStream {
    let arbitrary_impl = if let Shape::Struct(struct_shape) = &shape {
        arbitrary_builder(struct_shape, crate_ident)
    } else {
        arbitrary_other(shape.name(), shape.schema())
    };

    quote! {
        const _: () = {
            extern crate arbitrary as _arbitrary;
            use _arbitrary::Unstructured as _Unstructured;
            use _arbitrary::Arbitrary as _Arbitrary;
            use _arbitrary::MaxRecursionReached as _MaxRecursionReached;
            use #crate_ident::features::arbitrary::ArbitraryDeserializer as _ArbitraryDeserializer;
            use #crate_ident::features::arbitrary::TrySizeHint as _TrySizeHint;
            use #crate_ident::serde::deserializers::DeserializableShape as _DeserializableShape;

            #arbitrary_impl
        };
    }
}

fn arbitrary_builder(struct_shape: &StructShape, crate_ident: &TokenStream) -> TokenStream {
    let builder_name = struct_shape.builder();
    let name = &struct_shape.ident;
    let schema = &struct_shape.schema;
    quote! {
        use #crate_ident::serde::ShapeBuilder as _ShapeBuilder;

        #[automatically_derived]
        impl<'a> _Arbitrary<'a> for #name {
            fn arbitrary(u: &mut _Unstructured<'a>) -> _arbitrary::Result<Self> {
                <#builder_name as _DeserializableShape>::deserialize(_ArbitraryDeserializer::new(u))?
                .build()
                .map_err(|_| _arbitrary::Error::IncorrectFormat)
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                Self::try_size_hint(depth).unwrap_or_default()
            }

            #[inline]
            fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), _MaxRecursionReached> {
                #schema.try_size_hint(depth)
            }
        }

        #[automatically_derived]
        impl<'a> _Arbitrary<'a> for #builder_name {
            fn arbitrary(u: &mut _Unstructured<'a>) -> _arbitrary::Result<Self> {
                <#builder_name as _DeserializableShape>::deserialize(_ArbitraryDeserializer::new(u))
                .map_err(|_| _arbitrary::Error::IncorrectFormat)
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                Self::try_size_hint(depth).unwrap_or_default()
            }

            #[inline]
            fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), _MaxRecursionReached> {
                #schema.try_size_hint(depth)
            }
        }
    }
}

fn arbitrary_other(shape_name: &Ident, schema_ident: &Ident) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl<'a> _Arbitrary<'a> for #shape_name {
            fn arbitrary(u: &mut _Unstructured<'a>) -> _arbitrary::Result<Self> {
                <#shape_name as _DeserializableShape>::deserialize(_ArbitraryDeserializer::new(u))
                .map_err(|_| _arbitrary::Error::IncorrectFormat)
            }

            fn try_size_hint(depth: usize) -> _arbitrary::Result<(usize, Option<usize>), _MaxRecursionReached> {
                #schema_ident.try_size_hint(depth)
            }
        }
    }
}
