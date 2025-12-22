---
number: 5
title: UUID Validation
category: foundation
priority: medium
status: draft
dependencies: [1, 2]
created: 2025-12-21
---

# Specification 5: UUID Validation

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: Spec 1 (Project Foundation), Spec 2 (Error Types)

## Context

UUIDs (Universally Unique Identifiers) are widely used for:
- Database primary keys
- Distributed system identifiers
- Session tokens
- Trace/correlation IDs

Different UUID versions have different use cases:
- **v4**: Random - most common for general unique IDs
- **v7**: Time-ordered random - ideal for database keys (sortable by creation time)
- **v1/v6**: Time-based with MAC address
- **v5**: Namespace-based (SHA-1)

Platypus provides validation for any UUID version, plus version-specific types using const generics for compile-time version specification.

## Objective

Implement UUID validation with:
- `ValidUuid` - any valid UUID (any version)
- `UuidVersion<N>` - specific version using const generics
- Type aliases for common versions (v4, v7)
- Conversion helper to `uuid::Uuid`

## Requirements

### Functional Requirements

1. **ValidUuid Predicate**
   - Implements `stillwater::refined::Predicate<String>`
   - Uses `uuid::Uuid::parse_str()` for validation
   - Accepts any UUID version
   - Returns `DomainError` with appropriate context on failure

2. **UuidVersion<N> Predicate**
   - Const generic over version number
   - Validates that UUID is specific version
   - Returns error with version mismatch details

3. **Type Aliases**
   - `Uuid = Refined<String, ValidUuid>` - any UUID
   - `UuidV4 = Refined<String, UuidVersion<4>>` - random UUID
   - `UuidV7 = Refined<String, UuidVersion<7>>` - time-ordered UUID

4. **Conversion Helper**
   - `Uuid::to_uuid(&self) -> uuid::Uuid` - infallible conversion

5. **Feature Gating**
   - Only compiled when `uuid` feature is enabled

### Non-Functional Requirements

1. **RFC Compliance**: RFC 4122 UUID format
2. **Type Safety**: Version enforced at type level
3. **Zero Unsafe Code**: Pure safe Rust implementation

## Acceptance Criteria

- [ ] `ValidUuid` struct implements `Predicate<String>`
- [ ] `UuidVersion<N>` struct implements `Predicate<String>` with const generic
- [ ] `Uuid`, `UuidV4`, `UuidV7` type aliases defined
- [ ] Invalid UUIDs return `DomainErrorKind::InvalidFormat`
- [ ] Wrong version returns `DomainErrorKind::InvalidComponent`
- [ ] `to_uuid()` conversion method works
- [ ] Compiles only with `uuid` feature enabled
- [ ] Unit tests for all UUID versions

## Technical Details

### Implementation Approach

```rust
// src/uuid.rs

use crate::error::{DomainError, DomainErrorKind};
use stillwater::refined::{Predicate, Refined};
use uuid::Uuid as UuidImpl;

/// Any valid UUID (any version).
///
/// Uses the `uuid` crate for parsing and validation.
///
/// # Example
/// ```
/// use platypus::uuid::Uuid;
///
/// let id = Uuid::new("550e8400-e29b-41d4-a716-446655440000".to_string());
/// assert!(id.is_ok());
/// ```
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

/// UUID must be specific version.
///
/// Uses const generics to specify the required version at compile time.
///
/// # Example
/// ```
/// use platypus::uuid::UuidV4;
///
/// // v4 UUID passes
/// let v4 = UuidV4::new("550e8400-e29b-41d4-a716-446655440000".to_string());
/// assert!(v4.is_ok());
///
/// // v7 UUID fails (wrong version)
/// let v7_as_v4 = UuidV4::new("018f6b8e-e4a0-7000-8000-000000000000".to_string());
/// assert!(v7_as_v4.is_err());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct UuidVersion<const V: usize>;

impl<const V: usize> Predicate<String> for UuidVersion<V> {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        let parsed = UuidImpl::parse_str(value).map_err(|_| DomainError {
            format_name: "UUID",
            value: value.clone(),
            reason: DomainErrorKind::InvalidFormat {
                expected: "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
            },
            example: "550e8400-e29b-41d4-a716-446655440000",
        })?;

        let actual_version = parsed.get_version_num();
        if actual_version == V {
            Ok(())
        } else {
            Err(DomainError {
                format_name: uuid_version_name::<V>(),
                value: value.clone(),
                reason: DomainErrorKind::InvalidComponent {
                    component: "version",
                    reason: format!("expected v{}, got v{}", V, actual_version),
                },
                example: uuid_version_example::<V>(),
            })
        }
    }

    fn description() -> &'static str {
        "UUID with specific version"
    }
}

/// Returns format name for UUID version.
const fn uuid_version_name<const V: usize>() -> &'static str {
    match V {
        1 => "UUID v1",
        4 => "UUID v4",
        5 => "UUID v5",
        6 => "UUID v6",
        7 => "UUID v7",
        _ => "UUID",
    }
}

/// Returns example for UUID version.
const fn uuid_version_example<const V: usize>() -> &'static str {
    match V {
        4 => "550e8400-e29b-41d4-a716-446655440000",
        7 => "018f6b8e-e4a0-7000-8000-000000000000",
        _ => "xxxxxxxx-xxxx-Vxxx-xxxx-xxxxxxxxxxxx",
    }
}

/// Any valid UUID.
pub type Uuid = Refined<String, ValidUuid>;

