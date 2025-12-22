//! Convenient imports for platypus.
//!
//! ```rust
//! use platypus::prelude::*;
//! ```

pub use crate::error::{DomainError, DomainErrorKind};

// Allow unused imports for stub modules during initial development.
// These will export types once the modules are implemented.
#[allow(unused_imports)]
#[cfg(feature = "email")]
pub use crate::email::*;

#[allow(unused_imports)]
#[cfg(feature = "url")]
pub use crate::url::*;

#[allow(unused_imports)]
#[cfg(feature = "uuid")]
pub use crate::uuid::*;

#[allow(unused_imports)]
#[cfg(feature = "phone")]
pub use crate::phone::*;

#[allow(unused_imports)]
#[cfg(feature = "financial")]
pub use crate::financial::*;
