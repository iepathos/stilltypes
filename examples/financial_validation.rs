//! Financial validation example demonstrating payment form validation with stilltypes.
//!
//! This example shows how to validate IBAN and credit card numbers with
//! automatic security features (masking in errors and display).
//!
//! Run with: cargo run --example financial_validation --features full

use stilltypes::financial::{CreditCardExt, CreditCardNumber, Iban, IbanExt};
use stilltypes::prelude::*;
use stillwater::validation::{ValidateAll, Validation};

/// Raw payment method input from user.
#[derive(Debug)]
struct PaymentInput {
    method: PaymentMethod,
}

#[derive(Debug)]
enum PaymentMethod {
    Card { number: String, name: String },
    BankTransfer { iban: String, bic: String },
}

/// Validated payment method - financial data guaranteed valid.
#[derive(Debug)]
enum ValidPaymentMethod {
    Card {
        number: CreditCardNumber,
        name: String,
    },
    BankTransfer {
        iban: Iban,
        bic: String,
    },
}

/// Validates a payment method, accumulating all errors.
fn validate_payment(input: PaymentInput) -> Validation<ValidPaymentMethod, Vec<DomainError>> {
    match input.method {
        PaymentMethod::Card { number, name } => {
            Validation::from_result(CreditCardNumber::new(number).map_err(|e| vec![e]))
                .map(|card| ValidPaymentMethod::Card { number: card, name })
        }
        PaymentMethod::BankTransfer { iban, bic } => {
            Validation::from_result(Iban::new(iban).map_err(|e| vec![e]))
                .map(|iban| ValidPaymentMethod::BankTransfer { iban, bic })
        }
    }
}

/// Raw checkout form with multiple payment options.
#[derive(Debug)]
struct CheckoutForm {
    primary_card: String,
    backup_card: Option<String>,
    bank_account: Option<String>,
}

/// Validated checkout with all payment methods verified.
#[derive(Debug)]
struct ValidCheckout {
    primary_card: CreditCardNumber,
    backup_card: Option<CreditCardNumber>,
    bank_account: Option<Iban>,
}

/// Validates a full checkout form, accumulating all errors.
fn validate_checkout(form: CheckoutForm) -> Validation<ValidCheckout, Vec<DomainError>> {
    let primary_v =
        Validation::from_result(CreditCardNumber::new(form.primary_card).map_err(|e| vec![e]));

    let backup_v: Validation<Option<CreditCardNumber>, Vec<DomainError>> = match form.backup_card {
        Some(card) => {
            Validation::from_result(CreditCardNumber::new(card).map_err(|e| vec![e])).map(Some)
        }
        None => Validation::Success(None),
    };

    let bank_v: Validation<Option<Iban>, Vec<DomainError>> = match form.bank_account {
        Some(iban) => Validation::from_result(Iban::new(iban).map_err(|e| vec![e])).map(Some),
        None => Validation::Success(None),
    };

    (primary_v, backup_v, bank_v)
        .validate_all()
        .map(|(primary_card, backup_card, bank_account)| ValidCheckout {
            primary_card,
            backup_card,
            bank_account,
        })
}

/// Pure function - generates payment receipt from validated data.
/// No validation needed because types guarantee correctness.
fn generate_receipt(checkout: &ValidCheckout) -> String {
    let mut receipt = String::from("Payment Receipt\n");
    receipt.push_str("===============\n");
    receipt.push_str(&format!(
        "Primary Card: {} ({})\n",
        checkout.primary_card.masked(),
        checkout.primary_card.last_four()
    ));

    if let Some(ref backup) = checkout.backup_card {
        receipt.push_str(&format!("Backup Card: {}\n", backup.masked()));
    }

    if let Some(ref iban) = checkout.bank_account {
        receipt.push_str(&format!(
            "Bank Account: {} ({})\n",
            iban.masked(),
            iban.country_code()
        ));
    }

    receipt
}

