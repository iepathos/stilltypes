---
number: 1
title: Network Types
category: foundation
priority: high
status: draft
dependencies: []
created: 2025-12-22
---

# Specification 001: Network Types

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: none

## Context

Network programming is one of the most common use cases for refined types. IP addresses, domain names, and port numbers are ubiquitous in web applications, APIs, configuration files, and system administration tools. Currently, developers must manually validate these formats or use raw strings/integers, leading to potential runtime errors.

The Rust standard library provides `std::net::IpAddr`, `Ipv4Addr`, and `Ipv6Addr` for parsing, but these don't integrate with stilltypes' `DomainError` system. Additionally, domain names require DNS-compliant validation (RFC 1035), and port numbers need range validation with semantic meaning (privileged vs unprivileged ports).

## Objective

Add a `network` feature to stilltypes providing refined types for IP addresses (v4 and v6), domain names, and port numbers. All types must integrate with `DomainError` for rich, user-facing error messages and follow the existing module patterns established by email, url, and other stilltypes modules.

## Requirements

### Functional Requirements

1. **IPv4 Address Type**
   - Parse and validate IPv4 addresses in dotted-decimal notation (e.g., "192.168.1.1")
   - Reject invalid formats (wrong number of octets, octet > 255, leading zeros ambiguity)
   - Provide conversion to `std::net::Ipv4Addr`
   - Support both `"192.168.1.1"` string format and construction from octets

2. **IPv6 Address Type**
   - Parse and validate IPv6 addresses per RFC 4291
   - Support full notation, compressed notation (::), and mixed notation (::ffff:192.168.1.1)
   - Provide conversion to `std::net::Ipv6Addr`
   - Handle case-insensitive hex digits

3. **Domain Name Type**
   - Validate DNS domain names per RFC 1035
   - Labels must be 1-63 characters, total length <= 253 characters
   - Labels must start with a letter, contain only alphanumerics and hyphens
   - Labels cannot end with a hyphen
   - Support internationalized domain names (IDN) via Punycode is optional for v1

4. **Port Type**
   - Validate port numbers in range 1-65535 (or 0-65535 if including ephemeral)
   - Provide semantic helpers: `is_privileged()` (< 1024), `is_well_known()` (< 1024), `is_registered()` (1024-49151), `is_dynamic()` (49152-65535)
   - Use `u16` as the underlying type, not String

### Non-Functional Requirements

- All types must produce `DomainError` with helpful messages
- Zero external dependencies for basic validation (use `std::net` for IP parsing)
- Optional `addr` or `idna` crate dependency for IDN support
- Serde support when `serde` feature is enabled
- All predicates must be zero-sized types (ZSTs) for zero-cost abstraction

## Acceptance Criteria

- [ ] `Ipv4Addr` type validates and parses IPv4 addresses with proper error messages
- [ ] `Ipv6Addr` type validates and parses IPv6 addresses with proper error messages
- [ ] `DomainName` type validates DNS names per RFC 1035 with label-specific errors
- [ ] `Port` type validates port numbers with range errors and semantic helpers
- [ ] Extension trait `IpAddrExt` provides `to_std()` conversion for IP types
- [ ] Extension trait `PortExt` provides `is_privileged()`, `is_registered()`, `is_dynamic()`
- [ ] All types include comprehensive doc comments with examples
- [ ] Unit tests cover valid cases, invalid cases, and edge cases for each type
- [ ] Integration tests verify serde round-trip when feature enabled
- [ ] Error messages include format examples (e.g., "example: 192.168.1.1")

## Technical Details

### Implementation Approach

Follow the established stilltypes pattern:

```rust
// src/network.rs

use stillwater::refined::Refined;
use crate::error::{DomainError, DomainErrorKind};

/// Predicate for valid IPv4 addresses.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidIpv4;

impl Predicate<String> for ValidIpv4 {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        value.parse::<std::net::Ipv4Addr>()
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
}

/// A validated IPv4 address.
pub type Ipv4Addr = Refined<String, ValidIpv4>;

/// Extension trait for IP address operations.
pub trait Ipv4Ext {
    /// Convert to standard library type.
    fn to_std(&self) -> std::net::Ipv4Addr;

    /// Check if this is a loopback address (127.0.0.0/8).
    fn is_loopback(&self) -> bool;

    /// Check if this is a private address (RFC 1918).
    fn is_private(&self) -> bool;
}
```

### Module Structure

```
src/
├── network.rs          # All network types in one module
│   ├── ValidIpv4       # IPv4 predicate
│   ├── ValidIpv6       # IPv6 predicate
│   ├── ValidDomainName # Domain name predicate
│   ├── ValidPort       # Port number predicate
│   ├── Ipv4Addr        # type alias
│   ├── Ipv6Addr        # type alias
│   ├── DomainName      # type alias
│   ├── Port            # type alias
│   └── extension traits
```

### Feature Flag

```toml
[features]
network = []  # No external deps for basic validation

[dependencies]
# Optional for IDN support
idna = { version = "0.5", optional = true }

[features]
network-idn = ["network", "dep:idna"]
```

### Error Messages

```
"invalid IPv4 address: invalid format, expected dotted decimal (A.B.C.D) (example: 192.168.1.1)"
"invalid IPv6 address: invalid format, expected hex groups separated by colons (example: 2001:db8::1)"
"invalid domain name: label 'my--host' contains consecutive hyphens (example: example.com)"
"invalid port: 70000 is out of range 1-65535 (example: 8080)"
```

## Dependencies

- **Prerequisites**: None (first spec in series)
- **Affected Components**: `src/lib.rs` (add module), `src/prelude.rs` (add exports), `Cargo.toml` (add feature)
- **External Dependencies**: None required; `idna` optional for IDN support

## Testing Strategy

- **Unit Tests**:
  - Valid IPv4: "0.0.0.0", "255.255.255.255", "192.168.1.1", "10.0.0.1"
  - Invalid IPv4: "256.0.0.0", "1.2.3", "1.2.3.4.5", "abc.def.ghi.jkl", ""
  - Valid IPv6: "::1", "2001:db8::1", "fe80::1%eth0" (if scope supported)
  - Invalid IPv6: "12345::1", ":::", "2001:db8::g"
  - Valid domains: "example.com", "sub.example.co.uk", "localhost", "a.b"
  - Invalid domains: "-example.com", "exam ple.com", "a" * 64 + ".com"
  - Valid ports: 1, 80, 443, 8080, 65535
  - Invalid ports: 0 (if excluded), 65536, -1 (if signed input)

- **Integration Tests**: Serde serialize/deserialize round-trip
- **Property Tests**: Consider proptest for IPv4/IPv6 format fuzzing

## Documentation Requirements

- **Code Documentation**: Full rustdoc with examples for each type and trait
- **User Documentation**: Update lib.rs feature table
- **README**: Add network feature to examples

## Implementation Notes

- Use `std::net::Ipv4Addr::from_str()` and `Ipv6Addr::from_str()` for parsing
- Domain name validation should be done manually to provide label-specific errors
- Port validation is trivial range check but semantic helpers add value
- Consider whether to store IP addresses as String or as the std type internally

## Migration and Compatibility

- New feature, no breaking changes
- Optional feature flag means no impact on existing users
