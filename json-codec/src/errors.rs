use jiter::JiterError;
use smithy4rs_core::schema::documents::DocumentError;
use thiserror::Error;

#[derive(Error, Debug, Default)]
pub enum JsonSerdeError {
    #[error("Failed to serialize member to JSON: {0}")]
    SerializationError(String),
    #[error("Failed to deserialize member from JSON: {0}")]
    DeserializationError(String),
    #[error("Failed to convert integer type")]
    IntConversionError(#[from] std::num::TryFromIntError),
    #[error("Failed serializing")]
    #[default]
    Default,
    #[error("Failed Document conversion")]
    DocumentConversionError(#[from] DocumentError),
}

impl From<JiterError> for JsonSerdeError {
    fn from(value: JiterError) -> Self {
        JsonSerdeError::DeserializationError(format!("{}", value))
    }
}
