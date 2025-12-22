//! Phone number validation types.
//!
//! Provides E.164 compliant phone number validation using Google's libphonenumber
//! via the `phonenumber` crate.
//!
//! # Example
//!
//! ```
//! use stilltypes::phone::PhoneNumber;
//!
//! // Valid phone numbers (E.164 format)
//! let us = PhoneNumber::new("+14155551234".to_string());
//! assert!(us.is_ok());
//!
//! let uk = PhoneNumber::new("+442071234567".to_string());
//! assert!(uk.is_ok());
//!
//! // Invalid phone numbers fail validation
//! let invalid = PhoneNumber::new("not-a-phone".to_string());
//! assert!(invalid.is_err());
//! ```

use crate::error::{DomainError, DomainErrorKind};
use phonenumber::{Mode, parse};
use stillwater::refined::{Predicate, Refined};

/// E.164 international phone number predicate.
///
/// Validates phone numbers according to the ITU-T E.164 standard.
/// Uses Google's libphonenumber via the `phonenumber` crate for proper
/// international phone number handling.
///
/// # Example
///
/// ```
/// use stilltypes::phone::PhoneNumber;
///
/// // US number
/// let us = PhoneNumber::new("+14155551234".to_string());
/// assert!(us.is_ok());
///
/// // UK number
/// let uk = PhoneNumber::new("+442071234567".to_string());
/// assert!(uk.is_ok());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidPhoneNumber;

impl Predicate<String> for ValidPhoneNumber {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.is_empty() {
            return Err(DomainError {
                format_name: "phone number",
                value: value.clone(),
                reason: DomainErrorKind::Empty,
                example: "+14155551234",
            });
        }

        let parsed = parse(None, value).map_err(|_| DomainError {
            format_name: "phone number",
            value: value.clone(),
            reason: DomainErrorKind::InvalidFormat {
                expected: "E.164 format (+[country][number])",
            },
            example: "+14155551234",
        })?;

        if phonenumber::is_valid(&parsed) {
            Ok(())
        } else {
            Err(DomainError {
                format_name: "phone number",
                value: value.clone(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "valid phone number for region",
                },
                example: "+14155551234",
            })
        }
    }

    fn description() -> &'static str {
        "E.164 phone number"
    }
}

/// Validated E.164 phone number.
///
/// A phone number that has been validated according to the E.164 standard.
///
/// # Example
///
/// ```
/// use stilltypes::phone::{PhoneNumber, PhoneNumberExt};
///
/// let phone = PhoneNumber::new("+1 (415) 555-1234".to_string()).unwrap();
///
/// // Normalize to E.164 for storage
/// assert_eq!(phone.to_e164(), "+14155551234");
/// ```
pub type PhoneNumber = Refined<String, ValidPhoneNumber>;

/// Extension trait for phone number operations.
///
/// Provides methods for working with validated phone numbers.
///
/// # Example
///
/// ```
/// use stilltypes::phone::{PhoneNumber, PhoneNumberExt};
///
/// let phone = PhoneNumber::new("+1 (415) 555-1234".to_string()).unwrap();
/// assert_eq!(phone.to_e164(), "+14155551234");
/// assert_eq!(phone.country_code(), 1);
/// ```
pub trait PhoneNumberExt {
    /// Normalize to E.164 format.
    ///
    /// Strips all formatting and returns the canonical E.164 representation
    /// suitable for storage and comparison.
    ///
    /// # Example
    ///
    /// ```
    /// use stilltypes::phone::{PhoneNumber, PhoneNumberExt};
    ///
    /// let phone = PhoneNumber::new("+1 (415) 555-1234".to_string()).unwrap();
    /// assert_eq!(phone.to_e164(), "+14155551234");
    ///
    /// let uk = PhoneNumber::new("+44 20 7123 4567".to_string()).unwrap();
    /// assert_eq!(uk.to_e164(), "+442071234567");
    /// ```
    fn to_e164(&self) -> String;

    /// Get the country code.
    ///
    /// Returns the numeric country code (e.g., 1 for US, 44 for UK).
    ///
    /// # Example
    ///
    /// ```
    /// use stilltypes::phone::{PhoneNumber, PhoneNumberExt};
    ///
    /// let us = PhoneNumber::new("+14155551234".to_string()).unwrap();
    /// assert_eq!(us.country_code(), 1);
    ///
    /// let uk = PhoneNumber::new("+442071234567".to_string()).unwrap();
    /// assert_eq!(uk.country_code(), 44);
    /// ```
    fn country_code(&self) -> u16;
}

