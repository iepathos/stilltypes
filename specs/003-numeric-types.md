---
number: 3
title: Numeric Types
category: foundation
priority: medium
status: draft
dependencies: []
created: 2025-12-22
---

# Specification 003: Numeric Types

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: none

## Philosophy Alignment

This specification follows the [Stillwater Philosophy](../../../stillwater/PHILOSOPHY.md):

- **Parse, Don't Validate** (§7): `Percentage` encodes the 0-100 invariant at the type level. Once constructed, it's guaranteed within bounds.
- **Errors Should Tell Stories** (§3): Errors use domain terminology ("percentage must be 0-100") not generic "out of range".
- **Composition Over Complexity** (§4): Simple types that compose with stillwater's validation.
- **Types Guide, Don't Restrict** (§5): `Percentage` clearly communicates intent vs raw `f64`.

## Context

Business applications frequently deal with numeric values that have semantic constraints beyond their raw type. Percentages, rates, probabilities, and other bounded numerics are common across domains like finance, statistics, e-commerce (discounts), and UI development (opacity, progress).

While stillwater provides `between(min, max)` for generic range validation, domain-specific types offer clearer APIs and better error messages. A `Percentage` type communicates intent better than `f64`, enables percentage-specific operations, and provides user-friendly error messages ("percentage must be 0-100").

## Objective

Add a `numeric` feature to stilltypes providing refined types for common bounded numeric values, starting with `Percentage`. The implementation should demonstrate a pattern that can be extended to other bounded numerics (rates, probabilities, etc.) in the future.

## Requirements

### Functional Requirements

1. **Percentage Type**
   - Accept `f64` values in range 0.0 to 100.0 (inclusive)
   - Reject NaN and infinity values
   - Provide conversion to decimal form: `to_decimal() -> f64` (e.g., 50% -> 0.5)
   - Provide construction from decimal: `from_decimal(f64) -> Result<Self, Error>` (e.g., 0.5 -> 50%)
   - Support common operations: `complement()` (100 - self), `of(value)` (calculate percentage of)

2. **UnitInterval Type (0.0 to 1.0)**
   - Accept `f64` values in range 0.0 to 1.0 (inclusive)
   - Common for probabilities, opacity, normalized values
   - Provide conversion to percentage: `to_percentage() -> Percentage`
   - Interoperability with `Percentage` type

3. **PositiveFloat / NonNegativeFloat (Optional)**
   - `PositiveFloat`: f64 > 0.0
   - `NonNegativeFloat`: f64 >= 0.0
   - Common constraints for amounts, distances, durations

### Non-Functional Requirements

- Zero external dependencies
- Serde support when `serde` feature is enabled
- Handle floating-point edge cases (NaN, infinity, negative zero)
- All predicates must be zero-sized types (ZSTs)
- Error messages should use domain terminology ("percentage", "probability")

## Acceptance Criteria

- [ ] `Percentage` type validates f64 values in range [0.0, 100.0]
- [ ] `UnitInterval` type validates f64 values in range [0.0, 1.0]
- [ ] Both types reject NaN and infinity with appropriate errors
- [ ] `PercentageExt` provides `to_decimal()`, `complement()`, `of()`
- [ ] `UnitIntervalExt` provides `to_percentage()`
- [ ] `Percentage::from_decimal()` constructs from 0.0-1.0 range
- [ ] Error messages use domain terminology (not just "out of range")
- [ ] Unit tests cover boundaries: 0.0, 50.0, 100.0 for percentage; 0.0, 0.5, 1.0 for unit interval
- [ ] Unit tests cover invalid values: negative, >100, NaN, infinity
- [ ] Conversion between Percentage and UnitInterval is accurate
- [ ] `examples/discount_validation.rs` demonstrates error accumulation pattern
- [ ] README.md feature table updated with numeric types
- [ ] lib.rs feature table updated with numeric types
- [ ] `full` feature includes `numeric`

## Technical Details

### Implementation Approach

