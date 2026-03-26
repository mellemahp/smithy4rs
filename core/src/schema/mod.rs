mod traits;

use std::ops::Deref;

use regex::Regex;
pub use traits::*;

mod documents;
pub use documents::*;

mod shapes;
pub use shapes::*;

mod schemas;
pub use schemas::*;

mod unit;

// Do not include the unit trait as it can remain private.
pub use unit::{UNIT, Unit};

use crate::serde::{
    de::{DeserializeWithSchema, Deserializer, Error},
    se::{SerializeWithSchema, Serializer},
};

/// Transparent wrapper for [`Regex`] type.
///
/// Allows caching of pre-compiled regex in pattern trait.
#[repr(transparent)]
#[derive(Clone)]
pub struct RegexWrapper(pub Regex);
impl PartialEq<RegexWrapper> for RegexWrapper {
    fn eq(&self, other: &RegexWrapper) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}
impl Deref for RegexWrapper {
    type Target = Regex;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<&str> for RegexWrapper {
    fn from(value: &str) -> Self {
        Self(value.parse().unwrap())
    }
}
impl SerializeWithSchema for RegexWrapper {
    fn serialize_with_schema<S: Serializer>(
        &self,
        schema: &Schema,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.write_string(schema, self.as_str())
    }
}
impl<'de> DeserializeWithSchema<'de> for RegexWrapper {
    fn deserialize_with_schema<D>(schema: &Schema, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = deserializer.read_string(schema)?;
        Ok(Self(Regex::new(&s).map_err(D::Error::custom)?))
    }
}

/// Core Smithy shape and trait definitions
#[allow(deprecated, dead_code, missing_docs, clippy::doc_markdown)]
pub mod prelude {
    use crate::generated_shapes;

    generated_shapes![];
}
