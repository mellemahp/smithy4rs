use std::error::Error;
use std::fmt::Display;
use bigdecimal::BigDecimal;
use thiserror::Error;
use crate::schema::{SchemaRef, ShapeType};

//////////////////////////////////////////////////////////////////////////////
// ERRORS
//////////////////////////////////////////////////////////////////////////////



/// Aggregated list of all validation errors encountered while building a shape.
///
/// When executing validation of a Builder, more than one field could be invalid.
/// All of these [`ValidationError`]'s are aggregated together into a list on this
/// aggregate error type.
#[derive(Error, Debug)]
pub struct ValidationErrors {
    errors: Vec<ValidationErrorWrapper>
}

impl Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:#?}", self.errors)
    }
}

impl ValidationErrors {
    /// Create a new [`ValidationErrors`] error.
    ///
    /// **NOTE**: This method instantiates the error type with an
    /// empty list of errors. Actual validation errors must be added
    /// using the [`ValidationErrors::extend`] or [`ValidationErrors::add`]
    /// methods.
    pub const fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Extends an aggregate validation error with the contents of
    /// another aggregate validation error.
    pub fn extend(&mut self, other: ValidationErrors) {
        self.errors.extend(other.errors);
    }

    /// Add a new validation error to the list of errors.
    pub fn add(&mut self, path: &SchemaRef, error: impl Into<Box<dyn ValidationError>>) {
        self.errors.push(ValidationErrorWrapper::new(path.clone(), error.into()));
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Wrapper that groups a validation error with the schema location at which it occured.
#[derive(Error, Debug)]
pub struct ValidationErrorWrapper {
    path: SchemaRef,
    error: Box<dyn ValidationError>
}
impl ValidationErrorWrapper {
    pub fn new(path: SchemaRef, error: Box<dyn ValidationError>) -> Self {
        Self { path, error }
    }
}
impl Display for ValidationErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}:{:#?}", self.path.id(), self.error)
    }
}

/// Marker trait for validation errors.
pub trait ValidationError: Error {}

// Implement conversion for any Error enums implementing Validation error
impl <T: ValidationError + 'static> From<T> for Box<dyn ValidationError> {
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

/// Captures validation failures that could happen for any validator.
///
/// These errors should only occur for manually constructed schemas.
/// If you encounter one of these in a generated shape using the default
/// validator then this is a bug.
#[derive(Error, Debug)]
pub enum ValidationFailure {
    #[error("Expected member: {0}")]
    ExpectedMember(String),
    /// This error should only ever occur for manual schema interactions,
    /// not for automatically generated Shapes.
    #[error("Invalid Shape type. Expected {0:?}, recieved {1:?}.")]
    InvalidType(ShapeType, ShapeType),
    #[error("Unsupported validation operation.")]
    Unsupported,
}
impl ValidationError for ValidationFailure {}

#[derive(Error, Debug)]
pub enum SmithyConstraints {
    /// [@required](https://smithy.io/2.0/spec/type-refinement-traits.html#smithy-api-required-trait)
    #[error("Field is Required.")]
    Required,
    /// [@length](https://smithy.io/2.0/spec/constraint-traits.html#length-trait)
    #[error("Size: {0} does not conform to @length constraint. Expected between {1} and {2}.")]
    Length(usize, usize, usize),
    /// [@pattern](https://smithy.io/2.0/spec/constraint-traits.html#pattern-trait)
    #[error("Value {0} did not conform to expected pattern {1}")]
    Pattern(String, String),
    /// [@range](https://smithy.io/2.0/spec/constraint-traits.html#range-trait)
    #[error("Size: {0} does not conform to @range constraint. Expected between {1} and {2}.")]
    Range(BigDecimal, BigDecimal, BigDecimal),
    // TODO(question): Could this be security risk if non-unique are returned?
    /// [@uniqueItems](https://smithy.io/2.0/spec/constraint-traits.html#uniqueitems-trait]
    #[error("Items in collection should be unique.")]
    UniqueItems
}
impl ValidationError for SmithyConstraints {}

#[cfg(test)]
mod tests {
    use crate::prelude::STRING;
    use super::*;

    #[test]
    fn test_validation_errors_aggregate() {
        let mut errors = ValidationErrors::new();
        errors.add(&STRING, SmithyConstraints::Required);
        errors.add(&STRING, SmithyConstraints::Length(1,2,3));
        errors.add(&STRING, SmithyConstraints::Required);
        assert_eq!(errors.errors.len(), 3);
        assert_eq!(&errors.errors[0].error.to_string(), "Field is Required.");
        assert_eq!(&errors.errors[2].error.to_string(), "Field is Required.");
    }
}