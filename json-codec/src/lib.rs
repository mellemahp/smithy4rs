mod deserialization;
mod errors;
mod serialization;

pub use deserialization::JsonDeserializer;
pub use errors::JsonSerdeError;
pub use serialization::JsonSerializer;
use smithy4rs_core::serde::codec::Codec;

/// A JSON codec for body serialization/deserialization.
#[derive(Clone)]
pub struct JsonCodec;

impl Codec for JsonCodec {
    type Serializer<'a> = JsonSerializer<'a>;
    type Deserializer<'de> = JsonDeserializer<'de>;

    fn media_type(&self) -> &str {
        "application/json"
    }

    fn serializer<'a>(&self, buf: &'a mut Vec<u8>) -> Self::Serializer<'a> {
        JsonSerializer::new(buf)
    }

    fn deserializer<'de>(&self, data: &'de [u8]) -> Self::Deserializer<'de> {
        JsonDeserializer::new(data)
    }
}
