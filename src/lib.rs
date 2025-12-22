//! # Platypus
//!
//! Domain-specific refined types for the Stillwater ecosystem.
//!
//! Platypus provides production-ready predicates and type aliases for common
//! domain types like email addresses, URLs, phone numbers, and more.

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

pub use error::{DomainError, DomainErrorKind};