```rust
// src/numeric.rs

use stillwater::refined::Refined;
use crate::error::{DomainError, DomainErrorKind};

/// Predicate for valid percentage values (0-100).
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidPercentage;

impl Predicate<f64> for ValidPercentage {
    type Error = DomainError;

    fn check(value: &f64) -> Result<(), Self::Error> {
        if value.is_nan() {
            return Err(DomainError {
                format_name: "percentage",
                value: "NaN".to_string(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "a valid number",
                },
                example: "50.0",
            });
        }

        if value.is_infinite() {
            return Err(DomainError {
                format_name: "percentage",
                value: "infinity".to_string(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "a finite number",
                },
                example: "50.0",
            });
        }

        if *value < 0.0 || *value > 100.0 {
            return Err(DomainError {
                format_name: "percentage",
                value: value.to_string(),
                reason: DomainErrorKind::InvalidComponent {
                    component: "value",
                    reason: format!("must be between 0 and 100, got {}", value),
                },
                example: "50.0",
            });
        }

        Ok(())
    }
}

/// A validated percentage value (0-100).
pub type Percentage = Refined<f64, ValidPercentage>;

/// Extension trait for percentage operations.
pub trait PercentageExt {
    /// Convert to decimal form (0.0 to 1.0).
    /// 50% -> 0.5
    fn to_decimal(&self) -> f64;

    /// Calculate the complement (100 - self).
    /// 30% -> 70%
    fn complement(&self) -> Percentage;

    /// Calculate this percentage of a value.
    /// 25%.of(200.0) -> 50.0
    fn of(&self, value: f64) -> f64;
}

impl PercentageExt for Percentage {
    fn to_decimal(&self) -> f64 {
        self.value() / 100.0
    }

    fn complement(&self) -> Percentage {
        // Safe because 100 - [0,100] is always in [0,100]
        Percentage::new(100.0 - self.value()).unwrap()
    }

    fn of(&self, value: f64) -> f64 {
        value * self.to_decimal()
    }
}

impl Percentage {
    /// Create a percentage from a decimal value (0.0 to 1.0).
    /// 0.5 -> 50%
    pub fn from_decimal(decimal: f64) -> Result<Self, DomainError> {
        Self::new(decimal * 100.0)
    }
}
```

### UnitInterval Implementation

```rust
/// Predicate for valid unit interval values (0-1).
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidUnitInterval;

impl Predicate<f64> for ValidUnitInterval {
    type Error = DomainError;

    fn check(value: &f64) -> Result<(), Self::Error> {
        // Similar to Percentage but with 0-1 range
        // Error messages use "probability" or "unit interval" terminology
    }
}

/// A validated unit interval value (0.0 to 1.0).
/// Common for probabilities, opacity, and normalized values.
pub type UnitInterval = Refined<f64, ValidUnitInterval>;
```

### Feature Flag

```toml
[features]
numeric = []  # No external dependencies
```

### Error Messages

```
"invalid percentage: -5.0 must be between 0 and 100 (example: 50.0)"
"invalid percentage: NaN is not a valid number (example: 50.0)"
"invalid unit interval: 1.5 must be between 0 and 1 (example: 0.5)"
"invalid probability: -0.1 must be between 0 and 1 (example: 0.75)"
```

## Error Accumulation Example

Following the "Fail Completely" pattern (PHILOSOPHY.md §2), numeric types integrate with `Validation::all()`:

```rust
use stilltypes::prelude::*;
use stilltypes::numeric::{Percentage, UnitInterval};
use stillwater::validation::{Validation, ValidateAll};

/// Raw discount configuration input.
struct DiscountInput {
    base_discount: f64,      // Expected 0-100
    member_bonus: f64,       // Expected 0-100
    probability: f64,        // Expected 0-1
}

/// Validated discount configuration.
struct ValidDiscount {
    base_discount: Percentage,
    member_bonus: Percentage,
    probability: UnitInterval,
}

fn validate_discount(input: DiscountInput) -> Validation<ValidDiscount, Vec<DomainError>> {
    let base_v = Validation::from_result(Percentage::new(input.base_discount).map_err(|e| vec![e]));
    let bonus_v = Validation::from_result(Percentage::new(input.member_bonus).map_err(|e| vec![e]));
    let prob_v = Validation::from_result(UnitInterval::new(input.probability).map_err(|e| vec![e]));

    (base_v, bonus_v, prob_v)
        .validate_all()
        .map(|(base_discount, member_bonus, probability)| ValidDiscount {
            base_discount,
            member_bonus,
            probability,
        })
}
```

