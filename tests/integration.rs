//! Integration tests for stilltypes.
//!
//! This file tests the public API of stilltypes to ensure all modules
//! compile and export correctly with various feature flag combinations.

use stilltypes::prelude::*;

#[test]
fn test_error_types_accessible() {
    // Verify error types are exported correctly
    let error = DomainError {
        format_name: "test field",
        value: "test".to_string(),
        reason: DomainErrorKind::InvalidFormat {
            expected: "valid format",
        },
        example: "valid-example",
    };
    assert_eq!(error.value, "test");
    assert!(matches!(
        error.reason,
        DomainErrorKind::InvalidFormat { .. }
    ));
}

#[test]
fn test_error_with_empty_value() {
    let error = DomainError {
        format_name: "email address",
        value: "".to_string(),
        reason: DomainErrorKind::Empty,
        example: "example@domain.com",
    };

    assert_eq!(error.example, "example@domain.com");
    assert!(matches!(error.reason, DomainErrorKind::Empty));
}

#[test]
fn test_error_display() {
    let error = DomainError {
        format_name: "input field",
        value: "bad-input".to_string(),
        reason: DomainErrorKind::InvalidFormat {
            expected: "proper format",
        },
        example: "valid-input",
    };

    let display = format!("{}", error);
    assert!(display.contains("invalid input field"));
    assert!(display.contains("proper format"));
    assert!(display.contains("valid-input"));
}

#[test]
fn test_error_accumulation_with_vec() {
    let errors: Vec<DomainError> = vec![
        DomainError {
            format_name: "email",
            value: "bad".to_string(),
            reason: DomainErrorKind::InvalidFormat {
                expected: "local@domain",
            },
            example: "user@example.com",
        },
        DomainError {
            format_name: "phone",
            value: "".to_string(),
            reason: DomainErrorKind::Empty,
            example: "+1-555-555-5555",
        },
    ];

    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].format_name, "email");
    assert_eq!(errors[1].format_name, "phone");
}
