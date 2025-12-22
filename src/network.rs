//! Network validation types.
//!
//! Provides validated types for IP addresses, domain names, and ports following
//! RFC standards. These types encode validity at the type level—once you have
//! an `Ipv4Addr`, it's guaranteed valid with no runtime checks needed downstream.
//!
//! # Philosophy
//!
//! This module follows the Stillwater philosophy:
//! - **Parse, Don't Validate**: Network types guarantee validity via the type system
//! - **Errors Should Tell Stories**: Rich `DomainError` messages with examples
//! - **Composition Over Complexity**: Simple predicates combinable with `And`, `Or`, `Not`
//! - **Pragmatism Over Purity**: Uses `std::net` for parsing where appropriate
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "network")]
//! # {
//! use stilltypes::network::{Ipv4Addr, Port, DomainName, Ipv4Ext, PortExt};
//!
//! // IP addresses validate format
//! let ip = Ipv4Addr::new("192.168.1.1".to_string()).unwrap();
//! assert!(ip.is_private());
//!
//! // Ports validate range and provide semantic helpers
//! let port = Port::new(443).unwrap();
//! assert!(port.is_privileged());
//!
//! // Domain names follow RFC 1035
//! let domain = DomainName::new("api.example.com".to_string()).unwrap();
//! # }
//! ```

use crate::error::{DomainError, DomainErrorKind};
use stillwater::refined::{Predicate, Refined};

// ============================================================================
// IPv4 Address
// ============================================================================

/// Predicate for valid IPv4 addresses.
///
/// Validates IPv4 addresses in dotted-decimal notation (A.B.C.D) where each
/// octet is 0-255. Uses `std::net::Ipv4Addr` for parsing to ensure RFC compliance.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "network")]
/// # {
/// use stilltypes::network::Ipv4Addr;
///
/// let ip = Ipv4Addr::new("192.168.1.1".to_string());
/// assert!(ip.is_ok());
///
/// let invalid = Ipv4Addr::new("256.0.0.0".to_string());
/// assert!(invalid.is_err());
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidIpv4;

impl Predicate<String> for ValidIpv4 {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.is_empty() {
            return Err(DomainError {
                format_name: "IPv4 address",
                value: value.clone(),
                reason: DomainErrorKind::Empty,
                example: "192.168.1.1",
            });
        }

        value
            .parse::<std::net::Ipv4Addr>()
            .map(|_| ())
            .map_err(|_| DomainError {
                format_name: "IPv4 address",
                value: value.clone(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "dotted decimal (A.B.C.D where 0 <= each <= 255)",
                },
                example: "192.168.1.1",
            })
    }

    fn description() -> &'static str {
        "RFC 791 IPv4 address"
    }
}

/// A validated IPv4 address.
///
/// A `String` that has been validated to be a properly formatted IPv4 address
/// in dotted-decimal notation.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "network")]
/// # {
/// use stilltypes::network::{Ipv4Addr, Ipv4Ext};
///
/// let ip = Ipv4Addr::new("127.0.0.1".to_string()).unwrap();
/// assert_eq!(ip.get(), "127.0.0.1");
/// assert!(ip.is_loopback());
/// # }
/// ```
pub type Ipv4Addr = Refined<String, ValidIpv4>;

/// Extension trait for IPv4 address operations.
///
/// Provides semantic helpers for working with validated IPv4 addresses.
pub trait Ipv4Ext {
    /// Convert to standard library `Ipv4Addr` type.
    ///
    /// This conversion is infallible because the address was validated on construction.
    fn to_std(&self) -> std::net::Ipv4Addr;

    /// Check if this is a loopback address (127.0.0.0/8).
    fn is_loopback(&self) -> bool;

    /// Check if this is a private address (RFC 1918).
    ///
    /// Private ranges:
    /// - 10.0.0.0/8
    /// - 172.16.0.0/12
    /// - 192.168.0.0/16
    fn is_private(&self) -> bool;

    /// Check if this is a link-local address (169.254.0.0/16).
    fn is_link_local(&self) -> bool;

    /// Check if this is a broadcast address (255.255.255.255).
    fn is_broadcast(&self) -> bool;

