# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-12-22

Significant expansion of Stilltypes with four new validation modules covering network types, geographic coordinates, bounded numerics, and identifiers.

### Added

#### Network Validation (`network` feature)
- `Ipv4Addr` refined type for valid IPv4 addresses (RFC 791)
- `Ipv4Ext` trait with `is_private()`, `is_loopback()`, `is_link_local()`, `is_broadcast()` methods
- `Ipv6Addr` refined type for valid IPv6 addresses (RFC 4291)
- `Ipv6Ext` trait with `is_loopback()`, `is_multicast()`, `is_unspecified()` methods
- `DomainName` refined type for RFC 1035 compliant domain names
- `DomainNameExt` trait with domain manipulation methods
- `Port` refined type for valid port numbers (1-65535)
- `PortExt` trait with `is_privileged()`, `is_ephemeral()`, `is_registered()` methods

#### Geographic Validation (`geo` feature)
- `Latitude` refined type for latitude values (-90 to 90 degrees)
- `LatitudeExt` trait with `to_dms()`, `is_north()`, `is_south()` methods
- `Longitude` refined type for longitude values (-180 to 180 degrees)
- `LongitudeExt` trait with `to_dms()`, `is_east()`, `is_west()` methods
- `latitude_from_dms()` and `longitude_from_dms()` helper functions for DMS conversion

#### Numeric Validation (`numeric` feature)
- `Percentage` refined type for percentage values (0-100)
- `PercentageExt` trait with `to_decimal()`, `of()`, `complement()` methods
- `UnitInterval` refined type for unit interval values (0-1)
- `UnitIntervalExt` trait with `to_percentage()`, `complement()` methods

#### Identifier Validation (`identifiers` feature)
- `Slug` refined type for URL-safe slug values
- `SlugExt` trait with `from_title()` method for converting prose to slugs

#### New Examples
- `uuid_validation` example for UUID types
- `financial_validation` example for IBAN and credit card types
- `network_validation` example for IP addresses, domains, and ports
- `geo_validation` example for latitude/longitude with DMS conversion
- `discount_validation` example for percentage calculations
- `slug_validation` example for slug generation from titles

### Fixed
- Broken intra-doc links for feature-gated types
- Broken intra-doc links in feature table documentation

### Changed
- Updated `full` feature to include all new modules: `network`, `geo`, `numeric`, `identifiers`

### Features

| Feature | Description | Default |
|---------|-------------|---------|
| `email` | Email validation | ✓ |
| `url` | URL validation | ✓ |
| `uuid` | UUID validation | |
| `phone` | Phone number validation | |
| `financial` | IBAN and credit card validation | |
| `network` | IP address, domain, and port validation | |
| `geo` | Latitude and longitude validation | |
| `numeric` | Percentage and unit interval validation | |
| `identifiers` | URL slug validation | |
| `serde` | Serialization support | |
| `full` | All validators | |

## [0.1.0] - 2024-12-22

Initial release of Stilltypes - domain-specific refined types for the Stillwater ecosystem.

### Added

#### Core Infrastructure
- `DomainError` type with rich error context for user-facing messages
- Integration with `stillwater` for `Refined` types and error accumulation
- Modular feature flags to enable only needed validators

#### Email Validation
- `Email` refined type with RFC 5321 compliant validation
- Support for plus addressing (e.g., `user+tag@example.com`)
- `email_address` crate integration for robust validation

#### URL Validation
- `Url` refined type for any valid URL (RFC 3986)
- `HttpUrl` refined type for HTTP/HTTPS URLs only
- `SecureUrl` refined type for HTTPS-only URLs
- `url` crate integration

#### UUID Validation
- `Uuid` refined type for any valid UUID
- `UuidV4` refined type for version 4 UUIDs
- `UuidV7` refined type for version 7 UUIDs
- `ToUuid` trait for conversion to `uuid::Uuid`
- `uuid` crate integration

#### Phone Number Validation
- `PhoneNumber` refined type with E.164 format support
- `PhoneNumberExt` trait with `to_e164()` and `country_code()` methods
- Support for various input formats (parentheses, spaces, dashes)
- `phonenumber` crate integration

#### Financial Validation
- `Iban` refined type with checksum validation
- `IbanExt` trait with `country_code()` and `masked()` methods
- `CreditCardNumber` refined type with Luhn algorithm validation
- `CreditCardExt` trait with `masked()` and `last_four()` methods
- `iban_validate` and `creditcard` crate integration

#### Serde Integration
- Optional `serde` feature for Serialize/Deserialize support
- Validation during deserialization
- Transparent serialization of inner values

#### Documentation & Examples
- Comprehensive README with usage examples
- `form_validation` example demonstrating error accumulation
- `api_handler` example demonstrating effect composition
- Full rustdoc documentation

### Features

| Feature | Description | Default |
|---------|-------------|---------|
| `email` | Email validation | ✓ |
| `url` | URL validation | ✓ |
| `uuid` | UUID validation | |
| `phone` | Phone number validation | |
| `financial` | IBAN and credit card validation | |
| `serde` | Serialization support | |
| `full` | All validators | |

[0.2.0]: https://github.com/iepathos/stilltypes/releases/tag/v0.2.0
[0.1.0]: https://github.com/iepathos/stilltypes/releases/tag/v0.1.0
