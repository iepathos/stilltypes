---
number: 2
title: Error Types
category: foundation
priority: critical
status: draft
dependencies: [1]
created: 2025-12-21
---

# Specification 2: Error Types

**Category**: foundation
**Priority**: critical
**Status**: draft
**Dependencies**: Spec 1 (Project Foundation)

## Context

All domain predicates in Platypus need rich, contextual error types that provide helpful user-facing messages. Following the Stillwater philosophy that "Errors Should Tell Stories", the error types must include:

- What format was being validated
- The invalid value (optionally masked for sensitive data)
- Why it failed with specific details
- An example of a valid format

These errors must work seamlessly with stillwater's `Validation` error accumulation pattern, allowing multiple validation errors to be collected and reported together.

## Objective

Implement the `DomainError` struct and `DomainErrorKind` enum that serve as the error type for all domain predicates, providing rich context for user-facing error messages.

## Requirements

### Functional Requirements

1. **DomainError Struct**
   - `format_name: &'static str` - What we were validating ("email address", "phone number")
   - `value: String` - The invalid value (may be masked for sensitive data)
   - `reason: DomainErrorKind` - Why it failed
   - `example: &'static str` - Example of valid format

2. **DomainErrorKind Enum**
   - `Empty` - Value cannot be empty
   - `TooLong { max: usize, actual: usize }` - Value exceeds maximum length
   - `TooShort { min: usize, actual: usize }` - Value below minimum length
   - `InvalidFormat { expected: &'static str }` - Wrong format pattern
   - `InvalidCharacter { char: char, position: usize }` - Invalid character at position
   - `InvalidChecksum` - Checksum validation failed
   - `InvalidComponent { component: &'static str, reason: String }` - Specific component invalid

3. **Trait Implementations**
   - `std::fmt::Display` for both types with clear, user-friendly messages
   - `std::error::Error` for `DomainError`
   - `Debug`, `Clone`, `PartialEq`, `Eq` for both types

4. **Display Format**
   - DomainError: `"invalid {format_name}: {reason} (example: {example})"`
   - DomainErrorKind variants have specific messages

### Non-Functional Requirements

1. **Zero-Cost Abstractions**: Static strings where possible (`&'static str`)
2. **Minimal Allocations**: Only `value` and `InvalidComponent::reason` allocate
3. **Type Safety**: Exhaustive enum for error kinds
4. **Composability**: Works with `Vec<DomainError>` for error accumulation

## Acceptance Criteria

- [ ] `DomainError` struct defined with all required fields
- [ ] `DomainErrorKind` enum defined with all variants
- [ ] `Display` implementation for `DomainError` produces readable messages
- [ ] `Display` implementation for `DomainErrorKind` produces readable messages
- [ ] `Error` trait implemented for `DomainError`
- [ ] `Debug`, `Clone`, `PartialEq`, `Eq` derived for both types
- [ ] Unit tests verify Display output for all error variants
- [ ] Works with `Vec<DomainError>` for stillwater's Validation accumulation
- [ ] No clippy warnings

## Technical Details

### Implementation Approach

```rust
// src/error.rs

use std::fmt;

/// Rich error for domain validation failures.
///
/// Includes enough context for helpful user-facing messages.
///
/// # Example
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
```

### Error Accumulation Pattern

```rust
// Example of how errors work with stillwater's Validation
use stillwater::Validation;

fn validate_form(email: String, phone: String) -> Validation<ValidForm, Vec<DomainError>> {
    Validation::all((
        Email::new(email).map_err(|e| vec![e]),
        PhoneNumber::new(phone).map_err(|e| vec![e]),
    ))
    .map(|(email, phone)| ValidForm { email, phone })
}
```

## Dependencies

- **Prerequisites**: Spec 1 (Project Foundation) - crate structure must exist
- **Affected Components**: All future domain predicates will use these error types
- **External Dependencies**: None (uses only std)

## Testing Strategy

- **Unit Tests**: Test Display output for each DomainErrorKind variant
- **Unit Tests**: Test Display output for DomainError with various combinations
- **Unit Tests**: Verify Clone, PartialEq work correctly
- **Integration Tests**: Verify error accumulation with Vec<DomainError>

### Test Cases

```rust
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
        let kind = DomainErrorKind::TooLong { max: 100, actual: 150 };
        assert_eq!(kind.to_string(), "too long (150 chars, max 100)");
    }

    #[test]
    fn too_short_displays_correctly() {
        let kind = DomainErrorKind::TooShort { min: 5, actual: 3 };
        assert_eq!(kind.to_string(), "too short (3 chars, min 5)");
    }

    #[test]
    fn invalid_format_displays_correctly() {
        let kind = DomainErrorKind::InvalidFormat { expected: "local@domain" };
        assert_eq!(kind.to_string(), "invalid format, expected local@domain");
    }

    #[test]
    fn invalid_character_displays_correctly() {
        let kind = DomainErrorKind::InvalidCharacter { char: '@', position: 5 };
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
            reason: DomainErrorKind::InvalidFormat { expected: "local@domain" },
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
}
```

## Documentation Requirements

- **Code Documentation**: Doc comments on struct, enum, and all public fields
- **Examples**: Include usage example in module docs
- **User Documentation**: Update README if needed

## Implementation Notes

- Use `&'static str` for format_name, expected, component, and example to avoid allocations
- The `value` field is `String` to allow masking of sensitive data (e.g., credit cards)
- Consider adding a `masked()` method for DomainError that masks the value field

## Migration and Compatibility

N/A - New type with no existing code to migrate.