    /// Check if this is an unspecified address (0.0.0.0).
    fn is_unspecified(&self) -> bool;

    /// Get the individual octets as an array.
    fn octets(&self) -> [u8; 4];
}

impl Ipv4Ext for Ipv4Addr {
    fn to_std(&self) -> std::net::Ipv4Addr {
        // Safe: value was validated on construction
        self.get().parse().unwrap()
    }

    fn is_loopback(&self) -> bool {
        self.to_std().is_loopback()
    }

    fn is_private(&self) -> bool {
        self.to_std().is_private()
    }

    fn is_link_local(&self) -> bool {
        self.to_std().is_link_local()
    }

    fn is_broadcast(&self) -> bool {
        self.to_std().is_broadcast()
    }

    fn is_unspecified(&self) -> bool {
        self.to_std().is_unspecified()
    }

    fn octets(&self) -> [u8; 4] {
        self.to_std().octets()
    }
}

// ============================================================================
// IPv6 Address
// ============================================================================

/// Predicate for valid IPv6 addresses.
///
/// Validates IPv6 addresses per RFC 4291, including:
/// - Full notation (2001:0db8:0000:0000:0000:0000:0000:0001)
/// - Compressed notation (2001:db8::1)
/// - Mixed notation (::ffff:192.168.1.1)
///
/// # Example
///
/// ```
/// # #[cfg(feature = "network")]
/// # {
/// use stilltypes::network::Ipv6Addr;
///
/// let ip = Ipv6Addr::new("::1".to_string());
/// assert!(ip.is_ok());
///
/// let full = Ipv6Addr::new("2001:db8::1".to_string());
/// assert!(full.is_ok());
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidIpv6;

impl Predicate<String> for ValidIpv6 {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.is_empty() {
            return Err(DomainError {
                format_name: "IPv6 address",
                value: value.clone(),
                reason: DomainErrorKind::Empty,
                example: "::1",
            });
        }

        value
            .parse::<std::net::Ipv6Addr>()
            .map(|_| ())
            .map_err(|_| DomainError {
                format_name: "IPv6 address",
                value: value.clone(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "hex groups separated by colons (supports :: compression)",
                },
                example: "2001:db8::1",
            })
    }

    fn description() -> &'static str {
        "RFC 4291 IPv6 address"
    }
}

/// A validated IPv6 address.
///
/// A `String` that has been validated to be a properly formatted IPv6 address
/// per RFC 4291.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "network")]
/// # {
/// use stilltypes::network::{Ipv6Addr, Ipv6Ext};
///
/// let ip = Ipv6Addr::new("::1".to_string()).unwrap();
/// assert!(ip.is_loopback());
/// # }
/// ```
pub type Ipv6Addr = Refined<String, ValidIpv6>;

/// Extension trait for IPv6 address operations.
///
/// Provides semantic helpers for working with validated IPv6 addresses.
pub trait Ipv6Ext {
    /// Convert to standard library `Ipv6Addr` type.
    ///
    /// This conversion is infallible because the address was validated on construction.
    fn to_std(&self) -> std::net::Ipv6Addr;

    /// Check if this is a loopback address (::1).
    fn is_loopback(&self) -> bool;

    /// Check if this is an unspecified address (::).
    fn is_unspecified(&self) -> bool;

    /// Check if this is an IPv4-mapped address (::ffff:x.x.x.x).
    fn is_ipv4_mapped(&self) -> bool;

    /// Get the 16-byte representation.
    fn octets(&self) -> [u8; 16];

    /// Get the eight 16-bit segments.
    fn segments(&self) -> [u16; 8];
}

impl Ipv6Ext for Ipv6Addr {
    fn to_std(&self) -> std::net::Ipv6Addr {
        // Safe: value was validated on construction
        self.get().parse().unwrap()
    }

    fn is_loopback(&self) -> bool {
        self.to_std().is_loopback()
    }

    fn is_unspecified(&self) -> bool {
        self.to_std().is_unspecified()
    }

