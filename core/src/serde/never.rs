//! Uninhabited types for partial serializer/deserializer implementations.
//!
//! These types implement the compound serializer/reader traits but are
//! uninhabited (empty enums), making their methods unreachable. Use them
//! when implementing a serializer or deserializer that only handles scalars
//! (e.g., string serialization for HTTP headers).
//!
//! # Example
//!
//! ```ignore
//! use smithy4rs_core::serde::never::Never;
//!
//! impl<'de> Deserializer<'de> for StringDeserializer<'de> {
//!     type Error = MyError;
//!     type StructReader = Never<Self::Error>;
//!     type ListReader = Never<Self::Error>;
//!     type MapReader = Never<Self::Error>;
//!
//!     fn read_struct(self, _: &Schema) -> Result<Self::StructReader, Self::Error> {
//!         Err(MyError::new("cannot deserialize struct from string"))
//!     }
//!     // ... scalar methods ...
//! }
//! ```

use std::marker::PhantomData;

use super::{
    de::{self, DeserializeWithSchema},
    se::{self, SerializeWithSchema},
};
use crate::schema::Schema;

/// An uninhabited type that implements all compound serializer and reader traits.
///
/// This struct contains [`Infallible`](std::convert::Infallible), making it
/// impossible to construct. The `match self.0 {}` pattern in each method proves
/// to the compiler that the code is unreachable.
pub struct Never<E>(std::convert::Infallible, PhantomData<E>);

// ============================================================================
// Deserializer Reader Implementations
// ============================================================================

impl<'de, E: de::Error> de::StructReader<'de> for Never<E> {
    type Error = E;

    fn read_member<'a>(&mut self, _schema: &'a Schema) -> Result<Option<&'a Schema>, Self::Error> {
        match self.0 {}
    }

    fn read_value<T: DeserializeWithSchema<'de>>(
        &mut self,
        _schema: &Schema,
    ) -> Result<T, Self::Error> {
        match self.0 {}
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        match self.0 {}
    }
}

impl<'de, E: de::Error> de::ListReader<'de> for Never<E> {
    type Error = E;

    fn read_element<T: DeserializeWithSchema<'de>>(
        &mut self,
        _schema: &Schema,
    ) -> Result<Option<T>, Self::Error> {
        match self.0 {}
    }
}

impl<'de, E: de::Error> de::MapReader<'de> for Never<E> {
    type Error = E;

    fn read_key(&mut self) -> Result<Option<String>, Self::Error> {
        match self.0 {}
    }

    fn read_value<V: DeserializeWithSchema<'de>>(
        &mut self,
        _schema: &Schema,
    ) -> Result<V, Self::Error> {
        match self.0 {}
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        match self.0 {}
    }
}

// ============================================================================
// Serializer Writer Implementations
// ============================================================================

impl<E: se::Error> se::StructWriter for Never<E> {
    type Error = E;
    type Ok = ();

    fn write_member<T: SerializeWithSchema>(
        &mut self,
        _member_schema: &Schema,
        _value: &T,
    ) -> Result<(), Self::Error> {
        match self.0 {}
    }

    fn skip_member(&mut self, _schema: &Schema) -> Result<(), Self::Error> {
        match self.0 {}
    }

    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        match self.0 {}
    }
}

impl<E: se::Error> se::ListWriter for Never<E> {
    type Error = E;
    type Ok = ();

    fn write_element<T: SerializeWithSchema>(
        &mut self,
        _element_schema: &Schema,
        _value: &T,
    ) -> Result<(), Self::Error> {
        match self.0 {}
    }

    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        match self.0 {}
    }
}

impl<E: se::Error> se::MapWriter for Never<E> {
    type Error = E;
    type Ok = ();

    fn write_entry<K, V>(
        &mut self,
        _key_schema: &Schema,
        _value_schema: &Schema,
        _key: &K,
        _value: &V,
    ) -> Result<(), Self::Error>
    where
        K: SerializeWithSchema,
        V: SerializeWithSchema,
    {
        match self.0 {}
    }

    fn end(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        match self.0 {}
    }
}
