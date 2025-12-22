# Platypus Implementation Plan

> Domain-specific refined types for the Stillwater ecosystem

## Overview

**Platypus** extends stillwater with production-ready domain predicates. It provides predicates and type aliases - nothing more. All composition, validation accumulation, and effect bridging use stillwater's existing patterns.

## What Platypus Provides

```
┌─────────────────────────────────────────────────────────┐
│  Platypus                                               │
│  ─────────                                              │
│  • Predicates: ValidEmail, ValidUrl, ValidPhone, ...    │
│  • Type aliases: Email, Url, PhoneNumber, ...           │
│  • Rich errors: DomainError with context                │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│  Stillwater (already exists)                            │
│  ──────────────────────────                             │
│  • Refined<T, P> - the container                        │
│  • Validation::all() - error accumulation               │
│  • from_validation() - bridge to Effect                 │
│  • from_result() - lift Result to Effect                │
│  • And, Or, Not - predicate combinators                 │
└─────────────────────────────────────────────────────────┘
```

## What Platypus Does NOT Provide

- No wrapper functions around stillwater
- No new traits (uses `stillwater::refined::Predicate` directly)
- No new composition patterns
- No effect constructors

## Philosophy Alignment

| Principle | Application |
|-----------|-------------|
| **Pragmatism Over Purity** | No unnecessary abstractions; just predicates |
| **Parse, Don't Validate** | Domain types encode invariants in the type |
| **Composition Over Complexity** | Uses stillwater's existing `And`, `Or`, `Not` |
| **Errors Should Tell Stories** | `DomainError` includes format, example, reason |

## When NOT to Use Platypus

**Skip platypus if:**
- Validating a single field in a simple script
- Your domain already has validation (e.g., your ORM validates emails)
- You only need one domain type (just copy the predicate)

**Use platypus when:**
- Validating forms with multiple fields (accumulate all errors)
- Building APIs that need comprehensive input validation
- You want type-level guarantees throughout your codebase

---

## Stage 1: Project Foundation

**Goal**: Crate structure and CI

**Success Criteria**:
- [ ] Cargo.toml with feature flags
- [ ] CI passes (clippy, fmt, test, doc)
- [ ] stillwater dependency configured
- [ ] Basic lib.rs structure

**Directory Structure**:
```
platypus/
├── Cargo.toml
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── src/
│   ├── lib.rs
│   ├── error.rs      # DomainError type
│   └── prelude.rs    # Convenient imports
├── tests/
│   └── integration.rs
└── examples/
    └── form_validation.rs
```

**Cargo.toml**:
```toml
[package]
name = "platypus"
version = "0.1.0"
edition = "2024"
rust-version = "1.89"
description = "Domain-specific refined types for the Stillwater ecosystem"
license = "MIT OR Apache-2.0"
repository = "https://github.com/iepathos/platypus"
keywords = ["validation", "types", "refinement", "domain", "stillwater"]
categories = ["development-tools", "rust-patterns"]

[dependencies]
stillwater = { version = "1.0", path = "../stillwater" }

# Optional - each domain has its own dependency
regex = { version = "1", optional = true }
url = { version = "2", optional = true }
uuid = { version = "1", optional = true }
phonenumber = { version = "0.3", optional = true }
email_address = { version = "0.2", optional = true }
iban_validate = { version = "4", optional = true }
creditcard = { version = "0.3", optional = true }

[features]
default = ["email", "url"]
full = ["email", "url", "uuid", "phone", "financial"]

email = ["dep:email_address"]
url = ["dep:url"]
uuid = ["dep:uuid"]
phone = ["dep:phonenumber"]
financial = ["dep:iban_validate", "dep:creditcard"]

serde = ["stillwater/serde", "uuid?/serde", "url?/serde"]
```

**Status**: Not Started

---

## Stage 2: Error Types

**Goal**: Rich, contextual error type for all domain predicates

**Success Criteria**:
- [ ] `DomainError` struct with context
- [ ] `DomainErrorKind` enum for specific failures
- [ ] `Display` and `Error` implementations
- [ ] Works with stillwater's `Validation` error accumulation

**Implementation**:

```rust
// src/error.rs

use std::fmt;

/// Rich error for domain validation failures
///
/// Includes enough context for helpful user-facing messages.
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainErrorKind {
    Empty,
    TooLong { max: usize, actual: usize },
    TooShort { min: usize, actual: usize },
    InvalidFormat { expected: &'static str },
    InvalidCharacter { char: char, position: usize },
    InvalidChecksum,
    InvalidComponent { component: &'static str, reason: String },
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

**Status**: Not Started

---

## Stage 3: Email

**Goal**: RFC 5321 email validation

**Success Criteria**:
- [ ] `ValidEmail` predicate
- [ ] `Email` type alias
- [ ] Test suite with RFC edge cases

**Implementation**:

```rust
// src/email.rs

use crate::{DomainError, DomainErrorKind};
use email_address::EmailAddress;
use stillwater::refined::{Predicate, Refined};

/// RFC 5321 compliant email address
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

/// RFC 5321 compliant email address
pub type Email = Refined<String, ValidEmail>;
```

**Test Cases**:
- Valid: `user@example.com`, `user+tag@example.com`, `"quoted"@example.com`
- Valid per RFC: `user@localhost`, `user@[192.168.1.1]`
- Invalid: `@example.com`, `user@`, `user`, `user@@example.com`, ``

**Status**: Not Started

---

## Stage 4: URL

**Goal**: RFC 3986 URL validation

**Success Criteria**:
- [ ] `ValidUrl` predicate (any URL)
- [ ] `HttpScheme` predicate (http/https only)
- [ ] `HttpsOnly` predicate (https only)
- [ ] Type aliases using stillwater's `And` combinator

**Implementation**:

```rust
// src/url.rs

use crate::{DomainError, DomainErrorKind};
use stillwater::refined::{And, Predicate, Refined};
use url::Url as UrlParser;

/// Any valid RFC 3986 URL
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidUrl;

impl Predicate<String> for ValidUrl {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        UrlParser::parse(value).map(|_| ()).map_err(|e| DomainError {
            format_name: "URL",
            value: value.clone(),
            reason: DomainErrorKind::InvalidFormat {
                expected: "scheme://host/path",
            },
            example: "https://example.com",
        })
    }

    fn description() -> &'static str {
        "RFC 3986 URL"
    }
}

/// URL scheme must be http or https
#[derive(Debug, Clone, Copy, Default)]
pub struct HttpScheme;

impl Predicate<String> for HttpScheme {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        // Assumes ValidUrl already passed
        let parsed = UrlParser::parse(value).map_err(|_| DomainError {
            format_name: "HTTP URL",
            value: value.clone(),
            reason: DomainErrorKind::InvalidFormat { expected: "valid URL" },
            example: "https://example.com",
        })?;

        match parsed.scheme() {
            "http" | "https" => Ok(()),
            scheme => Err(DomainError {
                format_name: "HTTP URL",
                value: value.clone(),
                reason: DomainErrorKind::InvalidComponent {
                    component: "scheme",
                    reason: format!("expected http or https, got {}", scheme),
                },
                example: "https://example.com",
            }),
        }
    }

    fn description() -> &'static str {
        "HTTP or HTTPS scheme"
    }
}

/// URL scheme must be https
#[derive(Debug, Clone, Copy, Default)]
pub struct HttpsOnly;

impl Predicate<String> for HttpsOnly {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        let parsed = UrlParser::parse(value).map_err(|_| DomainError {
            format_name: "HTTPS URL",
            value: value.clone(),
            reason: DomainErrorKind::InvalidFormat { expected: "valid URL" },
            example: "https://example.com",
        })?;

        if parsed.scheme() == "https" {
            Ok(())
        } else {
            Err(DomainError {
                format_name: "HTTPS URL",
                value: value.clone(),
                reason: DomainErrorKind::InvalidComponent {
                    component: "scheme",
                    reason: format!("expected https, got {}", parsed.scheme()),
                },
                example: "https://example.com",
            })
        }
    }

    fn description() -> &'static str {
        "HTTPS scheme only"
    }
}

// Type aliases - composition uses stillwater's And
pub type Url = Refined<String, ValidUrl>;
pub type HttpUrl = Refined<String, And<ValidUrl, HttpScheme>>;
pub type SecureUrl = Refined<String, And<ValidUrl, HttpsOnly>>;
```

**Status**: Not Started

---

## Stage 5: UUID

**Goal**: UUID validation with version-specific types

**Success Criteria**:
- [ ] `ValidUuid` predicate (any version)
- [ ] `UuidVersion<N>` predicate (specific version)
- [ ] Type aliases for common versions (v4, v7)
- [ ] Conversion helper to `uuid::Uuid`

**Implementation**:

```rust
// src/uuid.rs

