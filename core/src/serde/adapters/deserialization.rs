
//========================================================================
// Errors
//========================================================================

use std::fmt::{Display, Formatter};
use std::error::Error as StdError;
use std::fmt;
use std::marker::PhantomData;
use bigdecimal::BigDecimal;
use bytebuffer::ByteBuffer;
use num_bigint::BigInt;
use serde::de;
use crate::serde::de::{Deserializer, Error};
use serde::de::{Error as SerdeError, Visitor};
use temporal_rs::Instant;
use crate::schema::{Document, SchemaRef};

/// Wrapper type that bridges `serde` and `smithy` Serialization error types.
#[derive(Debug)]
#[repr(transparent)]
pub struct DeErrorWrapper<E: SerdeError>(E);
impl<E: SerdeError> DeErrorWrapper<E> {
    #[inline]
    pub fn inner(self) -> E {
        self.0
    }
}
impl<E: SerdeError> Display for DeErrorWrapper<E> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
impl<E: SerdeError> StdError for DeErrorWrapper<E> {}
impl<E: SerdeError> Error for DeErrorWrapper<E> {
    #[inline]
    fn custom<T: Display>(msg: T) -> Self {
        DeErrorWrapper(E::custom(msg))
    }
}
impl<E: SerdeError> From<E> for DeErrorWrapper<E> {
    #[inline]
    fn from(e: E) -> Self {
        DeErrorWrapper(e)
    }
}

//========================================================================
// Deserialization Adapter
//========================================================================

#[cfg(test)]
mod test {
    fn test() {

    }
}