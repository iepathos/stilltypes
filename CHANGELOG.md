# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-12-22

Initial release of Platypus - domain-specific refined types for the Stillwater ecosystem.

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

[0.1.0]: https://github.com/iepathos/platypus/releases/tag/v0.1.0