use crate::{DomainError, DomainErrorKind};
use stillwater::refined::{Predicate, Refined};
use uuid::Uuid as UuidImpl;

/// Any valid UUID
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidUuid;

impl Predicate<String> for ValidUuid {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        UuidImpl::parse_str(value).map(|_| ()).map_err(|_| DomainError {
            format_name: "UUID",
            value: value.clone(),
            reason: DomainErrorKind::InvalidFormat {
                expected: "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
            },
            example: "550e8400-e29b-41d4-a716-446655440000",
        })
    }

    fn description() -> &'static str {
        "UUID"
    }
}

/// UUID must be specific version
#[derive(Debug, Clone, Copy, Default)]
pub struct UuidVersion<const V: usize>;

impl<const V: usize> Predicate<String> for UuidVersion<V> {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        let parsed = UuidImpl::parse_str(value).map_err(|_| DomainError {
            format_name: "UUID",
            value: value.clone(),
            reason: DomainErrorKind::InvalidFormat {
                expected: "valid UUID",
            },
            example: "550e8400-e29b-41d4-a716-446655440000",
        })?;

        if parsed.get_version_num() == V {
            Ok(())
        } else {
            Err(DomainError {
                format_name: &format!("UUID v{}", V).leak(),
                value: value.clone(),
                reason: DomainErrorKind::InvalidComponent {
                    component: "version",
                    reason: format!("expected v{}, got v{}", V, parsed.get_version_num()),
                },
                example: "550e8400-e29b-41d4-a716-446655440000",
            })
        }
    }

    fn description() -> &'static str {
        "UUID with specific version"
    }
}

pub type Uuid = Refined<String, ValidUuid>;
pub type UuidV4 = Refined<String, UuidVersion<4>>;
pub type UuidV7 = Refined<String, UuidVersion<7>>;

// Conversion helper
impl Uuid {
    /// Convert to uuid::Uuid (infallible - already validated)
    pub fn to_uuid(&self) -> UuidImpl {
        UuidImpl::parse_str(self.get()).expect("already validated")
    }
}
```

**Status**: Not Started

---

## Stage 6: Phone Number

**Goal**: E.164 phone number validation

**Success Criteria**:
- [ ] `ValidPhoneNumber` predicate (E.164)
- [ ] `PhoneNumber` type alias
- [ ] Normalization helper `to_e164()`

**Implementation**:

```rust
// src/phone.rs

use crate::{DomainError, DomainErrorKind};
use phonenumber::{Mode, parse};
use stillwater::refined::{Predicate, Refined};

/// E.164 international phone number
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidPhoneNumber;

impl Predicate<String> for ValidPhoneNumber {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
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

pub type PhoneNumber = Refined<String, ValidPhoneNumber>;

impl PhoneNumber {
    /// Normalize to E.164 format
    pub fn to_e164(&self) -> String {
        let parsed = parse(None, self.get()).expect("already validated");
        parsed.format().mode(Mode::E164).to_string()
    }
}
```

**Status**: Not Started

---

## Stage 7: Financial

**Goal**: IBAN and credit card validation

**Success Criteria**:
- [ ] `ValidIban` predicate with checksum
- [ ] `ValidCreditCard` predicate with Luhn
- [ ] Masked values in errors (security)

**Implementation**:

```rust
// src/financial.rs

use crate::{DomainError, DomainErrorKind};
use creditcard::CreditCard;
use iban_validate::Iban as IbanImpl;
use stillwater::refined::{Predicate, Refined};

/// Valid IBAN with checksum
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidIban;

impl Predicate<String> for ValidIban {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
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

/// Valid credit card number (Luhn validated)
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidCreditCard;

impl Predicate<String> for ValidCreditCard {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
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

pub type Iban = Refined<String, ValidIban>;
pub type CreditCardNumber = Refined<String, ValidCreditCard>;

fn mask_card(card: &str) -> String {
    let digits: String = card.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 4 {
        format!("****{}", &digits[digits.len() - 4..])
    } else {
        "****".to_string()
    }
}

fn mask_iban(iban: &str) -> String {
    if iban.len() > 6 {
        format!("{}****{}", &iban[..4], &iban[iban.len() - 4..])
    } else {
        "****".to_string()
    }
}
```

**Status**: Not Started

---

## Stage 8: Serde Integration

**Goal**: Validation during deserialization

**Success Criteria**:
- [ ] Feature flag `serde`
- [ ] Tests showing JSON deserialization validates
- [ ] Works via stillwater's existing serde support

**Note**: No code needed - stillwater's `Refined<T, P>` already implements Deserialize when the `serde` feature is enabled. We just need tests.

**Tests**:

```rust
#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct User {
        email: Email,
        website: Url,
    }

