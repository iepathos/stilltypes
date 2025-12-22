//! UUID validation example demonstrating resource ID management with stilltypes.
//!
//! This example shows how to use UUID types for validating resource identifiers,
//! including version-specific validation (v4 for random IDs, v7 for time-ordered).
//!
//! Run with: cargo run --example uuid_validation --features full

use stilltypes::prelude::*;
use stilltypes::uuid::{ToUuid, Uuid, UuidV4, UuidV7};
use stillwater::validation::{ValidateAll, Validation};

/// Raw API request containing resource identifiers.
#[derive(Debug)]
struct ResourceRequest {
    /// User ID (any valid UUID)
    user_id: String,
    /// Session ID (must be v4 random UUID)
    session_id: String,
    /// Transaction ID (must be v7 time-ordered UUID)
    transaction_id: String,
}

/// Validated resource request - all IDs guaranteed valid.
#[derive(Debug)]
struct ValidResourceRequest {
    user_id: Uuid,
    session_id: UuidV4,
    transaction_id: UuidV7,
}

/// Validates a resource request, accumulating all UUID errors.
fn validate_request(
    request: ResourceRequest,
) -> Validation<ValidResourceRequest, Vec<DomainError>> {
    let user_v = Validation::from_result(Uuid::new(request.user_id).map_err(|e| vec![e]));

    let session_v = Validation::from_result(UuidV4::new(request.session_id).map_err(|e| vec![e]));

    let tx_v = Validation::from_result(UuidV7::new(request.transaction_id).map_err(|e| vec![e]));

    (user_v, session_v, tx_v)
        .validate_all()
        .map(
            |(user_id, session_id, transaction_id)| ValidResourceRequest {
                user_id,
                session_id,
                transaction_id,
            },
        )
}

/// Pure function - extracts version info from validated UUIDs.
/// No validation needed because types guarantee correctness.
fn describe_uuids(request: &ValidResourceRequest) -> String {
    let user_uuid = request.user_id.to_uuid();
    let session_uuid = request.session_id.to_uuid();
    let tx_uuid = request.transaction_id.to_uuid();

    format!(
        "User (v{}): {}\nSession (v{}): {}\nTransaction (v{}): {}",
        user_uuid.get_version_num(),
        user_uuid,
        session_uuid.get_version_num(),
        session_uuid,
        tx_uuid.get_version_num(),
        tx_uuid
    )
}

fn main() {
    println!("Stilltypes UUID Validation Example");
    println!("===================================\n");

    // Example 1: Valid UUIDs with correct versions
    println!("=== Valid UUIDs ===");
    let valid_request = ResourceRequest {
        user_id: "550e8400-e29b-41d4-a716-446655440000".into(), // v4
        session_id: "550e8400-e29b-41d4-a716-446655440000".into(), // v4
        transaction_id: "018f6b8e-e4a0-7000-8000-000000000000".into(), // v7
    };

    match validate_request(valid_request) {
        Validation::Success(req) => {
            println!("Validation successful!");
            println!("{}", describe_uuids(&req));
        }
        Validation::Failure(errors) => {
            println!("Validation failed with {} errors:", errors.len());
            for err in errors {
                println!("  - {}", err);
            }
        }
    }

    // Example 2: Wrong UUID version
    println!("\n=== Wrong UUID Version ===");
    let wrong_version = ResourceRequest {
        user_id: "550e8400-e29b-41d4-a716-446655440000".into(),
        session_id: "018f6b8e-e4a0-7000-8000-000000000000".into(), // v7 passed to v4 field!
        transaction_id: "550e8400-e29b-41d4-a716-446655440000".into(), // v4 passed to v7 field!
    };

    match validate_request(wrong_version) {
        Validation::Success(_) => println!("Unexpected success!"),
        Validation::Failure(errors) => {
            println!("Validation failed with {} errors:", errors.len());
            for err in &errors {
                println!("  - {}", err);
            }
        }
    }

    // Example 3: Invalid UUID format
    println!("\n=== Invalid UUID Format ===");
    let invalid_format = ResourceRequest {
        user_id: "not-a-uuid".into(),
        session_id: "also-invalid".into(),
        transaction_id: "12345".into(),
    };

    match validate_request(invalid_format) {
        Validation::Success(_) => println!("Unexpected success!"),
        Validation::Failure(errors) => {
            println!("All {} UUID errors collected:", errors.len());
            for err in &errors {
                println!("  - {}", err);
            }
        }
    }

    // Example 4: Individual UUID validation
    println!("\n=== Individual UUID Validation ===");

    println!("\nAny UUID (any version accepted):");
    for input in [
        "550e8400-e29b-41d4-a716-446655440000",
        "018f6b8e-e4a0-7000-8000-000000000000",
        "invalid",
    ] {
        match Uuid::new(input.to_string()) {
            Ok(uuid) => {
                let impl_uuid = uuid.to_uuid();
                println!(
                    "  '{}' -> valid (version {})",
                    input,
                    impl_uuid.get_version_num()
                );
            }
            Err(e) => println!("  '{}' -> {}", input, e),
        }
    }

    println!("\nUUID v4 (random only):");
    for input in [
        "550e8400-e29b-41d4-a716-446655440000",
        "018f6b8e-e4a0-7000-8000-000000000000",
    ] {
        match UuidV4::new(input.to_string()) {
            Ok(_) => println!("  '{}' -> valid v4", input),
            Err(e) => println!("  '{}' -> {}", input, e),
        }
    }

    println!("\nUUID v7 (time-ordered only):");
    for input in [
        "018f6b8e-e4a0-7000-8000-000000000000",
        "550e8400-e29b-41d4-a716-446655440000",
    ] {
        match UuidV7::new(input.to_string()) {
            Ok(_) => println!("  '{}' -> valid v7", input),
            Err(e) => println!("  '{}' -> {}", input, e),
        }
    }

    println!("\n=== Key Concepts ===");
    println!("1. Uuid accepts any valid UUID regardless of version");
    println!("2. UuidV4 enforces version 4 (random) UUIDs");
    println!("3. UuidV7 enforces version 7 (time-ordered) UUIDs");
    println!("4. ToUuid trait converts to uuid::Uuid for further operations");
    println!("5. Version mismatches produce clear error messages");
}
