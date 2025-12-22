//! # Stilltypes
//!
//! Domain-specific refined types for the Stillwater ecosystem.
//!
//! Stilltypes provides production-ready predicates and type aliases for common
//! domain types like email addresses, URLs, phone numbers, and more. All types
//! integrate seamlessly with [stillwater](https://docs.rs/stillwater)'s
//! `Validation` for error accumulation and `Effect` for composable I/O.
//!
//! ## Quick Start
//!
//! ```
//! use stilltypes::prelude::*;
//!
//! // Types validate on construction
//! # #[cfg(feature = "email")]
//! # {
//! let email = Email::new("user@example.com".to_string());
//! assert!(email.is_ok());
//!
//! // Invalid values fail with helpful errors
//! let bad = Email::new("invalid".to_string());
//! assert!(bad.is_err());
//! println!("{}", bad.unwrap_err());
//! // invalid email address: invalid format, expected local@domain (example: user@example.com)
//! # }
//! ```
//!
//! ## Feature Flags
//!
//! Enable only what you need:
//!
//! ```toml
//! [dependencies]
//! stilltypes = { version = "0.1", default-features = false, features = ["email", "url"] }
//! ```
//!
//! | Feature | Types | Dependencies |
//! |---------|-------|--------------|
//! | `email` (default) | [`Email`](email::Email) | `email_address` |
//! | `url` (default) | [`Url`](url::Url), [`HttpUrl`](url::HttpUrl), [`SecureUrl`](url::SecureUrl) | `url` |
//! | `uuid` | [`Uuid`](uuid::Uuid), [`UuidV4`](uuid::UuidV4), [`UuidV7`](uuid::UuidV7) | `uuid` |
//! | `phone` | [`PhoneNumber`](phone::PhoneNumber) | `phonenumber` |
//! | `financial` | [`Iban`](financial::Iban), [`CreditCardNumber`](financial::CreditCardNumber) | `iban_validate`, `creditcard` |
//! | `network` | [`Ipv4Addr`](network::Ipv4Addr), [`Ipv6Addr`](network::Ipv6Addr), [`DomainName`](network::DomainName), [`Port`](network::Port) | - |
//! | `geo` | [`Latitude`](geo::Latitude), [`Longitude`](geo::Longitude) | - |
//! | `numeric` | [`Percentage`](numeric::Percentage), [`UnitInterval`](numeric::UnitInterval) | - |
//! | `serde` | Serialize/Deserialize for all types | - |
//! | `full` | All of the above | - |
//!
//! ## Error Accumulation with Stillwater
//!
//! Collect all validation errors at once using stillwater's `Validation`:
//!
//! ```ignore
//! use stilltypes::prelude::*;
//! use stillwater::validation::Validation;
//!
//! fn validate_form(email: String, phone: String) -> Validation<ValidForm, Vec<DomainError>> {
//!     Validation::all((
//!         Email::new(email).map_err(|e| vec![e]),
//!         PhoneNumber::new(phone).map_err(|e| vec![e]),
//!     ))
//!     .map(|(email, phone)| ValidForm { email, phone })
//! }
//! ```
//!
//! ## Serde Integration
//!
//! With the `serde` feature, types validate during deserialization:
//!
//! ```ignore
//! use stilltypes::prelude::*;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct User {
//!     email: Email,
//!     website: Option<SecureUrl>,
//! }
//!
//! // Invalid JSON fails to deserialize
//! let result: Result<User, _> = serde_json::from_str(json);
//! ```
//!
//! ## Philosophy
//!
//! Stilltypes follows the [Stillwater philosophy](https://github.com/iepathos/stillwater):
//!
//! - **Parse, Don't Validate** - Domain types encode invariants in the type
//! - **Errors Should Tell Stories** - Rich context for user-facing messages
//! - **Composition Over Complexity** - Uses stillwater's `And`, `Or`, `Not`
//! - **Pragmatism Over Purity** - Real-world formats, not theoretical perfection

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod error;
pub mod prelude;

#[cfg(feature = "email")]
pub mod email;

#[cfg(feature = "url")]
pub mod url;

#[cfg(feature = "uuid")]
pub mod uuid;

#[cfg(feature = "phone")]
pub mod phone;

#[cfg(feature = "financial")]
pub mod financial;

#[cfg(feature = "network")]
pub mod network;

#[cfg(feature = "geo")]
pub mod geo;

#[cfg(feature = "numeric")]
pub mod numeric;

pub use error::{DomainError, DomainErrorKind};
