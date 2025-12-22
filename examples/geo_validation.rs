//! Geographic coordinate validation example demonstrating error accumulation with stilltypes.
//!
//! This example shows how to validate geographic coordinates, convert between
//! decimal degrees and DMS format, and calculate distances using the Haversine formula.
//!
//! Run with: cargo run --example geo_validation --features full

use stilltypes::geo::{
    Latitude, LatitudeExt, Longitude, LongitudeExt, latitude_from_dms, longitude_from_dms,
};
use stilltypes::prelude::*;
use stillwater::validation::Validation;

/// Raw location input before validation.
#[derive(Debug)]
struct LocationInput {
    name: String,
    lat: f64,
    lon: f64,
}

/// Validated location - coordinates guaranteed within valid ranges.
#[derive(Debug)]
struct ValidLocation {
    name: String,
    lat: Latitude,
    lon: Longitude,
}

/// Validates a location, accumulating all errors.
fn validate_location(input: LocationInput) -> Validation<ValidLocation, Vec<DomainError>> {
    use stillwater::validation::ValidateAll;

    let lat_v: Validation<Latitude, Vec<DomainError>> =
        Validation::from_result(Latitude::new(input.lat).map_err(|e| vec![e]));
    let lon_v: Validation<Longitude, Vec<DomainError>> =
        Validation::from_result(Longitude::new(input.lon).map_err(|e| vec![e]));

    (lat_v, lon_v)
        .validate_all()
        .map(|(lat, lon)| ValidLocation {
            name: input.name,
            lat,
            lon,
        })
}

/// Calculate distance between two points using the Haversine formula.
///
/// This is a pure function - no validation needed because the types guarantee
/// the coordinates are valid.
fn haversine_distance(lat1: &Latitude, lon1: &Longitude, lat2: &Latitude, lon2: &Longitude) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let lat1_rad = lat1.get().to_radians();
    let lat2_rad = lat2.get().to_radians();
    let dlat = (lat2.get() - lat1.get()).to_radians();
    let dlon = (lon2.get() - lon1.get()).to_radians();

    let a =
        (dlat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    EARTH_RADIUS_KM * c
}

