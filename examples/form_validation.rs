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

    // Demonstrate error creation and formatting
    let errors = vec![
        DomainError::new(
            DomainErrorKind::InvalidFormat,
            "not-an-email",
            "Invalid email format",
        )
        .with_example("user@example.com"),
        DomainError::new(DomainErrorKind::Empty, "", "Phone number cannot be empty")
            .with_example("+1-555-123-4567"),
        DomainError::new(
            DomainErrorKind::TooShort,
            "abc",
            "Password must be at least 8 characters",
        ),
    ];

    println!("Simulated validation errors:\n");
    for (i, error) in errors.iter().enumerate() {
        println!("Error {}:", i + 1);
        println!("{}\n", error);
    }

    println!("Error kinds available:");
    println!("  - InvalidFormat: For malformed input");
    println!("  - Empty: For required fields that are empty");
    println!("  - TooLong: For values exceeding max length");
    println!("  - TooShort: For values below min length");
    println!("  - Custom: For domain-specific validation rules");

    println!("\n\nNote: Domain types (Email, Url, etc.) will be");
    println!("implemented in subsequent specifications.");
}
