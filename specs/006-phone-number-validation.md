---
number: 6
title: Phone Number Validation
category: foundation
priority: medium
status: draft
dependencies: [1, 2]
created: 2025-12-21
---

# Specification 6: Phone Number Validation

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: Spec 1 (Project Foundation), Spec 2 (Error Types)

## Context

Phone number validation is surprisingly complex due to:
- Different country formats (length, prefixes, area codes)
- Multiple valid representations (with/without country code, spaces, dashes)
- E.164 international standard for storage/comparison
- Local vs international formats

The E.164 standard defines the international format: `+[country code][subscriber number]` with a maximum of 15 digits. Examples:
- `+14155551234` (US)
- `+442071234567` (UK)
- `+33123456789` (France)

Platypus uses the `phonenumber` crate which implements libphonenumber (Google's phone validation library) for proper international phone number handling.

## Objective

Implement E.164 phone number validation with:
- `ValidPhoneNumber` predicate for E.164 format
- `PhoneNumber` type alias
- `to_e164()` normalization helper for consistent storage

## Requirements

### Functional Requirements

1. **ValidPhoneNumber Predicate**
   - Implements `stillwater::refined::Predicate<String>`
   - Uses `phonenumber` crate for parsing and validation
   - Validates against E.164 format rules
   - Handles international formats with country codes
   - Returns `DomainError` with appropriate context on failure

2. **PhoneNumber Type Alias**
   - `PhoneNumber = Refined<String, ValidPhoneNumber>`
   - Zero-cost abstraction over validated String

3. **Normalization Helper**
   - `PhoneNumber::to_e164()` - converts to canonical E.164 format
   - Strips formatting, ensures leading `+`

4. **Feature Gating**
   - Only compiled when `phone` feature is enabled

### Non-Functional Requirements

1. **ITU-T E.164 Compliance**: Maximum 15 digits, proper country codes
2. **International Support**: All countries supported by libphonenumber
3. **Zero Unsafe Code**: Pure safe Rust implementation

## Acceptance Criteria

- [ ] `ValidPhoneNumber` struct implements `Predicate<String>`
- [ ] `PhoneNumber` type alias defined and exported
- [ ] Valid E.164 numbers are accepted
- [ ] Invalid numbers return `DomainErrorKind::InvalidFormat`
- [ ] `to_e164()` normalizes to canonical format
- [ ] Compiles only with `phone` feature enabled
- [ ] Unit tests for various international formats
- [ ] Error messages are user-friendly

## Technical Details

### Implementation Approach

```rust
// src/phone.rs

use crate::error::{DomainError, DomainErrorKind};
use phonenumber::{parse, Mode};
use stillwater::refined::{Predicate, Refined};

/// E.164 international phone number.
///
/// Validates phone numbers according to the ITU-T E.164 standard.
/// Uses Google's libphonenumber via the `phonenumber` crate.
///
/// # Example
/// ```
/// use platypus::phone::PhoneNumber;
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
/// ```
/// use platypus::phone::PhoneNumber;
///
/// let phone = PhoneNumber::new("+1 (415) 555-1234".to_string()).unwrap();
///
/// // Normalize to E.164 for storage
/// assert_eq!(phone.to_e164(), "+14155551234");
/// ```
pub type PhoneNumber = Refined<String, ValidPhoneNumber>;

impl PhoneNumber {
    /// Normalize to E.164 format.
    ///
    /// Strips all formatting and returns the canonical E.164 representation
    /// suitable for storage and comparison.
    ///
    /// # Example
    /// ```
    /// use platypus::phone::PhoneNumber;
    ///
    /// let phone = PhoneNumber::new("+1 (415) 555-1234".to_string()).unwrap();
    /// assert_eq!(phone.to_e164(), "+14155551234");
    ///
    /// let uk = PhoneNumber::new("+44 20 7123 4567".to_string()).unwrap();
    /// assert_eq!(uk.to_e164(), "+442071234567");
    /// ```
    pub fn to_e164(&self) -> String {
        let parsed = parse(None, self.get()).expect("already validated");
        parsed.format().mode(Mode::E164).to_string()
    }

    /// Get the country code.
    ///
    /// Returns the numeric country code (e.g., 1 for US, 44 for UK).
    ///
    /// # Example
    /// ```
    /// use platypus::phone::PhoneNumber;
    ///
    /// let us = PhoneNumber::new("+14155551234".to_string()).unwrap();
    /// assert_eq!(us.country_code(), 1);
    ///
    /// let uk = PhoneNumber::new("+442071234567".to_string()).unwrap();
    /// assert_eq!(uk.country_code(), 44);
    /// ```
    pub fn country_code(&self) -> u16 {
        let parsed = parse(None, self.get()).expect("already validated");
        parsed.code().value()
    }
}
```

### Module Integration

```rust
// In src/lib.rs
#[cfg(feature = "phone")]
pub mod phone;

#[cfg(feature = "phone")]
pub use phone::{PhoneNumber, ValidPhoneNumber};
```

## Dependencies

- **Prerequisites**: Spec 1 (crate structure), Spec 2 (error types)
- **Affected Components**: prelude.rs, lib.rs exports
- **External Dependencies**: `phonenumber` crate (version 0.3)

## Testing Strategy

### Valid Phone Numbers

```rust
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
```

### Invalid Phone Numbers

```rust
#[test]
fn invalid_empty() {
    let result = PhoneNumber::new(String::new());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.reason, DomainErrorKind::Empty));
}

#[test]
fn invalid_no_country_code() {
    // Without country code, we can't validate
    assert!(PhoneNumber::new("4155551234".to_string()).is_err());
}

#[test]
fn invalid_too_short() {
    assert!(PhoneNumber::new("+1234".to_string()).is_err());
}

#[test]
fn invalid_too_long() {
    // E.164 max is 15 digits
    assert!(PhoneNumber::new("+12345678901234567890".to_string()).is_err());
}

#[test]
fn invalid_letters() {
    assert!(PhoneNumber::new("+1415555CALL".to_string()).is_err());
}

#[test]
fn invalid_random_text() {
    assert!(PhoneNumber::new("not a phone number".to_string()).is_err());
}
```

### Normalization Tests

```rust
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
```

### Country Code Tests

```rust
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
```

## Documentation Requirements

- **Code Documentation**: Doc comments with international examples
- **Format Guide**: Explain E.164 format and normalization
- **User Documentation**: Usage examples in README

## Implementation Notes

- The `phonenumber` crate is a Rust port of Google's libphonenumber
- Parsing without a default region (`None`) requires the number to include country code
- `is_valid()` checks if the number is valid for the detected region
- Consider adding region-specific validation in future (e.g., `UsPhoneNumber`)
- Note: The `phonenumber` crate has a large compiled size due to metadata

## Migration and Compatibility

N/A - New type with no existing code to migrate.

## Future Considerations

Not in scope for initial implementation:
- `PhoneNumber::national_format()` - format without country code
- `PhoneNumber::international_format()` - format with spaces/dashes
- `PhoneNumber::region()` - ISO country code (US, GB, etc.)
- Region-specific types (UsPhoneNumber, UkPhoneNumber)
- SMS capability detection
