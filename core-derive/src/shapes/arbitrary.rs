use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput};

/// `Arbitrary` implementation for generated shapes
pub(crate) fn arbitrary_impl(
    crate_ident: &TokenStream,
    shape_name: &Ident,
    schema_ident: &Ident,
    input: &DeriveInput,
) -> TokenStream {
    let arbitrary_impl = match &input.data {
        Data::Struct(_) => arbitrary_struct(crate_ident, shape_name, schema_ident),
        Data::Enum(_) => arbitrary_enum(shape_name, schema_ident),
        _ => panic!("SerializableShape can only be derived for structs, enum, or unions"),
    };
    quote! {
        use _arbitrary::Unstructured as _Unstructured;
        use _arbitrary::Arbitrary as _Arbitrary;
        use _arbitrary::MaxRecursionReached as _MaxRecursionReached;
        use #crate_ident::features::arbitrary::ArbitraryDeserializer as _ArbitraryDeserializer;
        use #crate_ident::features::arbitrary::TrySizeHint as _TrySizeHint;
        use #crate_ident::serde::deserializers::DeserializableShape as _DeserializableShape;

        #arbitrary_impl
    }
}

/// Generates an `Arbitrary` impl for a shape and its builder
fn arbitrary_struct(
    crate_ident: &TokenStream,
    shape_name: &Ident,
    schema_ident: &Ident,
) -> TokenStream {
    let builder_name = Ident::new(&format!("{shape_name}Builder"), Span::call_site());
    quote! {
        use #crate_ident::serde::ShapeBuilder as _ShapeBuilder;

        #[automatically_derived]
        impl<'a> _Arbitrary<'a> for #shape_name {
            fn arbitrary(u: &mut _Unstructured<'a>) -> _arbitrary::Result<Self> {
                #builder_name::deserialize(&mut _ArbitraryDeserializer(u))?
                .build()
                .map_err(|_| _arbitrary::Error::IncorrectFormat)
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                Self::try_size_hint(depth).unwrap_or_default()
            }

            #[inline]
            fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), _MaxRecursionReached> {
                #schema_ident.try_size_hint(depth)
            }
        }

        #[automatically_derived]
        impl<'a> _Arbitrary<'a> for #builder_name {
            fn arbitrary(u: &mut _Unstructured<'a>) -> _arbitrary::Result<Self> {
                #builder_name::deserialize(&mut _ArbitraryDeserializer(u))
                    .map_err(|_| _arbitrary::Error::IncorrectFormat)
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                Self::try_size_hint(depth).unwrap_or_default()
            }

            #[inline]
            fn try_size_hint(depth: usize) -> Result<(usize, Option<usize>), _MaxRecursionReached> {
                #schema_ident.try_size_hint(depth)
            }
        }
    }
}

fn arbitrary_enum(shape_name: &Ident, schema_ident: &Ident) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl<'a> _Arbitrary<'a> for #shape_name {
            fn arbitrary(u: &mut _Unstructured<'a>) -> _arbitrary::Result<Self> {
                #shape_name::deserialize(&mut _ArbitraryDeserializer(u))
                    .map_err(|_| _arbitrary::Error::IncorrectFormat)
            }

            fn try_size_hint(depth: usize) -> _arbitrary::Result<(usize, Option<usize>), _MaxRecursionReached> {
                #schema_ident.try_size_hint(depth)
            }
        }
    }
}