    #[test]
    fn valid_json_deserializes() {
        let json = r#"{"email": "user@example.com", "website": "https://example.com"}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.email.get(), "user@example.com");
    }

    #[test]
    fn invalid_email_fails_deserialization() {
        let json = r#"{"email": "invalid", "website": "https://example.com"}"#;
        let result: Result<User, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
```

**Status**: Not Started

---

## Stage 9: Documentation and Examples

**Goal**: Clear docs showing stillwater integration

**Success Criteria**:
- [ ] README with usage patterns
- [ ] Doc comments with examples
- [ ] Example: Form validation (error accumulation)
- [ ] Example: API handler (Effect composition)

**Key Example - Form Validation**:

```rust
// examples/form_validation.rs
//! Demonstrates error accumulation with platypus types

use platypus::prelude::*;
use stillwater::prelude::*;

#[derive(Debug)]
struct RegistrationForm {
    email: String,
    phone: String,
    website: String,
}

#[derive(Debug)]
struct ValidRegistration {
    email: Email,
    phone: PhoneNumber,
    website: HttpUrl,
}

fn validate(form: RegistrationForm) -> Validation<ValidRegistration, Vec<DomainError>> {
    Validation::all((
        Email::new(form.email).map_err(|e| vec![e]),
        PhoneNumber::new(form.phone).map_err(|e| vec![e]),
        HttpUrl::new(form.website).map_err(|e| vec![e]),
    ))
    .map(|(email, phone, website)| ValidRegistration { email, phone, website })
}

fn main() {
    let form = RegistrationForm {
        email: "bad".into(),
        phone: "also bad".into(),
        website: "not a url".into(),
    };

    match validate(form) {
        Validation::Success(reg) => println!("Valid: {:?}", reg),
        Validation::Failure(errors) => {
            println!("Validation failed with {} errors:", errors.len());
            for err in errors {
                println!("  - {}", err);
            }
        }
    }
}
```

**Key Example - API Handler**:

```rust
// examples/api_handler.rs
//! Demonstrates Effect composition with platypus types

use platypus::prelude::*;
use stillwater::prelude::*;

struct AppEnv {
    db: Database,
}

fn register_user(
    email: String,
    phone: String,
) -> impl Effect<Output = UserId, Error = AppError, Env = AppEnv> {
    // Validate with accumulation
    let validated = Validation::all((
        Email::new(email).map_err(|e| vec![e]),
        PhoneNumber::new(phone).map_err(|e| vec![e]),
    ));

    // Bridge to Effect, then do I/O
    from_validation(validated)
        .map_err(AppError::Validation)
        .and_then(|(email, phone)| {
            asks(move |env: &AppEnv| env.db.create_user(&email, &phone))
        })
        .context("registering user")
}
```

**Status**: Not Started

---

## Testing Strategy

### Unit Tests
- Each predicate: valid cases, invalid cases, edge cases from RFCs
- Error messages: verify they include format_name, example, reason

### Integration Tests
- Composition with `And`, `Or`, `Not`
- Error accumulation with `Validation::all`
- Serde round-trip

### Property Tests (optional)
- Valid refined values always pass predicate re-check
- Round-trip through serde preserves validity

---

## Release Checklist

- [ ] All stages complete
- [ ] CI green (clippy, fmt, test, doc)
- [ ] README complete
- [ ] CHANGELOG.md written
- [ ] Version 0.1.0 tagged
- [ ] Published to crates.io
- [ ] Added to stillwater ecosystem docs

---

## Future Considerations (Post 1.0)

Not in scope for initial release:

**Additional Domains**:
- `Ipv4`, `Ipv6`, `IpNetwork`
- `Slug` (URL-safe strings)
- `Semver` (version strings)
- `Cron` (cron expressions)

**Features**:
- `no_std` support
- WASM support
- Compile-time literal validation (proc-macro)
