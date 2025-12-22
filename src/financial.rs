//! Financial validation types.
//!
//! Provides IBAN and credit card number validation with built-in security
//! features like automatic value masking in error messages.
//!
//! # Security
//!
//! Financial identifiers are sensitive data. This module automatically masks
//! values in error messages to prevent leakage in logs:
//! - IBAN: Shows first 4 and last 4 characters (`DE89****3000`)
//! - Credit Card: Shows only last 4 digits (`****1111`)
//!
//! # Example
//!
//! ```
//! use platypus::financial::{Iban, CreditCardNumber};
//!
//! // Valid IBAN (German example)
//! let iban = Iban::new("DE89370400440532013000".to_string());
//! assert!(iban.is_ok());
//!
//! // Valid credit card (Visa test card)
//! let card = CreditCardNumber::new("4111111111111111".to_string());
//! assert!(card.is_ok());
//! ```

use crate::error::{DomainError, DomainErrorKind};
use creditcard::CreditCard;
use iban::Iban as IbanImpl;
use stillwater::refined::{Predicate, Refined};

/// Mask credit card number, showing only last 4 digits.
fn mask_card(card: &str) -> String {
    let digits: String = card.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 4 {
        format!("****{}", &digits[digits.len() - 4..])
    } else {
        "****".to_string()
    }
}

/// Mask IBAN, showing first 4 and last 4 characters.
fn mask_iban(iban: &str) -> String {
    if iban.len() > 8 {
        format!("{}****{}", &iban[..4], &iban[iban.len() - 4..])
    } else {
        "****".to_string()
    }
}

/// Valid IBAN (International Bank Account Number).
///
/// Validates according to ISO 13616, including country-specific
/// formats and check digit validation.
///
/// # Security
/// Error messages automatically mask the IBAN value to prevent
/// sensitive data leakage in logs.
///
/// # Example
/// ```
/// use platypus::financial::Iban;
///
/// let iban = Iban::new("DE89370400440532013000".to_string());
/// assert!(iban.is_ok());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidIban;

impl Predicate<String> for ValidIban {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.is_empty() {
            return Err(DomainError {
                format_name: "IBAN",
                value: "****".to_string(),
                reason: DomainErrorKind::Empty,
                example: "DE89370400440532013000",
            });
        }

        // Convert to uppercase for validation (IBANs are case-insensitive)
        let normalized = value.to_uppercase();
        normalized
            .parse::<IbanImpl>()
            .map(|_| ())
            .map_err(|_| DomainError {
                format_name: "IBAN",
                value: mask_iban(value),
                reason: DomainErrorKind::InvalidChecksum,
                example: "DE89370400440532013000",
            })
    }

    fn description() -> &'static str {
        "IBAN"
    }
}

/// Valid credit card number (Luhn validated).
///
/// Validates credit card numbers using the Luhn algorithm (mod 10 checksum).
/// Supports all major card networks (Visa, Mastercard, Amex, etc.).
///
/// # Security
/// Error messages automatically mask the card number to prevent
/// sensitive data leakage in logs. Only the last 4 digits are shown.
///
/// # Example
/// ```
/// use platypus::financial::CreditCardNumber;
///
/// // Visa test card
/// let card = CreditCardNumber::new("4111111111111111".to_string());
/// assert!(card.is_ok());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidCreditCard;

impl Predicate<String> for ValidCreditCard {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.is_empty() {
            return Err(DomainError {
                format_name: "credit card number",
                value: "****".to_string(),
                reason: DomainErrorKind::Empty,
                example: "4111111111111111",
            });
        }

        // Remove spaces and dashes for parsing
        let cleaned: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
        cleaned
            .parse::<CreditCard>()
            .map(|_| ())
            .map_err(|_| DomainError {
                format_name: "credit card number",
                value: mask_card(value),
                reason: DomainErrorKind::InvalidChecksum,
                example: "4111111111111111",
            })
    }

    fn description() -> &'static str {
        "credit card number"
    }
}

/// Validated IBAN.
pub type Iban = Refined<String, ValidIban>;

/// Validated credit card number.
pub type CreditCardNumber = Refined<String, ValidCreditCard>;

/// Extension trait for IBAN operations.
pub trait IbanExt {
    /// Get country code (first 2 characters).
    fn country_code(&self) -> &str;
    /// Get masked representation for display.
    fn masked(&self) -> String;
}

impl IbanExt for Iban {
    /// Get country code (first 2 characters).
    ///
    /// # Example
    /// ```
    /// use platypus::financial::{Iban, IbanExt};
    ///
    /// let iban = Iban::new("DE89370400440532013000".to_string()).unwrap();
    /// assert_eq!(iban.country_code(), "DE");
    /// ```
    fn country_code(&self) -> &str {
        &self.get()[..2]
    }

    /// Get masked representation for display.
    ///
    /// # Example
    /// ```
    /// use platypus::financial::{Iban, IbanExt};
    ///
    /// let iban = Iban::new("DE89370400440532013000".to_string()).unwrap();
    /// assert_eq!(iban.masked(), "DE89****3000");
    /// ```
    fn masked(&self) -> String {
        mask_iban(self.get())
    }
}

/// Extension trait for credit card operations.
pub trait CreditCardExt {
    /// Get masked representation for display (last 4 digits).
    fn masked(&self) -> String;
    /// Get last 4 digits.
    fn last_four(&self) -> String;
}