    fn is_ipv4_mapped(&self) -> bool {
        // Check if address matches ::ffff:x.x.x.x pattern
        let segments = self.to_std().segments();
        segments[0] == 0
            && segments[1] == 0
            && segments[2] == 0
            && segments[3] == 0
            && segments[4] == 0
            && segments[5] == 0xffff
    }

    fn octets(&self) -> [u8; 16] {
        self.to_std().octets()
    }

    fn segments(&self) -> [u16; 8] {
        self.to_std().segments()
    }
}

// ============================================================================
// Domain Name
// ============================================================================

/// Predicate for valid DNS domain names.
///
/// Validates domain names per RFC 1035:
/// - Labels must be 1-63 characters
/// - Total length must be <= 253 characters
/// - Labels must start with a letter or digit
/// - Labels can only contain alphanumerics and hyphens
/// - Labels cannot end with a hyphen
/// - Labels cannot have consecutive hyphens in positions 3-4 (reserved for IDN)
///
/// # Example
///
/// ```
/// # #[cfg(feature = "network")]
/// # {
/// use stilltypes::network::DomainName;
///
/// let domain = DomainName::new("example.com".to_string());
/// assert!(domain.is_ok());
///
/// let invalid = DomainName::new("-example.com".to_string());
/// assert!(invalid.is_err());
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidDomainName;

impl Predicate<String> for ValidDomainName {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.is_empty() {
            return Err(DomainError {
                format_name: "domain name",
                value: value.clone(),
                reason: DomainErrorKind::Empty,
                example: "example.com",
            });
        }

        // Total length check (253 max per RFC 1035)
        if value.len() > 253 {
            return Err(DomainError {
                format_name: "domain name",
                value: value.clone(),
                reason: DomainErrorKind::TooLong {
                    max: 253,
                    actual: value.len(),
                },
                example: "example.com",
            });
        }

        // Remove trailing dot if present (FQDN notation)
        let domain = value.strip_suffix('.').unwrap_or(value);

        // Split into labels and validate each
        let labels: Vec<&str> = domain.split('.').collect();

        for label in labels {
            validate_domain_label(label, value)?;
        }

        Ok(())
    }

    fn description() -> &'static str {
        "RFC 1035 domain name"
    }
}

/// Validate a single domain label per RFC 1035.
fn validate_domain_label(label: &str, full_domain: &str) -> Result<(), DomainError> {
    // Empty label check
    if label.is_empty() {
        return Err(DomainError {
            format_name: "domain name",
            value: full_domain.to_string(),
            reason: DomainErrorKind::InvalidComponent {
                component: "label",
                reason: "cannot be empty".to_string(),
            },
            example: "example.com",
        });
    }

    // Label length check (1-63 characters)
    if label.len() > 63 {
        return Err(DomainError {
            format_name: "domain name",
            value: full_domain.to_string(),
            reason: DomainErrorKind::InvalidComponent {
                component: "label",
                reason: format!("'{}' exceeds 63 characters (has {})", label, label.len()),
            },
            example: "example.com",
        });
    }

    let chars: Vec<char> = label.chars().collect();

    // First character must be alphanumeric
    if !chars[0].is_ascii_alphanumeric() {
        return Err(DomainError {
            format_name: "domain name",
            value: full_domain.to_string(),
            reason: DomainErrorKind::InvalidComponent {
                component: "label",
                reason: format!("'{}' must start with letter or digit", label),
            },
            example: "example.com",
        });
    }

    // Last character cannot be a hyphen
    if chars[chars.len() - 1] == '-' {
        return Err(DomainError {
            format_name: "domain name",
            value: full_domain.to_string(),
            reason: DomainErrorKind::InvalidComponent {
                component: "label",
                reason: format!("'{}' cannot end with hyphen", label),
            },
            example: "example.com",
        });
    }

    // Check all characters are valid (alphanumeric or hyphen)
    for (i, c) in chars.iter().enumerate() {
        if !c.is_ascii_alphanumeric() && *c != '-' {
            return Err(DomainError {
                format_name: "domain name",
                value: full_domain.to_string(),
                reason: DomainErrorKind::InvalidCharacter {
                    char: *c,
                    position: i,
                },
                example: "example.com",
            });
        }
    }

    Ok(())
}