impl PhoneNumberExt for PhoneNumber {
    fn to_e164(&self) -> String {
        let parsed = parse(None, self.get()).expect("already validated");
        parsed.format().mode(Mode::E164).to_string()
    }

    fn country_code(&self) -> u16 {
        let parsed = parse(None, self.get()).expect("already validated");
        parsed.code().value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Valid phone numbers
    #[test]
    fn valid_us_e164() {
        assert!(PhoneNumber::new("+14155551234".to_string()).is_ok());
    }

    #[test]
    fn valid_us_formatted() {
        assert!(PhoneNumber::new("+1 (415) 555-1234".to_string()).is_ok());
    }

    #[test]
    fn valid_uk_e164() {
        assert!(PhoneNumber::new("+442071234567".to_string()).is_ok());
    }

    #[test]
    fn valid_uk_formatted() {
        assert!(PhoneNumber::new("+44 20 7123 4567".to_string()).is_ok());
    }

    #[test]
    fn valid_france() {
        assert!(PhoneNumber::new("+33123456789".to_string()).is_ok());
    }

    #[test]
    fn valid_germany() {
        assert!(PhoneNumber::new("+4930123456".to_string()).is_ok());
    }

    #[test]
    fn valid_japan() {
        assert!(PhoneNumber::new("+81312345678".to_string()).is_ok());
    }

    // Invalid phone numbers
    #[test]
    fn invalid_empty() {
        let result = PhoneNumber::new(String::new());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.reason, DomainErrorKind::Empty));
    }

    #[test]
    fn invalid_no_country_code() {
        assert!(PhoneNumber::new("4155551234".to_string()).is_err());
    }

    #[test]
    fn invalid_too_short() {
        assert!(PhoneNumber::new("+1234".to_string()).is_err());
    }

    #[test]
    fn invalid_too_long() {
        assert!(PhoneNumber::new("+12345678901234567890".to_string()).is_err());
    }

    #[test]
    fn invalid_letters_only() {
        // Vanity numbers with letters are actually parsed by libphonenumber
        // but strings that are only letters should fail
        assert!(PhoneNumber::new("ABCDEFGHIJ".to_string()).is_err());
    }

    #[test]
    fn invalid_random_text() {
        assert!(PhoneNumber::new("not a phone number".to_string()).is_err());
    }

    // Normalization tests
    #[test]
    fn to_e164_strips_formatting() {
        let phone = PhoneNumber::new("+1 (415) 555-1234".to_string()).unwrap();
        assert_eq!(phone.to_e164(), "+14155551234");
    }

    #[test]
    fn to_e164_preserves_country_code() {
        let phone = PhoneNumber::new("+44 20 7123 4567".to_string()).unwrap();
        assert_eq!(phone.to_e164(), "+442071234567");
    }

    #[test]
    fn to_e164_idempotent() {
        let phone = PhoneNumber::new("+14155551234".to_string()).unwrap();
        assert_eq!(phone.to_e164(), "+14155551234");
    }

    // Country code tests
    #[test]
    fn country_code_us() {
        let phone = PhoneNumber::new("+14155551234".to_string()).unwrap();
        assert_eq!(phone.country_code(), 1);
    }

    #[test]
    fn country_code_uk() {
        let phone = PhoneNumber::new("+442071234567".to_string()).unwrap();
        assert_eq!(phone.country_code(), 44);
    }

    #[test]
    fn country_code_france() {
        let phone = PhoneNumber::new("+33123456789".to_string()).unwrap();
        assert_eq!(phone.country_code(), 33);
    }

    // Error message tests
    #[test]
    fn error_includes_format_name() {
        let result = PhoneNumber::new("invalid".to_string());
        let err = result.unwrap_err();
        assert_eq!(err.format_name, "phone number");
    }

    #[test]
    fn error_includes_example() {
        let result = PhoneNumber::new("invalid".to_string());
        let err = result.unwrap_err();
        assert_eq!(err.example, "+14155551234");
    }

    #[test]
    fn error_display_is_readable() {
        let result = PhoneNumber::new("invalid".to_string());
        let err = result.unwrap_err();
        let display = err.to_string();
        assert!(display.contains("phone number"));
        assert!(display.contains("+14155551234"));
    }

    #[test]
    fn description_returns_expected() {
        assert_eq!(ValidPhoneNumber::description(), "E.164 phone number");
    }
}
