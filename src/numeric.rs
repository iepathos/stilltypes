//! Bounded numeric types for common constraints.
//!
//! Provides validated types for percentages, unit intervals, and other
//! constrained numeric values. These types encode validity at the type level—
//! once you have a `Percentage`, it's guaranteed to be in the 0-100 range
//! with no runtime checks needed.
//!
//! # Philosophy
//!
//! This module follows the Stillwater philosophy:
//! - **Parse, Don't Validate**: Numeric types guarantee validity via the type system
//! - **Errors Should Tell Stories**: Rich `DomainError` messages with domain terminology
//! - **Composition Over Complexity**: Simple predicates that compose with stillwater
//! - **Types Guide, Don't Restrict**: `Percentage` communicates intent vs raw `f64`
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "numeric")]
//! # {
//! use stilltypes::numeric::{Percentage, UnitInterval, PercentageExt, UnitIntervalExt};
//!
//! // Percentage validates range 0 to 100
//! let discount = Percentage::new(25.0).unwrap();
//! assert_eq!(discount.to_decimal(), 0.25);
//!
//! // Calculate percentage of a value
//! let price = 100.0;
//! let savings = discount.of(price);  // 25.0
//! assert_eq!(savings, 25.0);
//!
//! // UnitInterval validates range 0 to 1
//! let probability = UnitInterval::new(0.75).unwrap();
//! let as_percent = probability.to_percentage();
//! assert_eq!(*as_percent.get(), 75.0);
//! # }
//! ```

use crate::error::{DomainError, DomainErrorKind};
use stillwater::refined::{Predicate, Refined};

// ============================================================================
// Percentage
// ============================================================================

/// Predicate for valid percentage values (0-100).
///
/// Validates percentage values in the range 0.0 to 100.0 (inclusive).
/// Rejects NaN and infinity values with appropriate error messages.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "numeric")]
/// # {
/// use stilltypes::numeric::Percentage;
///
/// let pct = Percentage::new(50.0);
/// assert!(pct.is_ok());
///
/// let invalid = Percentage::new(101.0);
/// assert!(invalid.is_err());
/// # }
/// ```
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
                    expected: "a valid number between 0 and 100",
                },
                example: "50.0",
            });
        }

        if value.is_infinite() {
            return Err(DomainError {
                format_name: "percentage",
                value: if value.is_sign_positive() {
                    "infinity"
                } else {
                    "-infinity"
                }
                .to_string(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "a finite number between 0 and 100",
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

    fn description() -> &'static str {
        "percentage (0 to 100)"
    }
}

/// A validated percentage value (0-100).
///
/// An `f64` that has been validated to be in the valid percentage range (0.0 to 100.0).
///
/// # Example
///
/// ```
/// # #[cfg(feature = "numeric")]
/// # {
/// use stilltypes::numeric::{Percentage, PercentageExt};
///
/// let discount = Percentage::new(25.0).unwrap();
/// assert_eq!(*discount.get(), 25.0);
/// assert_eq!(discount.to_decimal(), 0.25);
///
/// // Calculate 25% of 200
/// assert_eq!(discount.of(200.0), 50.0);
/// # }
/// ```
pub type Percentage = Refined<f64, ValidPercentage>;

/// Extension trait for percentage operations.
///
/// Provides semantic helpers for working with validated percentage values.
pub trait PercentageExt {
    /// Convert to decimal form (0.0 to 1.0).
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "numeric")]
    /// # {
    /// use stilltypes::numeric::{Percentage, PercentageExt};
    ///
    /// let pct = Percentage::new(50.0).unwrap();
    /// assert_eq!(pct.to_decimal(), 0.5);
    /// # }
    /// ```
    fn to_decimal(&self) -> f64;

    /// Calculate the complement (100 - self).
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "numeric")]
    /// # {
    /// use stilltypes::numeric::{Percentage, PercentageExt};
    ///
    /// let discount = Percentage::new(30.0).unwrap();
    /// let remaining = discount.complement();
    /// assert_eq!(*remaining.get(), 70.0);
    /// # }
    /// ```
    fn complement(&self) -> Percentage;

    /// Calculate this percentage of a value.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "numeric")]
    /// # {
    /// use stilltypes::numeric::{Percentage, PercentageExt};
    ///
    /// let tax_rate = Percentage::new(10.0).unwrap();
    /// let tax = tax_rate.of(50.0);
    /// assert_eq!(tax, 5.0);
    /// # }
    /// ```
    fn of(&self, value: f64) -> f64;

    /// Create a percentage from a decimal value (0.0 to 1.0).
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "numeric")]
    /// # {
    /// use stilltypes::numeric::{Percentage, PercentageExt};
    ///
    /// let pct = Percentage::from_decimal(0.5).unwrap();
    /// assert_eq!(*pct.get(), 50.0);
    ///
    /// // Values outside 0-1 fail
    /// let invalid = Percentage::from_decimal(1.5);
    /// assert!(invalid.is_err());
    /// # }
    /// ```
    fn from_decimal(decimal: f64) -> Result<Percentage, DomainError>;
}