impl CreditCardExt for CreditCardNumber {
    /// Get masked representation for display (last 4 digits).
    ///
    /// # Example
    /// ```
    /// use platypus::financial::{CreditCardNumber, CreditCardExt};
    ///
    /// let card = CreditCardNumber::new("4111111111111111".to_string()).unwrap();
    /// assert_eq!(card.masked(), "****1111");
    /// ```
    fn masked(&self) -> String {
        mask_card(self.get())
    }

    /// Get last 4 digits.
    ///
    /// # Example
    /// ```
    /// use platypus::financial::{CreditCardNumber, CreditCardExt};
    ///
    /// let card = CreditCardNumber::new("4111111111111111".to_string()).unwrap();
    /// assert_eq!(card.last_four(), "1111");
    /// ```
    fn last_four(&self) -> String {
        let digits: String = self.get().chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 4 {
            digits[digits.len() - 4..].to_string()
        } else {
            digits
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // IBAN valid cases
    #[test]
    fn valid_iban_germany() {
        assert!(Iban::new("DE89370400440532013000".to_string()).is_ok());
    }

    #[test]
    fn valid_iban_uk() {
        assert!(Iban::new("GB82WEST12345698765432".to_string()).is_ok());
    }

    #[test]
    fn valid_iban_france() {
        assert!(Iban::new("FR7630006000011234567890189".to_string()).is_ok());
    }

    #[test]
    fn valid_iban_lowercase() {
        assert!(Iban::new("de89370400440532013000".to_string()).is_ok());
    }

    // IBAN invalid cases
    #[test]
    fn invalid_iban_empty() {
        let result = Iban::new(String::new());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.reason, DomainErrorKind::Empty));
        assert_eq!(err.value, "****");
    }

    #[test]
    fn invalid_iban_checksum() {
        let result = Iban::new("DE00370400440532013000".to_string());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.reason, DomainErrorKind::InvalidChecksum));
    }

    #[test]
    fn invalid_iban_too_short() {
        assert!(Iban::new("DE89".to_string()).is_err());
    }

    #[test]
    fn invalid_iban_wrong_country() {
        assert!(Iban::new("XX89370400440532013000".to_string()).is_err());
    }

    // Credit Card valid cases
    #[test]
    fn valid_visa_test_card() {
        assert!(CreditCardNumber::new("4111111111111111".to_string()).is_ok());
    }

    #[test]
    fn valid_mastercard_test() {
        assert!(CreditCardNumber::new("5500000000000004".to_string()).is_ok());
    }

    #[test]
    fn valid_amex_test() {
        assert!(CreditCardNumber::new("340000000000009".to_string()).is_ok());
    }

    #[test]
    fn valid_with_spaces() {
        assert!(CreditCardNumber::new("4111 1111 1111 1111".to_string()).is_ok());
    }

    #[test]
    fn valid_with_dashes() {
        assert!(CreditCardNumber::new("4111-1111-1111-1111".to_string()).is_ok());
    }

    // Credit Card invalid cases
    #[test]
    fn invalid_card_empty() {
        let result = CreditCardNumber::new(String::new());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.reason, DomainErrorKind::Empty));
    }

    #[test]
    fn invalid_card_luhn() {
        let result = CreditCardNumber::new("4111111111111112".to_string());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err.reason, DomainErrorKind::InvalidChecksum));
    }

    #[test]
    fn invalid_card_too_short() {
        assert!(CreditCardNumber::new("411111".to_string()).is_err());
    }

    #[test]
    fn invalid_card_letters() {
        assert!(CreditCardNumber::new("4111111111111abc".to_string()).is_err());
    }

    // Masking tests
    #[test]
    fn mask_card_shows_last_four() {
        let card = CreditCardNumber::new("4111111111111111".to_string()).unwrap();
        assert_eq!(card.masked(), "****1111");
    }

    #[test]
    fn mask_iban_shows_prefix_and_suffix() {
        let iban = Iban::new("DE89370400440532013000".to_string()).unwrap();
        assert_eq!(iban.masked(), "DE89****3000");
    }

    #[test]
    fn error_contains_masked_value() {
        let result = CreditCardNumber::new("4111111111111112".to_string());
        let err = result.unwrap_err();
        assert!(err.value.starts_with("****"));
        assert!(!err.value.contains("4111"));
    }

    // Helper method tests
    #[test]
    fn iban_country_code() {
        let iban = Iban::new("DE89370400440532013000".to_string()).unwrap();
        assert_eq!(iban.country_code(), "DE");
    }

    #[test]
    fn credit_card_last_four() {
        let card = CreditCardNumber::new("4111111111111111".to_string()).unwrap();
        assert_eq!(card.last_four(), "1111");
    }

    #[test]
    fn description_returns_expected_iban() {
        assert_eq!(ValidIban::description(), "IBAN");
    }

    #[test]
    fn description_returns_expected_credit_card() {
        assert_eq!(ValidCreditCard::description(), "credit card number");
    }

    // Edge case masking tests
    #[test]
    fn mask_card_short_input() {
        assert_eq!(mask_card("123"), "****");
    }

    #[test]
    fn mask_card_with_separators() {
        assert_eq!(mask_card("4111-1111-1111-1111"), "****1111");
    }

    #[test]
    fn mask_iban_short_input() {
        assert_eq!(mask_iban("SHORT"), "****");
    }
}
