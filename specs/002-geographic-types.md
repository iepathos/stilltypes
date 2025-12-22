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

## Philosophy Alignment

This specification follows the [Stillwater Philosophy](../../../stillwater/PHILOSOPHY.md):

- **Parse, Don't Validate** (§7): `Latitude` and `Longitude` encode valid ranges at the type level. Once constructed, they're guaranteed within bounds—no runtime checks needed.
- **Errors Should Tell Stories** (§3): Errors use geographic terminology ("91.5 degrees is outside valid latitude range") not generic messages.
- **Types Guide, Don't Restrict** (§5): Simple types that make invalid coordinates impossible to represent.
- **Pragmatism Over Purity** (§6): Uses `f64` directly rather than generic numerics—practical for real-world use.

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
- [ ] `examples/geo_validation.rs` demonstrates error accumulation pattern
- [ ] README.md feature table updated with geo types
- [ ] lib.rs feature table updated with geo types
- [ ] `full` feature includes `geo`

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

## Error Accumulation Example

Following the "Fail Completely" pattern (PHILOSOPHY.md §2), geographic types integrate with `Validation::all()`:

```rust
use stilltypes::prelude::*;
use stilltypes::geo::{Latitude, Longitude};
use stillwater::validation::{Validation, ValidateAll};

/// Raw location input from user or API.
struct LocationInput {
    lat: f64,
    lon: f64,
    name: String,
}

/// Validated location - coordinates guaranteed within valid ranges.
struct ValidLocation {
    lat: Latitude,
    lon: Longitude,
    name: String,
}

fn validate_location(input: LocationInput) -> Validation<ValidLocation, Vec<DomainError>> {
    let lat_v = Validation::from_result(Latitude::new(input.lat).map_err(|e| vec![e]));
    let lon_v = Validation::from_result(Longitude::new(input.lon).map_err(|e| vec![e]));

    (lat_v, lon_v)
        .validate_all()
        .map(|(lat, lon)| ValidLocation { lat, lon, name: input.name })
}

// Returns both errors at once:
// - "invalid latitude: 91.5 degrees must be between -90 and 90"
// - "invalid longitude: 200.0 degrees must be between -180 and 180"
```

## Pure Core Example

Once validated, geographic types enable pure business logic:

```rust
use std::f64::consts::PI;

/// Pure function - no validation needed, types guarantee correctness.
fn haversine_distance(lat1: &Latitude, lon1: &Longitude, lat2: &Latitude, lon2: &Longitude) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let lat1_rad = lat1.get().to_radians();
    let lat2_rad = lat2.get().to_radians();
    let dlat = (lat2.get() - lat1.get()).to_radians();
    let dlon = (lon2.get() - lon1.get()).to_radians();

    let a = (dlat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    EARTH_RADIUS_KM * c
}

/// Pure function - operates on guaranteed-valid coordinates.
fn is_in_northern_hemisphere(lat: &Latitude) -> bool {
    lat.is_north()
}
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

### Code Documentation
- Full rustdoc with examples for each type and trait
- Module-level documentation explaining coordinate systems

### lib.rs Feature Table Update
Add row to the feature table in `src/lib.rs`:
```markdown
//! | `geo` | [`Latitude`](geo::Latitude), [`Longitude`](geo::Longitude) | - |
```

### README.md Updates
Add to feature table:
```markdown
| `geo` | `Latitude`, `Longitude` | - |
```

Add usage section:
```markdown
### Geographic Coordinates

\`\`\`rust,ignore
use stilltypes::geo::{Latitude, Longitude, LatitudeExt};

let lat = Latitude::new(37.7749)?;
let lon = Longitude::new(-122.4194)?;

assert!(lat.is_north());
let (deg, min, sec, hemi) = lat.to_dms();
// 37° 46' 29.64" N
\`\`\`
```

### Example File
Create `examples/geo_validation.rs`:
- Demonstrate coordinate validation with error accumulation
- Show Haversine distance calculation with validated coordinates
- Include boundary cases (poles, antimeridian)
- Pattern after `examples/form_validation.rs`

## Implementation Notes

- Use `f64` rather than a generic float type for simplicity
- The `to_dms()` method should handle negative values by returning absolute degrees with hemisphere indicator
- Consider whether -180.0 and 180.0 should be considered equivalent for longitude
- DMS seconds should be f64 to preserve precision

## Migration and Compatibility

- New feature, no breaking changes
- Optional feature flag means no impact on existing users