impl PercentageExt for Percentage {
    fn to_decimal(&self) -> f64 {
        self.get() / 100.0
    }

    fn complement(&self) -> Percentage {
        // Safe because 100 - [0,100] is always in [0,100]
        Percentage::new(100.0 - self.get()).unwrap()
    }

    fn of(&self, value: f64) -> f64 {
        value * self.to_decimal()
    }

    fn from_decimal(decimal: f64) -> Result<Percentage, DomainError> {
        Percentage::new(decimal * 100.0)
    }
}

// ============================================================================
// UnitInterval
// ============================================================================

/// Predicate for valid unit interval values (0-1).
///
/// Validates values in the range 0.0 to 1.0 (inclusive).
/// Rejects NaN and infinity values with appropriate error messages.
/// Common for probabilities, opacity, and normalized values.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "numeric")]
/// # {
/// use stilltypes::numeric::UnitInterval;
///
/// let prob = UnitInterval::new(0.75);
/// assert!(prob.is_ok());
///
/// let invalid = UnitInterval::new(1.5);
/// assert!(invalid.is_err());
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidUnitInterval;

impl Predicate<f64> for ValidUnitInterval {
    type Error = DomainError;

    fn check(value: &f64) -> Result<(), Self::Error> {
        if value.is_nan() {
            return Err(DomainError {
                format_name: "unit interval",
                value: "NaN".to_string(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "a valid number between 0 and 1",
                },
                example: "0.5",
            });
        }

        if value.is_infinite() {
            return Err(DomainError {
                format_name: "unit interval",
                value: if value.is_sign_positive() {
                    "infinity"
                } else {
                    "-infinity"
                }
                .to_string(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "a finite number between 0 and 1",
                },
                example: "0.5",
            });
        }

        if *value < 0.0 || *value > 1.0 {
            return Err(DomainError {
                format_name: "unit interval",
                value: value.to_string(),
                reason: DomainErrorKind::InvalidComponent {
                    component: "value",
                    reason: format!("must be between 0 and 1, got {}", value),
                },
                example: "0.5",
            });
        }

        Ok(())
    }

    fn description() -> &'static str {
        "unit interval (0 to 1)"
    }
}

/// A validated unit interval value (0.0 to 1.0).
///
/// An `f64` that has been validated to be in the valid unit interval range.
/// Common for probabilities, opacity, and normalized values.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "numeric")]
/// # {
/// use stilltypes::numeric::{UnitInterval, UnitIntervalExt};
///
/// let probability = UnitInterval::new(0.75).unwrap();
/// assert_eq!(*probability.get(), 0.75);
///
/// // Convert to percentage
/// let as_percent = probability.to_percentage();
/// assert_eq!(*as_percent.get(), 75.0);
/// # }
/// ```
pub type UnitInterval = Refined<f64, ValidUnitInterval>;

/// Extension trait for unit interval operations.
///
/// Provides semantic helpers for working with validated unit interval values.
pub trait UnitIntervalExt {
    /// Convert to a Percentage (0 to 100).
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "numeric")]
    /// # {
    /// use stilltypes::numeric::{UnitInterval, UnitIntervalExt};
    ///
    /// let unit = UnitInterval::new(0.75).unwrap();
    /// let pct = unit.to_percentage();
    /// assert_eq!(*pct.get(), 75.0);
    /// # }
    /// ```
    fn to_percentage(&self) -> Percentage;

    /// Calculate the complement (1 - self).
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "numeric")]
    /// # {
    /// use stilltypes::numeric::{UnitInterval, UnitIntervalExt};
    ///
    /// let prob = UnitInterval::new(0.3).unwrap();
    /// let complement = prob.complement();
    /// assert!((complement.get() - 0.7).abs() < 0.0001);
    /// # }
    /// ```
    fn complement(&self) -> UnitInterval;

    /// Scale a value by this unit interval.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "numeric")]
    /// # {
    /// use stilltypes::numeric::{UnitInterval, UnitIntervalExt};
    ///
    /// let opacity = UnitInterval::new(0.5).unwrap();
    /// let scaled = opacity.scale(255.0);
    /// assert_eq!(scaled, 127.5);
    /// # }
    /// ```
    fn scale(&self, value: f64) -> f64;

    /// Create a unit interval from a percentage (0-100).
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "numeric")]
    /// # {
    /// use stilltypes::numeric::{UnitInterval, UnitIntervalExt};
    ///
    /// let unit = UnitInterval::from_percentage(75.0).unwrap();
    /// assert_eq!(*unit.get(), 0.75);
    ///
    /// // Values outside 0-100 fail
    /// let invalid = UnitInterval::from_percentage(150.0);
    /// assert!(invalid.is_err());
    /// # }
    /// ```
    fn from_percentage(percentage: f64) -> Result<UnitInterval, DomainError>;
}