fn main() {
    println!("Stilltypes Geographic Validation Example");
    println!("=========================================\n");

    // Example 1: Valid location
    println!("=== Valid Location (San Francisco) ===");
    let sf = LocationInput {
        name: "San Francisco".into(),
        lat: 37.7749,
        lon: -122.4194,
    };

    match validate_location(sf) {
        Validation::Success(loc) => {
            println!("Location validated: {}", loc.name);
            println!(
                "  Latitude: {} ({} hemisphere)",
                loc.lat.get(),
                if loc.lat.is_north() {
                    "Northern"
                } else {
                    "Southern"
                }
            );
            println!(
                "  Longitude: {} ({} hemisphere)",
                loc.lon.get(),
                if loc.lon.is_west() {
                    "Western"
                } else {
                    "Eastern"
                }
            );

            let (deg, min, sec, hemi) = loc.lat.to_dms();
            println!("  Lat DMS: {}° {}' {:.2}\" {}", deg, min, sec, hemi);

            let (deg, min, sec, hemi) = loc.lon.to_dms();
            println!("  Lon DMS: {}° {}' {:.2}\" {}", deg, min, sec, hemi);
        }
        Validation::Failure(errors) => {
            for err in errors {
                println!("  Error: {}", err);
            }
        }
    }

    // Example 2: Invalid coordinates - both fail
    println!("\n=== Invalid Location (both coordinates wrong) ===");
    let invalid = LocationInput {
        name: "Invalid".into(),
        lat: 91.5,  // Out of range (max 90)
        lon: 200.0, // Out of range (max 180)
    };

    match validate_location(invalid) {
        Validation::Success(_) => println!("Unexpected success!"),
        Validation::Failure(errors) => {
            println!("Validation failed with {} errors:", errors.len());
            for err in &errors {
                println!("  - {}", err);
            }
        }
    }

    // Example 3: NaN and infinity handling
    println!("\n=== Special Values (NaN, Infinity) ===");
    println!("Testing NaN latitude:");
    match Latitude::new(f64::NAN) {
        Ok(_) => println!("  Unexpected success!"),
        Err(e) => println!("  {}", e),
    }

    println!("Testing infinity longitude:");
    match Longitude::new(f64::INFINITY) {
        Ok(_) => println!("  Unexpected success!"),
        Err(e) => println!("  {}", e),
    }

    // Example 4: Boundary values
    println!("\n=== Boundary Values ===");
    println!("Testing poles:");
    let north_pole = Latitude::new(90.0).unwrap();
    let south_pole = Latitude::new(-90.0).unwrap();
    println!(
        "  North Pole (90°): is_north={}, is_equator={}",
        north_pole.is_north(),
        north_pole.is_equator()
    );
    println!(
        "  South Pole (-90°): is_south={}, is_equator={}",
        south_pole.is_south(),
        south_pole.is_equator()
    );

    println!("\nTesting antimeridian:");
    let am_pos = Longitude::new(180.0).unwrap();
    let am_neg = Longitude::new(-180.0).unwrap();
    println!("  +180°: is_antimeridian={}", am_pos.is_antimeridian());
    println!("  -180°: is_antimeridian={}", am_neg.is_antimeridian());

    println!("\nTesting equator and prime meridian:");
    let equator = Latitude::new(0.0).unwrap();
    let prime_meridian = Longitude::new(0.0).unwrap();
    println!("  Equator (0° lat): is_equator={}", equator.is_equator());
    println!(
        "  Prime Meridian (0° lon): is_prime_meridian={}",
        prime_meridian.is_prime_meridian()
    );

    // Example 5: DMS conversion
    println!("\n=== DMS Conversion ===");
    println!("Creating coordinates from DMS:");

    match latitude_from_dms(37, 46, 29.64, 'N') {
        Ok(lat) => println!("  37° 46' 29.64\" N -> {:.4}°", lat.get()),
        Err(e) => println!("  Error: {}", e),
    }

    match longitude_from_dms(122, 25, 9.84, 'W') {
        Ok(lon) => println!("  122° 25' 9.84\" W -> {:.4}°", lon.get()),
        Err(e) => println!("  Error: {}", e),
    }

    println!("\nInvalid DMS values:");
    match latitude_from_dms(91, 0, 0.0, 'N') {
        Ok(_) => println!("  Unexpected success!"),
        Err(e) => println!("  91° N: {}", e),
    }

    match longitude_from_dms(122, 25, 9.84, 'N') {
        Ok(_) => println!("  Unexpected success!"),
        Err(e) => println!("  Using 'N' for longitude: {}", e),
    }

    // Example 6: Distance calculation with validated coordinates
    println!("\n=== Distance Calculation (Haversine) ===");
    let sf_lat = Latitude::new(37.7749).unwrap();
    let sf_lon = Longitude::new(-122.4194).unwrap();
    let nyc_lat = Latitude::new(40.7128).unwrap();
    let nyc_lon = Longitude::new(-74.0060).unwrap();
    let sydney_lat = Latitude::new(-33.8688).unwrap();
    let sydney_lon = Longitude::new(151.2093).unwrap();

    let sf_to_nyc = haversine_distance(&sf_lat, &sf_lon, &nyc_lat, &nyc_lon);
    let sf_to_sydney = haversine_distance(&sf_lat, &sf_lon, &sydney_lat, &sydney_lon);
    let nyc_to_sydney = haversine_distance(&nyc_lat, &nyc_lon, &sydney_lat, &sydney_lon);

    println!("  San Francisco to New York: {:.0} km", sf_to_nyc);
    println!("  San Francisco to Sydney: {:.0} km", sf_to_sydney);
    println!("  New York to Sydney: {:.0} km", nyc_to_sydney);

    // Example 7: Known locations
    println!("\n=== Known Locations ===");
    let locations = [
        ("San Francisco", 37.7749, -122.4194),
        ("New York", 40.7128, -74.0060),
        ("London", 51.5074, -0.1278),
        ("Sydney", -33.8688, 151.2093),
        ("Tokyo", 35.6762, 139.6503),
        ("Cape Town", -33.9249, 18.4241),
    ];

    for (name, lat_val, lon_val) in &locations {
        let lat = Latitude::new(*lat_val).unwrap();
        let lon = Longitude::new(*lon_val).unwrap();
        let n_s = if lat.is_north() { "N" } else { "S" };
        let e_w = if lon.is_east() { "E" } else { "W" };
        println!(
            "  {}: {:.4}° {}, {:.4}° {}",
            name,
            lat.get().abs(),
            n_s,
            lon.get().abs(),
            e_w
        );
    }
}
