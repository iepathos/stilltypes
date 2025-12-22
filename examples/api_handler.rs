//! API handler example demonstrating Effect composition with stilltypes.
//!
//! This example shows how to bridge from Validation to Effect for async I/O
//! operations after validation, following the "pure core, effects at boundary"
//! philosophy.
//!
//! Run with: cargo run --example api_handler --features full

use stilltypes::prelude::*;
use stillwater::effect::prelude::*;
use stillwater::validation::{ValidateAll, Validation};

/// Application environment providing dependencies.
#[derive(Clone)]
struct AppEnv {
    db_connected: bool,
}

/// Result of creating a user.
#[derive(Debug)]
struct UserId(#[allow(dead_code)] u64);

/// Application-level error type.
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
///
/// This is where I/O happens - at the boundary after validation.
fn create_user_in_db(
    email: Email,
    phone: PhoneNumber,
) -> impl Effect<Output = UserId, Error = AppError, Env = AppEnv> {
    asks(move |env: &AppEnv| {
        if env.db_connected {
            // In real code, this would be an async database call
            println!("  [DB] Creating user with email: {}", email.get());
            println!("  [DB] Normalized phone: {}", phone.to_e164());
            Ok::<UserId, AppError>(UserId(12345))
        } else {
            Err(AppError::Database("Not connected".to_string()))
        }
    })
    .and_then(from_result)
}

/// Full registration flow: validate inputs, then create user.
///
/// Demonstrates the pattern:
/// 1. Validate with accumulation (pure)
/// 2. Bridge to Effect (from_validation)
/// 3. Perform I/O (and_then with database call)
fn register_user(
    email_input: String,
    phone_input: String,
) -> impl Effect<Output = UserId, Error = AppError, Env = AppEnv> {
    // Step 1: Validate all inputs with error accumulation (pure function)
    let email_v: Validation<Email, Vec<DomainError>> =
        Validation::from_result(Email::new(email_input).map_err(|e| vec![e]));
    let phone_v: Validation<PhoneNumber, Vec<DomainError>> =
        Validation::from_result(PhoneNumber::new(phone_input).map_err(|e| vec![e]));

    let validated = (email_v, phone_v).validate_all();

    // Step 2: Bridge Validation to Effect
    from_validation::<_, _, AppEnv>(validated)
        .map_err(AppError::Validation)
        // Step 3: Perform I/O only if validation passed
        .and_then(|(email, phone)| create_user_in_db(email, phone))
}

#[tokio::main]
async fn main() {
    println!("Stilltypes API Handler Example");
    println!("=============================\n");

    let env = AppEnv { db_connected: true };

    // Example 1: Valid registration
    println!("=== Valid Registration ===");
    let effect = register_user("user@example.com".into(), "+14155551234".into());

    match effect.execute(&env).await {
        Ok(user_id) => println!("Created user: {:?}\n", user_id),
        Err(e) => println!("Failed: {}\n", e),
    }

    // Example 2: Invalid registration - multiple errors
    println!("=== Invalid Registration (multiple errors) ===");
    let effect = register_user("bad-email".into(), "bad-phone".into());

    match effect.execute(&env).await {
        Ok(user_id) => println!("Created user: {:?}\n", user_id),
        Err(e) => println!("Failed: {}\n", e),
    }

    // Example 3: Database error (valid input, but DB fails)
    println!("=== Database Error ===");
    let disconnected_env = AppEnv {
        db_connected: false,
    };
    let effect = register_user("user@example.com".into(), "+14155551234".into());

    match effect.execute(&disconnected_env).await {
        Ok(user_id) => println!("Created user: {:?}\n", user_id),
        Err(e) => println!("Failed: {}\n", e),
    }

    // Example 4: Show effect composition without executing
    println!("=== Effect Composition Demo ===");
    println!("Effects are lazy - nothing happens until execute() is called.");
    println!("This allows building complex pipelines before running them.");

    // Build a complex effect without running it
    let _complex_effect = register_user("test@example.com".into(), "+442071234567".into());

    println!("Effect built but not executed yet!");
    println!("\nKey concepts demonstrated:");
    println!("  1. Validation::from_result() converts Result to Validation");
    println!("  2. validate_all() accumulates all errors");
    println!("  3. from_validation() bridges to Effect");
    println!("  4. and_then() chains I/O operations");
    println!("  5. Environment (AppEnv) provides dependencies");
    println!("  6. Effects are lazy and composable");
}
