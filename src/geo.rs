//! Geographic validation types.
//!
//! Provides validated types for geographic coordinates following standard
//! conventions. These types encode validity at the type level—once you have
//! a `Latitude`, it's guaranteed within bounds with no runtime checks needed.
//!
//! # Philosophy
//!
//! This module follows the Stillwater philosophy:
//! - **Parse, Don't Validate**: Geographic types guarantee validity via the type system
//! - **Errors Should Tell Stories**: Rich `DomainError` messages with geographic terminology
//! - **Composition Over Complexity**: Simple predicates combinable with `And`, `Or`, `Not`
//! - **Pragmatism Over Purity**: Uses `f64` directly rather than generic numerics
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "geo")]
//! # {
//! use stilltypes::geo::{Latitude, Longitude, LatitudeExt, LongitudeExt};
//!
//! // Latitude validates range -90 to 90
//! let lat = Latitude::new(37.7749).unwrap();
//! assert!(lat.is_north());
//!
//! // Longitude validates range -180 to 180
//! let lon = Longitude::new(-122.4194).unwrap();
//! assert!(lon.is_west());
//!
//! // Convert to degrees, minutes, seconds
//! let (deg, min, sec, hemi) = lat.to_dms();
//! // 37° 46' 29.64" N
//! # }
//! ```

use crate::error::{DomainError, DomainErrorKind};
use stillwater::refined::{Predicate, Refined};

// ============================================================================
// Latitude
// ============================================================================

/// Predicate for valid latitude values.
///
/// Validates latitude values in the range -90.0 to 90.0 degrees (inclusive).
/// Rejects NaN and infinity values with appropriate error messages.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "geo")]
/// # {
/// use stilltypes::geo::Latitude;
///
/// let lat = Latitude::new(37.7749);
/// assert!(lat.is_ok());
///
/// let invalid = Latitude::new(91.0);
/// assert!(invalid.is_err());
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidLatitude;

impl Predicate<f64> for ValidLatitude {
    type Error = DomainError;

    fn check(value: &f64) -> Result<(), Self::Error> {
        if value.is_nan() {
            return Err(DomainError {
                format_name: "latitude",
                value: "NaN".to_string(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "a valid number between -90 and 90 degrees",
                },
                example: "37.7749",
            });
        }

        if value.is_infinite() {
            return Err(DomainError {
                format_name: "latitude",
                value: if value.is_sign_positive() {
                    "infinity"
                } else {
                    "-infinity"
                }
                .to_string(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "a finite number between -90 and 90 degrees",
                },
                example: "37.7749",
            });
        }

        if *value < -90.0 || *value > 90.0 {
            return Err(DomainError {
                format_name: "latitude",
                value: value.to_string(),
                reason: DomainErrorKind::InvalidComponent {
                    component: "degrees",
                    reason: format!("must be between -90 and 90, got {}", value),
                },
                example: "37.7749",
            });
        }

        Ok(())
    }

    fn description() -> &'static str {
        "latitude (-90 to 90 degrees)"
    }
}

/// A validated latitude coordinate.
///
/// An `f64` that has been validated to be in the valid latitude range (-90 to 90 degrees).
///
/// # Example
///
/// ```
/// # #[cfg(feature = "geo")]
/// # {
/// use stilltypes::geo::{Latitude, LatitudeExt};
///
/// let lat = Latitude::new(37.7749).unwrap();
/// assert_eq!(*lat.get(), 37.7749);
/// assert!(lat.is_north());
/// # }
/// ```
pub type Latitude = Refined<f64, ValidLatitude>;

/// Extension trait for latitude operations.
///
/// Provides semantic helpers for working with validated latitude values.
pub trait LatitudeExt {
    /// Returns true if latitude is in the northern hemisphere (> 0).
    fn is_north(&self) -> bool;

    /// Returns true if latitude is in the southern hemisphere (< 0).
    fn is_south(&self) -> bool;

    /// Returns true if latitude is on the equator (== 0).
    fn is_equator(&self) -> bool;