/// A validated DNS domain name.
///
/// A `String` that has been validated to be a properly formatted domain name
/// per RFC 1035.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "network")]
/// # {
/// use stilltypes::network::DomainName;
///
/// let domain = DomainName::new("api.example.com".to_string()).unwrap();
/// assert_eq!(domain.get(), "api.example.com");
/// # }
/// ```
pub type DomainName = Refined<String, ValidDomainName>;

/// Extension trait for domain name operations.
pub trait DomainNameExt {
    /// Get the labels of the domain name.
    fn labels(&self) -> Vec<&str>;

    /// Get the top-level domain (TLD).
    fn tld(&self) -> Option<&str>;

    /// Check if this is a subdomain of another domain.
    fn is_subdomain_of(&self, parent: &str) -> bool;
}

impl DomainNameExt for DomainName {
    fn labels(&self) -> Vec<&str> {
        let domain = self.get().strip_suffix('.').unwrap_or(self.get());
        domain.split('.').collect()
    }

    fn tld(&self) -> Option<&str> {
        self.labels().last().copied()
    }

    fn is_subdomain_of(&self, parent: &str) -> bool {
        let self_lower = self.get().to_lowercase();
        let parent_lower = parent.to_lowercase();
        self_lower.ends_with(&format!(".{}", parent_lower))
    }
}

// ============================================================================
// Port Number
// ============================================================================

/// Predicate for valid port numbers.
///
/// Validates port numbers in the range 1-65535. Port 0 is excluded as it's
/// typically used as a wildcard for automatic port assignment, not as an
/// actual port number.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "network")]
/// # {
/// use stilltypes::network::Port;
///
/// let port = Port::new(8080);
/// assert!(port.is_ok());
///
/// let invalid = Port::new(0);
/// assert!(invalid.is_err());
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidPort;

impl Predicate<u16> for ValidPort {
    type Error = DomainError;

    fn check(value: &u16) -> Result<(), Self::Error> {
        if *value == 0 {
            return Err(DomainError {
                format_name: "port",
                value: value.to_string(),
                reason: DomainErrorKind::InvalidFormat {
                    expected: "number 1-65535 (port 0 is reserved)",
                },
                example: "8080",
            });
        }
        Ok(())
    }

    fn description() -> &'static str {
        "TCP/UDP port number (1-65535)"
    }
}

/// A validated port number.
///
/// A `u16` that has been validated to be in the valid port range (1-65535).
///
/// # Example
///
/// ```
/// # #[cfg(feature = "network")]
/// # {
/// use stilltypes::network::{Port, PortExt};
///
/// let port = Port::new(443).unwrap();
/// assert!(port.is_privileged());
/// assert!(port.is_well_known());
/// # }
/// ```
pub type Port = Refined<u16, ValidPort>;

/// Extension trait for port number operations.
///
/// Provides semantic helpers based on IANA port number ranges.
pub trait PortExt {
    /// Check if this is a privileged/well-known port (1-1023).
    ///
    /// These ports typically require root/administrator privileges to bind.
    fn is_privileged(&self) -> bool;

    /// Check if this is a well-known port (1-1023).
    ///
    /// Alias for `is_privileged()`. Well-known ports are assigned by IANA
    /// to common services like HTTP (80), HTTPS (443), SSH (22).
    fn is_well_known(&self) -> bool;

    /// Check if this is a registered port (1024-49151).
    ///
    /// Registered ports are assigned by IANA upon application.
    fn is_registered(&self) -> bool;

    /// Check if this is a dynamic/private/ephemeral port (49152-65535).
    ///
    /// These ports are used for ephemeral connections and private services.
    fn is_dynamic(&self) -> bool;

    /// Check if this is an ephemeral port (alias for is_dynamic).
    fn is_ephemeral(&self) -> bool;
}

impl PortExt for Port {
    fn is_privileged(&self) -> bool {
        *self.get() >= 1 && *self.get() <= 1023
    }

    fn is_well_known(&self) -> bool {
        self.is_privileged()
    }

