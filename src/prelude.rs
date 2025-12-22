//! Convenient imports for common platypus usage.
//!
//! This module re-exports the most commonly used types from platypus,
//! allowing you to get started quickly with a single import.
//!
//! # Example
//!
//! ```
//! use platypus::prelude::*;
//!
//! // Now you have access to all enabled domain types
//! # #[cfg(feature = "email")]
//! # {
//! let email = Email::new("user@example.com".to_string());
//! assert!(email.is_ok());
//! # }
//! ```
//!
//! # Feature-Gated Exports
//!
//! The prelude only exports types for enabled features. Enable features
//! in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! platypus = { version = "0.1", features = ["email", "url", "phone"] }
//! ```

pub use crate::error::{DomainError, DomainErrorKind};

#[cfg(feature = "email")]
pub use crate::email::{Email, ValidEmail};

#[cfg(feature = "url")]
pub use crate::url::{HttpScheme, HttpUrl, HttpsOnly, SecureUrl, Url, ValidUrl};

#[cfg(feature = "uuid")]
pub use crate::uuid::{ToUuid, Uuid, UuidV4, UuidV7, UuidVersion, ValidUuid};

#[cfg(feature = "phone")]
pub use crate::phone::{PhoneNumber, PhoneNumberExt, ValidPhoneNumber};

#[cfg(feature = "financial")]
pub use crate::financial::{
    CreditCardExt, CreditCardNumber, Iban, IbanExt, ValidCreditCard, ValidIban,
};
