//! Form validation example demonstrating error accumulation with stilltypes.
//!
//! This example shows how to use stillwater's `Validation` to collect all
//! validation errors at once, rather than failing on the first error.
//!
//! Run with: cargo run --example form_validation --features full

use stilltypes::prelude::*;
use stillwater::validation::Validation;

/// Raw form input before validation.
#[derive(Debug)]
struct RegistrationForm {
    email: String,
    phone: String,
    website: String,
}

/// Validated registration data - these types guarantee validity.
#[derive(Debug)]
struct ValidRegistration {
    email: Email,
    phone: PhoneNumber,
    website: HttpUrl,
}

/// Converts an HttpUrl error to a DomainError.
///
/// HttpUrl uses `And<ValidUrl, HttpScheme>` which returns `AndError`.
/// We extract the first available error for display.
fn http_url_error_to_domain(
    e: stillwater::refined::AndError<DomainError, DomainError>,
) -> DomainError {
    match e {
        stillwater::refined::AndError::First(e) => e,
        stillwater::refined::AndError::Second(e) => e,
        stillwater::refined::AndError::Both(e, _) => e,
    }
}

/// Validates a registration form, accumulating all errors.
///
/// Uses `Validation::all` to run all validations and collect failures,
/// rather than short-circuiting on the first error.
fn validate(form: RegistrationForm) -> Validation<ValidRegistration, Vec<DomainError>> {
    let email_v: Validation<Email, Vec<DomainError>> =
        Validation::from_result(Email::new(form.email).map_err(|e| vec![e]));
    let phone_v: Validation<PhoneNumber, Vec<DomainError>> =
        Validation::from_result(PhoneNumber::new(form.phone).map_err(|e| vec![e]));
    let url_v: Validation<HttpUrl, Vec<DomainError>> = Validation::from_result(
        HttpUrl::new(form.website).map_err(|e| vec![http_url_error_to_domain(e)]),
    );

    // Use .validate_all() directly from the trait
    use stillwater::validation::ValidateAll;
    (email_v, phone_v, url_v)
        .validate_all()
        .map(|(email, phone, website)| ValidRegistration {
            email,
            phone,
            website,
        })
}

fn main() {
    println!("Stilltypes Form Validation Example");
    println!("=================================\n");

    // Example 1: Valid form - all fields pass validation
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

    // Example 2: Invalid form - all fields fail validation
    println!("\n=== Invalid Form (all fields wrong) ===");
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

    // Example 3: Partially invalid form - some fields fail
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

    // Example 4: Demonstrate individual type validation
    println!("\n=== Individual Type Validation ===");

    // Email validation
    println!("\nEmail validation:");
    for input in ["user@example.com", "invalid", "user+tag@gmail.com"] {
        match Email::new(input.to_string()) {
            Ok(email) => println!("  '{}' -> valid: {}", input, email.get()),
            Err(e) => println!("  '{}' -> {}", input, e),
        }
    }

    // Phone validation with normalization
    println!("\nPhone validation (with E.164 normalization):");
    for input in ["+1 (415) 555-1234", "+442071234567", "invalid"] {
        match PhoneNumber::new(input.to_string()) {
            Ok(phone) => println!(
                "  '{}' -> normalized: {}, country: {}",
                input,
                phone.to_e164(),
                phone.country_code()
            ),
            Err(e) => println!("  '{}' -> {}", input, e),
        }
    }

    // URL validation (different schemes)
    println!("\nURL validation:");

    println!("  Url (any valid URL):");
    for input in [
        "https://example.com",
        "http://example.com",
        "ftp://files.example.com",
        "file:///path/to/file",
    ] {
        match stilltypes::url::Url::new(input.to_string()) {
            Ok(url) => println!("    '{}' -> valid: {}", input, url.get()),
            Err(e) => println!("    '{}' -> {}", input, e),
        }
    }

    println!("  HttpUrl (HTTP or HTTPS only):");
    for input in [
        "https://example.com",
        "http://example.com",
        "ftp://files.example.com",
    ] {
        match HttpUrl::new(input.to_string()) {
            Ok(url) => println!("    '{}' -> valid: {}", input, url.get()),
            Err(_) => println!("    '{}' -> rejected (not HTTP/HTTPS)", input),
        }
    }

    println!("  SecureUrl (HTTPS only):");
    for input in ["https://example.com", "http://example.com"] {
        match SecureUrl::new(input.to_string()) {
            Ok(url) => println!("    '{}' -> valid: {}", input, url.get()),
            Err(_) => println!("    '{}' -> rejected (insecure)", input),
        }
    }

    println!("\n=== URL Type Hierarchy ===");
    println!("  Url       -> any valid RFC 3986 URL (ftp, file, http, etc.)");
    println!("  HttpUrl   -> HTTP or HTTPS only (web URLs)");
    println!("  SecureUrl -> HTTPS only (secure web URLs)");
}
