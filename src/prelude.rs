//! Convenient imports for common stilltypes usage.
//!
//! This module re-exports the most commonly used types from stilltypes,
//! allowing you to get started quickly with a single import.
//!
//! # Example
//!
//! ```
//! use stilltypes::prelude::*;
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
//! stilltypes = { version = "0.1", features = ["email", "url", "phone"] }
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

#[cfg(feature = "network")]
pub use crate::network::{
    DomainName, DomainNameExt, Ipv4Addr, Ipv4Ext, Ipv6Addr, Ipv6Ext, Port, PortExt,
    ValidDomainName, ValidIpv4, ValidIpv6, ValidPort,
};

#[cfg(feature = "geo")]
pub use crate::geo::{
    Latitude, LatitudeExt, Longitude, LongitudeExt, ValidLatitude, ValidLongitude,
    latitude_from_dms, longitude_from_dms,
};

#[cfg(feature = "numeric")]
pub use crate::numeric::{
    Percentage, PercentageExt, UnitInterval, UnitIntervalExt, ValidPercentage, ValidUnitInterval,
};

#[cfg(feature = "identifiers")]
pub use crate::identifiers::{Slug, SlugExt, ValidSlug};
