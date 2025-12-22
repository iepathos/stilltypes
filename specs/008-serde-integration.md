---
number: 8
title: Serde Integration
category: compatibility
priority: high
status: draft
dependencies: [1, 2, 3, 4, 5, 6, 7]
created: 2025-12-21
---

# Specification 8: Serde Integration

**Category**: compatibility
**Priority**: high
**Status**: draft
**Dependencies**: Specs 1-7 (All domain types)

## Context

Serde integration enables validation during JSON/YAML/TOML deserialization, which is critical for:
- API request validation (JSON bodies)
- Configuration file parsing
- Database record mapping
- Message queue payloads

The key insight is that **Platypus needs no serde implementation code**. Stillwater's `Refined<T, P>` already implements `Deserialize` when:
1. The `serde` feature is enabled
2. The inner type `T` implements `Deserialize`
3. The predicate's error type implements `Display`

This specification focuses on testing and documenting this integration.

## Objective

Enable and test serde integration for all Platypus types through:
- Feature flag configuration
- Comprehensive test suite
- Clear documentation of validation-on-deserialize behavior

## Requirements

### Functional Requirements

1. **Feature Flag Configuration**
   - `serde` feature enables serde support
   - Properly propagates to stillwater dependency
   - Enables serde for optional deps (uuid, url) when both features active

2. **Validation on Deserialize**
   - Invalid JSON values fail deserialization
   - Error messages include validation context
   - Works for all domain types (Email, Url, Uuid, PhoneNumber, Iban, CreditCardNumber)

3. **Serialization Support**
   - All types serialize to their string representation
   - Round-trip (serialize → deserialize) preserves values

4. **Struct Composition**
   - Works in user-defined structs
   - Multiple validated fields in single struct
   - Optional fields with `Option<Email>` work correctly

### Non-Functional Requirements

1. **Zero Additional Code**: Uses stillwater's existing serde impl
2. **Type Safety**: Compile-time errors if serde not enabled
3. **Performance**: No overhead beyond stillwater's implementation

## Acceptance Criteria

- [ ] `serde` feature properly configured in Cargo.toml
- [ ] All domain types deserialize with validation
- [ ] Invalid values produce clear error messages
- [ ] Valid values round-trip correctly
- [ ] Works in composite structs
- [ ] Works with Option<T> fields
- [ ] Documentation explains behavior
- [ ] Tests cover all domain types

## Technical Details

### Feature Configuration

```toml
# In Cargo.toml
[features]
serde = ["stillwater/serde", "uuid?/serde", "url?/serde"]

[dev-dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### No Implementation Required

Stillwater's `Refined<T, P>` already provides:

```rust
// From stillwater (not platypus)
impl<'de, T, P> Deserialize<'de> for Refined<T, P>
where
    T: Deserialize<'de>,
    P: Predicate<T>,
    P::Error: Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Refined::new(value).map_err(de::Error::custom)
    }
}
```

### Test Module Structure

```rust
// tests/serde_integration.rs

#[cfg(all(feature = "serde", feature = "email", feature = "url"))]
mod serde_tests {
    use platypus::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct User {
        email: Email,
        website: Option<Url>,
    }

    #[test]
    fn valid_json_deserializes() {
        let json = r#"{"email": "user@example.com", "website": "https://example.com"}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.email.get(), "user@example.com");
        assert_eq!(user.website.as_ref().unwrap().get(), "https://example.com");
    }

    #[test]
    fn invalid_email_fails_deserialization() {
        let json = r#"{"email": "invalid", "website": "https://example.com"}"#;
        let result: Result<User, _> = serde_json::from_str(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("email"));
    }

    #[test]
    fn missing_optional_field_ok() {
        let json = r#"{"email": "user@example.com"}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert!(user.website.is_none());
    }

    #[test]
    fn null_optional_field_ok() {
        let json = r#"{"email": "user@example.com", "website": null}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert!(user.website.is_none());
    }

