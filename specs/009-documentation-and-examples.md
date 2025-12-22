---
number: 9
title: Documentation and Examples
category: foundation
priority: high
status: draft
dependencies: [1, 2, 3, 4, 5, 6, 7, 8]
created: 2025-12-21
---

# Specification 9: Documentation and Examples

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: Specs 1-8 (All implementation specs)

## Context

Good documentation is essential for library adoption. Platypus needs clear documentation that:
- Shows practical usage patterns
- Demonstrates stillwater integration (error accumulation, Effect composition)
- Explains when to use Platypus vs simpler alternatives
- Provides copy-paste examples for common scenarios

The implementation plan specifically calls out two key examples:
1. **Form Validation** - demonstrating error accumulation with `Validation::all()`
2. **API Handler** - demonstrating Effect composition with `from_validation()`

## Objective

Create comprehensive documentation and practical examples that demonstrate:
- Basic usage of all domain types
- Error accumulation patterns
- Effect composition patterns
- Serde integration
- When (and when not) to use Platypus

## Requirements

### Functional Requirements

1. **README.md**
   - Project overview and philosophy
   - Quick start example
   - Feature flag documentation
   - Links to detailed examples
   - When to use / when not to use

2. **Crate-level Documentation**
   - Module overview in lib.rs
   - Each module has doc comments
   - All public items documented
   - Examples in doc comments

3. **Example: Form Validation**
   - Demonstrates `Validation::all()` for error accumulation
   - Shows collecting all errors before responding
   - Practical form validation scenario

4. **Example: API Handler**
   - Demonstrates `from_validation()` bridge to Effect
   - Shows async I/O after validation
   - Practical API endpoint scenario

5. **Prelude Module**
   - Re-exports common types for convenient import
   - Feature-gated exports

### Non-Functional Requirements

1. **Copy-Paste Ready**: Examples should work as-is
2. **Progressive Complexity**: Start simple, build up
3. **Real-World Scenarios**: Not toy examples

## Acceptance Criteria

- [ ] README.md is complete and accurate
- [ ] All public items have doc comments
- [ ] `cargo doc` builds without warnings
- [ ] `examples/form_validation.rs` compiles and runs
- [ ] `examples/api_handler.rs` compiles and runs (if async runtime available)
- [ ] Prelude exports all common types
- [ ] Doc examples pass `cargo test --doc`

## Technical Details

### README Structure

```markdown
# Platypus

> Domain-specific refined types for the Stillwater ecosystem

[![Crates.io](https://img.shields.io/crates/v/platypus.svg)](https://crates.io/crates/platypus)
[![Documentation](https://docs.rs/platypus/badge.svg)](https://docs.rs/platypus)

## Quick Start

```rust
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

```rust
use platypus::prelude::*;
use stillwater::prelude::*;

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

```rust
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

## When to Use Platypus

**Use Platypus when:**
- Validating forms with multiple fields (accumulate all errors)
- Building APIs that need comprehensive input validation
- You want type-level guarantees throughout your codebase

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

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
```

### Form Validation Example

```rust
// examples/form_validation.rs
//! Demonstrates error accumulation with platypus types.
//!
//! Run with: cargo run --example form_validation --features full

use platypus::prelude::*;
use stillwater::validation::Validation;

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

/// Validates a registration form, accumulating all errors.
fn validate(form: RegistrationForm) -> Validation<ValidRegistration, Vec<DomainError>> {
    Validation::all((
        Email::new(form.email).map_err(|e| vec![e]),
        PhoneNumber::new(form.phone).map_err(|e| vec![e]),
        HttpUrl::new(form.website).map_err(|e| vec![e]),
    ))
    .map(|(email, phone, website)| ValidRegistration {
        email,
        phone,
        website,
    })
}

fn main() {
    println!("=== Valid Form ===");
    let valid_form = RegistrationForm {
        email: "user@example.com".into(),
        phone: "+14155551234".into(),
        website: "https://example.com".into(),
    };

    match validate(valid_form) {
        Validation::Success(reg) => {
            println!("Registration successful!");
            println!("  Email: {}", reg.email.get());
            println!("  Phone: {}", reg.phone.to_e164());
            println!("  Website: {}", reg.website.get());
        }
        Validation::Failure(errors) => {
            println!("Validation failed!");
            for err in errors {
                println!("  - {}", err);
            }
        }
    }

    println!("\n=== Invalid Form ===");
    let invalid_form = RegistrationForm {
        email: "not-an-email".into(),
        phone: "also-not-valid".into(),
        website: "not a url".into(),
    };

    match validate(invalid_form) {
        Validation::Success(_) => println!("Unexpected success!"),
        Validation::Failure(errors) => {
            println!("Validation failed with {} errors:", errors.len());
            for err in &errors {
                println!("  - {}", err);
            }
        }
    }

    println!("\n=== Partially Invalid Form ===");
    let partial_form = RegistrationForm {
        email: "valid@example.com".into(),
        phone: "bad-phone".into(),
        website: "https://valid-url.com".into(),
    };

    match validate(partial_form) {
        Validation::Success(_) => println!("Unexpected success!"),
        Validation::Failure(errors) => {
            println!("Validation failed with {} error(s):", errors.len());
            for err in &errors {
                println!("  - {}", err);
            }
        }
    }
}
```

### API Handler Example

```rust
// examples/api_handler.rs
//! Demonstrates Effect composition with platypus types.
//!
//! This example shows how to bridge from Validation to Effect
//! for async I/O operations after validation.
//!
//! Run with: cargo run --example api_handler --features full

