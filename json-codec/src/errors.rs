use std::fmt::Display;

use smithy4rs_core::serde::{
    deserializers::Error as DeserializerError, serializers::Error as SerializerError,
};
use thiserror::Error;

// TODO(errors): Better errors
#[derive(Error, Debug)]
pub enum JsonSerdeError {
    #[error("Failed to serialize: {0}")]
    SerializationError(String),
    #[error("Failed to deserialize: {0}")]
    DeserializationError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Format error: {0}")]
    FmtError(#[from] std::fmt::Error),
}

impl SerializerError for JsonSerdeError {
    fn custom<T: Display>(msg: T) -> Self {
        JsonSerdeError::SerializationError(msg.to_string())
    }
}

impl DeserializerError for JsonSerdeError {
    fn custom<T: Display>(msg: T) -> Self {
        JsonSerdeError::DeserializationError(msg.to_string())
    }
}