/// UUID version 4 (random).
///
/// The most common UUID type, generated from random bytes.
pub type UuidV4 = Refined<String, UuidVersion<4>>;

/// UUID version 7 (time-ordered random).
///
/// Ideal for database primary keys as they sort by creation time.
pub type UuidV7 = Refined<String, UuidVersion<7>>;

// Conversion helper implementations
impl Uuid {
    /// Convert to `uuid::Uuid`.
    ///
    /// This is infallible because the value has already been validated.
    ///
    /// # Example
    /// ```
    /// use platypus::uuid::Uuid;
    ///
    /// let validated = Uuid::new("550e8400-e29b-41d4-a716-446655440000".to_string()).unwrap();
    /// let uuid_impl = validated.to_uuid();
    /// assert_eq!(uuid_impl.get_version_num(), 4);
    /// ```
    pub fn to_uuid(&self) -> UuidImpl {
        UuidImpl::parse_str(self.get()).expect("already validated")
    }
}

impl UuidV4 {
    /// Convert to `uuid::Uuid`.
    pub fn to_uuid(&self) -> UuidImpl {
        UuidImpl::parse_str(self.get()).expect("already validated")
    }
}

impl UuidV7 {
    /// Convert to `uuid::Uuid`.
    pub fn to_uuid(&self) -> UuidImpl {
        UuidImpl::parse_str(self.get()).expect("already validated")
    }
}
```

### Module Integration

```rust
// In src/lib.rs
#[cfg(feature = "uuid")]
pub mod uuid;

#[cfg(feature = "uuid")]
pub use uuid::{Uuid, UuidV4, UuidV7, UuidVersion, ValidUuid};
```

## Dependencies

- **Prerequisites**: Spec 1 (crate structure), Spec 2 (error types)
- **Affected Components**: prelude.rs, lib.rs exports
- **External Dependencies**: `uuid` crate (version 1)

## Testing Strategy

### ValidUuid Tests

```rust
#[test]
fn valid_uuid_v4() {
    assert!(Uuid::new("550e8400-e29b-41d4-a716-446655440000".to_string()).is_ok());
}

#[test]
fn valid_uuid_v7() {
    assert!(Uuid::new("018f6b8e-e4a0-7000-8000-000000000000".to_string()).is_ok());
}

#[test]
fn valid_uuid_lowercase() {
    assert!(Uuid::new("550e8400-e29b-41d4-a716-446655440000".to_string()).is_ok());
}

#[test]
fn valid_uuid_uppercase() {
    assert!(Uuid::new("550E8400-E29B-41D4-A716-446655440000".to_string()).is_ok());
}

#[test]
fn invalid_uuid_format() {
    assert!(Uuid::new("not-a-uuid".to_string()).is_err());
}

#[test]
fn invalid_uuid_too_short() {
    assert!(Uuid::new("550e8400-e29b-41d4".to_string()).is_err());
}

#[test]
fn invalid_uuid_wrong_chars() {
    assert!(Uuid::new("550e8400-e29b-41d4-a716-44665544gggg".to_string()).is_err());
}
```

### UuidVersion Tests

```rust
#[test]
fn uuid_v4_accepts_v4() {
    let v4 = "550e8400-e29b-41d4-a716-446655440000";
    assert!(UuidV4::new(v4.to_string()).is_ok());
}

#[test]
fn uuid_v4_rejects_v7() {
    let v7 = "018f6b8e-e4a0-7000-8000-000000000000";
    let result = UuidV4::new(v7.to_string());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.reason, DomainErrorKind::InvalidComponent { .. }));
}

#[test]
fn uuid_v7_accepts_v7() {
    let v7 = "018f6b8e-e4a0-7000-8000-000000000000";
    assert!(UuidV7::new(v7.to_string()).is_ok());
}

#[test]
fn uuid_v7_rejects_v4() {
    let v4 = "550e8400-e29b-41d4-a716-446655440000";
    let result = UuidV7::new(v4.to_string());
    assert!(result.is_err());
}
```

### Conversion Tests

```rust
#[test]
fn to_uuid_returns_correct_type() {
    let validated = Uuid::new("550e8400-e29b-41d4-a716-446655440000".to_string()).unwrap();
    let uuid_impl = validated.to_uuid();
    assert_eq!(uuid_impl.get_version_num(), 4);
}

#[test]
fn uuid_v4_to_uuid_is_version_4() {
    let validated = UuidV4::new("550e8400-e29b-41d4-a716-446655440000".to_string()).unwrap();
    let uuid_impl = validated.to_uuid();
    assert_eq!(uuid_impl.get_version_num(), 4);
}
```

## Documentation Requirements

- **Code Documentation**: Doc comments with examples on all types
- **Version Guide**: Explain when to use v4 vs v7
- **User Documentation**: Usage examples in README

## Implementation Notes

- Const generic `UuidVersion<N>` allows compile-time version specification
- The `to_uuid()` method uses `expect()` because value is already validated
- Consider adding `From<Uuid>` impl for `uuid::Uuid` in future
- UUID parsing is case-insensitive per RFC 4122

## Migration and Compatibility

N/A - New type with no existing code to migrate.

## Future Considerations

Not in scope for initial implementation:
- `Uuid::generate_v4()` - generate new random UUID
- `Uuid::generate_v7()` - generate new time-ordered UUID
- `UuidV1`, `UuidV5`, `UuidV6` type aliases
- `From<Uuid>` impl for `uuid::Uuid`
