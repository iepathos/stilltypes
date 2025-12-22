//! Form validation example demonstrating platypus domain types.
//!
//! This example shows how domain types can be used to validate user input
//! in a form-like scenario, accumulating all errors rather than failing
//! on the first one.
//!
//! Run with: cargo run --example form_validation --all-features

use platypus::prelude::*;

fn main() {
    println!("Platypus Form Validation Example");
    println!("=================================\n");

    // Demonstrate error creation and formatting with rich context
    let errors = [
        DomainError {
            format_name: "email address",
            value: "not-an-email".to_string(),
            reason: DomainErrorKind::InvalidFormat {
                expected: "local@domain",
            },
            example: "user@example.com",
        },
        DomainError {
            format_name: "phone number",
            value: "".to_string(),
            reason: DomainErrorKind::Empty,
            example: "+1-555-123-4567",
        },
        DomainError {
            format_name: "password",
            value: "abc".to_string(),
            reason: DomainErrorKind::TooShort { min: 8, actual: 3 },
            example: "MySecurePassword123",
        },
        DomainError {
            format_name: "username",
            value: "user@name".to_string(),
            reason: DomainErrorKind::InvalidCharacter {
                char: '@',
                position: 4,
            },
            example: "valid_username",
        },
    ];

    println!("Simulated validation errors:\n");
    for (i, error) in errors.iter().enumerate() {
        println!("Error {}:", i + 1);
        println!("  {}\n", error);
    }

    println!("Error kinds available:");
    println!("  - Empty: For required fields that are empty");
    println!("  - TooLong: For values exceeding max length");
    println!("  - TooShort: For values below min length");
    println!("  - InvalidFormat: For malformed input");
    println!("  - InvalidCharacter: For invalid characters at specific positions");
    println!("  - InvalidChecksum: For checksum validation failures");
    println!("  - InvalidComponent: For invalid parts of complex values");

    println!("\n\nNote: Domain types (Email, Url, etc.) will be");
    println!("implemented in subsequent specifications.");
}