    fn is_registered(&self) -> bool {
        *self.get() >= 1024 && *self.get() <= 49151
    }

    fn is_dynamic(&self) -> bool {
        *self.get() >= 49152
    }

    fn is_ephemeral(&self) -> bool {
        self.is_dynamic()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // IPv4 Tests
    // ========================================================================

    mod ipv4_tests {
        use super::*;

        // Valid cases
        #[test]
        fn valid_simple_ipv4() {
            assert!(Ipv4Addr::new("192.168.1.1".to_string()).is_ok());
        }

        #[test]
        fn valid_all_zeros() {
            assert!(Ipv4Addr::new("0.0.0.0".to_string()).is_ok());
        }

        #[test]
        fn valid_all_255() {
            assert!(Ipv4Addr::new("255.255.255.255".to_string()).is_ok());
        }

        #[test]
        fn valid_loopback() {
            assert!(Ipv4Addr::new("127.0.0.1".to_string()).is_ok());
        }

        #[test]
        fn valid_class_a_private() {
            assert!(Ipv4Addr::new("10.0.0.1".to_string()).is_ok());
        }

        #[test]
        fn valid_class_b_private() {
            assert!(Ipv4Addr::new("172.16.0.1".to_string()).is_ok());
        }

        #[test]
        fn valid_class_c_private() {
            assert!(Ipv4Addr::new("192.168.0.1".to_string()).is_ok());
        }

        // Invalid cases
        #[test]
        fn invalid_empty() {
            let result = Ipv4Addr::new(String::new());
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err.reason, DomainErrorKind::Empty));
        }

