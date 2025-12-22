# Platypus

> Domain-specific refined types for the Stillwater ecosystem

[![Crates.io](https://img.shields.io/crates/v/platypus.svg)](https://crates.io/crates/platypus)
[![Documentation](https://docs.rs/platypus/badge.svg)](https://docs.rs/platypus)
[![License](https://img.shields.io/crates/l/platypus.svg)](LICENSE-MIT)

Platypus provides production-ready domain predicates and refined types that integrate seamlessly with [stillwater](https://github.com/iepathos/stillwater). Validate emails, URLs, phone numbers, and more—with errors that accumulate and types that prove validity.

## Quick Start

```rust
use platypus::prelude::*;
use stillwater::prelude::*;

// Domain types encode validity in the type system
fn register_user(input: RawInput) -> Validation<User, Vec<DomainError>> {
    Validation::all((
        Email::new(input.email),
        PhoneNumber::new(input.phone),
        SecureUrl::new(input.website),
    ))
    .map(|(email, phone, website)| User { email, phone, website })
}

// Pure business logic works with guaranteed-valid types
fn send_welcome(user: &User) {
    // user.email is GUARANTEED valid - no runtime checks needed
    println!("Welcome! Confirmation sent to {}", user.email);
}

// Effects at the boundary
async fn handle_registration(input: RawInput, env: &AppEnv) -> Result<User, AppError> {
    from_validation(register_user(input))
        .map_err(AppError::Validation)
        .and_then(|user| {
            asks(|env: &AppEnv| env.db.save_user(&user))
                .map(|_| user)
        })
        .run(env)
        .await
}
```

## Available Domain Types

### Email (RFC 5321)
```rust
use platypus::email::{Email, PracticalEmail};

let email = Email::new("user@example.com".into())?;
let strict = PracticalEmail::new("user@company.co".into())?; // Requires TLD
```

### URL (RFC 3986)
```rust
use platypus::url::{Url, HttpUrl, SecureUrl};

let any_url = Url::new("ftp://files.example.com".into())?;
let http = HttpUrl::new("http://example.com".into())?;
let secure = SecureUrl::new("https://secure.example.com".into())?;
```

### UUID
```rust
use platypus::uuid::{Uuid, UuidV4, UuidV7};

let any = Uuid::new("550e8400-e29b-41d4-a716-446655440000".into())?;
let v4 = UuidV4::new("550e8400-e29b-41d4-a716-446655440000".into())?;
let v7 = UuidV7::new("01902c6f-3c9b-7abc-8def-0123456789ab".into())?;
```

### Phone Numbers (E.164)
```rust
use platypus::phone::{PhoneNumber, UsPhoneNumber};

let intl = PhoneNumber::new("+14155551234".into())?;
let us = UsPhoneNumber::new("(415) 555-1234".into())?;

// Normalize to E.164
assert_eq!(intl.to_e164(), "+14155551234");
```

### Financial
```rust
use platypus::financial::{Iban, CreditCardNumber};

let iban = Iban::new("DE89370400440532013000".into())?;
let card = CreditCardNumber::new("4111111111111111".into())?;
```

## Features

```toml
[dependencies]
platypus = { version = "0.1", features = ["full"] }

# Or pick what you need:
platypus = { version = "0.1", features = ["email", "url"] }
```

| Feature | Types Included |
|---------|---------------|
| `email` | `Email`, `PracticalEmail` |
| `url` | `Url`, `HttpUrl`, `SecureUrl` |
| `uuid` | `Uuid`, `UuidV4`, `UuidV7` |
| `phone` | `PhoneNumber`, `UsPhoneNumber` |
| `financial` | `Iban`, `CreditCardNumber` |
| `serde` | Serialization support |
| `full` | All of the above |

## Error Messages That Help

```rust
let result = Email::new("invalid".into());
// Error: invalid email address
//   value: "invalid"
//   expected: local@domain format per RFC 5321
//   example: user@example.com
```

## The Stillwater Ecosystem

| Library | Purpose |
|---------|---------|
| [stillwater](https://github.com/iepathos/stillwater) | Effect composition and validation core |
| **platypus** | Domain-specific refined types |
| [mindset](https://github.com/iepathos/mindset) | Zero-cost state machines |
| [premortem](https://github.com/iepathos/premortem) | Configuration validation |
| [postmortem](https://github.com/iepathos/postmortem) | JSON validation with path tracking |

## Philosophy

Platypus follows stillwater's core beliefs:

- **Parse, don't validate** - Validity is encoded in types, not checked repeatedly
- **Error accumulation** - Show users all problems, not just the first
- **Pure core, effects at boundary** - Domain types are pure; I/O happens at edges
- **Pragmatism over purity** - Real-world formats, not theoretical perfection

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