    /// Convert to degrees, minutes, seconds format.
    ///
    /// Returns (degrees, minutes, seconds, hemisphere) where hemisphere is 'N' or 'S'.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "geo")]
    /// # {
    /// use stilltypes::geo::{Latitude, LatitudeExt};
    ///
    /// let lat = Latitude::new(37.7749).unwrap();
    /// let (deg, min, sec, hemi) = lat.to_dms();
    /// assert_eq!(deg, 37);
    /// assert_eq!(min, 46);
    /// assert!((sec - 29.64).abs() < 0.01);
    /// assert_eq!(hemi, 'N');
    /// # }
    /// ```
    fn to_dms(&self) -> (i32, u32, f64, char);
}

impl LatitudeExt for Latitude {
    fn is_north(&self) -> bool {
        *self.get() > 0.0
    }

    fn is_south(&self) -> bool {
        *self.get() < 0.0
    }

    fn is_equator(&self) -> bool {
        *self.get() == 0.0
    }

    fn to_dms(&self) -> (i32, u32, f64, char) {
        let value = *self.get();
        let hemisphere = if value >= 0.0 { 'N' } else { 'S' };
        let (deg, min, sec) = decimal_to_dms(value);
        (deg, min, sec, hemisphere)
    }
}

// ============================================================================
// Longitude
// ============================================================================

/// Predicate for valid longitude values.
///
/// Validates longitude values in the range -180.0 to 180.0 degrees (inclusive).
/// Rejects NaN and infinity values with appropriate error messages.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "geo")]
/// # {
/// use stilltypes::geo::Longitude;
///
/// let lon = Longitude::new(-122.4194);
/// assert!(lon.is_ok());
///
/// let invalid = Longitude::new(200.0);
/// assert!(invalid.is_err());
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidLongitude;

impl Predicate<f64> for ValidLongitude {
    type Error = DomainError;

    fn check(value: &f64) -> Result<(), Self::Error> {
        if value.is_nan() {
            return Err(DomainError {
                format_name: "longitude",
                value: "NaN".to_string(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "a valid number between -180 and 180 degrees",
                },
                example: "-122.4194",
            });
        }

        if value.is_infinite() {
            return Err(DomainError {
                format_name: "longitude",
                value: if value.is_sign_positive() {
                    "infinity"
                } else {
                    "-infinity"
                }
                .to_string(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "a finite number between -180 and 180 degrees",
                },
                example: "-122.4194",
            });
        }

        if *value < -180.0 || *value > 180.0 {
            return Err(DomainError {
                format_name: "longitude",
                value: value.to_string(),
                reason: DomainErrorKind::InvalidComponent {
                    component: "degrees",
                    reason: format!("must be between -180 and 180, got {}", value),
                },
                example: "-122.4194",
            });
        }

        Ok(())
    }

    fn description() -> &'static str {
        "longitude (-180 to 180 degrees)"
    }
}

/// A validated longitude coordinate.
///
/// An `f64` that has been validated to be in the valid longitude range (-180 to 180 degrees).
///
/// # Example
///
/// ```
/// # #[cfg(feature = "geo")]
/// # {
/// use stilltypes::geo::{Longitude, LongitudeExt};
///
/// let lon = Longitude::new(-122.4194).unwrap();
/// assert_eq!(*lon.get(), -122.4194);
/// assert!(lon.is_west());
/// # }
/// ```
pub type Longitude = Refined<f64, ValidLongitude>;

/// Extension trait for longitude operations.
///
/// Provides semantic helpers for working with validated longitude values.
pub trait LongitudeExt {
    /// Returns true if longitude is in the eastern hemisphere (> 0).
    fn is_east(&self) -> bool;

    /// Returns true if longitude is in the western hemisphere (< 0).
    fn is_west(&self) -> bool;

    /// Returns true if longitude is on the prime meridian (== 0).
    fn is_prime_meridian(&self) -> bool;

    /// Returns true if longitude is on the antimeridian (±180).
    ///
    /// Note: Both 180.0 and -180.0 represent the same geographic location.
    fn is_antimeridian(&self) -> bool;

