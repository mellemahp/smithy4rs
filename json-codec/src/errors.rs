use jiter::JiterError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JsonSerdeError {
    #[error("Failed to serialize member to JSON: {0}")]
    SerializationError(String),
    #[error("Failed to deserialize member from JSON: {0}")]
    DeserializationError(String),
    #[error("Failed to convert integer type")]
    IntConversionError(#[from] std::num::TryFromIntError),
}

impl From<JiterError> for JsonSerdeError {
    fn from(value: JiterError) -> Self {
        JsonSerdeError::DeserializationError(format!("{}", value))
    }
}