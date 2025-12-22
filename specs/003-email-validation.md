---
number: 3
title: Email Validation
category: foundation
priority: high
status: draft
dependencies: [1, 2]
created: 2025-12-21
---

# Specification 3: Email Validation

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: Spec 1 (Project Foundation), Spec 2 (Error Types)

## Context

Email addresses are one of the most commonly validated domain types. RFC 5321 defines the format for email addresses, which is more permissive than many naive implementations assume. Examples of valid addresses that many validators reject:

- `"quoted local part"@example.com`
- `user+tag@example.com`
- `user@localhost`
- `user@[192.168.1.1]` (IP address literal)

Platypus uses the `email_address` crate for RFC-compliant validation rather than implementing a custom regex, following the principle of "Parse, Don't Validate" - we delegate to a well-tested library.

## Objective

Implement RFC 5321 compliant email validation through the `ValidEmail` predicate and `Email` type alias, with proper integration with stillwater's `Refined` type and `DomainError` for rich error messages.

## Requirements

### Functional Requirements

1. **ValidEmail Predicate**
   - Implements `stillwater::refined::Predicate<String>`
   - Uses `email_address::EmailAddress::is_valid()` for RFC 5321 compliance
   - Returns `DomainError` with appropriate context on failure
   - Provides `description()` returning "RFC 5321 email address"

2. **Email Type Alias**
   - `pub type Email = Refined<String, ValidEmail>;`
   - Zero-cost abstraction over validated String

3. **Validation Behavior**
   - Empty strings return `DomainErrorKind::Empty`
   - Invalid format returns `DomainErrorKind::InvalidFormat`
   - Example in error: `"user@example.com"`

4. **Feature Gating**
   - Only compiled when `email` feature is enabled
   - Dependency on `email_address` crate is optional

### Non-Functional Requirements

1. **RFC Compliance**: Follows RFC 5321 specification
2. **Performance**: Minimal overhead beyond library call
3. **Zero Unsafe Code**: Pure safe Rust implementation

## Acceptance Criteria

- [ ] `ValidEmail` struct implements `Predicate<String>`
- [ ] `Email` type alias is defined and exported
- [ ] Empty email returns `DomainErrorKind::Empty`
- [ ] Invalid email returns `DomainErrorKind::InvalidFormat`
- [ ] Error includes example "user@example.com"
- [ ] `description()` returns "RFC 5321 email address"
- [ ] Compiles only with `email` feature enabled
- [ ] All RFC 5321 edge cases pass (see test cases)
- [ ] Unit tests for valid and invalid cases
- [ ] Integration with stillwater's `Refined::new()` works

## Technical Details

### Implementation Approach

```rust
// src/email.rs

use crate::error::{DomainError, DomainErrorKind};
use email_address::EmailAddress;
use stillwater::refined::{Predicate, Refined};

/// RFC 5321 compliant email address predicate.
///
/// Uses the `email_address` crate for validation to ensure compliance
/// with the RFC specification, including edge cases like quoted local parts.
///
/// # Example
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
```

### Module Integration

```rust
// In src/lib.rs
#[cfg(feature = "email")]
pub mod email;

#[cfg(feature = "email")]
pub use email::{Email, ValidEmail};
```

### Prelude Export

```rust
// In src/prelude.rs
#[cfg(feature = "email")]
pub use crate::email::{Email, ValidEmail};
```

## Dependencies

- **Prerequisites**: Spec 1 (crate structure), Spec 2 (error types)
- **Affected Components**: prelude.rs, lib.rs exports
- **External Dependencies**: `email_address` crate (version 0.2)

## Testing Strategy

### Valid Cases (should pass)

```rust
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
    // RFC allows this but may want to consider if practical
    assert!(Email::new("user@localhost".to_string()).is_ok());
}

#[test]
fn valid_ip_literal() {
    assert!(Email::new("user@[192.168.1.1]".to_string()).is_ok());
}
```

### Invalid Cases (should fail)

```rust
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
```

### Error Message Tests

```rust
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
```

## Documentation Requirements

- **Code Documentation**: Doc comments with examples on ValidEmail and Email
- **Module Documentation**: Overview of RFC 5321 compliance
- **User Documentation**: Usage examples in README

## Implementation Notes

- The `email_address` crate is well-maintained and follows RFC 5321
- Consider whether to accept `user@localhost` - RFC allows it but may not be practical
- The predicate is deliberately simple - complex email rules (MX lookup, etc.) are out of scope
- Normalize consideration: emails are case-insensitive in domain part - may want helper method

## Migration and Compatibility

N/A - New type with no existing code to migrate.

## Future Considerations

Not in scope for initial implementation:
- `Email::normalize()` - lowercase domain
- `Email::domain()` - extract domain part
- `Email::local()` - extract local part
- DNS MX record validation
