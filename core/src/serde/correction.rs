//! Implementations of Smithy [Error Correction](https://smithy.io/2.0/spec/aggregate-types.html#client-error-correction).
//!
//! Error correction fills missing required values to allow invalid shapes to still be correctly
//! constructed. This is primarily useful for validation logic and to avoid deserialization
//! issues in some clients.

use bigdecimal::Zero;
use indexmap::IndexMap;
use crate::prelude::DOCUMENT;
use crate::{Instant, BigDecimal, BigInt};
use crate::schema::{Document, DocumentValue};
use crate::serde::builders::MaybeBuilt;
use crate::serde::serializers::SerializeWithSchema;

//////////////////////////////////////////////////////////////////////////////
// Traits
//////////////////////////////////////////////////////////////////////////////

/// A Shape that can be "corrected" to fill missing default values
pub trait ErrorCorrection {
    /// Type of filled default value
    type Value;

    /// Fills missing required values.
    fn correct(self) -> Self::Value;
}

/// Provides a static default value for a field for use in error correction
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
correction_default_impl!(Instant, Instant::from_epoch_milliseconds(0).expect("Instant default should always be instantiatable"));
correction_default_impl!(String, String::new());
correction_default_impl!(BigDecimal, BigDecimal::zero());
correction_default_impl!(BigInt, BigInt::zero());

impl ErrorCorrectionDefault for Document {
    fn default() -> Self {
        Document {
            schema: DOCUMENT.clone(),
            value: DocumentValue::Null,
            discriminator: None,
        }
    }
}

impl <E> ErrorCorrectionDefault for Vec<E> {
    fn default() -> Self {
        Vec::new()
    }
}

impl <E> ErrorCorrectionDefault for IndexMap<String, E> {
    fn default() -> Self {
        IndexMap::new()
    }
}

// TODO: ENUM AND INT ENUM IMPLS + Byte buffer impls

// Fill a missing required builder
impl<'de, S: ErrorCorrectionDefault + SerializeWithSchema, B: ErrorCorrection<Value=S> + SerializeWithSchema> ErrorCorrectionDefault for MaybeBuilt<S, B> {
    fn default() -> Self {
        MaybeBuilt::Struct(S::default())
    }
}

//////////////////////////////////////////////////////////////////////////////
// Error Correction Implementations
//////////////////////////////////////////////////////////////////////////////

// Get the contained struct or convert the contained builder
impl <S: ErrorCorrectionDefault + SerializeWithSchema, B: ErrorCorrection<Value=S> + SerializeWithSchema> ErrorCorrection for MaybeBuilt<S, B> {
    type Value = S;

    fn correct(self) -> Self::Value {
        match self {
            MaybeBuilt::Struct(s) => s,
            MaybeBuilt::Builder(b) => b.correct(),
        }
    }
}

// Convert and optional of a builder to an optional of the built shape
impl <S, B: ErrorCorrection<Value=S>> ErrorCorrection for Option<B> {
    type Value = Option<S>;

    fn correct(self) -> Self::Value {
        match self {
            None => None,
            Some(b) => Some(b.correct())
        }
    }
}

// Convert a vector of builders into a vector of built shapes
impl <S, B: ErrorCorrection<Value=S>> ErrorCorrection for Vec<B> {
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
impl <S, B: ErrorCorrection<Value=S>> ErrorCorrection for IndexMap<String, B> {
    type Value = IndexMap<String, S>;

    fn correct(self) -> Self::Value {
        let mut results = IndexMap::with_capacity(self.len());
        for (key, builder) in self.into_iter() {
            let _ = results.insert(key, builder.correct());
        }
        results
    }
}
