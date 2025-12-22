//! UUID validation types.
//!
//! Provides RFC 4122 compliant UUID validation using the `uuid` crate.
//!
//! This module provides validation for any UUID version, plus version-specific types
//! using const generics for compile-time version specification.
//!
//! # Example
//!
//! ```
//! use platypus::uuid::{Uuid, UuidV4, UuidV7, ToUuid};
//!
//! // Any valid UUID (any version)
//! let uuid = Uuid::new("550e8400-e29b-41d4-a716-446655440000".to_string());
//! assert!(uuid.is_ok());
//!
//! // Version-specific UUIDs
//! let v4 = UuidV4::new("550e8400-e29b-41d4-a716-446655440000".to_string());
//! assert!(v4.is_ok());
//!
//! // Wrong version fails
//! let v7_as_v4 = UuidV4::new("018f6b8e-e4a0-7000-8000-000000000000".to_string());
//! assert!(v7_as_v4.is_err());
//!
//! // Convert to uuid::Uuid
//! let validated = v4.unwrap();
//! let uuid_impl = validated.to_uuid();
//! assert_eq!(uuid_impl.get_version_num(), 4);
//! ```

use crate::error::{DomainError, DomainErrorKind};
use stillwater::refined::{Predicate, Refined};
use uuid::Uuid as UuidImpl;

/// Any valid UUID (any version).
///
/// Uses the `uuid` crate for parsing and validation according to RFC 4122.
///
/// # Example
///
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
        UuidImpl::parse_str(value)
            .map(|_| ())
            .map_err(|_| DomainError {
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

/// UUID must be a specific version.
///
/// Uses const generics to specify the required version at compile time.
///
/// # Example
///
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
///
/// # Panics
/// Panics if V is not 4 or 7. Only these versions are publicly exposed.
const fn uuid_version_name<const V: usize>() -> &'static str {
    match V {
        4 => "UUID v4",
        7 => "UUID v7",
        _ => unreachable!(),
    }
}

/// Returns example for UUID version.
///
/// # Panics
/// Panics if V is not 4 or 7. Only these versions are publicly exposed.
const fn uuid_version_example<const V: usize>() -> &'static str {
    match V {
        4 => "550e8400-e29b-41d4-a716-446655440000",
        7 => "018f6b8e-e4a0-7000-8000-000000000000",
        _ => unreachable!(),
    }
}

/// Any valid UUID.
///
/// A `String` that has been validated to be a properly formatted UUID
/// according to RFC 4122.
///
/// # Example
///
/// ```
/// use platypus::uuid::Uuid;
///
/// let validated = Uuid::new("550e8400-e29b-41d4-a716-446655440000".to_string()).unwrap();
/// assert_eq!(validated.get(), "550e8400-e29b-41d4-a716-446655440000");
/// ```
pub type Uuid = Refined<String, ValidUuid>;

/// UUID version 4 (random).
///
/// The most common UUID type, generated from random bytes.
///
/// # Example
///
/// ```
/// use platypus::uuid::UuidV4;
///
/// let v4 = UuidV4::new("550e8400-e29b-41d4-a716-446655440000".to_string()).unwrap();
/// ```
pub type UuidV4 = Refined<String, UuidVersion<4>>;

/// UUID version 7 (time-ordered random).
///
/// Ideal for database primary keys as they sort by creation time.
///
/// # Example
///
/// ```
/// use platypus::uuid::UuidV7;
///
/// let v7 = UuidV7::new("018f6b8e-e4a0-7000-8000-000000000000".to_string()).unwrap();
/// ```
pub type UuidV7 = Refined<String, UuidVersion<7>>;

/// Trait for converting validated UUID types to `uuid::Uuid`.
///
/// This trait is automatically implemented for all validated UUID types.
///
/// # Example
///
/// ```
/// use platypus::uuid::{Uuid, ToUuid};
///
/// let validated = Uuid::new("550e8400-e29b-41d4-a716-446655440000".to_string()).unwrap();
/// let uuid_impl = validated.to_uuid();
/// assert_eq!(uuid_impl.get_version_num(), 4);
/// ```
pub trait ToUuid {
    /// Convert to `uuid::Uuid`.
    ///
    /// This is infallible because the value has already been validated.
    fn to_uuid(&self) -> UuidImpl;
}

impl<P: Predicate<String>> ToUuid for Refined<String, P> {
    fn to_uuid(&self) -> UuidImpl {
        UuidImpl::parse_str(self.get()).expect("already validated")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ValidUuid tests
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

    // UuidVersion tests
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
        assert!(matches!(
            err.reason,
            DomainErrorKind::InvalidComponent { .. }
        ));
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

    // Conversion tests
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

    #[test]
    fn uuid_v7_to_uuid_is_version_7() {
        let validated = UuidV7::new("018f6b8e-e4a0-7000-8000-000000000000".to_string()).unwrap();
        let uuid_impl = validated.to_uuid();
        assert_eq!(uuid_impl.get_version_num(), 7);
    }

    // Error message tests
    #[test]
    fn error_includes_format_name() {
        let result = Uuid::new("invalid".to_string());
        let err = result.unwrap_err();
        assert_eq!(err.format_name, "UUID");
    }

    #[test]
    fn error_includes_example() {
        let result = Uuid::new("invalid".to_string());
        let err = result.unwrap_err();
        assert_eq!(err.example, "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn version_error_is_invalid_component() {
        let result = UuidV4::new("018f6b8e-e4a0-7000-8000-000000000000".to_string());
        let err = result.unwrap_err();
        match err.reason {
            DomainErrorKind::InvalidComponent { component, reason } => {
                assert_eq!(component, "version");
                assert!(reason.contains("v4"));
                assert!(reason.contains("v7"));
            }
            _ => panic!("Expected InvalidComponent error"),
        }
    }

    #[test]
    fn valid_uuid_description() {
        assert_eq!(ValidUuid::description(), "UUID");
    }

    #[test]
    fn uuid_version_description() {
        assert_eq!(
            UuidVersion::<4>::description(),
            "UUID with specific version"
        );
    }

    // Test malformed UUID passed to version-specific type
    #[test]
    fn uuid_v4_rejects_malformed() {
        let result = UuidV4::new("not-a-uuid".to_string());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.format_name, "UUID");
        assert!(matches!(
            err.reason,
            DomainErrorKind::InvalidFormat { .. }
        ));
    }

    #[test]
    fn uuid_v7_rejects_malformed() {
        let result = UuidV7::new("invalid".to_string());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err.reason,
            DomainErrorKind::InvalidFormat { .. }
        ));
    }
}