impl UnitIntervalExt for UnitInterval {
    fn to_percentage(&self) -> Percentage {
        // Safe because [0,1] * 100 is always in [0,100]
        Percentage::new(self.get() * 100.0).unwrap()
    }

    fn complement(&self) -> UnitInterval {
        // Safe because 1 - [0,1] is always in [0,1]
        UnitInterval::new(1.0 - self.get()).unwrap()
    }

    fn scale(&self, value: f64) -> f64 {
        value * self.get()
    }

    fn from_percentage(percentage: f64) -> Result<UnitInterval, DomainError> {
        UnitInterval::new(percentage / 100.0)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Percentage Tests
    // ========================================================================

    mod percentage_tests {
        use super::*;

        // Valid cases
        #[test]
        fn valid_zero() {
            assert!(Percentage::new(0.0).is_ok());
        }

        #[test]
        fn valid_hundred() {
            assert!(Percentage::new(100.0).is_ok());
        }

        #[test]
        fn valid_mid_values() {
            assert!(Percentage::new(25.0).is_ok());
            assert!(Percentage::new(50.0).is_ok());
            assert!(Percentage::new(75.0).is_ok());
        }

        #[test]
        fn valid_decimal_values() {
            assert!(Percentage::new(33.33).is_ok());
            assert!(Percentage::new(0.01).is_ok());
            assert!(Percentage::new(99.99).is_ok());
        }

        #[test]
        fn valid_negative_zero() {
            // -0.0 should be treated as 0.0
            let result = Percentage::new(-0.0);
            assert!(result.is_ok());
            // IEEE 754: -0.0 == 0.0
            assert!(*result.unwrap().get() >= 0.0);
        }

        // Invalid cases
        #[test]
        fn invalid_negative() {
            let result = Percentage::new(-0.1);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.format_name, "percentage");
            assert!(matches!(
                err.reason,
                DomainErrorKind::InvalidComponent { .. }
            ));
        }

        #[test]
        fn invalid_over_hundred() {
            let result = Percentage::new(100.1);
            assert!(result.is_err());
        }

        #[test]
        fn invalid_nan() {
            let result = Percentage::new(f64::NAN);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.value, "NaN");
            assert!(matches!(err.reason, DomainErrorKind::InvalidFormat { .. }));
        }

