//! Integration tests for platypus.
//!
//! This file tests the public API of platypus to ensure all modules
//! compile and export correctly with various feature flag combinations.

use platypus::prelude::*;

#[test]
fn test_error_types_accessible() {
    // Verify error types are exported correctly
    let error = DomainError::new(DomainErrorKind::InvalidFormat, "test", "Test error message");
    assert_eq!(error.value, "test");
    assert_eq!(error.kind, DomainErrorKind::InvalidFormat);
}

#[test]
fn test_error_with_example() {
    let error = DomainError::new(DomainErrorKind::Empty, "", "Value cannot be empty")
        .with_example("example@domain.com");

    assert_eq!(error.example, Some("example@domain.com".to_string()));
}

#[test]
fn test_error_display() {
    let error = DomainError::new(
        DomainErrorKind::InvalidFormat,
        "bad-input",
        "Invalid format provided",
    )
    .with_example("valid-input");

    let display = format!("{}", error);
    assert!(display.contains("Invalid format provided"));
    assert!(display.contains("bad-input"));
    assert!(display.contains("valid-input"));
}
