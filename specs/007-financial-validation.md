---
number: 7
title: Financial Validation
category: foundation
priority: medium
status: draft
dependencies: [1, 2]
created: 2025-12-21
---

# Specification 7: Financial Validation

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: Spec 1 (Project Foundation), Spec 2 (Error Types)

## Context

Financial identifiers require careful validation due to:
- **Security**: These values are sensitive and must never be logged in full
- **Checksum validation**: Both IBAN and credit cards have built-in checksums
- **Fraud prevention**: Invalid numbers should be rejected early
- **Compliance**: Proper validation is often required by regulations

### IBAN (International Bank Account Number)
- ISO 13616 standard
- Country prefix + check digits + BBAN (Basic Bank Account Number)
- Length varies by country (15-34 characters)
- Example: `DE89370400440532013000` (Germany)

### Credit Card Numbers
- Luhn algorithm (mod 10) checksum
- 13-19 digits depending on issuer
- Issuer identification via first digits (BIN/IIN)
- Example: `4111111111111111` (Visa test card)

## Objective

Implement financial identifier validation with:
- `ValidIban` predicate with checksum validation
- `ValidCreditCard` predicate with Luhn validation
- Automatic masking of sensitive values in error messages
- `Iban` and `CreditCardNumber` type aliases

## Requirements

### Functional Requirements

1. **ValidIban Predicate**
   - Implements `stillwater::refined::Predicate<String>`
   - Uses `iban_validate` crate for ISO 13616 compliance
   - Validates structure and checksum
   - Returns `DomainError` with masked value on failure

2. **ValidCreditCard Predicate**
   - Implements `stillwater::refined::Predicate<String>`
   - Uses `creditcard` crate for Luhn validation
   - Validates checksum digit
   - Returns `DomainError` with masked value on failure

3. **Value Masking**
   - IBAN: Show first 4 and last 4 characters (`DE89****3000`)
   - Credit Card: Show only last 4 digits (`****1111`)
   - Masking applied automatically in error messages

4. **Type Aliases**
   - `Iban = Refined<String, ValidIban>`
   - `CreditCardNumber = Refined<String, ValidCreditCard>`

5. **Feature Gating**
   - Only compiled when `financial` feature is enabled

### Non-Functional Requirements

1. **Security**: Never log/display full financial identifiers
2. **Checksum Validation**: Catch typos and invalid numbers early
3. **Zero Unsafe Code**: Pure safe Rust implementation

## Acceptance Criteria

- [ ] `ValidIban` struct implements `Predicate<String>`
- [ ] `ValidCreditCard` struct implements `Predicate<String>`
- [ ] `Iban` and `CreditCardNumber` type aliases defined
- [ ] Invalid values return `DomainErrorKind::InvalidChecksum`
- [ ] Error messages contain masked values, never full numbers
- [ ] Compiles only with `financial` feature enabled
- [ ] Unit tests for valid and invalid values
- [ ] Masking functions work correctly for edge cases

## Technical Details

### Implementation Approach

```rust
// src/financial.rs

use crate::error::{DomainError, DomainErrorKind};
use creditcard::CreditCard;
use iban_validate::Iban as IbanImpl;
use stillwater::refined::{Predicate, Refined};

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

        value.parse::<IbanImpl>().map(|_| ()).map_err(|_| DomainError {
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

        let card = CreditCard::from(value.as_str());
        if card.is_valid() {
            Ok(())
        } else {
            Err(DomainError {
                format_name: "credit card number",
                value: mask_card(value),
                reason: DomainErrorKind::InvalidChecksum,
                example: "4111111111111111",
            })
        }
    }

    fn description() -> &'static str {
        "credit card number"
    }
}

/// Validated IBAN.
pub type Iban = Refined<String, ValidIban>;

/// Validated credit card number.
pub type CreditCardNumber = Refined<String, ValidCreditCard>;

/// Mask credit card number, showing only last 4 digits.
///
/// # Examples
/// ```
/// assert_eq!(mask_card("4111111111111111"), "****1111");
/// assert_eq!(mask_card("4111-1111-1111-1111"), "****1111");
/// assert_eq!(mask_card("123"), "****");
/// ```
fn mask_card(card: &str) -> String {
    let digits: String = card.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 4 {
        format!("****{}", &digits[digits.len() - 4..])
    } else {
        "****".to_string()
    }
}

/// Mask IBAN, showing first 4 and last 4 characters.
///
/// # Examples
/// ```
/// assert_eq!(mask_iban("DE89370400440532013000"), "DE89****3000");
/// assert_eq!(mask_iban("GB82WEST12345698765432"), "GB82****5432");
/// assert_eq!(mask_iban("SHORT"), "****");
/// ```
fn mask_iban(iban: &str) -> String {
    if iban.len() > 8 {
        format!("{}****{}", &iban[..4], &iban[iban.len() - 4..])
    } else {
        "****".to_string()
    }
}