        #[test]
        fn invalid_positive_infinity() {
            let result = Percentage::new(f64::INFINITY);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.value, "infinity");
        }

        #[test]
        fn invalid_negative_infinity() {
            let result = Percentage::new(f64::NEG_INFINITY);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.value, "-infinity");
        }

        // Extension trait tests
        #[test]
        fn ext_to_decimal() {
            let pct = Percentage::new(50.0).unwrap();
            assert_eq!(pct.to_decimal(), 0.5);

            let zero = Percentage::new(0.0).unwrap();
            assert_eq!(zero.to_decimal(), 0.0);

            let hundred = Percentage::new(100.0).unwrap();
            assert_eq!(hundred.to_decimal(), 1.0);
        }

        #[test]
        fn ext_complement() {
            let pct = Percentage::new(30.0).unwrap();
            let comp = pct.complement();
            assert_eq!(*comp.get(), 70.0);

            let zero = Percentage::new(0.0).unwrap();
            assert_eq!(*zero.complement().get(), 100.0);

            let hundred = Percentage::new(100.0).unwrap();
            assert_eq!(*hundred.complement().get(), 0.0);
        }

        #[test]
        fn ext_of() {
            let pct = Percentage::new(25.0).unwrap();
            assert_eq!(pct.of(200.0), 50.0);

            let ten = Percentage::new(10.0).unwrap();
            assert_eq!(ten.of(50.0), 5.0);
        }

        #[test]
        fn from_decimal_valid() {
            let pct = Percentage::from_decimal(0.5).unwrap();
            assert_eq!(*pct.get(), 50.0);

            let zero = Percentage::from_decimal(0.0).unwrap();
            assert_eq!(*zero.get(), 0.0);

            let one = Percentage::from_decimal(1.0).unwrap();
            assert_eq!(*one.get(), 100.0);
        }

        #[test]
        fn from_decimal_invalid() {
            assert!(Percentage::from_decimal(1.5).is_err());
            assert!(Percentage::from_decimal(-0.1).is_err());
        }

        #[test]
        fn description_returns_expected() {
            assert_eq!(ValidPercentage::description(), "percentage (0 to 100)");
        }

        #[test]
        fn error_includes_example() {
            let result = Percentage::new(150.0);
            let err = result.unwrap_err();
            assert_eq!(err.example, "50.0");
        }
    }

    // ========================================================================
    // UnitInterval Tests
    // ========================================================================

    mod unit_interval_tests {
        use super::*;

        // Valid cases
        #[test]
        fn valid_zero() {
            assert!(UnitInterval::new(0.0).is_ok());
        }

        #[test]
        fn valid_one() {
            assert!(UnitInterval::new(1.0).is_ok());
        }

        #[test]
        fn valid_mid_values() {
            assert!(UnitInterval::new(0.25).is_ok());
            assert!(UnitInterval::new(0.5).is_ok());
            assert!(UnitInterval::new(0.75).is_ok());
        }

        #[test]
        fn valid_small_values() {
            assert!(UnitInterval::new(0.001).is_ok());
            assert!(UnitInterval::new(0.999).is_ok());
        }

        #[test]
        fn valid_negative_zero() {
            let result = UnitInterval::new(-0.0);
            assert!(result.is_ok());
        }

        // Invalid cases
        #[test]
        fn invalid_negative() {
            let result = UnitInterval::new(-0.01);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.format_name, "unit interval");
        }

        #[test]
        fn invalid_over_one() {
            let result = UnitInterval::new(1.01);
            assert!(result.is_err());
        }

        #[test]
        fn invalid_nan() {
            let result = UnitInterval::new(f64::NAN);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.value, "NaN");
        }

        #[test]
        fn invalid_positive_infinity() {
            let result = UnitInterval::new(f64::INFINITY);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.value, "infinity");
        }

        #[test]
        fn invalid_negative_infinity() {
            let result = UnitInterval::new(f64::NEG_INFINITY);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.value, "-infinity");
        }

        // Extension trait tests
        #[test]
        fn ext_to_percentage() {
            let unit = UnitInterval::new(0.75).unwrap();
            let pct = unit.to_percentage();
            assert_eq!(*pct.get(), 75.0);

            let zero = UnitInterval::new(0.0).unwrap();
            assert_eq!(*zero.to_percentage().get(), 0.0);

            let one = UnitInterval::new(1.0).unwrap();
            assert_eq!(*one.to_percentage().get(), 100.0);
        }

        #[test]
        fn ext_complement() {
            let unit = UnitInterval::new(0.3).unwrap();
            let comp = unit.complement();
            assert!((comp.get() - 0.7).abs() < 0.0001);

            let zero = UnitInterval::new(0.0).unwrap();
            assert_eq!(*zero.complement().get(), 1.0);

            let one = UnitInterval::new(1.0).unwrap();
            assert_eq!(*one.complement().get(), 0.0);
        }

        #[test]
        fn ext_scale() {
            let unit = UnitInterval::new(0.5).unwrap();
            assert_eq!(unit.scale(100.0), 50.0);

            let opacity = UnitInterval::new(0.8).unwrap();
            assert_eq!(opacity.scale(255.0), 204.0);
        }

        #[test]
        fn from_percentage_valid() {
            let unit = UnitInterval::from_percentage(75.0).unwrap();
            assert_eq!(*unit.get(), 0.75);

            let zero = UnitInterval::from_percentage(0.0).unwrap();
            assert_eq!(*zero.get(), 0.0);

            let hundred = UnitInterval::from_percentage(100.0).unwrap();
            assert_eq!(*hundred.get(), 1.0);
        }

        #[test]
        fn from_percentage_invalid() {
            assert!(UnitInterval::from_percentage(150.0).is_err());
            assert!(UnitInterval::from_percentage(-10.0).is_err());
        }

        #[test]
        fn description_returns_expected() {
            assert_eq!(ValidUnitInterval::description(), "unit interval (0 to 1)");
        }

        #[test]
        fn error_includes_example() {
            let result = UnitInterval::new(1.5);
            let err = result.unwrap_err();
            assert_eq!(err.example, "0.5");
        }
    }

    // ========================================================================
    // Conversion Tests
    // ========================================================================

    mod conversion_tests {
        use super::*;

        #[test]
        fn percentage_to_unit_interval_roundtrip() {
            let original = 75.0;
            let pct = Percentage::new(original).unwrap();
            let unit = UnitInterval::new(pct.to_decimal()).unwrap();
            let back = unit.to_percentage();
            assert_eq!(*back.get(), original);
        }

        #[test]
        fn unit_interval_to_percentage_roundtrip() {
            let original = 0.25;
            let unit = UnitInterval::new(original).unwrap();
            let pct = unit.to_percentage();
            let back = Percentage::from_decimal(pct.to_decimal()).unwrap();
            assert!((back.get() - (original * 100.0)).abs() < 0.0001);
        }

        #[test]
        fn from_decimal_to_decimal_roundtrip() {
            let original = 0.333;
            let pct = Percentage::from_decimal(original).unwrap();
            let back = pct.to_decimal();
            assert!((back - original).abs() < 0.0001);
        }

        #[test]
        fn from_percentage_roundtrip() {
            let original = 42.5;
            let unit = UnitInterval::from_percentage(original).unwrap();
            let back = unit.to_percentage();
            assert!((back.get() - original).abs() < 0.0001);
        }
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    mod edge_case_tests {
        use super::*;

        #[test]
        fn boundary_values_percentage() {
            assert!(Percentage::new(0.0).is_ok());
            assert!(Percentage::new(50.0).is_ok());
            assert!(Percentage::new(100.0).is_ok());
        }

        #[test]
        fn just_outside_percentage_bounds() {
            // Just below 0
            assert!(Percentage::new(-0.0001).is_err());
            // Just above 100
            assert!(Percentage::new(100.0001).is_err());
        }

        #[test]
        fn boundary_values_unit_interval() {
            assert!(UnitInterval::new(0.0).is_ok());
            assert!(UnitInterval::new(0.5).is_ok());
            assert!(UnitInterval::new(1.0).is_ok());
        }

        #[test]
        fn just_outside_unit_interval_bounds() {
            // Just below 0
            assert!(UnitInterval::new(-0.0001).is_err());
            // Just above 1
            assert!(UnitInterval::new(1.0001).is_err());
        }

        #[test]
        fn very_small_valid_values() {
            assert!(Percentage::new(0.0000001).is_ok());
            assert!(UnitInterval::new(0.0000001).is_ok());
        }

        #[test]
        fn complement_at_boundaries() {
            // 0% complement = 100%
            let zero = Percentage::new(0.0).unwrap();
            assert_eq!(*zero.complement().get(), 100.0);

            // 100% complement = 0%
            let hundred = Percentage::new(100.0).unwrap();
            assert_eq!(*hundred.complement().get(), 0.0);

            // 0.0 complement = 1.0
            let zero_unit = UnitInterval::new(0.0).unwrap();
            assert_eq!(*zero_unit.complement().get(), 1.0);

            // 1.0 complement = 0.0
            let one_unit = UnitInterval::new(1.0).unwrap();
            assert_eq!(*one_unit.complement().get(), 0.0);
        }

        #[test]
        fn of_with_edge_values() {
            let zero_pct = Percentage::new(0.0).unwrap();
            assert_eq!(zero_pct.of(100.0), 0.0);

            let hundred_pct = Percentage::new(100.0).unwrap();
            assert_eq!(hundred_pct.of(100.0), 100.0);

            // Test with negative value (percentage stays valid)
            let half = Percentage::new(50.0).unwrap();
            assert_eq!(half.of(-100.0), -50.0);
        }

        #[test]
        fn scale_with_edge_values() {
            let zero_unit = UnitInterval::new(0.0).unwrap();
            assert_eq!(zero_unit.scale(255.0), 0.0);

            let one_unit = UnitInterval::new(1.0).unwrap();
            assert_eq!(one_unit.scale(255.0), 255.0);
        }
    }
}
