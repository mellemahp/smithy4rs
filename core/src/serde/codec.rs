//! Codec trait for format-specific serialization/deserialization.

use super::{de, se};

/// A codec provides format-specific serializer/deserializers.
///
/// Implementations handle a specific wire format (e.g. JSON, CBOR) and may
/// used by protocols to serialize/deserialize messages.
pub trait Codec {
    /// The serializer type produced by this codec.
    type Serializer<'a>: se::Serializer<Ok = ()> + 'a
    where
        Self: 'a;

    /// The deserializer type produced by this codec.
    type Deserializer<'de>: de::Deserializer<'de>;

    /// The media type this codec handles (e.g. `"application/json"`).
    fn media_type(&self) -> &str;

    /// Create a serializer that writes into the given buffer.
    fn serializer<'a>(&self, buf: &'a mut Vec<u8>) -> Self::Serializer<'a>;

    /// Create a deserializer that reads from the given byte slice.
    fn deserializer<'de>(&self, data: &'de [u8]) -> Self::Deserializer<'de>;
}
