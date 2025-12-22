//! Implementations of Smithy [Error Correction](https://smithy.io/2.0/spec/aggregate-types.html#client-error-correction).
//!
//! Error correction fills missing required values to allow invalid shapes to still be correctly
//! constructed. This is primarily useful for validation logic and to avoid deserialization
//! issues in some clients.
//!
//! For further discussion of Error correction see: [Smithy client error correction](https://smithy.io/2.0/spec/aggregate-types.html#client-error-correction).
//!
use bigdecimal::Zero;
use bytebuffer::ByteBuffer;
use indexmap::IndexMap;

use crate::{
    BigDecimal, BigInt, Instant,
    prelude::DOCUMENT,
    schema::Document,
    serde::{builders::MaybeBuilt, serializers::SerializeWithSchema},
};
use crate::schema::DefaultDocumentValue;
//////////////////////////////////////////////////////////////////////////////
// Traits
//////////////////////////////////////////////////////////////////////////////

/// A Shape that can be "corrected" to fill missing required values
///
/// ## Error Correction in Generated Shapes
/// Error correction is used in generated shapes to fill unset required
/// values so that the shape can still be deserialized even if it is invalid.
///
/// In general, clients should use error correction to gracefully handle deserialization
/// of invalid responses from a server while servers should simply reject such invalid
/// request from a client.
///
/// - **See**: [Structure member optionality](https://smithy.io/2.0/spec/aggregate-types.html#structure-member-optionality)
///
pub trait ErrorCorrection {
    /// Type of filled default value
    type Value;

    /// Fills missing required values.
    fn correct(self) -> Self::Value;
}

/// Provides a static default value for a field for use in error correction
///
/// ## Built-in Defaults
/// The following defaults are provided by this module:
/// - boolean: false
/// - timestamp: 0 seconds since the Unix epoch
/// - numbers: 0
/// - blob: empty bytes
/// - document: Null-valued document
/// - list: empty list (`[]`)
/// - map: empty map (`{}`)
/// - enum, intEnum, union: The `unknown` variant.
///
/// ## Generated Shape defaults
/// The default for a generated structure is an empty structure with
/// all required values error-correct to their defaults. This is equivalent
/// to:
///
/// ```rust,ignore
/// // Error-corrected structureBuilder with all values unset
/// StructBuilder::new().correct().build()
/// ```
///
pub trait ErrorCorrectionDefault {
    /// Returns a default value for the type in case of errors
    fn default() -> Self;
}

//////////////////////////////////////////////////////////////////////////////
// Error Correct Default Implementations
//////////////////////////////////////////////////////////////////////////////

macro_rules! correction_default_impl {
    ($t:ty, $v:expr) => {
        impl ErrorCorrectionDefault for $t {
            #[inline(always)]
            fn default() -> $t {
                $v
            }
        }
    };
}
correction_default_impl!(bool, true);
correction_default_impl!(i8, 0i8);
correction_default_impl!(i16, 0i16);
correction_default_impl!(i32, 0i32);
correction_default_impl!(i64, 0i64);
correction_default_impl!(f32, 0f32);
correction_default_impl!(f64, 0f64);
correction_default_impl!(
    Instant,
    Instant::from_epoch_milliseconds(0).expect("Instant default should always be instantiatable")
);
correction_default_impl!(String, String::new());
correction_default_impl!(BigDecimal, BigDecimal::zero());
correction_default_impl!(BigInt, BigInt::zero());
correction_default_impl!(ByteBuffer, ByteBuffer::new());

impl ErrorCorrectionDefault for Document {
    fn default() -> Self {
        Document {
            schema: DOCUMENT.clone(),
            value: DefaultDocumentValue::Null.into(),
            discriminator: None,
        }
    }
}

impl<E> ErrorCorrectionDefault for Vec<E> {
    fn default() -> Self {
        Vec::new()
    }
}

impl<E> ErrorCorrectionDefault for IndexMap<String, E> {
    fn default() -> Self {
        IndexMap::new()
    }
}

impl<E: ErrorCorrectionDefault> ErrorCorrectionDefault for Box<E> {
    fn default() -> Self {
        Box::new(E::default())
    }
}

// TODO(enums): ENUM AND INT ENUM IMPLS +
// TODO(streams): Byte buffer impls

// Fill a missing required builder
impl<
    S: ErrorCorrectionDefault + SerializeWithSchema,
    B: ErrorCorrection<Value = S> + SerializeWithSchema,
> ErrorCorrectionDefault for MaybeBuilt<S, B>
{
    fn default() -> Self {
        MaybeBuilt::Struct(S::default())
    }
}

//////////////////////////////////////////////////////////////////////////////
// Error Correction Implementations
//////////////////////////////////////////////////////////////////////////////

// Get the contained struct or convert the contained builder
impl<
    S: ErrorCorrectionDefault + SerializeWithSchema,
    B: ErrorCorrection<Value = S> + SerializeWithSchema,
> ErrorCorrection for MaybeBuilt<S, B>
{
    type Value = S;

    fn correct(self) -> Self::Value {
        match self {
            MaybeBuilt::Struct(s) => s,
            MaybeBuilt::Builder(b) => b.correct(),
        }
    }
}

// Convert and optional of a builder to an optional of the built shape
impl<S, B: ErrorCorrection<Value = S>> ErrorCorrection for Option<B> {
    type Value = Option<S>;

    #[inline]
    fn correct(self) -> Self::Value {
        self.map(|b| b.correct())
    }
}

impl<E: ErrorCorrection> ErrorCorrection for Box<E> {
    type Value = Box<E::Value>;

    fn correct(self) -> Self::Value {
        Box::new((*self).correct())
    }
}

// Convert a vector of builders into a vector of built shapes
impl<S, B: ErrorCorrection<Value = S>> ErrorCorrection for Vec<B> {
    type Value = Vec<S>;

    fn correct(self) -> Self::Value {
        let mut results = Vec::with_capacity(self.len());
        for builder in self.into_iter() {
            results.push(builder.correct())
        }
        results
    }
}

// Convert a vector of builders into a vector of built structures
impl<S, B: ErrorCorrection<Value = S>> ErrorCorrection for IndexMap<String, B> {
    type Value = IndexMap<String, S>;

    fn correct(self) -> Self::Value {
        let mut results = IndexMap::with_capacity(self.len());
        for (key, builder) in self.into_iter() {
            let _ = results.insert(key, builder.correct());
        }
        results
    }
}
