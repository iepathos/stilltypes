//! Discount validation example demonstrating percentage and pricing calculations.
//!
//! This example shows how to use stilltypes' numeric types for discount/pricing
//! calculations with error accumulation.
//!
//! Run with: cargo run --example discount_validation --features full

use stilltypes::prelude::*;
use stillwater::validation::Validation;

/// Raw discount configuration input.
#[derive(Debug)]
struct DiscountInput {
    base_discount: f64,
    member_bonus: f64,
    probability: f64,
}

/// Validated discount configuration - these types guarantee validity.
#[derive(Debug)]
struct ValidDiscount {
    base_discount: Percentage,
    member_bonus: Percentage,
    probability: UnitInterval,
}

/// Validates a discount configuration, accumulating all errors.
fn validate_discount(input: DiscountInput) -> Validation<ValidDiscount, Vec<DomainError>> {
    let base_v: Validation<Percentage, Vec<DomainError>> =
        Validation::from_result(Percentage::new(input.base_discount).map_err(|e| vec![e]));
    let bonus_v: Validation<Percentage, Vec<DomainError>> =
        Validation::from_result(Percentage::new(input.member_bonus).map_err(|e| vec![e]));
    let prob_v: Validation<UnitInterval, Vec<DomainError>> =
        Validation::from_result(UnitInterval::new(input.probability).map_err(|e| vec![e]));

    use stillwater::validation::ValidateAll;
    (base_v, bonus_v, prob_v)
        .validate_all()
        .map(|(base_discount, member_bonus, probability)| ValidDiscount {
            base_discount,
            member_bonus,
            probability,
        })
}

/// Pure function - calculates final price with validated discounts.
fn calculate_discounted_price(
    price: f64,
    base_discount: &Percentage,
    member_bonus: &Percentage,
) -> f64 {
    // No validation needed - types guarantee 0-100 range
    let base_off = base_discount.of(price);
    let bonus_off = member_bonus.of(price);
    let total_off = (base_off + bonus_off).min(price); // Cap at price (100% max)
    price - total_off
}

/// Pure function - applies probability-weighted scoring.
fn weighted_score(base_score: f64, weight: &UnitInterval) -> f64 {
    // Types guarantee weight is 0-1, safe to multiply
    weight.scale(base_score)
}

fn main() {
    println!("Stilltypes Discount Validation Example");
    println!("======================================\n");

    // Example 1: Valid discount configuration
    println!("=== Valid Discount Configuration ===");
    let valid_input = DiscountInput {
        base_discount: 20.0,
        member_bonus: 10.0,
        probability: 0.75,
    };

    match validate_discount(valid_input) {
        Validation::Success(discount) => {
            println!("Discount configuration valid!");
            println!("  Base discount: {}%", discount.base_discount.get());
            println!("  Member bonus: {}%", discount.member_bonus.get());
            println!(
                "  Probability: {} ({}%)",
                discount.probability.get(),
                discount.probability.to_percentage().get()
            );

            // Calculate discounted price
            let original_price = 100.0;
            let final_price = calculate_discounted_price(
                original_price,
                &discount.base_discount,
                &discount.member_bonus,
            );
            println!("\n  Price calculation:");
            println!("    Original: ${:.2}", original_price);
            println!(
                "    Base discount ({}%): -${:.2}",
                discount.base_discount.get(),
                discount.base_discount.of(original_price)
            );
            println!(
                "    Member bonus ({}%): -${:.2}",
                discount.member_bonus.get(),
                discount.member_bonus.of(original_price)
            );
            println!("    Final price: ${:.2}", final_price);

            // Calculate weighted score
            let base_score = 100.0;
            let weighted = weighted_score(base_score, &discount.probability);
            println!("\n  Weighted score:");
            println!("    Base score: {}", base_score);
            println!("    Weight: {}", discount.probability.get());
            println!("    Result: {}", weighted);
        }
        Validation::Failure(errors) => {
            println!("Validation failed!");
            for err in errors {
                println!("  - {}", err);
            }
        }
    }

    // Example 2: Invalid discount configuration (all fields wrong)
    println!("\n=== Invalid Discount Configuration (all fields wrong) ===");
    let invalid_input = DiscountInput {
        base_discount: 150.0, // Too high
        member_bonus: -10.0,  // Negative
        probability: 1.5,     // Over 1
    };

    match validate_discount(invalid_input) {
        Validation::Success(_) => println!("Unexpected success!"),
        Validation::Failure(errors) => {
            println!("Validation failed with {} errors:", errors.len());
            for err in &errors {
                println!("  - {}", err);
            }
        }
    }

    // Example 3: Percentage conversions
    println!("\n=== Percentage Conversions ===");

    println!("\nConverting decimals to percentages:");
    for decimal in [0.0, 0.25, 0.5, 0.75, 1.0] {
        match Percentage::from_decimal(decimal) {
            Ok(pct) => println!("  {:.2} -> {}%", decimal, pct.get()),
            Err(e) => println!("  {:.2} -> Error: {}", decimal, e),
        }
    }

    println!("\nConverting percentages to decimals:");
    for percent in [0.0, 25.0, 50.0, 75.0, 100.0] {
        let pct = Percentage::new(percent).unwrap();
        println!("  {}% -> {:.2}", pct.get(), pct.to_decimal());
    }

    // Example 4: Complement calculations
    println!("\n=== Complement Calculations ===");
    let discount = Percentage::new(30.0).unwrap();
    let remaining = discount.complement();
    println!(
        "If discount is {}%, remaining is {}%",
        discount.get(),
        remaining.get()
    );

    let probability = UnitInterval::new(0.3).unwrap();
    let complement = probability.complement();
    println!(
        "If probability is {}, complement is {}",
        probability.get(),
        complement.get()
    );

    // Example 5: UnitInterval for opacity/alpha
    println!("\n=== UnitInterval for Opacity ===");
    for alpha in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let opacity = UnitInterval::new(alpha).unwrap();
        let byte_value = opacity.scale(255.0);
        println!(
            "  Opacity {} -> {} byte value ({:.0}/255)",
            alpha,
            opacity.to_percentage().get(),
            byte_value
        );
    }

    // Example 6: Edge cases and error messages
    println!("\n=== Edge Cases and Error Messages ===");

    println!("\nInvalid percentages:");
    for value in [-1.0, 101.0, f64::NAN, f64::INFINITY] {
        match Percentage::new(value) {
            Ok(_) => println!("  {} -> valid (unexpected)", value),
            Err(e) => println!("  {} -> {}", value, e),
        }
    }

    println!("\nInvalid unit intervals:");
    for value in [-0.1, 1.1, f64::NAN, f64::NEG_INFINITY] {
        match UnitInterval::new(value) {
            Ok(_) => println!("  {} -> valid (unexpected)", value),
            Err(e) => println!("  {} -> {}", value, e),
        }
    }

    // Example 7: Conversion between types
    println!("\n=== Type Conversions ===");
    let unit = UnitInterval::new(0.42).unwrap();
    let pct = unit.to_percentage();
    println!("UnitInterval {} -> Percentage {}%", unit.get(), pct.get());

    let pct2 = Percentage::new(67.5).unwrap();
    let unit2 = UnitInterval::from_percentage(*pct2.get()).unwrap();
    println!("Percentage {}% -> UnitInterval {}", pct2.get(), unit2.get());
}