fn main() {
    println!("Stilltypes Financial Validation Example");
    println!("========================================\n");

    // Example 1: Valid credit card payment
    println!("=== Valid Credit Card ===");
    let card_payment = PaymentInput {
        method: PaymentMethod::Card {
            number: "4111111111111111".into(),
            name: "John Doe".into(),
        },
    };

    match validate_payment(card_payment) {
        Validation::Success(ValidPaymentMethod::Card { number, name }) => {
            println!("Payment method valid!");
            println!("  Cardholder: {}", name);
            println!(
                "  Card: {} (last 4: {})",
                number.masked(),
                number.last_four()
            );
        }
        Validation::Success(_) => unreachable!(),
        Validation::Failure(errors) => {
            for err in errors {
                println!("  Error: {}", err);
            }
        }
    }

    // Example 2: Valid IBAN payment
    println!("\n=== Valid Bank Transfer ===");
    let bank_payment = PaymentInput {
        method: PaymentMethod::BankTransfer {
            iban: "DE89370400440532013000".into(),
            bic: "COBADEFFXXX".into(),
        },
    };

    match validate_payment(bank_payment) {
        Validation::Success(ValidPaymentMethod::BankTransfer { iban, bic }) => {
            println!("Payment method valid!");
            println!(
                "  IBAN: {} (country: {})",
                iban.masked(),
                iban.country_code()
            );
            println!("  BIC: {}", bic);
        }
        Validation::Success(_) => unreachable!(),
        Validation::Failure(errors) => {
            for err in errors {
                println!("  Error: {}", err);
            }
        }
    }

    // Example 3: Invalid credit card (Luhn check fails)
    println!("\n=== Invalid Credit Card (Luhn failure) ===");
    let bad_card = PaymentInput {
        method: PaymentMethod::Card {
            number: "4111111111111112".into(), // Wrong check digit
            name: "Jane Doe".into(),
        },
    };

    match validate_payment(bad_card) {
        Validation::Success(_) => println!("Unexpected success!"),
        Validation::Failure(errors) => {
            println!("Validation failed (card number masked in error):");
            for err in &errors {
                println!("  - {}", err);
                // Note: The card number is automatically masked
                assert!(err.value.starts_with("****"));
            }
        }
    }

    // Example 4: Invalid IBAN (checksum fails)
    println!("\n=== Invalid IBAN (checksum failure) ===");
    let bad_iban = PaymentInput {
        method: PaymentMethod::BankTransfer {
            iban: "DE00370400440532013000".into(), // Wrong check digits
            bic: "COBADEFFXXX".into(),
        },
    };

    match validate_payment(bad_iban) {
        Validation::Success(_) => println!("Unexpected success!"),
        Validation::Failure(errors) => {
            println!("Validation failed (IBAN masked in error):");
            for err in &errors {
                println!("  - {}", err);
            }
        }
    }

    // Example 5: Full checkout validation with multiple payment methods
    println!("\n=== Full Checkout Validation ===");
    let checkout = CheckoutForm {
        primary_card: "4111111111111111".into(),
        backup_card: Some("5500000000000004".into()), // Mastercard test
        bank_account: Some("GB82WEST12345698765432".into()), // UK IBAN
    };

    match validate_checkout(checkout) {
        Validation::Success(valid) => {
            println!("Checkout valid!");
            println!("\n{}", generate_receipt(&valid));
        }
        Validation::Failure(errors) => {
            println!("Checkout validation failed with {} errors:", errors.len());
            for err in errors {
                println!("  - {}", err);
            }
        }
    }

    // Example 6: Checkout with invalid backup card
    println!("\n=== Checkout with Invalid Backup Card ===");
    let bad_checkout = CheckoutForm {
        primary_card: "4111111111111111".into(),
        backup_card: Some("1234567890123456".into()), // Invalid card
        bank_account: Some("INVALID_IBAN".into()),    // Invalid IBAN
    };

    match validate_checkout(bad_checkout) {
        Validation::Success(_) => println!("Unexpected success!"),
        Validation::Failure(errors) => {
            println!("All {} errors collected:", errors.len());
            for err in &errors {
                println!("  - {}", err);
            }
        }
    }

    // Example 7: Various card formats
    println!("\n=== Card Format Variations ===");
    let formats = [
        "4111111111111111",    // No separators
        "4111 1111 1111 1111", // Spaces
        "4111-1111-1111-1111", // Dashes
        "5500000000000004",    // Mastercard
        "340000000000009",     // Amex (15 digits)
        "4111111111111112",    // Invalid (Luhn fails)
    ];

    for card in formats {
        match CreditCardNumber::new(card.to_string()) {
            Ok(valid) => println!("  '{}' -> valid ({})", card, valid.masked()),
            Err(e) => println!("  '{}' -> {}", card, e),
        }
    }

    // Example 8: Various IBAN formats
    println!("\n=== IBAN Country Examples ===");
    let ibans = [
        ("DE89370400440532013000", "Germany"),
        ("GB82WEST12345698765432", "UK"),
        ("FR7630006000011234567890189", "France"),
        ("ES9121000418450200051332", "Spain"),
        ("de89370400440532013000", "Germany (lowercase)"),
        ("INVALID", "Invalid"),
    ];

    for (iban, country) in ibans {
        match Iban::new(iban.to_string()) {
            Ok(valid) => println!(
                "  {} ({}): {} -> {}",
                country,
                valid.country_code(),
                iban,
                valid.masked()
            ),
            Err(e) => println!("  {}: {} -> {}", country, iban, e),
        }
    }

    println!("\n=== Security Features ===");
    println!("1. Credit card numbers are masked in error messages (****1234)");
    println!("2. IBANs are partially masked (DE89****3000)");
    println!("3. Extension traits provide safe display methods");
    println!("4. Sensitive data never appears in full in logs");
}