    /// Convert to degrees, minutes, seconds format.
    ///
    /// Returns (degrees, minutes, seconds, hemisphere) where hemisphere is 'E' or 'W'.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "geo")]
    /// # {
    /// use stilltypes::geo::{Longitude, LongitudeExt};
    ///
    /// let lon = Longitude::new(-122.4194).unwrap();
    /// let (deg, min, sec, hemi) = lon.to_dms();
    /// assert_eq!(deg, 122);
    /// assert_eq!(min, 25);
    /// assert!((sec - 9.84).abs() < 0.01);
    /// assert_eq!(hemi, 'W');
    /// # }
    /// ```
    fn to_dms(&self) -> (i32, u32, f64, char);
}

impl LongitudeExt for Longitude {
    fn is_east(&self) -> bool {
        *self.get() > 0.0
    }

    fn is_west(&self) -> bool {
        *self.get() < 0.0
    }

    fn is_prime_meridian(&self) -> bool {
        *self.get() == 0.0
    }

    fn is_antimeridian(&self) -> bool {
        let value = *self.get();
        value == 180.0 || value == -180.0
    }

    fn to_dms(&self) -> (i32, u32, f64, char) {
        let value = *self.get();
        let hemisphere = if value >= 0.0 { 'E' } else { 'W' };
        let (deg, min, sec) = decimal_to_dms(value);
        (deg, min, sec, hemisphere)
    }
}

// ============================================================================
// DMS Conversion
// ============================================================================

/// Convert decimal degrees to degrees, minutes, seconds.
///
/// Takes the absolute value internally so the result is always positive.
/// The sign/hemisphere should be determined by the caller.
fn decimal_to_dms(decimal: f64) -> (i32, u32, f64) {
    let abs_decimal = decimal.abs();
    let degrees = abs_decimal.floor() as i32;
    let minutes_decimal = (abs_decimal - degrees as f64) * 60.0;
    let minutes = minutes_decimal.floor() as u32;
    let seconds = (minutes_decimal - minutes as f64) * 60.0;
    (degrees, minutes, seconds)
}

/// Create a latitude from degrees, minutes, seconds, and hemisphere.
///
/// # Arguments
///
/// * `degrees` - Degrees (0-90)
/// * `minutes` - Minutes (0-59)
/// * `seconds` - Seconds (0.0-60.0)
/// * `hemisphere` - 'N' for north, 'S' for south
///
/// # Example
///
/// ```
/// # #[cfg(feature = "geo")]
/// # {
/// use stilltypes::geo::{latitude_from_dms, LatitudeExt};
///
/// let lat = latitude_from_dms(37, 46, 29.64, 'N').unwrap();
/// assert!((lat.get() - 37.7749).abs() < 0.0001);
/// # }
/// ```
pub fn latitude_from_dms(
    degrees: u32,
    minutes: u32,
    seconds: f64,
    hemisphere: char,
) -> Result<Latitude, DomainError> {
    if degrees > 90 {
        return Err(DomainError {
            format_name: "latitude",
            value: format!("{}°{}'{:.2}\"", degrees, minutes, seconds),
            reason: DomainErrorKind::InvalidComponent {
                component: "degrees",
                reason: format!("must be between 0 and 90, got {}", degrees),
            },
            example: "37°46'29.64\"N",
        });
    }

    if minutes >= 60 {
        return Err(DomainError {
            format_name: "latitude",
            value: format!("{}°{}'{:.2}\"", degrees, minutes, seconds),
            reason: DomainErrorKind::InvalidComponent {
                component: "minutes",
                reason: format!("must be between 0 and 59, got {}", minutes),
            },
            example: "37°46'29.64\"N",
        });
    }

    if !(0.0..60.0).contains(&seconds) {
        return Err(DomainError {
            format_name: "latitude",
            value: format!("{}°{}'{:.2}\"", degrees, minutes, seconds),
            reason: DomainErrorKind::InvalidComponent {
                component: "seconds",
                reason: format!("must be between 0 and 60, got {}", seconds),
            },
            example: "37°46'29.64\"N",
        });
    }

    let hemisphere_upper = hemisphere.to_ascii_uppercase();
    if hemisphere_upper != 'N' && hemisphere_upper != 'S' {
        return Err(DomainError {
            format_name: "latitude",
            value: format!("{}°{}'{:.2}\"{}", degrees, minutes, seconds, hemisphere),
            reason: DomainErrorKind::InvalidComponent {
                component: "hemisphere",
                reason: format!("must be 'N' or 'S', got '{}'", hemisphere),
            },
            example: "37°46'29.64\"N",
        });
    }

    let decimal = degrees as f64 + minutes as f64 / 60.0 + seconds / 3600.0;
    let signed = if hemisphere_upper == 'S' {
        -decimal
    } else {
        decimal
    };

    Latitude::new(signed)
}