    #[test]
    fn roundtrip_preserves_values() {
        let original = User {
            email: Email::new("test@example.com".to_string()).unwrap(),
            website: Some(Url::new("https://example.com".to_string()).unwrap()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: User = serde_json::from_str(&json).unwrap();
        assert_eq!(original.email.get(), restored.email.get());
    }
}
```

## Dependencies

- **Prerequisites**: All domain type specs (1-7) must be complete
- **Affected Components**: Test files, documentation
- **External Dependencies**:
  - `serde` (dev-dependency for testing)
  - `serde_json` (dev-dependency for testing)

## Testing Strategy

### Email Serde Tests

```rust
#[cfg(all(feature = "serde", feature = "email"))]
mod email_serde {
    use platypus::email::Email;

    #[test]
    fn email_deserializes() {
        let json = r#""user@example.com""#;
        let email: Email = serde_json::from_str(json).unwrap();
        assert_eq!(email.get(), "user@example.com");
    }

    #[test]
    fn invalid_email_fails() {
        let json = r#""not-an-email""#;
        let result: Result<Email, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn email_serializes() {
        let email = Email::new("test@example.com".to_string()).unwrap();
        let json = serde_json::to_string(&email).unwrap();
        assert_eq!(json, r#""test@example.com""#);
    }
}
```

### URL Serde Tests

```rust
#[cfg(all(feature = "serde", feature = "url"))]
mod url_serde {
    use platypus::url::{Url, HttpUrl, SecureUrl};

    #[test]
    fn url_deserializes() {
        let json = r#""https://example.com""#;
        let url: Url = serde_json::from_str(json).unwrap();
        assert_eq!(url.get(), "https://example.com");
    }

    #[test]
    fn http_url_rejects_ftp() {
        let json = r#""ftp://example.com""#;
        let result: Result<HttpUrl, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn secure_url_rejects_http() {
        let json = r#""http://example.com""#;
        let result: Result<SecureUrl, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
```

### UUID Serde Tests

```rust
#[cfg(all(feature = "serde", feature = "uuid"))]
mod uuid_serde {
    use platypus::uuid::{Uuid, UuidV4, UuidV7};

    #[test]
    fn uuid_deserializes() {
        let json = r#""550e8400-e29b-41d4-a716-446655440000""#;
        let uuid: Uuid = serde_json::from_str(json).unwrap();
        assert!(uuid.get().contains("550e8400"));
    }

    #[test]
    fn uuid_v4_validates_version() {
        // v7 UUID fails for UuidV4 type
        let json = r#""018f6b8e-e4a0-7000-8000-000000000000""#;
        let result: Result<UuidV4, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
```

### Phone Serde Tests

```rust
#[cfg(all(feature = "serde", feature = "phone"))]
mod phone_serde {
    use platypus::phone::PhoneNumber;

    #[test]
    fn phone_deserializes() {
        let json = r#""+14155551234""#;
        let phone: PhoneNumber = serde_json::from_str(json).unwrap();
        assert_eq!(phone.to_e164(), "+14155551234");
    }

    #[test]
    fn invalid_phone_fails() {
        let json = r#""not-a-phone""#;
        let result: Result<PhoneNumber, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
```

### Financial Serde Tests

```rust
#[cfg(all(feature = "serde", feature = "financial"))]
mod financial_serde {
    use platypus::financial::{Iban, CreditCardNumber};

    #[test]
    fn iban_deserializes() {
        let json = r#""DE89370400440532013000""#;
        let iban: Iban = serde_json::from_str(json).unwrap();
        assert_eq!(iban.country_code(), "DE");
    }

    #[test]
    fn credit_card_deserializes() {
        let json = r#""4111111111111111""#;
        let card: CreditCardNumber = serde_json::from_str(json).unwrap();
        assert_eq!(card.last_four(), "1111");
    }

    #[test]
    fn invalid_luhn_fails() {
        let json = r#""4111111111111112""#;
        let result: Result<CreditCardNumber, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
```

### Complex Struct Tests

```rust
#[cfg(all(feature = "serde", feature = "full"))]
mod complex_serde {
    use platypus::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct RegistrationForm {
        email: Email,
        phone: PhoneNumber,
        website: Option<HttpUrl>,
        #[serde(default)]
        terms_accepted: bool,
    }

    #[test]
    fn complex_form_deserializes() {
        let json = r#"{
            "email": "user@example.com",
            "phone": "+14155551234",
            "website": "https://example.com"
        }"#;
        let form: RegistrationForm = serde_json::from_str(json).unwrap();
        assert_eq!(form.email.get(), "user@example.com");
        assert_eq!(form.phone.to_e164(), "+14155551234");
    }

    #[test]
    fn partial_invalid_shows_first_error() {
        let json = r#"{
            "email": "invalid",
            "phone": "also-invalid"
        }"#;
        let result: Result<RegistrationForm, _> = serde_json::from_str(json);
        assert!(result.is_err());
        // Note: serde stops at first error, doesn't accumulate
    }
}
```

## Documentation Requirements

- **Code Documentation**: Explain that serde uses stillwater's impl
- **Behavior Notes**: Document that errors don't accumulate in serde
- **User Documentation**: Example in README showing JSON validation

### README Example

```markdown
## JSON Validation

With the `serde` feature, types validate during deserialization:

```rust
use platypus::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct UserInput {
    email: Email,
    website: Option<SecureUrl>,
}

fn handle_request(json: &str) -> Result<UserInput, serde_json::Error> {
    // Validation happens automatically!
    serde_json::from_str(json)
}
```

Note: Serde stops at the first validation error. For accumulating
multiple errors, use stillwater's `Validation::all()` pattern instead.
```

## Implementation Notes

- **No Code Required**: This spec is about testing and documentation
- Serde's error handling is first-error-wins, not accumulating
- For multi-field validation with error accumulation, use `Validation::all()` after deserializing raw strings
- The `#[serde(default)]` attribute works normally with validated types

## Migration and Compatibility

N/A - New feature integration with no existing code to migrate.

## Future Considerations

Not in scope for initial implementation:
- Custom deserializer for error accumulation
- Support for other serde formats (bincode, postcard)
- `#[serde(try_from)]` pattern documentation
