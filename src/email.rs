//! Email validation types.
//!
//! Provides RFC 5321 compliant email validation using the `email_address` crate.
//!
//! # Example
//!
//! ```
//! use platypus::email::Email;
//!
//! // Valid email addresses
//! let email = Email::new("user@example.com".to_string());
//! assert!(email.is_ok());
//!
//! // Invalid email addresses fail validation
//! let invalid = Email::new("not-an-email".to_string());
//! assert!(invalid.is_err());
//! ```

use crate::error::{DomainError, DomainErrorKind};
use email_address::EmailAddress;
use stillwater::refined::{Predicate, Refined};

/// RFC 5321 compliant email address predicate.
///
/// Uses the `email_address` crate for validation to ensure compliance
/// with the RFC specification, including edge cases like quoted local parts,
/// plus-addressing, and IP literals.
///
/// # Example
///
/// ```
/// use platypus::email::Email;
///
/// let email = Email::new("user@example.com".to_string());
/// assert!(email.is_ok());
///
/// let invalid = Email::new("not-an-email".to_string());
/// assert!(invalid.is_err());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidEmail;

impl Predicate<String> for ValidEmail {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.is_empty() {
            return Err(DomainError {
                format_name: "email address",
                value: value.clone(),
                reason: DomainErrorKind::Empty,
                example: "user@example.com",
            });
        }

        if EmailAddress::is_valid(value) {
            Ok(())
        } else {
            Err(DomainError {
                format_name: "email address",
                value: value.clone(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "local@domain",
                },
                example: "user@example.com",
            })
        }
    }

    fn description() -> &'static str {
        "RFC 5321 email address"
    }
}

/// RFC 5321 compliant email address.
///
/// A `String` that has been validated to be a properly formatted email address
/// according to RFC 5321.
///
/// # Example
///
/// ```
/// use platypus::email::Email;
///
/// // Create from valid email
/// let email = Email::new("hello@example.com".to_string()).unwrap();
/// assert_eq!(email.get(), "hello@example.com");
///
/// // Use in type signatures to enforce validation
/// fn send_newsletter(subscriber: Email) {
///     // `subscriber` is guaranteed to be a valid email
/// }
/// ```
pub type Email = Refined<String, ValidEmail>;

#[cfg(test)]
mod tests {
    use super::*;

    // Valid cases
    #[test]
    fn valid_simple_email() {
        assert!(Email::new("user@example.com".to_string()).is_ok());
    }

    #[test]
    fn valid_with_plus_tag() {
        assert!(Email::new("user+tag@example.com".to_string()).is_ok());
    }

    #[test]
    fn valid_with_subdomain() {
        assert!(Email::new("user@mail.example.com".to_string()).is_ok());
    }

    #[test]
    fn valid_quoted_local_part() {
        assert!(Email::new("\"quoted\"@example.com".to_string()).is_ok());
    }

    #[test]
    fn valid_localhost() {
        assert!(Email::new("user@localhost".to_string()).is_ok());
    }

    #[test]
    fn valid_ip_literal() {
        assert!(Email::new("user@[192.168.1.1]".to_string()).is_ok());
    }

    // Invalid cases
    #[test]
    fn invalid_empty() {
        let result = Email::new(String::new());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.reason, DomainErrorKind::Empty));
    }

    #[test]
    fn invalid_missing_at() {
        let result = Email::new("userexample.com".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_missing_local() {
        let result = Email::new("@example.com".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_missing_domain() {
        let result = Email::new("user@".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_double_at() {
        let result = Email::new("user@@example.com".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_spaces() {
        let result = Email::new("user @example.com".to_string());
        assert!(result.is_err());
    }

    // Error message tests
    #[test]
    fn error_includes_format_name() {
        let result = Email::new("invalid".to_string());
        let err = result.unwrap_err();
        assert_eq!(err.format_name, "email address");
    }

    #[test]
    fn error_includes_example() {
        let result = Email::new("invalid".to_string());
        let err = result.unwrap_err();
        assert_eq!(err.example, "user@example.com");
    }

    #[test]
    fn error_display_is_readable() {
        let result = Email::new("invalid".to_string());
        let err = result.unwrap_err();
        let display = err.to_string();
        assert!(display.contains("email address"));
        assert!(display.contains("user@example.com"));
    }

    #[test]
    fn description_returns_expected() {
        assert_eq!(ValidEmail::description(), "RFC 5321 email address");
    }
}
