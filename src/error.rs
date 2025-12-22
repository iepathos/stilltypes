//! Error types for domain validation.

use std::fmt;

/// Kinds of domain validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainErrorKind {
    /// Invalid format for the domain type.
    InvalidFormat,
    /// Value is empty when it should not be.
    Empty,
    /// Value is too long.
    TooLong,
    /// Value is too short.
    TooShort,
    /// Custom validation error.
    Custom(String),
}

/// A domain validation error with context.
#[derive(Debug, Clone)]
pub struct DomainError {
    /// The kind of error.
    pub kind: DomainErrorKind,
    /// The invalid value that was provided.
    pub value: String,
    /// A human-readable message.
    pub message: String,
    /// An example of a valid value.
    pub example: Option<String>,
}

impl DomainError {
    /// Create a new domain error.
    pub fn new(
        kind: DomainErrorKind,
        value: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            value: value.into(),
            message: message.into(),
            example: None,
        }
    }

    /// Add an example of a valid value.
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.example = Some(example.into());
        self
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        write!(f, "\n  value: {:?}", self.value)?;
        if let Some(ref example) = self.example {
            write!(f, "\n  example: {}", example)?;
        }
        Ok(())
    }
}

impl std::error::Error for DomainError {}