use platypus::prelude::*;
use stillwater::prelude::*;

// Simulated environment and types
struct AppEnv {
    db_connected: bool,
}

#[derive(Debug)]
struct UserId(u64);

#[derive(Debug)]
enum AppError {
    Validation(Vec<DomainError>),
    Database(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Validation(errors) => {
                write!(f, "Validation errors: ")?;
                for (i, e) in errors.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}", e)?;
                }
                Ok(())
            }
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

/// Simulates creating a user in the database.
fn create_user_in_db(
    email: &Email,
    phone: &PhoneNumber,
) -> impl Effect<Output = UserId, Error = AppError, Env = AppEnv> {
    let email = email.clone();
    let phone = phone.clone();

    asks(move |env: &AppEnv| {
        if env.db_connected {
            // In real code, this would be async database call
            println!("Creating user with email: {}", email.get());
            println!("  Normalized phone: {}", phone.to_e164());
            Ok(UserId(12345))
        } else {
            Err(AppError::Database("Not connected".to_string()))
        }
    })
    .flatten_result()
}

/// Full registration flow: validate, then create user.
fn register_user(
    email_input: String,
    phone_input: String,
) -> impl Effect<Output = UserId, Error = AppError, Env = AppEnv> {
    // Step 1: Validate with accumulation
    let validated = Validation::all((
        Email::new(email_input).map_err(|e| vec![e]),
        PhoneNumber::new(phone_input).map_err(|e| vec![e]),
    ));

    // Step 2: Bridge to Effect
    from_validation(validated)
        .map_err(AppError::Validation)
        // Step 3: Do I/O if validation passed
        .and_then(|(email, phone)| create_user_in_db(&email, &phone))
        .context("registering user")
}

fn main() {
    let env = AppEnv { db_connected: true };

    println!("=== Valid Registration ===");
    let effect = register_user("user@example.com".into(), "+14155551234".into());

    match effect.run(&env) {
        Ok(user_id) => println!("Created user: {:?}", user_id),
        Err(e) => println!("Failed: {}", e),
    }

    println!("\n=== Invalid Registration ===");
    let effect = register_user("bad-email".into(), "bad-phone".into());

    match effect.run(&env) {
        Ok(user_id) => println!("Created user: {:?}", user_id),
        Err(e) => println!("Failed: {}", e),
    }

    println!("\n=== Database Error ===");
    let disconnected_env = AppEnv {
        db_connected: false,
    };
    let effect = register_user("user@example.com".into(), "+14155551234".into());

    match effect.run(&disconnected_env) {
        Ok(user_id) => println!("Created user: {:?}", user_id),
        Err(e) => println!("Failed: {}", e),
    }
}
```

### Prelude Module

```rust
// src/prelude.rs
//! Convenient imports for common platypus usage.
//!
//! # Example
//! ```
//! use platypus::prelude::*;
//! ```

pub use crate::error::{DomainError, DomainErrorKind};

#[cfg(feature = "email")]
pub use crate::email::{Email, ValidEmail};

#[cfg(feature = "url")]
pub use crate::url::{HttpScheme, HttpUrl, HttpsOnly, SecureUrl, Url, ValidUrl};

#[cfg(feature = "uuid")]
pub use crate::uuid::{Uuid, UuidV4, UuidV7, UuidVersion, ValidUuid};

#[cfg(feature = "phone")]
pub use crate::phone::{PhoneNumber, ValidPhoneNumber};

#[cfg(feature = "financial")]
pub use crate::financial::{CreditCardNumber, Iban, ValidCreditCard, ValidIban};
```

## Dependencies

- **Prerequisites**: All implementation specs (1-8) complete
- **Affected Components**: README.md, lib.rs, prelude.rs, examples/
- **External Dependencies**: None (documentation only)

## Testing Strategy

### Doc Tests

All code examples in documentation must pass:

```bash
cargo test --doc --all-features
```

### Example Compilation

Examples must compile and run:

```bash
cargo run --example form_validation --features full
cargo run --example api_handler --features full
```

### Documentation Build

Documentation must build without warnings:

```bash
cargo doc --all-features --no-deps
```

## Documentation Requirements

- **Code Documentation**: All public items documented
- **README**: Complete usage guide
- **Examples**: Practical, runnable examples
- **Doc Tests**: Examples in docs are tested

## Implementation Notes

- Examples should be self-contained and not require external services
- Use `println!` for output in examples (no logging crate dependency)
- Keep examples focused on demonstrating platypus, not stillwater internals
- Consider adding a `CHANGELOG.md` for release history

## Migration and Compatibility

N/A - Documentation only.

## Future Considerations

Not in scope for initial implementation:
- mdBook or similar for extended documentation
- Video tutorials
- Interactive examples (playground)
- Comparison with other validation libraries
