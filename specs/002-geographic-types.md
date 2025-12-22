---
number: 2
title: Geographic Types
category: foundation
priority: medium
status: draft
dependencies: []
created: 2025-12-22
---

# Specification 002: Geographic Types

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: none

## Context

Geographic coordinates are fundamental to location-based applications, mapping services, logistics systems, and any application dealing with real-world positions. Latitude and longitude values have well-defined ranges that are frequently violated in user input or data imports, leading to subtle bugs or invalid map positions.

Stillwater already provides `between(min, max)` predicates for range validation, but raw range checks produce generic error messages. Geographic coordinates benefit from domain-specific types that:

1. Communicate intent clearly in type signatures
2. Provide coordinate-specific error messages ("latitude must be between -90 and 90 degrees")
3. Offer helper methods for common operations (DMS conversion, hemisphere detection)
4. Enable composition into a `Coordinate` or `GeoPoint` type

## Objective

Add a `geo` feature to stilltypes providing refined types for latitude and longitude values. These types leverage stillwater's numeric predicates but wrap them with geographic-specific semantics, error messages, and helper methods.

## Requirements

### Functional Requirements

1. **Latitude Type**
   - Accept `f64` values in range -90.0 to 90.0 (inclusive)
   - Reject NaN and infinity values
   - Provide hemisphere detection: `is_north()`, `is_south()`, `is_equator()`
   - Provide DMS (degrees, minutes, seconds) conversion: `to_dms() -> (i32, u32, f64, char)`
   - Support construction from DMS: `from_dms(degrees, minutes, seconds, hemisphere)`

2. **Longitude Type**
   - Accept `f64` values in range -180.0 to 180.0 (inclusive)
   - Reject NaN and infinity values
   - Provide hemisphere detection: `is_east()`, `is_west()`, `is_prime_meridian()`
   - Provide DMS conversion matching latitude
   - Handle the antimeridian edge case (180.0 == -180.0 semantically)

3. **Coordinate Type (Optional)**
   - Composite type combining validated Latitude and Longitude
   - Provide distance calculation between coordinates (Haversine formula)
   - Support common formats: "lat,lon", GeoJSON-style [lon, lat]

### Non-Functional Requirements

- Zero external dependencies (pure math operations)
- Serde support when `serde` feature is enabled
- Precision: maintain full f64 precision, don't round during validation
- All predicates must be zero-sized types (ZSTs)
- Error messages must use geographic terminology ("degrees", "north/south")

## Acceptance Criteria

- [ ] `Latitude` type validates f64 values in range [-90.0, 90.0]
- [ ] `Longitude` type validates f64 values in range [-180.0, 180.0]
- [ ] Both types reject NaN and infinity with appropriate errors
- [ ] `LatitudeExt` trait provides `is_north()`, `is_south()`, `to_dms()`
- [ ] `LongitudeExt` trait provides `is_east()`, `is_west()`, `to_dms()`
- [ ] Error messages are geographic-specific (e.g., "91.5 degrees is outside valid latitude range")
- [ ] Unit tests cover boundary values: -90, 0, 90 for lat; -180, 0, 180 for lon
- [ ] Unit tests cover invalid values: NaN, infinity, out of range
- [ ] DMS conversion is accurate to within 0.0001 seconds
- [ ] Serde integration tests pass when feature enabled

## Technical Details

### Implementation Approach

```rust
// src/geo.rs

use stillwater::refined::Refined;
use crate::error::{DomainError, DomainErrorKind};

/// Predicate for valid latitude values (-90 to 90 degrees).
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
                    expected: "a valid number",
                },
                example: "37.7749",
            });
        }

        if value.is_infinite() {
            return Err(DomainError {
                format_name: "latitude",
                value: if value.is_sign_positive() { "infinity" } else { "-infinity" }.to_string(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "a finite number",
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
}

/// A validated latitude coordinate.
pub type Latitude = Refined<f64, ValidLatitude>;

/// Extension trait for latitude operations.
pub trait LatitudeExt {
    /// Returns true if latitude is in the northern hemisphere (> 0).
    fn is_north(&self) -> bool;

    /// Returns true if latitude is in the southern hemisphere (< 0).
    fn is_south(&self) -> bool;

    /// Returns true if latitude is on the equator (== 0).
    fn is_equator(&self) -> bool;

    /// Convert to degrees, minutes, seconds format.
    /// Returns (degrees, minutes, seconds, hemisphere) where hemisphere is 'N' or 'S'.
    fn to_dms(&self) -> (i32, u32, f64, char);
}
```

### DMS Conversion Algorithm

```rust
fn decimal_to_dms(decimal: f64) -> (i32, u32, f64) {
    let abs_decimal = decimal.abs();
    let degrees = abs_decimal.floor() as i32;
    let minutes_decimal = (abs_decimal - degrees as f64) * 60.0;
    let minutes = minutes_decimal.floor() as u32;
    let seconds = (minutes_decimal - minutes as f64) * 60.0;
    (degrees, minutes, seconds)
}
```

### Feature Flag

```toml
[features]
geo = []  # No external dependencies
```

### Error Messages

```
"invalid latitude: 91.5 degrees must be between -90 and 90 (example: 37.7749)"
"invalid longitude: NaN is not a valid number (example: -122.4194)"
"invalid longitude: 200.0 degrees must be between -180 and 180 (example: -122.4194)"
```

## Dependencies

- **Prerequisites**: None
- **Affected Components**: `src/lib.rs`, `src/prelude.rs`, `Cargo.toml`
- **External Dependencies**: None

## Testing Strategy

- **Unit Tests**:
  - Valid latitudes: -90.0, -45.0, 0.0, 45.0, 90.0, 37.7749
  - Invalid latitudes: -90.1, 90.1, NaN, f64::INFINITY, f64::NEG_INFINITY
  - Valid longitudes: -180.0, -90.0, 0.0, 90.0, 180.0, -122.4194
  - Invalid longitudes: -180.1, 180.1, NaN, infinities
  - DMS conversion accuracy tests
  - Hemisphere detection tests

- **Integration Tests**: Serde round-trip for coordinates
- **Edge Cases**:
  - Exactly 0.0 (equator/prime meridian)
  - Exactly ±90.0 (poles)
  - Exactly ±180.0 (antimeridian)

## Documentation Requirements

- **Code Documentation**: Rustdoc with examples showing construction and DMS conversion
- **User Documentation**: Update lib.rs feature table
- **Examples**: Geographic validation example in examples/ directory

## Implementation Notes

- Use `f64` rather than a generic float type for simplicity
- The `to_dms()` method should handle negative values by returning absolute degrees with hemisphere indicator
- Consider whether -180.0 and 180.0 should be considered equivalent for longitude
- DMS seconds should be f64 to preserve precision

## Migration and Compatibility

- New feature, no breaking changes
- Optional feature flag means no impact on existing users
