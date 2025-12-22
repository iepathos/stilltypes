//! Error types for domain validation.
//!
//! This module provides rich, contextual error types that follow the Stillwater
//! philosophy: "Errors Should Tell Stories". Each error includes:
//!
//! - What format was being validated
//! - The invalid value (optionally masked for sensitive data)
//! - Why it failed with specific details
//! - An example of a valid format
//!
//! # Example
//!
//! ```
//! use platypus::error::{DomainError, DomainErrorKind};
//!
//! let error = DomainError {
//!     format_name: "email address",
//!     value: "invalid".to_string(),
//!     reason: DomainErrorKind::InvalidFormat {
//!         expected: "local@domain",
//!     },
//!     example: "user@example.com",
//! };
//!
//! assert_eq!(
//!     error.to_string(),
//!     "invalid email address: invalid format, expected local@domain (example: user@example.com)"
//! );
//! ```

use std::fmt;

/// Rich error for domain validation failures.
///
/// Includes enough context for helpful user-facing messages.
///
/// # Example
///
/// ```
/// use platypus::error::{DomainError, DomainErrorKind};
///
/// let error = DomainError {
///     format_name: "email address",
///     value: "invalid".to_string(),
///     reason: DomainErrorKind::InvalidFormat {
///         expected: "local@domain",
///     },
///     example: "user@example.com",
/// };
///
/// assert_eq!(
///     error.to_string(),
///     "invalid email address: invalid format, expected local@domain (example: user@example.com)"
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainError {
    /// What we were validating ("email address", "phone number")
    pub format_name: &'static str,
    /// The invalid value (may be masked for sensitive data)
    pub value: String,
    /// Why it failed
    pub reason: DomainErrorKind,
    /// Example of valid format
    pub example: &'static str,
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid {}: {} (example: {})",
            self.format_name, self.reason, self.example
        )
    }
}

impl std::error::Error for DomainError {}

/// Specific reason for domain validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainErrorKind {
    /// Value cannot be empty
    Empty,
    /// Value exceeds maximum length
    TooLong {
        /// Maximum allowed length
        max: usize,
        /// Actual length
        actual: usize,
    },
    /// Value below minimum length
    TooShort {
        /// Minimum required length
        min: usize,
        /// Actual length
        actual: usize,
    },
    /// Wrong format pattern
    InvalidFormat {
        /// Expected format description
        expected: &'static str,
    },
    /// Invalid character at position
    InvalidCharacter {
        /// The invalid character
        char: char,
        /// Position in string (0-indexed)
        position: usize,
    },
    /// Checksum validation failed
    InvalidChecksum,
    /// Specific component is invalid
    InvalidComponent {
        /// Component name ("scheme", "domain", etc.)
        component: &'static str,
        /// Why the component is invalid
        reason: String,
    },
}

impl fmt::Display for DomainErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "cannot be empty"),
            Self::TooLong { max, actual } => {
                write!(f, "too long ({} chars, max {})", actual, max)
            }
            Self::TooShort { min, actual } => {
                write!(f, "too short ({} chars, min {})", actual, min)
            }
            Self::InvalidFormat { expected } => {
                write!(f, "invalid format, expected {}", expected)
            }
            Self::InvalidCharacter { char, position } => {
                write!(f, "invalid character '{}' at position {}", char, position)
            }
            Self::InvalidChecksum => write!(f, "checksum validation failed"),
            Self::InvalidComponent { component, reason } => {
                write!(f, "invalid {}: {}", component, reason)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_displays_correctly() {
        let kind = DomainErrorKind::Empty;
        assert_eq!(kind.to_string(), "cannot be empty");
    }

    #[test]
    fn too_long_displays_correctly() {
        let kind = DomainErrorKind::TooLong {
            max: 100,
            actual: 150,
        };
        assert_eq!(kind.to_string(), "too long (150 chars, max 100)");
    }

    #[test]
    fn too_short_displays_correctly() {
        let kind = DomainErrorKind::TooShort { min: 5, actual: 3 };
        assert_eq!(kind.to_string(), "too short (3 chars, min 5)");
    }

    #[test]
    fn invalid_format_displays_correctly() {
        let kind = DomainErrorKind::InvalidFormat {
            expected: "local@domain",
        };
        assert_eq!(kind.to_string(), "invalid format, expected local@domain");
    }

    #[test]
    fn invalid_character_displays_correctly() {
        let kind = DomainErrorKind::InvalidCharacter {
            char: '@',
            position: 5,
        };
        assert_eq!(kind.to_string(), "invalid character '@' at position 5");
    }

    #[test]
    fn invalid_checksum_displays_correctly() {
        let kind = DomainErrorKind::InvalidChecksum;
        assert_eq!(kind.to_string(), "checksum validation failed");
    }

    #[test]
    fn invalid_component_displays_correctly() {
        let kind = DomainErrorKind::InvalidComponent {
            component: "scheme",
            reason: "expected https".to_string(),
        };
        assert_eq!(kind.to_string(), "invalid scheme: expected https");
    }

    #[test]
    fn domain_error_displays_correctly() {
        let error = DomainError {
            format_name: "email address",
            value: "bad".to_string(),
            reason: DomainErrorKind::InvalidFormat {
                expected: "local@domain",
            },
            example: "user@example.com",
        };
        assert_eq!(
            error.to_string(),
            "invalid email address: invalid format, expected local@domain (example: user@example.com)"
        );
    }

    #[test]
    fn domain_error_is_clone_and_eq() {
        let error1 = DomainError {
            format_name: "email",
            value: "test".to_string(),
            reason: DomainErrorKind::Empty,
            example: "user@example.com",
        };
        let error2 = error1.clone();
        assert_eq!(error1, error2);
    }

    #[test]
    fn domain_error_works_with_vec_for_accumulation() {
        let errors: Vec<DomainError> = vec![
            DomainError {
                format_name: "email",
                value: "".to_string(),
                reason: DomainErrorKind::Empty,
                example: "user@example.com",
            },
            DomainError {
                format_name: "phone",
                value: "abc".to_string(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "digits only",
                },
                example: "+1-555-555-5555",
            },
        ];
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].format_name, "email");
        assert_eq!(errors[1].format_name, "phone");
    }

    #[test]
    fn domain_error_is_error_trait() {
        fn accepts_error<E: std::error::Error>(_: E) {}
        let error = DomainError {
            format_name: "test",
            value: "val".to_string(),
            reason: DomainErrorKind::Empty,
            example: "example",
        };
        accepts_error(error);
    }
}
