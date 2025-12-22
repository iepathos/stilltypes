# Platypus

> Domain-specific refined types for the Stillwater ecosystem

[![Crates.io](https://img.shields.io/crates/v/platypus.svg)](https://crates.io/crates/platypus)
[![Documentation](https://docs.rs/platypus/badge.svg)](https://docs.rs/platypus)
[![License](https://img.shields.io/crates/l/platypus.svg)](LICENSE)

Platypus provides production-ready domain predicates and refined types that integrate seamlessly with [stillwater](https://github.com/iepathos/stillwater). Validate emails, URLs, phone numbers, and more with errors that accumulate and types that prove validity.

## Quick Start

```rust,ignore
use platypus::prelude::*;

// Types validate on construction
let email = Email::new("user@example.com".to_string())?;
let url = SecureUrl::new("https://example.com".to_string())?;

// Invalid values fail with helpful errors
let bad = Email::new("invalid".to_string());
assert!(bad.is_err());
println!("{}", bad.unwrap_err());
// invalid email address: invalid format, expected local@domain (example: user@example.com)
```

## Features

Enable only what you need:

```toml
[dependencies]
platypus = { version = "0.1", default-features = false, features = ["email", "url"] }
```

| Feature | Types | Dependencies |
|---------|-------|--------------|
| `email` (default) | `Email` | `email_address` |
| `url` (default) | `Url`, `HttpUrl`, `SecureUrl` | `url` |
| `uuid` | `Uuid`, `UuidV4`, `UuidV7` | `uuid` |
| `phone` | `PhoneNumber` | `phonenumber` |
| `financial` | `Iban`, `CreditCardNumber` | `iban_validate`, `creditcard` |
| `serde` | Serialize/Deserialize for all types | - |
| `full` | All of the above | - |

## Error Accumulation

Collect all validation errors at once using stillwater's `Validation`:

```rust,ignore
use platypus::prelude::*;
use stillwater::validation::Validation;

struct ValidForm {
    email: Email,
    phone: PhoneNumber,
}

fn validate_form(email: String, phone: String) -> Validation<ValidForm, Vec<DomainError>> {
    Validation::all((
        Email::new(email).map_err(|e| vec![e]),
        PhoneNumber::new(phone).map_err(|e| vec![e]),
    ))
    .map(|(email, phone)| ValidForm { email, phone })
}

match validate_form(email, phone) {
    Validation::Success(form) => handle_valid(form),
    Validation::Failure(errors) => {
        for err in errors {
            println!("Error: {}", err);
        }
    }
}
```

## JSON Validation

With the `serde` feature, types validate during deserialization:

```rust,ignore
use platypus::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct User {
    email: Email,
    website: Option<SecureUrl>,
}

// Invalid JSON fails to deserialize
let result: Result<User, _> = serde_json::from_str(json);
```

## Available Domain Types

### Email (RFC 5321)

```rust,ignore
use platypus::email::Email;

let email = Email::new("user@example.com".to_string())?;
assert_eq!(email.get(), "user@example.com");

// Plus addressing works
let plus = Email::new("user+tag@example.com".to_string())?;
```

### URL (RFC 3986)

```rust,ignore
use platypus::url::{Url, HttpUrl, SecureUrl};

// Any valid URL
let any_url = Url::new("ftp://files.example.com".to_string())?;

// HTTP or HTTPS only
let http = HttpUrl::new("http://example.com".to_string())?;

// HTTPS only (secure)
let secure = SecureUrl::new("https://secure.example.com".to_string())?;
let insecure = SecureUrl::new("http://example.com".to_string());
assert!(insecure.is_err()); // HTTP rejected
```

### UUID

```rust,ignore
use platypus::uuid::{Uuid, UuidV4, UuidV7, ToUuid};

// Any valid UUID
let any = Uuid::new("550e8400-e29b-41d4-a716-446655440000".to_string())?;

// Version-specific
let v4 = UuidV4::new("550e8400-e29b-41d4-a716-446655440000".to_string())?;
let v7 = UuidV7::new("018f6b8e-e4a0-7000-8000-000000000000".to_string())?;

// Convert to uuid::Uuid
let uuid_impl = v4.to_uuid();
assert_eq!(uuid_impl.get_version_num(), 4);
```

### Phone Numbers (E.164)

```rust,ignore
use platypus::phone::{PhoneNumber, PhoneNumberExt};

let phone = PhoneNumber::new("+1 (415) 555-1234".to_string())?;

// Normalize to E.164 for storage
assert_eq!(phone.to_e164(), "+14155551234");

// Get country code
assert_eq!(phone.country_code(), 1);
```

### Financial

```rust,ignore
use platypus::financial::{Iban, CreditCardNumber, IbanExt, CreditCardExt};

// IBAN validation
let iban = Iban::new("DE89370400440532013000".to_string())?;
assert_eq!(iban.country_code(), "DE");
assert_eq!(iban.masked(), "DE89****3000"); // For display

// Credit card validation (Luhn algorithm)
let card = CreditCardNumber::new("4111111111111111".to_string())?;
assert_eq!(card.masked(), "****1111"); // For display
assert_eq!(card.last_four(), "1111");
```

## When to Use Platypus

**Use Platypus when:**
- Validating forms with multiple fields (accumulate all errors)
- Building APIs that need comprehensive input validation
- You want type-level guarantees throughout your codebase
- Working with the Stillwater ecosystem

**Skip Platypus if:**
- Validating a single field in a simple script
- Your domain already has validation (e.g., ORM validates emails)
- You only need one domain type (just copy the predicate)

## Philosophy

Platypus follows the [Stillwater philosophy](https://github.com/iepathos/stillwater):

- **Pragmatism Over Purity** - No unnecessary abstractions; just predicates
- **Parse, Don't Validate** - Domain types encode invariants in the type
- **Composition Over Complexity** - Uses stillwater's `And`, `Or`, `Not`
- **Errors Should Tell Stories** - Rich context for user-facing messages

## Examples

See the `examples/` directory for complete working examples:

- `form_validation.rs` - Error accumulation with `Validation::all()`
- `api_handler.rs` - Effect composition with `from_validation()`

Run with:

```bash
cargo run --example form_validation --features full
cargo run --example api_handler --features full
```

## The Stillwater Ecosystem

| Library | Purpose |
|---------|---------|
| [stillwater](https://github.com/iepathos/stillwater) | Effect composition and validation core |
| **platypus** | Domain-specific refined types |
| [mindset](https://github.com/iepathos/mindset) | Zero-cost state machines |
| [premortem](https://github.com/iepathos/premortem) | Configuration validation |
| [postmortem](https://github.com/iepathos/postmortem) | JSON validation with path tracking |

## License

Licensed under the MIT license. See [LICENSE](LICENSE) for details.