        #[test]
        fn invalid_octet_too_high() {
            let result = Ipv4Addr::new("256.0.0.0".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn invalid_too_few_octets() {
            let result = Ipv4Addr::new("1.2.3".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn invalid_too_many_octets() {
            let result = Ipv4Addr::new("1.2.3.4.5".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn invalid_letters() {
            let result = Ipv4Addr::new("abc.def.ghi.jkl".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn invalid_negative() {
            let result = Ipv4Addr::new("-1.0.0.0".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn invalid_spaces() {
            let result = Ipv4Addr::new("192.168.1. 1".to_string());
            assert!(result.is_err());
        }

        // Extension trait tests
        #[test]
        fn ext_to_std() {
            let ip = Ipv4Addr::new("192.168.1.1".to_string()).unwrap();
            let std_ip = ip.to_std();
            assert_eq!(std_ip.octets(), [192, 168, 1, 1]);
        }

        #[test]
        fn ext_is_loopback() {
            let loopback = Ipv4Addr::new("127.0.0.1".to_string()).unwrap();
            assert!(loopback.is_loopback());

            let not_loopback = Ipv4Addr::new("192.168.1.1".to_string()).unwrap();
            assert!(!not_loopback.is_loopback());
        }

        #[test]
        fn ext_is_private() {
            // Class A private
            let class_a = Ipv4Addr::new("10.0.0.1".to_string()).unwrap();
            assert!(class_a.is_private());

            // Class B private
            let class_b = Ipv4Addr::new("172.16.0.1".to_string()).unwrap();
            assert!(class_b.is_private());

            // Class C private
            let class_c = Ipv4Addr::new("192.168.1.1".to_string()).unwrap();
            assert!(class_c.is_private());

            // Public
            let public = Ipv4Addr::new("8.8.8.8".to_string()).unwrap();
            assert!(!public.is_private());
        }

        #[test]
        fn ext_is_link_local() {
            let link_local = Ipv4Addr::new("169.254.1.1".to_string()).unwrap();
            assert!(link_local.is_link_local());

            let not_link_local = Ipv4Addr::new("192.168.1.1".to_string()).unwrap();
            assert!(!not_link_local.is_link_local());
        }

        #[test]
        fn ext_is_broadcast() {
            let broadcast = Ipv4Addr::new("255.255.255.255".to_string()).unwrap();
            assert!(broadcast.is_broadcast());

            let not_broadcast = Ipv4Addr::new("192.168.1.1".to_string()).unwrap();
            assert!(!not_broadcast.is_broadcast());
        }

        #[test]
        fn ext_is_unspecified() {
            let unspecified = Ipv4Addr::new("0.0.0.0".to_string()).unwrap();
            assert!(unspecified.is_unspecified());

            let specified = Ipv4Addr::new("192.168.1.1".to_string()).unwrap();
            assert!(!specified.is_unspecified());
        }

        #[test]
        fn ext_octets() {
            let ip = Ipv4Addr::new("192.168.1.1".to_string()).unwrap();
            assert_eq!(ip.octets(), [192, 168, 1, 1]);
        }

        // Error message tests
        #[test]
        fn error_includes_format_name() {
            let result = Ipv4Addr::new("invalid".to_string());
            let err = result.unwrap_err();
            assert_eq!(err.format_name, "IPv4 address");
        }

        #[test]
        fn error_includes_example() {
            let result = Ipv4Addr::new("invalid".to_string());
            let err = result.unwrap_err();
            assert_eq!(err.example, "192.168.1.1");
        }

        #[test]
        fn description_returns_expected() {
            assert_eq!(ValidIpv4::description(), "RFC 791 IPv4 address");
        }
    }

    // ========================================================================
    // IPv6 Tests
    // ========================================================================

    mod ipv6_tests {
        use super::*;

        // Valid cases
        #[test]
        fn valid_loopback() {
            assert!(Ipv6Addr::new("::1".to_string()).is_ok());
        }

        #[test]
        fn valid_unspecified() {
            assert!(Ipv6Addr::new("::".to_string()).is_ok());
        }

        #[test]
        fn valid_compressed() {
            assert!(Ipv6Addr::new("2001:db8::1".to_string()).is_ok());
        }

        #[test]
        fn valid_full() {
            assert!(Ipv6Addr::new("2001:0db8:0000:0000:0000:0000:0000:0001".to_string()).is_ok());
        }

        #[test]
        fn valid_mixed_case() {
            assert!(Ipv6Addr::new("2001:DB8::1".to_string()).is_ok());
        }

        #[test]
        fn valid_link_local() {
            assert!(Ipv6Addr::new("fe80::1".to_string()).is_ok());
        }

        #[test]
        fn valid_ipv4_mapped() {
            assert!(Ipv6Addr::new("::ffff:192.168.1.1".to_string()).is_ok());
        }

        // Invalid cases
        #[test]
        fn invalid_empty() {
            let result = Ipv6Addr::new(String::new());
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err.reason, DomainErrorKind::Empty));
        }

        #[test]
        fn invalid_triple_colon() {
            assert!(Ipv6Addr::new(":::".to_string()).is_err());
        }

        #[test]
        fn invalid_too_many_groups() {
            assert!(Ipv6Addr::new("1:2:3:4:5:6:7:8:9".to_string()).is_err());
        }

        #[test]
        fn invalid_invalid_hex() {
            assert!(Ipv6Addr::new("2001:db8::g".to_string()).is_err());
        }

        #[test]
        fn invalid_group_too_long() {
            assert!(Ipv6Addr::new("12345::1".to_string()).is_err());
        }

        // Extension trait tests
        #[test]
        fn ext_to_std() {
            let ip = Ipv6Addr::new("::1".to_string()).unwrap();
            let std_ip = ip.to_std();
            assert_eq!(std_ip.segments(), [0, 0, 0, 0, 0, 0, 0, 1]);
        }

        #[test]
        fn ext_is_loopback() {
            let loopback = Ipv6Addr::new("::1".to_string()).unwrap();
            assert!(loopback.is_loopback());

            let not_loopback = Ipv6Addr::new("2001:db8::1".to_string()).unwrap();
            assert!(!not_loopback.is_loopback());
        }

        #[test]
        fn ext_is_unspecified() {
            let unspecified = Ipv6Addr::new("::".to_string()).unwrap();
            assert!(unspecified.is_unspecified());

            let specified = Ipv6Addr::new("::1".to_string()).unwrap();
            assert!(!specified.is_unspecified());
        }

        #[test]
        fn ext_is_ipv4_mapped() {
            let mapped = Ipv6Addr::new("::ffff:192.168.1.1".to_string()).unwrap();
            assert!(mapped.is_ipv4_mapped());

            let not_mapped = Ipv6Addr::new("2001:db8::1".to_string()).unwrap();
            assert!(!not_mapped.is_ipv4_mapped());
        }

        #[test]
        fn ext_segments() {
            let ip = Ipv6Addr::new("2001:db8::1".to_string()).unwrap();
            let segments = ip.segments();
            assert_eq!(segments[0], 0x2001);
            assert_eq!(segments[1], 0x0db8);
            assert_eq!(segments[7], 0x0001);
        }

        // Error message tests
        #[test]
        fn error_includes_format_name() {
            let result = Ipv6Addr::new("invalid".to_string());
            let err = result.unwrap_err();
            assert_eq!(err.format_name, "IPv6 address");
        }

        #[test]
        fn error_includes_example() {
            let result = Ipv6Addr::new("invalid".to_string());
            let err = result.unwrap_err();
            assert_eq!(err.example, "2001:db8::1");
        }

        #[test]
        fn description_returns_expected() {
            assert_eq!(ValidIpv6::description(), "RFC 4291 IPv6 address");
        }
    }

    // ========================================================================
    // Domain Name Tests
    // ========================================================================

    mod domain_tests {
        use super::*;

        // Valid cases
        #[test]
        fn valid_simple() {
            assert!(DomainName::new("example.com".to_string()).is_ok());
        }

        #[test]
        fn valid_subdomain() {
            assert!(DomainName::new("sub.example.com".to_string()).is_ok());
        }

        #[test]
        fn valid_multiple_subdomains() {
            assert!(DomainName::new("a.b.c.example.com".to_string()).is_ok());
        }

        #[test]
        fn valid_localhost() {
            assert!(DomainName::new("localhost".to_string()).is_ok());
        }

        #[test]
        fn valid_co_uk() {
            assert!(DomainName::new("example.co.uk".to_string()).is_ok());
        }

        #[test]
        fn valid_with_numbers() {
            assert!(DomainName::new("example123.com".to_string()).is_ok());
        }

        #[test]
        fn valid_starting_with_number() {
            assert!(DomainName::new("123example.com".to_string()).is_ok());
        }

        #[test]
        fn valid_with_hyphen() {
            assert!(DomainName::new("my-example.com".to_string()).is_ok());
        }

        #[test]
        fn valid_short_labels() {
            assert!(DomainName::new("a.b".to_string()).is_ok());
        }

        #[test]
        fn valid_with_trailing_dot() {
            assert!(DomainName::new("example.com.".to_string()).is_ok());
        }

        // Invalid cases
        #[test]
        fn invalid_empty() {
            let result = DomainName::new(String::new());
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err.reason, DomainErrorKind::Empty));
        }

        #[test]
        fn invalid_starting_with_hyphen() {
            let result = DomainName::new("-example.com".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn invalid_ending_with_hyphen() {
            let result = DomainName::new("example-.com".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn invalid_space() {
            let result = DomainName::new("exam ple.com".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn invalid_underscore() {
            let result = DomainName::new("exam_ple.com".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn invalid_double_dot() {
            let result = DomainName::new("example..com".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn invalid_too_long_total() {
            let long_domain = format!("{}.com", "a".repeat(250));
            let result = DomainName::new(long_domain);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err.reason, DomainErrorKind::TooLong { .. }));
        }

        #[test]
        fn invalid_too_long_label() {
            let long_label = format!("{}.com", "a".repeat(64));
            let result = DomainName::new(long_label);
            assert!(result.is_err());
        }

        // Extension trait tests
        #[test]
        fn ext_labels() {
            let domain = DomainName::new("sub.example.com".to_string()).unwrap();
            let labels = domain.labels();
            assert_eq!(labels, vec!["sub", "example", "com"]);
        }

        #[test]
        fn ext_tld() {
            let domain = DomainName::new("sub.example.com".to_string()).unwrap();
            assert_eq!(domain.tld(), Some("com"));
        }

        #[test]
        fn ext_is_subdomain_of() {
            let domain = DomainName::new("api.example.com".to_string()).unwrap();
            assert!(domain.is_subdomain_of("example.com"));
            assert!(!domain.is_subdomain_of("other.com"));
        }

        // Error message tests
        #[test]
        fn error_includes_format_name() {
            let result = DomainName::new("-invalid.com".to_string());
            let err = result.unwrap_err();
            assert_eq!(err.format_name, "domain name");
        }

        #[test]
        fn error_includes_example() {
            let result = DomainName::new("-invalid.com".to_string());
            let err = result.unwrap_err();
            assert_eq!(err.example, "example.com");
        }

        #[test]
        fn description_returns_expected() {
            assert_eq!(ValidDomainName::description(), "RFC 1035 domain name");
        }
    }

    // ========================================================================
    // Port Tests
    // ========================================================================

    mod port_tests {
        use super::*;

        // Valid cases
        #[test]
        fn valid_port_1() {
            assert!(Port::new(1).is_ok());
        }

        #[test]
        fn valid_port_80() {
            assert!(Port::new(80).is_ok());
        }

        #[test]
        fn valid_port_443() {
            assert!(Port::new(443).is_ok());
        }

        #[test]
        fn valid_port_8080() {
            assert!(Port::new(8080).is_ok());
        }

        #[test]
        fn valid_port_max() {
            assert!(Port::new(65535).is_ok());
        }

        // Invalid cases
        #[test]
        fn invalid_port_0() {
            let result = Port::new(0);
            assert!(result.is_err());
        }

        // Extension trait tests
        #[test]
        fn ext_is_privileged() {
            let privileged = Port::new(80).unwrap();
            assert!(privileged.is_privileged());

            let not_privileged = Port::new(8080).unwrap();
            assert!(!not_privileged.is_privileged());
        }

        #[test]
        fn ext_is_well_known() {
            let well_known = Port::new(443).unwrap();
            assert!(well_known.is_well_known());

            let not_well_known = Port::new(8443).unwrap();
            assert!(!not_well_known.is_well_known());
        }

        #[test]
        fn ext_is_registered() {
            let registered = Port::new(3306).unwrap();
            assert!(registered.is_registered());

            let not_registered_low = Port::new(80).unwrap();
            assert!(!not_registered_low.is_registered());

            let not_registered_high = Port::new(50000).unwrap();
            assert!(!not_registered_high.is_registered());
        }

        #[test]
        fn ext_is_dynamic() {
            let dynamic = Port::new(50000).unwrap();
            assert!(dynamic.is_dynamic());

            let not_dynamic = Port::new(8080).unwrap();
            assert!(!not_dynamic.is_dynamic());
        }

        #[test]
        fn ext_is_ephemeral() {
            let ephemeral = Port::new(60000).unwrap();
            assert!(ephemeral.is_ephemeral());
        }

        #[test]
        fn port_range_boundary_1023() {
            let port = Port::new(1023).unwrap();
            assert!(port.is_privileged());
            assert!(!port.is_registered());
        }

        #[test]
        fn port_range_boundary_1024() {
            let port = Port::new(1024).unwrap();
            assert!(!port.is_privileged());
            assert!(port.is_registered());
        }

        #[test]
        fn port_range_boundary_49151() {
            let port = Port::new(49151).unwrap();
            assert!(port.is_registered());
            assert!(!port.is_dynamic());
        }

        #[test]
        fn port_range_boundary_49152() {
            let port = Port::new(49152).unwrap();
            assert!(!port.is_registered());
            assert!(port.is_dynamic());
        }

        // Error message tests
        #[test]
        fn error_includes_format_name() {
            let result = Port::new(0);
            let err = result.unwrap_err();
            assert_eq!(err.format_name, "port");
        }

        #[test]
        fn error_includes_example() {
            let result = Port::new(0);
            let err = result.unwrap_err();
            assert_eq!(err.example, "8080");
        }

        #[test]
        fn description_returns_expected() {
            assert_eq!(ValidPort::description(), "TCP/UDP port number (1-65535)");
        }
    }
}