## Pure Core Example

Once validated, numeric types enable pure business logic:

```rust
/// Pure function - calculates final price with validated discounts.
fn calculate_discounted_price(
    price: f64,
    base_discount: &Percentage,
    member_bonus: &Percentage,
) -> f64 {
    // No validation needed - types guarantee 0-100 range
    let total_discount = base_discount.get() + member_bonus.get();
    let capped = total_discount.min(100.0);  // Business rule: max 100% off
    price * (1.0 - capped / 100.0)
}

/// Pure function - applies probability-weighted scoring.
fn weighted_score(base_score: f64, weight: &UnitInterval) -> f64 {
    // Types guarantee weight is 0-1, safe to multiply
    base_score * weight.get()
}
```

## Dependencies

- **Prerequisites**: None
- **Affected Components**: `src/lib.rs`, `src/prelude.rs`, `Cargo.toml`
- **External Dependencies**: None

## Testing Strategy

- **Unit Tests**:
  - Valid percentages: 0.0, 25.0, 50.0, 75.0, 100.0
  - Invalid percentages: -0.1, 100.1, NaN, infinity
  - Valid unit intervals: 0.0, 0.25, 0.5, 0.75, 1.0
  - Invalid unit intervals: -0.01, 1.01, NaN, infinity
  - Conversion accuracy: `to_decimal()` and `from_decimal()` round-trip
  - Complement: 0% <-> 100%, 30% <-> 70%
  - `of()` calculation: 25%.of(200) == 50

- **Integration Tests**: Serde round-trip
- **Edge Cases**:
  - Exactly 0.0 and 100.0 (boundary values)
  - Negative zero (-0.0) should be treated as 0.0
  - Very small values near boundaries

## Documentation Requirements

### Code Documentation
- Full rustdoc with examples for each type and trait
- Module-level documentation explaining bounded numeric concepts

### lib.rs Feature Table Update
Add row to the feature table in `src/lib.rs`:
```markdown
//! | `numeric` | [`Percentage`](numeric::Percentage), [`UnitInterval`](numeric::UnitInterval) | - |
```

### README.md Updates
Add to feature table:
```markdown
| `numeric` | `Percentage`, `UnitInterval` | - |
```

Add usage section:
```markdown
### Bounded Numerics

\`\`\`rust,ignore
use stilltypes::numeric::{Percentage, UnitInterval, PercentageExt};

let discount = Percentage::new(25.0)?;
let price = 100.0;
let discounted = price - discount.of(price);  // 75.0

// Convert between representations
let probability = UnitInterval::new(0.75)?;
let as_percent = Percentage::from_decimal(0.75)?;  // 75%
\`\`\`
```

### Example File
Create `examples/discount_validation.rs`:
- Demonstrate discount/pricing calculations with validated percentages
- Show error accumulation for multiple numeric validations
- Include conversion between Percentage and UnitInterval
- Pattern after `examples/form_validation.rs`

## Implementation Notes

- Use `f64` for all numeric types for consistency
- Handle `-0.0` as equivalent to `0.0` (IEEE 754 semantics)
- Consider whether to provide arithmetic operations (Add, Sub, Mul, Div)
- The `complement()` method is infallible since input is already validated
- `from_decimal()` needs to validate the result, not just multiply

## Migration and Compatibility

- New feature, no breaking changes
- Optional feature flag means no impact on existing users

## Future Extensions

This specification establishes patterns for additional numeric types:
- `Rate` - positive float for rates (interest, exchange)
- `Score` - bounded integer for scores/ratings
- `Count` - non-negative integer
- `Money` - decimal with currency (would need external dep)