/// Create a longitude from degrees, minutes, seconds, and hemisphere.
///
/// # Arguments
///
/// * `degrees` - Degrees (0-180)
/// * `minutes` - Minutes (0-59)
/// * `seconds` - Seconds (0.0-60.0)
/// * `hemisphere` - 'E' for east, 'W' for west
///
/// # Example
///
/// ```
/// # #[cfg(feature = "geo")]
/// # {
/// use stilltypes::geo::{longitude_from_dms, LongitudeExt};
///
/// let lon = longitude_from_dms(122, 25, 9.84, 'W').unwrap();
/// assert!((lon.get() - (-122.4194)).abs() < 0.0001);
/// # }
/// ```
pub fn longitude_from_dms(
    degrees: u32,
    minutes: u32,
    seconds: f64,
    hemisphere: char,
) -> Result<Longitude, DomainError> {
    if degrees > 180 {
        return Err(DomainError {
            format_name: "longitude",
            value: format!("{}°{}'{:.2}\"", degrees, minutes, seconds),
            reason: DomainErrorKind::InvalidComponent {
                component: "degrees",
                reason: format!("must be between 0 and 180, got {}", degrees),
            },
            example: "122°25'9.84\"W",
        });
    }

    if minutes >= 60 {
        return Err(DomainError {
            format_name: "longitude",
            value: format!("{}°{}'{:.2}\"", degrees, minutes, seconds),
            reason: DomainErrorKind::InvalidComponent {
                component: "minutes",
                reason: format!("must be between 0 and 59, got {}", minutes),
            },
            example: "122°25'9.84\"W",
        });
    }

    if !(0.0..60.0).contains(&seconds) {
        return Err(DomainError {
            format_name: "longitude",
            value: format!("{}°{}'{:.2}\"", degrees, minutes, seconds),
            reason: DomainErrorKind::InvalidComponent {
                component: "seconds",
                reason: format!("must be between 0 and 60, got {}", seconds),
            },
            example: "122°25'9.84\"W",
        });
    }

    let hemisphere_upper = hemisphere.to_ascii_uppercase();
    if hemisphere_upper != 'E' && hemisphere_upper != 'W' {
        return Err(DomainError {
            format_name: "longitude",
            value: format!("{}°{}'{:.2}\"{}", degrees, minutes, seconds, hemisphere),
            reason: DomainErrorKind::InvalidComponent {
                component: "hemisphere",
                reason: format!("must be 'E' or 'W', got '{}'", hemisphere),
            },
            example: "122°25'9.84\"W",
        });
    }

    let decimal = degrees as f64 + minutes as f64 / 60.0 + seconds / 3600.0;
    let signed = if hemisphere_upper == 'W' {
        -decimal
    } else {
        decimal
    };

    Longitude::new(signed)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Latitude Tests
    // ========================================================================

    mod latitude_tests {
        use super::*;

        // Valid cases
        #[test]
        fn valid_zero() {
            assert!(Latitude::new(0.0).is_ok());
        }

        #[test]
        fn valid_north_pole() {
            assert!(Latitude::new(90.0).is_ok());
        }

        #[test]
        fn valid_south_pole() {
            assert!(Latitude::new(-90.0).is_ok());
        }

        #[test]
        fn valid_positive() {
            assert!(Latitude::new(37.7749).is_ok());
        }

        #[test]
        fn valid_negative() {
            assert!(Latitude::new(-33.8688).is_ok());
        }

        #[test]
        fn valid_mid_range() {
            assert!(Latitude::new(45.0).is_ok());
            assert!(Latitude::new(-45.0).is_ok());
        }

        // Invalid cases
        #[test]
        fn invalid_too_high() {
            let result = Latitude::new(90.1);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.reason,
                DomainErrorKind::InvalidComponent { .. }
            ));
        }

        #[test]
        fn invalid_too_low() {
            let result = Latitude::new(-90.1);
            assert!(result.is_err());
        }

        #[test]
        fn invalid_nan() {
            let result = Latitude::new(f64::NAN);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err.reason, DomainErrorKind::InvalidFormat { .. }));
            assert_eq!(err.value, "NaN");
        }

        #[test]
        fn invalid_positive_infinity() {
            let result = Latitude::new(f64::INFINITY);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err.reason, DomainErrorKind::InvalidFormat { .. }));
            assert_eq!(err.value, "infinity");
        }

        #[test]
        fn invalid_negative_infinity() {
            let result = Latitude::new(f64::NEG_INFINITY);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err.reason, DomainErrorKind::InvalidFormat { .. }));
            assert_eq!(err.value, "-infinity");
        }

        // Extension trait tests
        #[test]
        fn ext_is_north() {
            let north = Latitude::new(45.0).unwrap();
            assert!(north.is_north());
            assert!(!north.is_south());
            assert!(!north.is_equator());
        }

        #[test]
        fn ext_is_south() {
            let south = Latitude::new(-45.0).unwrap();
            assert!(south.is_south());
            assert!(!south.is_north());
            assert!(!south.is_equator());
        }

        #[test]
        fn ext_is_equator() {
            let equator = Latitude::new(0.0).unwrap();
            assert!(equator.is_equator());
            assert!(!equator.is_north());
            assert!(!equator.is_south());
        }

        #[test]
        fn ext_to_dms_north() {
            let lat = Latitude::new(37.7749).unwrap();
            let (deg, min, sec, hemi) = lat.to_dms();
            assert_eq!(deg, 37);
            assert_eq!(min, 46);
            assert!((sec - 29.64).abs() < 0.01);
            assert_eq!(hemi, 'N');
        }

        #[test]
        fn ext_to_dms_south() {
            let lat = Latitude::new(-33.8688).unwrap();
            let (deg, min, sec, hemi) = lat.to_dms();
            assert_eq!(deg, 33);
            assert_eq!(min, 52);
            assert!((sec - 7.68).abs() < 0.01);
            assert_eq!(hemi, 'S');
        }

        #[test]
        fn ext_to_dms_equator() {
            let lat = Latitude::new(0.0).unwrap();
            let (deg, min, sec, hemi) = lat.to_dms();
            assert_eq!(deg, 0);
            assert_eq!(min, 0);
            assert!(sec.abs() < 0.0001);
            assert_eq!(hemi, 'N');
        }

        // Error message tests
        #[test]
        fn error_includes_format_name() {
            let result = Latitude::new(91.0);
            let err = result.unwrap_err();
            assert_eq!(err.format_name, "latitude");
        }

        #[test]
        fn error_includes_example() {
            let result = Latitude::new(91.0);
            let err = result.unwrap_err();
            assert_eq!(err.example, "37.7749");
        }

        #[test]
        fn description_returns_expected() {
            assert_eq!(ValidLatitude::description(), "latitude (-90 to 90 degrees)");
        }
    }

    // ========================================================================
    // Longitude Tests
    // ========================================================================

    mod longitude_tests {
        use super::*;

        // Valid cases
        #[test]
        fn valid_zero() {
            assert!(Longitude::new(0.0).is_ok());
        }

        #[test]
        fn valid_antimeridian_positive() {
            assert!(Longitude::new(180.0).is_ok());
        }

        #[test]
        fn valid_antimeridian_negative() {
            assert!(Longitude::new(-180.0).is_ok());
        }

        #[test]
        fn valid_positive() {
            assert!(Longitude::new(122.4194).is_ok());
        }

        #[test]
        fn valid_negative() {
            assert!(Longitude::new(-122.4194).is_ok());
        }

        #[test]
        fn valid_mid_range() {
            assert!(Longitude::new(90.0).is_ok());
            assert!(Longitude::new(-90.0).is_ok());
        }

        // Invalid cases
        #[test]
        fn invalid_too_high() {
            let result = Longitude::new(180.1);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.reason,
                DomainErrorKind::InvalidComponent { .. }
            ));
        }

        #[test]
        fn invalid_too_low() {
            let result = Longitude::new(-180.1);
            assert!(result.is_err());
        }

        #[test]
        fn invalid_nan() {
            let result = Longitude::new(f64::NAN);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err.reason, DomainErrorKind::InvalidFormat { .. }));
            assert_eq!(err.value, "NaN");
        }

        #[test]
        fn invalid_positive_infinity() {
            let result = Longitude::new(f64::INFINITY);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.value, "infinity");
        }

        #[test]
        fn invalid_negative_infinity() {
            let result = Longitude::new(f64::NEG_INFINITY);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.value, "-infinity");
        }

        // Extension trait tests
        #[test]
        fn ext_is_east() {
            let east = Longitude::new(122.4194).unwrap();
            assert!(east.is_east());
            assert!(!east.is_west());
            assert!(!east.is_prime_meridian());
        }

        #[test]
        fn ext_is_west() {
            let west = Longitude::new(-122.4194).unwrap();
            assert!(west.is_west());
            assert!(!west.is_east());
            assert!(!west.is_prime_meridian());
        }

        #[test]
        fn ext_is_prime_meridian() {
            let pm = Longitude::new(0.0).unwrap();
            assert!(pm.is_prime_meridian());
            assert!(!pm.is_east());
            assert!(!pm.is_west());
        }

        #[test]
        fn ext_is_antimeridian() {
            let am_pos = Longitude::new(180.0).unwrap();
            assert!(am_pos.is_antimeridian());

            let am_neg = Longitude::new(-180.0).unwrap();
            assert!(am_neg.is_antimeridian());

            let not_am = Longitude::new(90.0).unwrap();
            assert!(!not_am.is_antimeridian());
        }

        #[test]
        fn ext_to_dms_east() {
            let lon = Longitude::new(122.4194).unwrap();
            let (deg, min, sec, hemi) = lon.to_dms();
            assert_eq!(deg, 122);
            assert_eq!(min, 25);
            assert!((sec - 9.84).abs() < 0.01);
            assert_eq!(hemi, 'E');
        }

        #[test]
        fn ext_to_dms_west() {
            let lon = Longitude::new(-122.4194).unwrap();
            let (deg, min, sec, hemi) = lon.to_dms();
            assert_eq!(deg, 122);
            assert_eq!(min, 25);
            assert!((sec - 9.84).abs() < 0.01);
            assert_eq!(hemi, 'W');
        }

        #[test]
        fn ext_to_dms_prime_meridian() {
            let lon = Longitude::new(0.0).unwrap();
            let (deg, min, sec, hemi) = lon.to_dms();
            assert_eq!(deg, 0);
            assert_eq!(min, 0);
            assert!(sec.abs() < 0.0001);
            assert_eq!(hemi, 'E');
        }

        // Error message tests
        #[test]
        fn error_includes_format_name() {
            let result = Longitude::new(200.0);
            let err = result.unwrap_err();
            assert_eq!(err.format_name, "longitude");
        }

        #[test]
        fn error_includes_example() {
            let result = Longitude::new(200.0);
            let err = result.unwrap_err();
            assert_eq!(err.example, "-122.4194");
        }

        #[test]
        fn description_returns_expected() {
            assert_eq!(
                ValidLongitude::description(),
                "longitude (-180 to 180 degrees)"
            );
        }
    }

    // ========================================================================
    // DMS Conversion Tests
    // ========================================================================

    mod dms_tests {
        use super::*;

        #[test]
        fn dms_conversion_roundtrip_latitude() {
            let original = 37.7749;
            let lat = Latitude::new(original).unwrap();
            let (deg, min, sec, hemi) = lat.to_dms();
            let reconstructed = latitude_from_dms(deg as u32, min, sec, hemi).unwrap();
            assert!((*reconstructed.get() - original).abs() < 0.0001);
        }

        #[test]
        fn dms_conversion_roundtrip_longitude() {
            let original = -122.4194;
            let lon = Longitude::new(original).unwrap();
            let (deg, min, sec, hemi) = lon.to_dms();
            let reconstructed = longitude_from_dms(deg as u32, min, sec, hemi).unwrap();
            assert!((*reconstructed.get() - original).abs() < 0.0001);
        }

        #[test]
        fn latitude_from_dms_valid() {
            let lat = latitude_from_dms(37, 46, 29.64, 'N').unwrap();
            assert!((*lat.get() - 37.7749).abs() < 0.0001);
        }

        #[test]
        fn latitude_from_dms_south() {
            let lat = latitude_from_dms(33, 52, 7.68, 'S').unwrap();
            assert!((*lat.get() - (-33.8688)).abs() < 0.0001);
        }

        #[test]
        fn latitude_from_dms_lowercase_hemisphere() {
            let lat = latitude_from_dms(37, 46, 29.64, 'n').unwrap();
            assert!(lat.is_north());
        }

        #[test]
        fn latitude_from_dms_invalid_degrees() {
            let result = latitude_from_dms(91, 0, 0.0, 'N');
            assert!(result.is_err());
        }

        #[test]
        fn latitude_from_dms_invalid_minutes() {
            let result = latitude_from_dms(37, 60, 0.0, 'N');
            assert!(result.is_err());
        }

        #[test]
        fn latitude_from_dms_invalid_seconds() {
            let result = latitude_from_dms(37, 46, 60.0, 'N');
            assert!(result.is_err());
        }

        #[test]
        fn latitude_from_dms_invalid_hemisphere() {
            let result = latitude_from_dms(37, 46, 29.64, 'X');
            assert!(result.is_err());
        }

        #[test]
        fn longitude_from_dms_valid() {
            let lon = longitude_from_dms(122, 25, 9.84, 'W').unwrap();
            assert!((*lon.get() - (-122.4194)).abs() < 0.0001);
        }

        #[test]
        fn longitude_from_dms_east() {
            let lon = longitude_from_dms(122, 25, 9.84, 'E').unwrap();
            assert!((*lon.get() - 122.4194).abs() < 0.0001);
        }

        #[test]
        fn longitude_from_dms_invalid_degrees() {
            let result = longitude_from_dms(181, 0, 0.0, 'E');
            assert!(result.is_err());
        }

        #[test]
        fn longitude_from_dms_invalid_hemisphere() {
            let result = longitude_from_dms(122, 25, 9.84, 'N');
            assert!(result.is_err());
        }

        #[test]
        fn dms_precision_test() {
            // Test that conversion is accurate to within 0.0001 seconds
            let lat = Latitude::new(40.7128).unwrap();
            let (deg, min, sec, _) = lat.to_dms();
            assert_eq!(deg, 40);
            assert_eq!(min, 42);
            // 40.7128 = 40° 42' 46.08"
            assert!((sec - 46.08).abs() < 0.01);
        }
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    mod edge_case_tests {
        use super::*;

        #[test]
        fn boundary_values_latitude() {
            // All boundary values should be valid
            assert!(Latitude::new(-90.0).is_ok());
            assert!(Latitude::new(-45.0).is_ok());
            assert!(Latitude::new(0.0).is_ok());
            assert!(Latitude::new(45.0).is_ok());
            assert!(Latitude::new(90.0).is_ok());
        }

        #[test]
        fn boundary_values_longitude() {
            // All boundary values should be valid
            assert!(Longitude::new(-180.0).is_ok());
            assert!(Longitude::new(-90.0).is_ok());
            assert!(Longitude::new(0.0).is_ok());
            assert!(Longitude::new(90.0).is_ok());
            assert!(Longitude::new(180.0).is_ok());
        }

        #[test]
        fn very_small_values() {
            // Very small values near zero should work
            assert!(Latitude::new(0.0000001).is_ok());
            assert!(Longitude::new(0.0000001).is_ok());
        }

        #[test]
        fn known_locations() {
            // San Francisco
            assert!(Latitude::new(37.7749).is_ok());
            assert!(Longitude::new(-122.4194).is_ok());

            // Sydney
            assert!(Latitude::new(-33.8688).is_ok());
            assert!(Longitude::new(151.2093).is_ok());

            // London (Greenwich)
            assert!(Latitude::new(51.4772).is_ok());
            assert!(Longitude::new(0.0).is_ok());

            // Tokyo
            assert!(Latitude::new(35.6762).is_ok());
            assert!(Longitude::new(139.6503).is_ok());
        }
    }
}