impl Iban {
    /// Get country code (first 2 characters).
    ///
    /// # Example
    /// ```
    /// use platypus::financial::Iban;
    ///
    /// let iban = Iban::new("DE89370400440532013000".to_string()).unwrap();
    /// assert_eq!(iban.country_code(), "DE");
    /// ```
    pub fn country_code(&self) -> &str {
        &self.get()[..2]
    }

    /// Get masked representation for display.
    ///
    /// # Example
    /// ```
    /// use platypus::financial::Iban;
    ///
    /// let iban = Iban::new("DE89370400440532013000".to_string()).unwrap();
    /// assert_eq!(iban.masked(), "DE89****3000");
    /// ```
    pub fn masked(&self) -> String {
        mask_iban(self.get())
    }
}

impl CreditCardNumber {
    /// Get masked representation for display (last 4 digits).
    ///
    /// # Example
    /// ```
    /// use platypus::financial::CreditCardNumber;
    ///
    /// let card = CreditCardNumber::new("4111111111111111".to_string()).unwrap();
    /// assert_eq!(card.masked(), "****1111");
    /// ```
    pub fn masked(&self) -> String {
        mask_card(self.get())
    }

    /// Get last 4 digits.
    ///
    /// # Example
    /// ```
    /// use platypus::financial::CreditCardNumber;
    ///
    /// let card = CreditCardNumber::new("4111111111111111".to_string()).unwrap();
    /// assert_eq!(card.last_four(), "1111");
    /// ```
    pub fn last_four(&self) -> String {
        let digits: String = self.get().chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 4 {
            digits[digits.len() - 4..].to_string()
        } else {
            digits
        }
    }
}
```

### Module Integration

```rust
// In src/lib.rs
#[cfg(feature = "financial")]
pub mod financial;

#[cfg(feature = "financial")]
pub use financial::{CreditCardNumber, Iban, ValidCreditCard, ValidIban};
```

## Dependencies

- **Prerequisites**: Spec 1 (crate structure), Spec 2 (error types)
- **Affected Components**: prelude.rs, lib.rs exports
- **External Dependencies**:
  - `iban_validate` crate (version 4)
  - `creditcard` crate (version 0.3)

## Testing Strategy

### IBAN Valid Cases

```rust
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
    // Should accept lowercase
    assert!(Iban::new("de89370400440532013000".to_string()).is_ok());
}

#[test]
fn valid_iban_with_spaces() {
    // Some implementations accept spaces
    assert!(Iban::new("DE89 3704 0044 0532 0130 00".to_string()).is_ok());
}
```

### IBAN Invalid Cases

```rust
#[test]
fn invalid_iban_empty() {
    let result = Iban::new(String::new());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.reason, DomainErrorKind::Empty));
    assert_eq!(err.value, "****");  // Masked even for empty
}

#[test]
fn invalid_iban_checksum() {
    // DE89... with wrong check digits
    let result = Iban::new("DE00370400440532013000".to_string());
    assert!(result.is_err());
}

#[test]
fn invalid_iban_too_short() {
    assert!(Iban::new("DE89".to_string()).is_err());
}

#[test]
fn invalid_iban_wrong_country() {
    assert!(Iban::new("XX89370400440532013000".to_string()).is_err());
}
```

### Credit Card Valid Cases

```rust
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
```

### Credit Card Invalid Cases

```rust
#[test]
fn invalid_card_empty() {
    let result = CreditCardNumber::new(String::new());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.reason, DomainErrorKind::Empty));
}

#[test]
fn invalid_card_luhn() {
    // Wrong checksum digit
    let result = CreditCardNumber::new("4111111111111112".to_string());
    assert!(result.is_err());
}

#[test]
fn invalid_card_too_short() {
    assert!(CreditCardNumber::new("411111".to_string()).is_err());
}

#[test]
fn invalid_card_letters() {
    assert!(CreditCardNumber::new("4111111111111abc".to_string()).is_err());
}
```

### Masking Tests

```rust
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
    assert!(!err.value.contains("4111"));  // No full number
}
```

## Documentation Requirements

- **Code Documentation**: Doc comments emphasizing security
- **Security Notes**: Explain masking behavior
- **User Documentation**: Usage examples in README

## Implementation Notes

- **Security First**: Always mask sensitive values, even in debug/test scenarios
- The masking functions are private but tested via public API
- `is_valid()` on creditcard crate validates Luhn checksum
- Consider adding card type detection (Visa, Mastercard, etc.) in future
- IBAN validation includes country-specific length and format checks

## Migration and Compatibility

N/A - New type with no existing code to migrate.

## Future Considerations

Not in scope for initial implementation:
- `CreditCardNumber::card_type()` - detect Visa, Mastercard, etc.
- `CreditCardNumber::issuer_id()` - first 6 digits (BIN/IIN)
- `Iban::bank_code()` - extract bank identifier
- `Iban::account_number()` - extract account number
- Currency validation (ISO 4217)
- Amount validation with decimal precision
