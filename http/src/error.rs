//! Error types for HTTP binding operations.

// TODO: Refactor to real structured error variants (with source chaining), we need to think about this sooner than later

/// Error type for HTTP binding operations.
#[derive(Debug)]
pub struct HttpBindingError {
    /// Human-readable error message
    message: String,
}

impl HttpBindingError {
    /// Create a new error with the given message.
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }

    /// Get the error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for HttpBindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for HttpBindingError {}

impl smithy4rs_core::serde::se::Error for HttpBindingError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::new(msg.to_string())
    }
}

impl smithy4rs_core::serde::de::Error for HttpBindingError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::new(msg.to_string())
    }
}
